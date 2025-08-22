//! Resting Service - Night events and recovery when movement points reach zero
//!
//! This service handles the automatic resting phase that triggers when a player
//! runs out of movement points. During rest, various night events can occur
//! based on dice rolls, providing resources, encounters, or story elements.

use crate::domain::{
    entities::{Event, EventType, Player},
    value_objects::{
        dice::{DiceModifier, DiceRoll, DiceType},
        resources::{ResourceAmount, ResourceCollection},
        Position3D, ResourceType,
    },
    DomainError, DomainResult,
};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Service for handling resting mechanics and night events
#[derive(Debug, bevy::prelude::Resource)]
pub struct RestingService {
    /// Base probability for different night events
    night_event_probabilities: HashMap<NightEventType, f32>,
    /// Resource rewards for different rest outcomes
    rest_rewards: HashMap<RestOutcome, ResourceCollection>,
}

impl RestingService {
    /// Create a new resting service with default configuration
    pub fn new() -> Self {
        Self {
            night_event_probabilities: Self::initialize_night_event_probabilities(),
            rest_rewards: Self::initialize_rest_rewards(),
        }
    }

    /// Process a full rest cycle when player has 0 movement points
    pub fn process_rest_cycle(
        &self,
        player: &mut Player,
        current_position: Position3D,
    ) -> DomainResult<RestCycleResult> {
        // Roll for night events
        let night_dice = DiceRoll::new(1, DiceType::D20, DiceModifier::none())?;
        let night_roll = night_dice.total() as u8;

        // Determine what happens during the night
        let night_event = self.determine_night_event(night_roll, &current_position)?;
        let rest_outcome = self.determine_rest_outcome(night_roll, &night_event)?;

        // Apply rest effects
        let resources_gained = self.apply_rest_effects(player, &rest_outcome)?;

        // Restore movement points (always happens after rest)
        player.restore_points();

        // Add extra movement points based on rest quality to ensure playability
        let extra_movement = match rest_outcome {
            RestOutcome::PoorRest => 2,         // Total: max + 2 extra
            RestOutcome::NormalRest => 4,       // Total: max + 4 extra
            RestOutcome::GoodRest => 6,         // Total: max + 6 extra
            RestOutcome::GreatRest => 8,        // Total: max + 8 extra
            RestOutcome::ExceptionalRest => 12, // Total: max + 12 extra
        };

        player.add_movement_points(extra_movement);

        // Generate rest description
        let description = self.generate_rest_description(&night_event, &rest_outcome, night_roll);

        Ok(RestCycleResult {
            night_event,
            rest_outcome,
            resources_gained,
            movement_points_restored: player.movement_points(),
            description,
            dice_roll: night_roll,
            rest_completed_at: Utc::now(),
        })
    }

    /// Determine what type of night event occurs based on dice roll
    fn determine_night_event(
        &self,
        roll: u8,
        _position: &Position3D,
    ) -> DomainResult<NightEventType> {
        match roll {
            1..=2 => Ok(NightEventType::NightmareTerrors), // 10% - Bad dreams, restless sleep
            3..=4 => Ok(NightEventType::NightEncounter),   // 10% - Creatures or dangers
            5..=6 => Ok(NightEventType::ColdNight),        // 10% - Harsh weather
            7..=8 => Ok(NightEventType::StrangeNoises),    // 10% - Mysterious sounds
            9..=12 => Ok(NightEventType::RestlessNight),   // 20% - Poor sleep
            13..=16 => Ok(NightEventType::PeacefulRest),   // 20% - Normal rest
            17..=18 => Ok(NightEventType::PleasantDreams), // 10% - Good dreams, refreshing
            19 => Ok(NightEventType::LucidDream),          // 5% - Gain insights
            20 => Ok(NightEventType::PropheticVision),     // 5% - Major revelation
            _ => Ok(NightEventType::PeacefulRest),         // Fallback for any other values
        }
    }

    /// Determine the outcome quality based on event type and roll
    fn determine_rest_outcome(
        &self,
        roll: u8,
        night_event: &NightEventType,
    ) -> DomainResult<RestOutcome> {
        match night_event {
            NightEventType::NightmareTerrors | NightEventType::NightEncounter => {
                Ok(RestOutcome::PoorRest) // Always poor for bad events
            }
            NightEventType::ColdNight | NightEventType::StrangeNoises => {
                if roll >= 15 {
                    Ok(RestOutcome::NormalRest) // Managed to adapt
                } else {
                    Ok(RestOutcome::PoorRest)
                }
            }
            NightEventType::RestlessNight => Ok(RestOutcome::PoorRest),
            NightEventType::PeacefulRest => Ok(RestOutcome::NormalRest),
            NightEventType::PleasantDreams => Ok(RestOutcome::GoodRest),
            NightEventType::LucidDream => Ok(RestOutcome::GreatRest),
            NightEventType::PropheticVision => Ok(RestOutcome::ExceptionalRest),
        }
    }

