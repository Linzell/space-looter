//! Bevy Resources - Global Game State Management for 3D Isometric RPG
//!
//! This module contains Bevy resources that manage global game state for the RPG,
//! providing shared access to domain entities and services across systems.
//! Resources are designed for turn-based gameplay with dice mechanics.

use crate::domain::{
    Base, DiceRoll, EntityId, GamePhase, GameSession, GameTime, Map, Player, Position3D,
    ResourceType, TerrainType, WorldBoundaries,
};
use bevy::prelude::*;
use std::collections::HashMap;

/// Bevy resource wrapper for the main player entity
#[derive(Resource, Debug, Clone)]
pub struct PlayerResource {
    pub player: Option<Player>,
}

impl PlayerResource {
    /// Create a new player resource
    pub fn new() -> Self {
        Self { player: None }
    }

    /// Create player and store in resource
    pub fn create_player(
        &mut self,
        _player_id: String,
        player_name: String,
        position: Position3D,
        stats: crate::domain::PlayerStats,
    ) -> Result<(), crate::domain::DomainError> {
        let entity_id = crate::domain::EntityId::generate();
        let player = Player::new(entity_id, player_name, position, stats)?;
        self.player = Some(player);
        Ok(())
    }

    /// Get reference to current player
    pub fn player(&self) -> Option<&Player> {
        self.player.as_ref()
    }

    /// Get mutable reference to current player
    pub fn player_mut(&mut self) -> Option<&mut Player> {
        self.player.as_mut()
    }

    /// Check if player exists
    pub fn has_player(&self) -> bool {
        self.player.is_some()
    }

    /// Get player position if exists
    pub fn player_position(&self) -> Option<Position3D> {
        self.player.as_ref().map(|p| *p.position())
    }

    /// Get player level if exists
    pub fn level(&self) -> Option<u32> {
        self.player.as_ref().map(|p| p.level())
    }

    /// Move player to new position
    pub fn move_player(
        &mut self,
        new_position: Position3D,
        movement_cost: u8,
    ) -> Result<(), crate::domain::DomainError> {
        if let Some(player) = &mut self.player {
            player.move_to(new_position, movement_cost)
        } else {
            Err(crate::domain::DomainError::PlayerError(
                "No player exists".to_string(),
            ))
        }
    }

    /// Get reference to current player (convenience method)
    pub fn get_player(&self) -> Option<&Player> {
        self.player()
    }

    /// Get mutable reference to current player (convenience method)
    pub fn get_player_mut(&mut self) -> Option<&mut Player> {
        self.player_mut()
    }

    /// Subtract movement points from player
    pub fn subtract_movement_points(
        &mut self,
        points: u8,
    ) -> Result<(), crate::domain::DomainError> {
        if let Some(player) = &mut self.player {
            player.subtract_movement_points(points);
            Ok(())
        } else {
            Err(crate::domain::DomainError::PlayerError(
                "No player exists".to_string(),
            ))
        }
    }
}

impl Default for PlayerResource {
    fn default() -> Self {
        Self::new()
    }
}

/// Bevy resource wrapper for the player's base
#[derive(Resource, Debug, Clone)]
pub struct BaseResource {
    pub base: Option<Base>,
}

impl BaseResource {
    /// Create a new base resource
    pub fn new() -> Self {
        Self { base: None }
    }

    /// Create base and store in resource
    pub fn create_base(
        &mut self,
        base_name: String,
        position: Position3D,
    ) -> Result<(), crate::domain::DomainError> {
        let base_id = crate::domain::EntityId::generate();
        let base = Base::new(base_id, base_name, position)?;
        self.base = Some(base);
        Ok(())
    }

    /// Get reference to current base
    pub fn base(&self) -> Option<&Base> {
        self.base.as_ref()
    }

    /// Get mutable reference to current base
    pub fn base_mut(&mut self) -> Option<&mut Base> {
        self.base.as_mut()
    }

    /// Check if base exists
    pub fn has_base(&self) -> bool {
        self.base.is_some()
    }

    /// Get base position if exists
    pub fn base_position(&self) -> Option<Position3D> {
        self.base.as_ref().map(|b| *b.position())
    }

    /// Get base level if exists
    pub fn base_level(&self) -> Option<u32> {
        self.base.as_ref().map(|b| b.level() as u32)
    }

