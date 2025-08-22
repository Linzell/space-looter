//! Tile Movement Service - Grid-based movement with dice roll events
//!
//! This service handles player movement on a tile-based grid system where
//! each movement triggers a dice roll that determines what event occurs
//! based on the roll result and player progression.

use crate::domain::entities::{Event, EventType, Map, Player};
use crate::domain::value_objects::{
    dice::{DiceModifier, DiceRoll, DiceType},
    Position3D, TileCoordinate,
};
use crate::domain::{DomainError, DomainResult};
use rand::Rng;
use std::collections::HashMap;

/// Service for handling tile-based movement with dice events
#[derive(Debug)]
pub struct TileMovementService {
    /// Cached event templates for different outcomes
    event_templates: HashMap<EventCategory, Vec<EventTemplate>>,
}

impl TileMovementService {
    /// Create a new tile movement service
    pub fn new() -> Self {
        let mut service = Self {
            event_templates: HashMap::new(),
        };
        service.initialize_event_templates();
        service
    }

    /// Execute a movement attempt from current position to target position
    /// Returns the movement result with any triggered events
    pub fn attempt_movement(
        &self,
        player: &Player,
        target_position: Position3D,
        map: &Map,
        player_level: u32,
    ) -> DomainResult<MovementResult> {
        // Check if movement is valid
        if !self.is_valid_movement(player.position(), &target_position) {
            return Err(DomainError::InvalidMapCoordinates(
                target_position.x,
                target_position.y,
                target_position.z,
            ));
        }

        // Check if target tile is passable
        if !map.is_passable(&target_position) {
            return Err(DomainError::TileNotAccessible(
                target_position.x,
                target_position.y,
                target_position.z,
            ));
        }

        // Calculate movement cost
        let movement_cost = map.movement_cost(&target_position);
        if player.movement_points() < movement_cost {
            return Err(DomainError::InsufficientResources(format!(
                "Not enough movement points. Need: {}, Have: {}",
                movement_cost,
                player.movement_points()
            )));
        }

        // Roll dice for movement event
        let dice_result = self.roll_movement_dice(player, map, &target_position, player_level)?;

        // Generate event based on dice result
        let event =
            self.generate_movement_event(&dice_result, &target_position, map, player_level)?;

        // Create movement result
        Ok(MovementResult {
            success: true,
            target_position,
            movement_cost,
            dice_result,
            triggered_event: event,
        })
    }

    /// Check if movement from current to target position is valid (adjacent tiles only)
    fn is_valid_movement(&self, current: &Position3D, target: &Position3D) -> bool {
        let dx = (target.x - current.x).abs();
        let dy = (target.y - current.y).abs();
        let dz = (target.z - current.z).abs();

        // Only allow movement to adjacent tiles (Manhattan distance of 1)
        (dx + dy + dz) == 1
    }