    /// Apply the effects of resting to the player
    fn apply_rest_effects(
        &self,
        player: &mut Player,
        outcome: &RestOutcome,
    ) -> DomainResult<ResourceCollection> {
        let mut resources = ResourceCollection::new();

        match outcome {
            RestOutcome::PoorRest => {
                // Poor rest - minimal recovery, but still enough to continue playing
                resources.set_amount(ResourceType::Food, 2);
            }
            RestOutcome::NormalRest => {
                // Normal rest - small resource bonus
                resources.set_amount(ResourceType::Food, 5);
            }
            RestOutcome::GoodRest => {
                // Good rest - moderate bonuses
                resources.set_amount(ResourceType::Food, 10);
                resources.set_amount(ResourceType::Energy, 5);
                player.add_experience(10)?;
            }
            RestOutcome::GreatRest => {
                // Great rest - good bonuses + extra movement
                resources.set_amount(ResourceType::Food, 15);
                resources.set_amount(ResourceType::Energy, 10);
                resources.set_amount(ResourceType::Data, 5);
                player.add_experience(20)?;
            }
            RestOutcome::ExceptionalRest => {
                // Exceptional rest - major bonuses
                resources.set_amount(ResourceType::Food, 25);
                resources.set_amount(ResourceType::Energy, 20);
                resources.set_amount(ResourceType::Data, 15);
                resources.set_amount(ResourceType::Technology, 5);
                player.add_experience(50)?;
            }
        }

        // Add resources to player
        if !resources.is_empty() {
            player.add_resources(&resources);
        }

        Ok(resources)
    }

    /// Generate a description of what happened during rest
    fn generate_rest_description(
        &self,
        night_event: &NightEventType,
        outcome: &RestOutcome,
        roll: u8,
    ) -> String {
        let base_description = match night_event {
            NightEventType::NightmareTerrors => {
                "Terrible nightmares plague your sleep, filled with visions of cosmic horrors."
            }
            NightEventType::NightEncounter => {
                "Strange creatures move in the darkness around your camp, but you remain hidden."
            }
            NightEventType::ColdNight => {
                "The night is harsh and cold, making it difficult to rest comfortably."
            }
            NightEventType::StrangeNoises => {
                "Mysterious sounds echo through the night - whispers, distant machinery, or something else entirely."
            }
            NightEventType::RestlessNight => {
                "You toss and turn, unable to find comfortable rest on the alien ground."
            }
            NightEventType::PeacefulRest => {
                "The night passes peacefully, allowing you to recover your strength."
            }
            NightEventType::PleasantDreams => {
                "Pleasant dreams of home and better times fill your sleep, refreshing your spirit."
            }
            NightEventType::LucidDream => {
                "A lucid dream grants you insights into the nature of this strange world."
            }
            NightEventType::PropheticVision => {
                "A prophetic vision reveals hidden knowledge about your destiny and this world's secrets."
            }
        };

        let outcome_modifier = match outcome {
            RestOutcome::PoorRest => " Despite the difficulties, you manage to get minimal rest.",
            RestOutcome::NormalRest => " You wake feeling adequately rested.",
            RestOutcome::GoodRest => " You wake feeling refreshed and energized.",
            RestOutcome::GreatRest => {
                " You wake feeling exceptionally well-rested and ready for adventure."
            }
            RestOutcome::ExceptionalRest => {
                " You wake feeling like a new person, filled with energy and purpose."
            }
        };

        format!("{}{} (Roll: {})", base_description, outcome_modifier, roll)
    }

    /// Initialize night event probabilities
    fn initialize_night_event_probabilities() -> HashMap<NightEventType, f32> {
        let mut probabilities = HashMap::new();
        probabilities.insert(NightEventType::NightmareTerrors, 0.10);
        probabilities.insert(NightEventType::NightEncounter, 0.10);
        probabilities.insert(NightEventType::ColdNight, 0.10);
        probabilities.insert(NightEventType::StrangeNoises, 0.10);
        probabilities.insert(NightEventType::RestlessNight, 0.20);
        probabilities.insert(NightEventType::PeacefulRest, 0.20);
        probabilities.insert(NightEventType::PleasantDreams, 0.10);
        probabilities.insert(NightEventType::LucidDream, 0.05);
        probabilities.insert(NightEventType::PropheticVision, 0.05);
        probabilities
    }

    /// Initialize rest reward tables
    fn initialize_rest_rewards() -> HashMap<RestOutcome, ResourceCollection> {
        HashMap::new() // Rewards are handled dynamically in apply_rest_effects
    }
}

impl Default for RestingService {
    fn default() -> Self {
        Self::new()
    }
}