    /// Upgrade base
    pub fn upgrade_base(&mut self) -> Result<bool, crate::domain::DomainError> {
        if let Some(_base) = &mut self.base {
            // Simple upgrade implementation - in a real system this would
            // check resources, upgrade level, etc.
            Ok(true)
        } else {
            Err(crate::domain::DomainError::BaseUpgradeError(
                "No base exists".to_string(),
            ))
        }
    }
}

impl Default for BaseResource {
    fn default() -> Self {
        Self::new()
    }
}

/// Bevy resource wrapper for GameSession domain entity
#[derive(Resource, Debug, Clone)]
pub struct GameSessionResource {
    pub session: Option<GameSession>,
}

impl GameSessionResource {
    /// Create a new game session resource
    pub fn new() -> Self {
        Self { session: None }
    }

    /// Create a new game session
    pub fn create_session(&mut self, session_id: String) -> Result<(), crate::domain::DomainError> {
        let player_id = crate::domain::EntityId::generate();
        let map_id = crate::domain::EntityId::generate();
        let difficulty = crate::domain::entities::game::DifficultyLevel::Normal;
        let boundaries = crate::domain::WorldBoundaries::standard();

        let session = GameSession::new(session_id, player_id, map_id, difficulty, boundaries)?;
        self.session = Some(session);
        Ok(())
    }

    /// Get reference to current session
    pub fn session(&self) -> Option<&GameSession> {
        self.session.as_ref()
    }

    /// Get mutable reference to current session
    pub fn session_mut(&mut self) -> Option<&mut GameSession> {
        self.session.as_mut()
    }

    /// Check if there's an active session
    pub fn has_active_session(&self) -> bool {
        self.session
            .as_ref()
            .map(|s| s.is_active())
            .unwrap_or(false)
    }

    /// Start the current session
    pub fn start_session(&mut self) -> Result<(), crate::domain::DomainError> {
        if let Some(session) = &mut self.session {
            session.start()
        } else {
            Err(crate::domain::DomainError::GameSessionError(
                "No session exists".to_string(),
            ))
        }
    }

    /// End the current session
    pub fn end_session(&mut self) -> Result<(), crate::domain::DomainError> {
        if let Some(session) = &mut self.session {
            session.end()?;
            self.session = None;
            Ok(())
        } else {
            Err(crate::domain::DomainError::GameSessionError(
                "No session exists".to_string(),
            ))
        }
    }

    /// Get session experience points
    pub fn experience_points(&self) -> u32 {
        self.session
            .as_ref()
            .map(|s| s.experience_points())
            .unwrap_or(0)
    }

    /// Add experience points
    pub fn add_experience(&mut self, points: u32) -> Result<(), crate::domain::DomainError> {
        if let Some(session) = &mut self.session {
            session.add_experience(points)
        } else {
            Err(crate::domain::DomainError::GameSessionError(
                "No session exists".to_string(),
            ))
        }
    }
}

impl Default for GameSessionResource {
    fn default() -> Self {
        Self::new()
    }
}

/// Bevy resource for world boundaries
#[derive(Resource, Debug, Clone)]
pub struct WorldBoundariesResource {
    pub boundaries: WorldBoundaries,
}

impl WorldBoundariesResource {
    /// Create new world boundaries resource
    pub fn new(boundaries: WorldBoundaries) -> Self {
        Self { boundaries }
    }

    /// Create resource with standard boundaries
    pub fn standard() -> Self {
        Self {
            boundaries: WorldBoundaries::standard(),
        }
    }

    /// Create resource with large world boundaries
    pub fn large() -> Self {
        Self {
            boundaries: WorldBoundaries::large(),
        }
    }

    /// Check if position is within boundaries
    pub fn contains(&self, position: &Position3D) -> bool {
        self.boundaries.contains(position)
    }

    /// Clamp position to boundaries
    pub fn clamp(&self, position: Position3D) -> Position3D {
        self.boundaries.clamp(position)
    }

    /// Get world size
    pub fn size(&self) -> (i32, i32, i32) {
        self.boundaries.size()
    }
}

impl Default for WorldBoundariesResource {
    fn default() -> Self {
        Self::standard()
    }
}