    /// Roll dice for movement with modifiers based on player and environment
    fn roll_movement_dice(
        &self,
        player: &Player,
        map: &Map,
        target_position: &Position3D,
        player_level: u32,
    ) -> DomainResult<MovementDiceResult> {
        // Base dice roll (d20)
        let base_dice = DiceRoll::new(1, DiceType::D20, DiceModifier::none())?;
        let base_result = base_dice.total();

        // Calculate modifiers
        let mut total_modifier = 0i8;

        // Player level modifier (higher level = better outcomes)
        let level_modifier = (player_level as i8 / 5).min(5); // +1 per 5 levels, max +5
        total_modifier += level_modifier;

        // Terrain modifier
        let tile_coord = TileCoordinate::from(*target_position);
        if let Some(tile) = map.get_tile(&tile_coord) {
            let terrain_modifier = match tile.terrain_type {
                crate::domain::value_objects::terrain::TerrainType::Plains => 2,
                crate::domain::value_objects::terrain::TerrainType::Forest => 0,
                crate::domain::value_objects::terrain::TerrainType::Mountains => -2,
                crate::domain::value_objects::terrain::TerrainType::Desert => -1,
                crate::domain::value_objects::terrain::TerrainType::Tundra => -3,
                crate::domain::value_objects::terrain::TerrainType::Swamp => -4,
                crate::domain::value_objects::terrain::TerrainType::Ocean => -5,
                crate::domain::value_objects::terrain::TerrainType::Volcanic => -4,
                crate::domain::value_objects::terrain::TerrainType::Anomaly => -6,
                crate::domain::value_objects::terrain::TerrainType::Constructed => 3,
                crate::domain::value_objects::terrain::TerrainType::Cave => -3,
                crate::domain::value_objects::terrain::TerrainType::Crystal => 1,
            };
            total_modifier += terrain_modifier;
        }

        // Danger level modifier (higher danger = worse outcomes but better rewards)
        let danger_level = map.danger_level(target_position);
        let danger_modifier = -(danger_level as i8 / 2); // Negative modifier for danger
        total_modifier += danger_modifier;

        // Apply modifier to roll
        let modified_result = (base_result as i8 + total_modifier).max(1) as u8;

        Ok(MovementDiceResult {
            base_roll: base_result as u8,
            level_modifier,
            terrain_modifier: if let Some(tile) =
                map.get_tile(&TileCoordinate::from(*target_position))
            {
                match tile.terrain_type {
                    crate::domain::value_objects::terrain::TerrainType::Plains => 2,
                    crate::domain::value_objects::terrain::TerrainType::Forest => 0,
                    crate::domain::value_objects::terrain::TerrainType::Mountains => -2,
                    crate::domain::value_objects::terrain::TerrainType::Desert => -1,
                    crate::domain::value_objects::terrain::TerrainType::Tundra => -3,
                    crate::domain::value_objects::terrain::TerrainType::Swamp => -4,
                    crate::domain::value_objects::terrain::TerrainType::Ocean => -5,
                    crate::domain::value_objects::terrain::TerrainType::Volcanic => -4,
                    crate::domain::value_objects::terrain::TerrainType::Anomaly => -6,
                    crate::domain::value_objects::terrain::TerrainType::Constructed => 3,
                    crate::domain::value_objects::terrain::TerrainType::Cave => -3,
                    crate::domain::value_objects::terrain::TerrainType::Crystal => 1,
                }
            } else {
                0
            },
            danger_modifier,
            total_modifier,
            final_result: modified_result,
            dice_roll: base_dice,
        })
    }

    /// Generate an event based on dice roll result
    fn generate_movement_event(
        &self,
        dice_result: &MovementDiceResult,
        position: &Position3D,
        _map: &Map,
        _player_level: u32,
    ) -> DomainResult<Option<Event>> {
        let result = dice_result.final_result;

        // Determine event category based on dice result
        let event_category = match result {
            1..=3 => EventCategory::CriticalFailure,
            4..=7 => EventCategory::Failure,
            8..=12 => EventCategory::Neutral,
            13..=16 => EventCategory::Success,
            17..=19 => EventCategory::GreatSuccess,
            20..=255 => EventCategory::CriticalSuccess,
            0 => EventCategory::CriticalFailure, // Edge case for 0
        };

        // Some rolls don't trigger events (neutral outcomes)
        if matches!(event_category, EventCategory::Neutral) && result >= 10 {
            return Ok(None); // Safe movement, no event
        }

        // Select event template
        let templates = self.event_templates.get(&event_category).ok_or_else(|| {
            DomainError::EventTriggerError("No event templates found".to_string())
        })?;

        if templates.is_empty() {
            return Ok(None);
        }

        let mut rng = rand::thread_rng();
        let template = &templates[rng.gen_range(0..templates.len())];

        // Create event from template
        let event = Event::new(
            template.event_type,
            template.title.clone(),
            template.description.clone(),
            Some(*position),
        )?;

        Ok(Some(event))
    }