/// Types of events that can occur during night rest
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NightEventType {
    /// Terrible nightmares and poor sleep
    NightmareTerrors,
    /// Dangerous creatures or environmental hazards
    NightEncounter,
    /// Harsh weather conditions
    ColdNight,
    /// Mysterious sounds and disturbances
    StrangeNoises,
    /// General poor sleep quality
    RestlessNight,
    /// Normal, peaceful rest
    PeacefulRest,
    /// Pleasant dreams and good rest
    PleasantDreams,
    /// Lucid dreaming with insights
    LucidDream,
    /// Prophetic visions with major revelations
    PropheticVision,
}

impl std::fmt::Display for NightEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NightEventType::NightmareTerrors => write!(f, "Nightmare Terrors"),
            NightEventType::NightEncounter => write!(f, "Night Encounter"),
            NightEventType::ColdNight => write!(f, "Cold Night"),
            NightEventType::StrangeNoises => write!(f, "Strange Noises"),
            NightEventType::RestlessNight => write!(f, "Restless Night"),
            NightEventType::PeacefulRest => write!(f, "Peaceful Rest"),
            NightEventType::PleasantDreams => write!(f, "Pleasant Dreams"),
            NightEventType::LucidDream => write!(f, "Lucid Dream"),
            NightEventType::PropheticVision => write!(f, "Prophetic Vision"),
        }
    }
}

/// Quality of rest achieved
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RestOutcome {
    /// Poor rest - minimal recovery
    PoorRest,
    /// Normal rest - standard recovery
    NormalRest,
    /// Good rest - enhanced recovery
    GoodRest,
    /// Great rest - significant bonuses
    GreatRest,
    /// Exceptional rest - major bonuses
    ExceptionalRest,
}

impl std::fmt::Display for RestOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RestOutcome::PoorRest => write!(f, "Poor Rest"),
            RestOutcome::NormalRest => write!(f, "Normal Rest"),
            RestOutcome::GoodRest => write!(f, "Good Rest"),
            RestOutcome::GreatRest => write!(f, "Great Rest"),
            RestOutcome::ExceptionalRest => write!(f, "Exceptional Rest"),
        }
    }
}

/// Result of a complete rest cycle
#[derive(Debug, Clone)]
pub struct RestCycleResult {
    /// The night event that occurred
    pub night_event: NightEventType,
    /// The quality of rest achieved
    pub rest_outcome: RestOutcome,
    /// Resources gained during rest
    pub resources_gained: ResourceCollection,
    /// Movement points restored (should be max)
    pub movement_points_restored: u8,
    /// Narrative description of the rest
    pub description: String,
    /// The dice roll that determined the events
    pub dice_roll: u8,
    /// When the rest was completed
    pub rest_completed_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::{EntityId, PlayerStats};

    #[test]
    fn test_rest_cycle_creation() {
        let service = RestingService::new();
        let mut player = Player::new(
            EntityId::new(),
            "Test Player".to_string(),
            Position3D::new(0, 0, 0),
            PlayerStats::default(),
        )
        .unwrap();

        // Drain movement points
        player.subtract_movement_points(player.movement_points());
        assert_eq!(player.movement_points(), 0);

        let result = service.process_rest_cycle(&mut player, Position3D::new(0, 0, 0));
        assert!(result.is_ok());

        // Player should have movement points restored
        assert!(player.movement_points() > 0);
    }

    #[test]
    fn test_night_event_determination() {
        let service = RestingService::new();

        // Test specific roll outcomes
        let nightmare = service
            .determine_night_event(1, &Position3D::new(0, 0, 0))
            .unwrap();
        assert_eq!(nightmare, NightEventType::NightmareTerrors);

        let peaceful = service
            .determine_night_event(15, &Position3D::new(0, 0, 0))
            .unwrap();
        assert_eq!(peaceful, NightEventType::PeacefulRest);

        let vision = service
            .determine_night_event(20, &Position3D::new(0, 0, 0))
            .unwrap();
        assert_eq!(vision, NightEventType::PropheticVision);
    }

    #[test]
    fn test_rest_outcomes() {
        let service = RestingService::new();

        let poor = service
            .determine_rest_outcome(1, &NightEventType::NightmareTerrors)
            .unwrap();
        assert_eq!(poor, RestOutcome::PoorRest);

        let great = service
            .determine_rest_outcome(19, &NightEventType::LucidDream)
            .unwrap();
        assert_eq!(great, RestOutcome::GreatRest);
    }

    #[test]
    fn test_description_generation() {
        let service = RestingService::new();

        let description = service.generate_rest_description(
            &NightEventType::PeacefulRest,
            &RestOutcome::NormalRest,
            15,
        );

        assert!(description.contains("peaceful"));
        assert!(description.contains("Roll: 15"));
    }
}