/// Resource for tracking RPG game timing and turns
#[derive(Resource, Debug, Clone)]
pub struct GameTimerResource {
    pub game_time: GameTime,
    pub turn_number: u32,
    pub phase: GamePhase,
    pub last_event_check: f32,
    pub last_resource_update: f32,
    pub paused_time: f32,
    pub real_time: f32,
}

impl GameTimerResource {
    /// Create new game timer resource
    pub fn new() -> Self {
        Self {
            game_time: GameTime::start_of_game(),
            turn_number: 1,
            phase: GamePhase::PlayerTurn,
            last_event_check: 0.0,
            last_resource_update: 0.0,
            paused_time: 0.0,
            real_time: 0.0,
        }
    }

    /// Update game time
    pub fn update(&mut self, delta_time: f32) {
        self.real_time += delta_time;
        if self.phase != GamePhase::Paused {
            self.game_time = self.game_time.advance_by_seconds(delta_time as u32);
        }
    }

    /// Advance to next turn
    pub fn next_turn(&mut self) {
        self.turn_number += 1;
        self.phase = GamePhase::PlayerTurn;
        self.game_time = self.game_time.advance_by_turns(1);
    }

    /// Set game phase
    pub fn set_phase(&mut self, phase: GamePhase) {
        self.phase = phase;
    }

    /// Check if enough time has passed for event check
    pub fn should_check_events(&self) -> bool {
        self.real_time - self.last_event_check
            >= crate::domain::constants::EVENT_CHECK_INTERVAL as f32
    }

    /// Record event check time
    pub fn record_event_check(&mut self) {
        self.last_event_check = self.real_time;
    }

    /// Check if enough time has passed for resource update
    pub fn should_update_resources(&self) -> bool {
        self.real_time - self.last_resource_update
            >= crate::domain::constants::RESOURCE_GATHERING_TIME as f32
    }

    /// Record resource update time
    pub fn record_resource_update(&mut self) {
        self.last_resource_update = self.real_time;
    }

    /// Pause timer
    pub fn pause(&mut self) {
        self.paused_time = self.real_time;
        self.phase = GamePhase::Paused;
    }

    /// Resume timer
    pub fn resume(&mut self) {
        if self.phase == GamePhase::Paused {
            self.phase = GamePhase::PlayerTurn;
        }
    }

    /// Reset timer
    pub fn reset(&mut self) {
        self.game_time = GameTime::start_of_game();
        self.turn_number = 1;
        self.phase = GamePhase::PlayerTurn;
        self.last_event_check = 0.0;
        self.last_resource_update = 0.0;
        self.paused_time = 0.0;
        self.real_time = 0.0;
    }

    /// Get current game phase
    pub fn phase(&self) -> GamePhase {
        self.phase
    }

    /// Get current turn number
    pub fn turn(&self) -> u32 {
        self.turn_number
    }

    /// Check if it's the player's turn
    pub fn is_player_turn(&self) -> bool {
        self.phase == GamePhase::PlayerTurn
    }
}

impl Default for GameTimerResource {
    fn default() -> Self {
        Self::new()
    }
}

/// Resource for tracking RPG game statistics
#[derive(Resource, Debug, Clone)]
pub struct GameStatsResource {
    pub quests_completed: u32,
    pub events_triggered: u32,
    pub resources_gathered: HashMap<ResourceType, i32>,
    pub dice_rolls_made: u32,
    pub successful_rolls: u32,
    pub critical_successes: u32,
    pub critical_failures: u32,
    pub tiles_explored: u32,
    pub experience_gained: u32,
    pub game_duration: f32,
}

impl GameStatsResource {
    /// Create new game stats resource
    pub fn new() -> Self {
        Self {
            quests_completed: 0,
            events_triggered: 0,
            resources_gathered: HashMap::new(),
            dice_rolls_made: 0,
            successful_rolls: 0,
            critical_successes: 0,
            critical_failures: 0,
            tiles_explored: 0,
            experience_gained: 0,
            game_duration: 0.0,
        }
    }

    /// Record quest completion
    pub fn record_quest_completion(&mut self) {
        self.quests_completed += 1;
    }

    /// Record event trigger
    pub fn record_event_trigger(&mut self) {
        self.events_triggered += 1;
    }