    /// Initialize event templates for different categories
    fn initialize_event_templates(&mut self) {
        // Critical Failure events (1-3)
        self.event_templates.insert(
            EventCategory::CriticalFailure,
            vec![
                EventTemplate {
                    event_type: EventType::Hazard,
                    title: "Equipment Malfunction".to_string(),
                    description: "Your equipment suffers a critical failure!".to_string(),
                },
                EventTemplate {
                    event_type: EventType::Combat,
                    title: "Ambush!".to_string(),
                    description: "Hostile entities emerge from the shadows!".to_string(),
                },
                EventTemplate {
                    event_type: EventType::Hazard,
                    title: "Environmental Hazard".to_string(),
                    description: "The ground gives way beneath your feet!".to_string(),
                },
            ],
        );

        // Failure events (4-7)
        self.event_templates.insert(
            EventCategory::Failure,
            vec![
                EventTemplate {
                    event_type: EventType::Hazard,
                    title: "Minor Setback".to_string(),
                    description: "You encounter a minor obstacle that slows your progress."
                        .to_string(),
                },
                EventTemplate {
                    event_type: EventType::Malfunction,
                    title: "Equipment Strain".to_string(),
                    description: "Your equipment shows signs of wear and tear.".to_string(),
                },
                EventTemplate {
                    event_type: EventType::Combat,
                    title: "Hostile Encounter".to_string(),
                    description: "You spot dangerous creatures in the area.".to_string(),
                },
            ],
        );

        // Neutral events (8-12)
        self.event_templates.insert(
            EventCategory::Neutral,
            vec![
                EventTemplate {
                    event_type: EventType::Narrative,
                    title: "Quiet Exploration".to_string(),
                    description: "You move through the area without incident.".to_string(),
                },
                EventTemplate {
                    event_type: EventType::Mystery,
                    title: "Strange Phenomenon".to_string(),
                    description: "You notice something unusual but can't quite identify what."
                        .to_string(),
                },
            ],
        );

        // Success events (13-16)
        self.event_templates.insert(
            EventCategory::Success,
            vec![
                EventTemplate {
                    event_type: EventType::ResourceDiscovery,
                    title: "Resource Cache".to_string(),
                    description: "You discover a small cache of useful resources.".to_string(),
                },
                EventTemplate {
                    event_type: EventType::Trade,
                    title: "Friendly Encounter".to_string(),
                    description: "You encounter a friendly trader willing to deal.".to_string(),
                },
                EventTemplate {
                    event_type: EventType::Boon,
                    title: "Favorable Conditions".to_string(),
                    description: "The environment provides unexpected advantages.".to_string(),
                },
            ],
        );

        // Great Success events (17-19)
        self.event_templates.insert(
            EventCategory::GreatSuccess,
            vec![
                EventTemplate {
                    event_type: EventType::ResourceDiscovery,
                    title: "Rich Deposit".to_string(),
                    description: "You discover a rich vein of valuable resources!".to_string(),
                },
                EventTemplate {
                    event_type: EventType::Boon,
                    title: "Ancient Technology".to_string(),
                    description: "You find remnants of advanced technology!".to_string(),
                },
                EventTemplate {
                    event_type: EventType::Mystery,
                    title: "Hidden Knowledge".to_string(),
                    description: "You uncover secrets that expand your understanding.".to_string(),
                },
            ],
        );

        // Critical Success events (20+)
        self.event_templates.insert(
            EventCategory::CriticalSuccess,
            vec![
                EventTemplate {
                    event_type: EventType::ResourceDiscovery,
                    title: "Jackpot Discovery".to_string(),
                    description: "You strike it rich with an incredible resource find!".to_string(),
                },
                EventTemplate {
                    event_type: EventType::Boon,
                    title: "Legendary Artifact".to_string(),
                    description: "You discover a powerful artifact of ancient origin!".to_string(),
                },
                EventTemplate {
                    event_type: EventType::Trade,
                    title: "Exclusive Opportunity".to_string(),
                    description: "A rare trading opportunity presents itself!".to_string(),
                },
            ],
        );
    }
}

impl Default for TileMovementService {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a movement attempt
#[derive(Debug, Clone, PartialEq)]
pub struct MovementResult {
    pub success: bool,
    pub target_position: Position3D,
    pub movement_cost: u8,
    pub dice_result: MovementDiceResult,
    pub triggered_event: Option<Event>,
}

/// Detailed result of movement dice roll
#[derive(Debug, Clone, PartialEq)]
pub struct MovementDiceResult {
    pub base_roll: u8,
    pub level_modifier: i8,
    pub terrain_modifier: i8,
    pub danger_modifier: i8,
    pub total_modifier: i8,
    pub final_result: u8,
    pub dice_roll: DiceRoll,
}

impl MovementDiceResult {
    /// Get a formatted description of the dice roll
    pub fn description(&self) -> String {
        format!(
            "🎲 Rolled {} + {} = {} (Base: {}, Level: {:+}, Terrain: {:+}, Danger: {:+})",
            self.base_roll,
            self.total_modifier,
            self.final_result,
            self.base_roll,
            self.level_modifier,
            self.terrain_modifier,
            self.danger_modifier
        )
    }

    /// Get the outcome category as a string
    pub fn outcome_category(&self) -> &'static str {
        match self.final_result {
            1..=3 => "Critical Failure",
            4..=7 => "Failure",
            8..=12 => "Neutral",
            13..=16 => "Success",
            17..=19 => "Great Success",
            20..=255 => "Critical Success",
            0 => "Critical Failure", // Edge case for 0
        }
    }
}