    /// Record resource gathering
    pub fn record_resource_gather(&mut self, resource_type: ResourceType, amount: u32) {
        *self.resources_gathered.entry(resource_type).or_insert(0) += amount as i32;
    }

    /// Record dice roll result
    pub fn record_dice_roll(&mut self, dice_roll: &DiceRoll, success_threshold: u8) {
        self.dice_rolls_made += 1;
        let total = dice_roll.total();

        if total >= success_threshold as i32 {
            self.successful_rolls += 1;
        }

        if total >= crate::domain::constants::CRITICAL_SUCCESS_THRESHOLD as i32 {
            self.critical_successes += 1;
        } else if total <= crate::domain::constants::CRITICAL_FAILURE_THRESHOLD as i32 {
            self.critical_failures += 1;
        }
    }

    /// Record tile exploration
    pub fn record_tile_explored(&mut self) {
        self.tiles_explored += 1;
    }

    /// Record experience gain
    pub fn record_experience_gain(&mut self, amount: u32) {
        self.experience_gained += amount;
    }

    /// Update game duration
    pub fn update_duration(&mut self, delta_time: f32) {
        self.game_duration += delta_time;
    }

    /// Reset all statistics
    pub fn reset(&mut self) {
        self.quests_completed = 0;
        self.events_triggered = 0;
        self.resources_gathered.clear();
        self.dice_rolls_made = 0;
        self.successful_rolls = 0;
        self.critical_successes = 0;
        self.critical_failures = 0;
        self.tiles_explored = 0;
        self.experience_gained = 0;
        self.game_duration = 0.0;
    }

    /// Calculate success rate percentage
    pub fn success_rate(&self) -> f32 {
        if self.dice_rolls_made == 0 {
            0.0
        } else {
            (self.successful_rolls as f32 / self.dice_rolls_made as f32) * 100.0
        }
    }

    /// Calculate critical success rate
    pub fn critical_success_rate(&self) -> f32 {
        if self.dice_rolls_made == 0 {
            0.0
        } else {
            (self.critical_successes as f32 / self.dice_rolls_made as f32) * 100.0
        }
    }

    /// Get total resources gathered
    pub fn total_resources_gathered(&self) -> i32 {
        self.resources_gathered.values().sum()
    }

    /// Get resource gathered for specific type
    pub fn resources_gathered_for_type(&self, resource_type: ResourceType) -> i32 {
        self.resources_gathered
            .get(&resource_type)
            .copied()
            .unwrap_or(0)
    }
}

impl Default for GameStatsResource {
    fn default() -> Self {
        Self::new()
    }
}

/// Resource for managing current map state
#[derive(Resource, Debug, Clone)]
pub struct MapResource {
    pub current_map: Option<Map>,
    pub loaded_chunks: HashMap<(i32, i32), bool>, // Track which map chunks are loaded
    pub visible_area: (Position3D, Position3D),   // Min and max positions currently visible
}

impl MapResource {
    /// Create new map resource
    pub fn new() -> Self {
        Self {
            current_map: None,
            loaded_chunks: HashMap::new(),
            visible_area: (Position3D::new(0, 0, 0), Position3D::new(0, 0, 0)),
        }
    }

    /// Load a new map
    pub fn load_map(&mut self, map: Map) {
        self.current_map = Some(map);
        self.loaded_chunks.clear();
    }

    /// Get reference to current map
    pub fn current_map(&self) -> Option<&Map> {
        self.current_map.as_ref()
    }

    /// Get mutable reference to current map
    pub fn current_map_mut(&mut self) -> Option<&mut Map> {
        self.current_map.as_mut()
    }

    /// Check if map is loaded
    pub fn has_map(&self) -> bool {
        self.current_map.is_some()
    }

    /// Update visible area based on camera/player position
    pub fn update_visible_area(&mut self, center: Position3D, view_distance: i32) {
        self.visible_area = (
            Position3D::new(
                center.x - view_distance,
                center.y - view_distance,
                center.z - 1,
            ),
            Position3D::new(
                center.x + view_distance,
                center.y + view_distance,
                center.z + 3,
            ),
        );
    }

    /// Check if position is in visible area
    pub fn is_visible(&self, position: Position3D) -> bool {
        let (min, max) = self.visible_area;
        position.x >= min.x
            && position.x <= max.x
            && position.y >= min.y
            && position.y <= max.y
            && position.z >= min.z
            && position.z <= max.z
    }

    /// Mark chunk as loaded
    pub fn mark_chunk_loaded(&mut self, chunk_x: i32, chunk_y: i32) {
        self.loaded_chunks.insert((chunk_x, chunk_y), true);
    }

    /// Check if chunk is loaded
    pub fn is_chunk_loaded(&self, chunk_x: i32, chunk_y: i32) -> bool {
        self.loaded_chunks
            .get(&(chunk_x, chunk_y))
            .copied()
            .unwrap_or(false)
    }

    /// Get terrain at position
    pub fn terrain_at(&self, _position: Position3D) -> Option<TerrainType> {
        // Simplified implementation - in a real system this would query the actual map
        Some(TerrainType::Plains)
    }

    /// Get or create map around a given position
    /// This method ensures there's always a map available for gameplay
    pub fn get_or_create_map(&mut self, center_position: Position3D) -> &Map {
        if self.current_map.is_none() {
            // Create a new map with procedural generation
            let map_id = EntityId::generate();
            let map_name = format!(
                "Sector {}-{}",
                center_position.x / 100,
                center_position.y / 100
            );
            let seed = ((center_position.x as u64) << 32) | (center_position.y as u64);

            let mut new_map = Map::new(map_id, map_name, seed).expect("Failed to create new map");

            // Use MapService for generation
            let map_service = crate::domain::services::MapService::new(seed);
            if let Err(e) = map_service.generate_chunk(&mut new_map, center_position, 20) {
                error!("Failed to generate map chunk: {}", e);
                // Continue with empty map rather than failing
            }

            // Ensure the immediate area is also generated
            self.ensure_area_generated(center_position);

            self.current_map = Some(new_map);

            info!(
                "🗺️ Generated new map sector at ({}, {})",
                center_position.x, center_position.y
            );
        }

        self.current_map.as_ref().unwrap()
    }

    /// Get or create map around a given position (mutable reference)
    /// This method ensures there's always a map available for gameplay
    pub fn get_or_create_map_mut(&mut self, center_position: Position3D) -> &mut Map {
        if self.current_map.is_none() {
            // Create a new map with procedural generation
            let map_id = EntityId::generate();
            let map_name = format!(
                "Sector {}-{}",
                center_position.x / 100,
                center_position.y / 100
            );
            let seed = ((center_position.x as u64) << 32) | (center_position.y as u64);

            let mut new_map = Map::new(map_id, map_name, seed).expect("Failed to create new map");

            // Use MapService for generation
            let map_service = crate::domain::services::MapService::new(seed);
            map_service
                .generate_chunk(&mut new_map, center_position, 8)
                .expect("Failed to generate map chunk");

            self.current_map = Some(new_map);
        }

        self.current_map.as_mut().unwrap()
    }

    /// Ensure the area around a position is generated
    fn ensure_area_generated(&mut self, position: Position3D) {
        let chunk_size = 20;
        let chunk_x = position.x / chunk_size;
        let chunk_y = position.y / chunk_size;

        // Check if we need to generate surrounding chunks
        for dx in -1..=1 {
            for dy in -1..=1 {
                let check_chunk_x = chunk_x + dx;
                let check_chunk_y = chunk_y + dy;

                if !self.is_chunk_loaded(check_chunk_x, check_chunk_y) {
                    if let Some(map) = &mut self.current_map {
                        let chunk_center = Position3D::new(
                            check_chunk_x * chunk_size,
                            check_chunk_y * chunk_size,
                            position.z,
                        );

                        let map_service = crate::domain::services::MapService::new(map.seed());
                        if let Err(e) = map_service.generate_chunk(map, chunk_center, chunk_size) {
                            warn!(
                                "Failed to generate chunk at ({}, {}): {}",
                                check_chunk_x, check_chunk_y, e
                            );
                        } else {
                            self.mark_chunk_loaded(check_chunk_x, check_chunk_y);
                            debug!("Generated chunk at ({}, {})", check_chunk_x, check_chunk_y);
                        }
                    }
                }
            }
        }
    }
}

impl Default for MapResource {
    fn default() -> Self {
        Self::new()
    }
}