/// Categories of events based on dice roll results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum EventCategory {
    CriticalFailure,
    Failure,
    Neutral,
    Success,
    GreatSuccess,
    CriticalSuccess,
}

/// Template for generating events
#[derive(Debug, Clone, PartialEq)]
struct EventTemplate {
    event_type: EventType,
    title: String,
    description: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::Map;
    use crate::domain::entities::MapTile;
    use crate::domain::value_objects::{
        terrain::Elevation, terrain::TerrainType, EntityId, PlayerStats,
    };

    fn create_test_player() -> Player {
        let stats = PlayerStats::starting_stats();
        Player::create_new_character("Test Player".to_string(), Position3D::origin()).unwrap()
    }

    fn create_test_map() -> Map {
        let mut map = Map::new(EntityId::generate(), "Test Map".to_string(), 12345).unwrap();

        // Add some test tiles
        for x in -2..=2 {
            for y in -2..=2 {
                let coord = TileCoordinate::new(x, y, 0);
                let tile = MapTile::new(TerrainType::Plains, Elevation::sea_level(), false);
                map.set_tile(coord, tile);
            }
        }

        map
    }

    #[test]
    fn service_creation() {
        let service = TileMovementService::new();
        assert!(!service.event_templates.is_empty());
    }

    #[test]
    fn valid_movement_check() {
        let service = TileMovementService::new();
        let current = Position3D::new(0, 0, 0);

        // Adjacent tiles should be valid
        assert!(service.is_valid_movement(&current, &Position3D::new(1, 0, 0)));
        assert!(service.is_valid_movement(&current, &Position3D::new(0, 1, 0)));
        assert!(service.is_valid_movement(&current, &Position3D::new(0, 0, 1)));
        assert!(service.is_valid_movement(&current, &Position3D::new(-1, 0, 0)));

        // Diagonal or distant tiles should be invalid
        assert!(!service.is_valid_movement(&current, &Position3D::new(1, 1, 0)));
        assert!(!service.is_valid_movement(&current, &Position3D::new(2, 0, 0)));
    }

    #[test]
    fn movement_dice_roll() {
        let service = TileMovementService::new();
        let player = create_test_player();
        let map = create_test_map();
        let target = Position3D::new(1, 0, 0);

        let result = service
            .roll_movement_dice(&player, &map, &target, 1)
            .unwrap();

        assert!(result.base_roll >= 1 && result.base_roll <= 20);
        assert!(result.final_result >= 1);
    }

    #[test]
    fn movement_attempt_success() {
        let service = TileMovementService::new();
        let player = create_test_player();
        let map = create_test_map();
        let target = Position3D::new(1, 0, 0);

        let result = service.attempt_movement(&player, target, &map, 1);
        assert!(result.is_ok());

        let movement_result = result.unwrap();
        assert!(movement_result.success);
        assert_eq!(movement_result.target_position, target);
    }

    #[test]
    fn movement_attempt_invalid_target() {
        let service = TileMovementService::new();
        let player = create_test_player();
        let map = create_test_map();
        let target = Position3D::new(2, 2, 0); // Too far

        let result = service.attempt_movement(&player, target, &map, 1);
        assert!(result.is_err());
    }

    #[test]
    fn dice_result_description() {
        let dice_roll = DiceRoll::new(1, DiceType::D20, DiceModifier::none()).unwrap();
        let result = MovementDiceResult {
            base_roll: 15,
            level_modifier: 2,
            terrain_modifier: 1,
            danger_modifier: -1,
            total_modifier: 2,
            final_result: 17,
            dice_roll,
        };

        let description = result.description();
        assert!(description.contains("🎲"));
        assert!(description.contains("15"));
        assert!(description.contains("17"));
    }

    #[test]
    fn outcome_categories() {
        let dice_roll = DiceRoll::new(1, DiceType::D20, DiceModifier::none()).unwrap();

        let critical_failure = MovementDiceResult {
            base_roll: 1,
            level_modifier: 0,
            terrain_modifier: 0,
            danger_modifier: 0,
            total_modifier: 0,
            final_result: 1,
            dice_roll: dice_roll.clone(),
        };
        assert_eq!(critical_failure.outcome_category(), "Critical Failure");

        let critical_success = MovementDiceResult {
            base_roll: 20,
            level_modifier: 0,
            terrain_modifier: 0,
            danger_modifier: 0,
            total_modifier: 0,
            final_result: 20,
            dice_roll,
        };
        assert_eq!(critical_success.outcome_category(), "Critical Success");
    }
}