/// Resource for managing dice roll animations and UI
#[derive(Resource, Debug, Clone)]
pub struct DiceUIResource {
    pub pending_rolls: Vec<(DiceRoll, String, f32)>, // (roll, context, display_time)
    pub show_dice_ui: bool,
    pub last_roll_result: Option<DiceRoll>,
}

impl DiceUIResource {
    /// Create new dice UI resource
    pub fn new() -> Self {
        Self {
            pending_rolls: Vec::new(),
            show_dice_ui: false,
            last_roll_result: None,
        }
    }

    /// Add a dice roll to display
    pub fn add_roll(&mut self, roll: DiceRoll, context: String, display_time: f32) {
        self.pending_rolls
            .push((roll.clone(), context, display_time));
        self.last_roll_result = Some(roll);
        self.show_dice_ui = true;
    }

    /// Update pending rolls (remove expired ones)
    pub fn update(&mut self, delta_time: f32) {
        self.pending_rolls.retain_mut(|(_, _, time)| {
            *time -= delta_time;
            *time > 0.0
        });

        if self.pending_rolls.is_empty() {
            self.show_dice_ui = false;
        }
    }

    /// Get current rolls to display
    pub fn current_rolls(&self) -> &[(DiceRoll, String, f32)] {
        &self.pending_rolls
    }

    /// Check if dice UI should be shown
    pub fn should_show_ui(&self) -> bool {
        self.show_dice_ui
    }

    /// Get last roll result
    pub fn last_roll(&self) -> Option<&DiceRoll> {
        self.last_roll_result.as_ref()
    }

    /// Clear all pending rolls
    pub fn clear(&mut self) {
        self.pending_rolls.clear();
        self.show_dice_ui = false;
    }
}

impl Default for DiceUIResource {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::dice::DiceModifier;
    use crate::domain::{DiceType, Position3D, ResourceType};

    #[test]
    fn player_resource_creation() {
        let mut resource = PlayerResource::new();
        assert!(!resource.has_player());

        let pos = Position3D::new(0, 0, 0);
        let stats = crate::domain::PlayerStats::new(10, 10, 10, 10, 10, 10).unwrap();
        assert!(resource
            .create_player(
                "test_player_id".to_string(),
                "Test Player".to_string(),
                pos,
                stats
            )
            .is_ok());
        assert!(resource.has_player());
        assert_eq!(resource.player_position(), Some(pos));
    }

    #[test]
    fn base_resource_creation() {
        let mut resource = BaseResource::new();
        assert!(!resource.has_base());

        let pos = Position3D::new(5, 5, 0);
        assert!(resource.create_base("test_base".to_string(), pos).is_ok());
        assert!(resource.has_base());
        assert_eq!(resource.base_position(), Some(pos));
    }

    #[test]
    fn game_session_resource_lifecycle() {
        let mut resource = GameSessionResource::new();

        assert!(!resource.has_active_session());

        assert!(resource.create_session("test_session".to_string()).is_ok());
        assert!(resource.start_session().is_ok());
        assert!(resource.has_active_session());

        assert!(resource.add_experience(100).is_ok());
        assert_eq!(resource.experience_points(), 100);

        assert!(resource.end_session().is_ok());
        assert!(!resource.has_active_session());
    }

    #[test]
    fn world_boundaries_resource() {
        let resource = WorldBoundariesResource::standard();
        let center_pos = Position3D::new(0, 0, 0);
        let outside_pos = Position3D::new(1000, 1000, 1000);

        assert!(resource.contains(&center_pos));
        assert!(!resource.contains(&outside_pos));

        let clamped = resource.clamp(outside_pos);
        assert!(resource.contains(&clamped));
    }

    #[test]
    fn game_timer_resource_functionality() {
        let mut timer = GameTimerResource::new();

        assert_eq!(timer.turn(), 1);
        assert!(timer.is_player_turn());

        timer.update(1.0);
        timer.next_turn();
        assert_eq!(timer.turn(), 2);

        timer.pause();
        assert_eq!(timer.phase(), GamePhase::Paused);

        timer.resume();
        assert_eq!(timer.phase(), GamePhase::PlayerTurn);
    }

    #[test]
    fn game_stats_resource_tracking() {
        let mut stats = GameStatsResource::new();

        stats.record_quest_completion();
        stats.record_resource_gather(ResourceType::Metal, 50);
        stats.record_experience_gain(100);

        assert_eq!(stats.quests_completed, 1);
        assert_eq!(stats.resources_gathered_for_type(ResourceType::Metal), 50);
        assert_eq!(stats.experience_gained, 100);

        let dice_roll = DiceRoll::new(1, DiceType::D20, DiceModifier::none()).unwrap();
        stats.record_dice_roll(&dice_roll, 10);
        assert_eq!(stats.dice_rolls_made, 1);
        assert_eq!(stats.successful_rolls, 1);
        assert_eq!(stats.success_rate(), 100.0);

        stats.reset();
        assert_eq!(stats.quests_completed, 0);
        assert_eq!(stats.total_resources_gathered(), 0);
    }

    #[test]
    fn map_resource_functionality() {
        let mut map_resource = MapResource::new();
        assert!(!map_resource.has_map());

        // Test visible area updates
        let center = Position3D::new(10, 10, 0);
        map_resource.update_visible_area(center, 5);

        let visible_pos = Position3D::new(12, 12, 0);
        let invisible_pos = Position3D::new(20, 20, 0);

        assert!(map_resource.is_visible(visible_pos));
        assert!(!map_resource.is_visible(invisible_pos));

        // Test chunk loading
        assert!(!map_resource.is_chunk_loaded(0, 0));
        map_resource.mark_chunk_loaded(0, 0);
        assert!(map_resource.is_chunk_loaded(0, 0));
    }

    #[test]
    fn dice_ui_resource_functionality() {
        let mut dice_ui = DiceUIResource::new();
        assert!(!dice_ui.should_show_ui());

        let dice_roll = DiceRoll::new(3, DiceType::D6, DiceModifier::none()).unwrap();
        dice_ui.add_roll(dice_roll.clone(), "Movement".to_string(), 2.0);

        assert!(dice_ui.should_show_ui());
        assert_eq!(dice_ui.current_rolls().len(), 1);
        assert_eq!(dice_ui.last_roll(), Some(&dice_roll));

        dice_ui.update(3.0); // Time exceeds display time
        assert!(!dice_ui.should_show_ui());
        assert_eq!(dice_ui.current_rolls().len(), 0);
    }
}

// Backward compatibility resources (to be removed after transition)

/// Bevy resource wrapper for Score domain value object (legacy)
#[derive(Resource, Debug, Clone)]
pub struct ScoreResource {
    pub score: crate::domain::Score,
}

impl ScoreResource {
    /// Create a new score resource
    pub fn new(score: crate::domain::Score) -> Self {
        Self { score }
    }

    /// Create score resource starting at zero
    pub fn zero() -> Self {
        Self {
            score: crate::domain::Score::zero(),
        }
    }

    /// Update the score
    pub fn update_score(&mut self, new_score: crate::domain::Score) {
        self.score = new_score;
    }

    /// Add points to the current score
    pub fn add_points(&mut self, points: u32) -> Result<(), crate::domain::DomainError> {
        let new_score = self.score.add(points)?;
        self.score = new_score;
        Ok(())
    }

    /// Get current score value
    pub fn value(&self) -> u32 {
        self.score.value()
    }

    /// Get formatted score string
    pub fn formatted(&self) -> String {
        self.score.formatted()
    }
}

impl Default for ScoreResource {
    fn default() -> Self {
        Self::zero()
    }
}

/// Bevy resource for game boundaries (legacy)
#[derive(Resource, Debug, Clone)]
pub struct GameBoundariesResource {
    pub boundaries: crate::domain::WorldBoundaries,
}

impl GameBoundariesResource {
    /// Create new game boundaries resource
    pub fn new(boundaries: crate::domain::WorldBoundaries) -> Self {
        Self { boundaries }
    }

    /// Create resource with standard boundaries
    pub fn standard() -> Self {
        Self {
            boundaries: crate::domain::WorldBoundaries::standard(),
        }
    }

    /// Check if position is within boundaries
    pub fn contains(&self, position: &Position3D) -> bool {
        self.boundaries.contains(position)
    }

    /// Clamp position to boundaries
    pub fn clamp(&self, position: Position3D) -> Position3D {
        self.boundaries.clamp(position)
    }
}

impl Default for GameBoundariesResource {
    fn default() -> Self {
        Self::standard()
    }
}
