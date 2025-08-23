//! GameSession Entity - Core game state and session management
//!
//! This entity represents the overall game session, managing the game state,
//! turn progression, and high-level game mechanics for the RPG system.

use crate::domain::value_objects::{EntityId, GameTime, Position3D};
use crate::domain::{DomainError, DomainResult, GamePhase, WorldBoundaries};
use chrono::{DateTime, Utc};

/// The main game session entity
#[derive(Debug, Clone, PartialEq)]
pub struct GameSession {
    id: EntityId,
    session_name: String,
    player_id: EntityId,
    map_id: EntityId,
    current_phase: GamePhase,
    turn_number: u32,
    game_time: GameTime,
    world_boundaries: WorldBoundaries,
    difficulty_level: DifficultyLevel,
    session_settings: SessionSettings,
    statistics: GameStatistics,
    created_at: DateTime<Utc>,
    last_updated: DateTime<Utc>,
    version: u64,
}

impl GameSession {
    /// Create a new game session
    pub fn new(
        session_name: String,
        player_id: EntityId,
        map_id: EntityId,
        difficulty_level: DifficultyLevel,
        world_boundaries: WorldBoundaries,
    ) -> DomainResult<Self> {
        if session_name.is_empty() || session_name.len() > 100 {
            return Err(DomainError::ValidationError(
                "Session name must be between 1 and 100 characters".to_string(),
            ));
        }

        let now = Utc::now();
        Ok(Self {
            id: EntityId::generate(),
            session_name,
            player_id,
            map_id,
            current_phase: GamePhase::PlayerTurn,
            turn_number: 1,
            game_time: GameTime::new(0),
            world_boundaries,
            difficulty_level,
            session_settings: SessionSettings::default(),
            statistics: GameStatistics::new(),
            created_at: now,
            last_updated: now,
            version: 1,
        })
    }

    /// Get session ID
    pub fn id(&self) -> &EntityId {
        &self.id
    }

    /// Get session name
    pub fn session_name(&self) -> &str {
        &self.session_name
    }

    /// Get player ID
    pub fn player_id(&self) -> &EntityId {
        &self.player_id
    }

    /// Get map ID
    pub fn map_id(&self) -> &EntityId {
        &self.map_id
    }

    /// Get current game phase
    pub fn current_phase(&self) -> GamePhase {
        self.current_phase
    }

    /// Get current turn number
    pub fn turn_number(&self) -> u32 {
        self.turn_number
    }

    /// Get current game time
    pub fn game_time(&self) -> GameTime {
        self.game_time
    }

    /// Get world boundaries
    pub fn world_boundaries(&self) -> &WorldBoundaries {
        &self.world_boundaries
    }

    /// Get difficulty level
    pub fn difficulty_level(&self) -> DifficultyLevel {
        self.difficulty_level
    }

    /// Get session settings
    pub fn session_settings(&self) -> &SessionSettings {
        &self.session_settings
    }

    /// Get game statistics
    pub fn statistics(&self) -> &GameStatistics {
        &self.statistics
    }

    /// Backward compatibility methods for legacy code

    /// Start the game session (legacy)
    pub fn start(&mut self) -> DomainResult<()> {
        if self.current_phase == GamePhase::GameOver {
            return Err(DomainError::InvalidGameState(
                "Cannot start game that is over".to_string(),
            ));
        }
        self.current_phase = GamePhase::PlayerTurn;
        self.update_timestamp();
        Ok(())
    }

    /// End the game session (legacy)
    pub fn end(&mut self) -> DomainResult<()> {
        self.end_game()
    }

    /// Check if session is active (legacy)
    pub fn is_active(&self) -> bool {
        !matches!(self.current_phase, GamePhase::GameOver | GamePhase::Paused)
    }

    /// Get experience points from statistics (legacy)
    pub fn experience_points(&self) -> u32 {
        self.statistics.experience_earned
    }

    /// Add experience points (legacy)
    pub fn add_experience(&mut self, points: u32) -> DomainResult<()> {
        self.statistics.experience_earned =
            self.statistics.experience_earned.saturating_add(points);
        self.update_timestamp();
        Ok(())
    }

    /// Get score equivalent (legacy)
    pub fn score(&self) -> &crate::domain::Score {
        // Convert experience to score for backward compatibility
        static mut CACHED_SCORE: Option<crate::domain::Score> = None;
        unsafe {
            CACHED_SCORE = Some(
                crate::domain::Score::new(self.statistics.experience_earned)
                    .unwrap_or_else(|_| crate::domain::Score::zero()),
            );
            CACHED_SCORE.as_ref().unwrap()
        }
    }

    /// Add points to score (legacy)
    pub fn add_points(&mut self, points: u32) -> DomainResult<()> {
        self.add_experience(points)
    }

    /// Check if it's the player's turn
    pub fn is_player_turn(&self) -> bool {
        self.current_phase == GamePhase::PlayerTurn
    }

    /// Check if game is paused
    pub fn is_paused(&self) -> bool {
        self.current_phase == GamePhase::Paused
    }

    /// Check if game is over
    pub fn is_game_over(&self) -> bool {
        self.current_phase == GamePhase::GameOver
    }

    /// Advance to next game phase
    pub fn advance_phase(&mut self) -> DomainResult<GamePhase> {
        let next_phase = match self.current_phase {
            GamePhase::PlayerTurn => GamePhase::Processing,
            GamePhase::Processing => GamePhase::EventPhase,
            GamePhase::EventPhase => GamePhase::EndTurn,
            GamePhase::EndTurn => {
                self.turn_number += 1;
                self.game_time = self.game_time.add(GameTime::from_minutes(1));
                GamePhase::PlayerTurn
            }
            GamePhase::Paused => {
                return Err(DomainError::InvalidGameState(
                    "Cannot advance phase while paused".to_string(),
                ))
            }
            GamePhase::GameOver => {
                return Err(DomainError::InvalidGameState(
                    "Cannot advance phase when game is over".to_string(),
                ))
            }
        };

        self.current_phase = next_phase;
        self.update_timestamp();
        Ok(next_phase)
    }

    /// Pause the game
    pub fn pause(&mut self) -> DomainResult<()> {
        if self.current_phase == GamePhase::GameOver {
            return Err(DomainError::InvalidGameState(
                "Cannot pause game that is over".to_string(),
            ));
        }

        self.current_phase = GamePhase::Paused;
        self.update_timestamp();
        Ok(())
    }

    /// Resume the game from pause
    pub fn resume(&mut self) -> DomainResult<()> {
        if self.current_phase != GamePhase::Paused {
            return Err(DomainError::InvalidGameState(
                "Game is not paused".to_string(),
            ));
        }

        self.current_phase = GamePhase::PlayerTurn;
        self.update_timestamp();
        Ok(())
    }

    /// End the game
    pub fn end_game(&mut self) -> DomainResult<()> {
        self.current_phase = GamePhase::GameOver;
        self.statistics.game_ended_at = Some(Utc::now());
        self.update_timestamp();
        Ok(())
    }

    /// Update session settings
    pub fn update_settings(&mut self, settings: SessionSettings) -> DomainResult<()> {
        self.session_settings = settings;
        self.update_timestamp();
        Ok(())
    }

    /// Record player action in statistics
    pub fn record_action(&mut self, action_type: ActionType) {
        self.statistics.record_action(action_type);
        self.update_timestamp();
    }

    /// Add game time
    pub fn add_game_time(&mut self, additional_time: GameTime) {
        self.game_time = self.game_time.add(additional_time);
        self.update_timestamp();
    }

    /// Check if position is within world boundaries
    pub fn is_position_valid(&self, position: &Position3D) -> bool {
        self.world_boundaries.contains(position)
    }

    /// Clamp position to world boundaries
    pub fn clamp_position(&self, position: Position3D) -> Position3D {
        self.world_boundaries.clamp(position)
    }

    /// Get session duration
    pub fn session_duration(&self) -> chrono::Duration {
        Utc::now().signed_duration_since(self.created_at)
    }

    /// Get version
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Update timestamp and increment version
    fn update_timestamp(&mut self) {
        self.last_updated = Utc::now();
        self.version += 1;
    }

    /// Get session summary for display
    pub fn summary(&self) -> GameSessionSummary {
        GameSessionSummary {
            session_name: self.session_name.clone(),
            turn_number: self.turn_number,
            current_phase: self.current_phase,
            game_time: self.game_time,
            difficulty_level: self.difficulty_level,
            total_actions: self.statistics.total_actions(),
            session_duration: self.session_duration(),
        }
    }
}

/// Difficulty levels for the game
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DifficultyLevel {
    Easy,
    Normal,
    Hard,
    Expert,
    Nightmare,
}

impl DifficultyLevel {
    /// Get experience multiplier for this difficulty
    pub fn experience_multiplier(&self) -> f32 {
        match self {
            DifficultyLevel::Easy => 0.75,
            DifficultyLevel::Normal => 1.0,
            DifficultyLevel::Hard => 1.25,
            DifficultyLevel::Expert => 1.5,
            DifficultyLevel::Nightmare => 2.0,
        }
    }

    /// Get resource scarcity multiplier
    pub fn resource_scarcity_multiplier(&self) -> f32 {
        match self {
            DifficultyLevel::Easy => 1.5,
            DifficultyLevel::Normal => 1.0,
            DifficultyLevel::Hard => 0.8,
            DifficultyLevel::Expert => 0.6,
            DifficultyLevel::Nightmare => 0.4,
        }
    }

    /// Get danger level multiplier
    pub fn danger_multiplier(&self) -> f32 {
        match self {
            DifficultyLevel::Easy => 0.7,
            DifficultyLevel::Normal => 1.0,
            DifficultyLevel::Hard => 1.3,
            DifficultyLevel::Expert => 1.6,
            DifficultyLevel::Nightmare => 2.0,
        }
    }
}

impl std::fmt::Display for DifficultyLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DifficultyLevel::Easy => write!(f, "Easy"),
            DifficultyLevel::Normal => write!(f, "Normal"),
            DifficultyLevel::Hard => write!(f, "Hard"),
            DifficultyLevel::Expert => write!(f, "Expert"),
            DifficultyLevel::Nightmare => write!(f, "Nightmare"),
        }
    }
}

impl Default for DifficultyLevel {
    fn default() -> Self {
        DifficultyLevel::Normal
    }
}

/// Session settings
#[derive(Debug, Clone, PartialEq)]
pub struct SessionSettings {
    pub auto_save_enabled: bool,
    pub auto_save_interval_minutes: u32,
    pub event_frequency_multiplier: f32,
    pub resource_regeneration_multiplier: f32,
    pub permadeath_enabled: bool,
    pub fog_of_war_enabled: bool,
}

impl Default for SessionSettings {
    fn default() -> Self {
        Self {
            auto_save_enabled: true,
            auto_save_interval_minutes: 5,
            event_frequency_multiplier: 1.0,
            resource_regeneration_multiplier: 1.0,
            permadeath_enabled: false,
            fog_of_war_enabled: true,
        }
    }
}

/// Game statistics tracking
#[derive(Debug, Clone, PartialEq)]
pub struct GameStatistics {
    pub actions_taken: std::collections::HashMap<ActionType, u32>,
    pub total_distance_traveled: u32,
    pub total_resources_gathered: u32,
    pub total_experience_gained: u32,
    pub experience_earned: u32, // Backward compatibility field
    pub events_encountered: u32,
    pub quests_completed: u32,
    pub game_started_at: DateTime<Utc>,
    pub game_ended_at: Option<DateTime<Utc>>,
}

impl GameStatistics {
    /// Create new game statistics
    pub fn new() -> Self {
        Self {
            actions_taken: std::collections::HashMap::new(),
            total_distance_traveled: 0,
            total_resources_gathered: 0,
            total_experience_gained: 0,
            experience_earned: 0, // Initialize backward compatibility field
            events_encountered: 0,
            quests_completed: 0,
            game_started_at: Utc::now(),
            game_ended_at: None,
        }
    }

    /// Record an action
    pub fn record_action(&mut self, action_type: ActionType) {
        *self.actions_taken.entry(action_type).or_insert(0) += 1;
    }

    /// Get total actions taken
    pub fn total_actions(&self) -> u32 {
        self.actions_taken.values().sum()
    }

    /// Add distance traveled
    pub fn add_distance(&mut self, distance: u32) {
        self.total_distance_traveled += distance;
    }

    /// Add resources gathered
    pub fn add_resources_gathered(&mut self, amount: u32) {
        self.total_resources_gathered += amount;
    }

    /// Add experience gained
    pub fn add_experience(&mut self, experience: u32) {
        self.total_experience_gained += experience;
    }

    /// Record event encounter
    pub fn record_event(&mut self) {
        self.events_encountered += 1;
    }

    /// Record quest completion
    pub fn record_quest_completion(&mut self) {
        self.quests_completed += 1;
    }
}

impl Default for GameStatistics {
    fn default() -> Self {
        Self::new()
    }
}

/// Types of actions that can be recorded
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionType {
    Move,
    GatherResources,
    BuildStructure,
    Rest,
    Explore,
    UseItem,
    Trade,
    Combat,
    Craft,
    Research,
}

impl std::fmt::Display for ActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionType::Move => write!(f, "Move"),
            ActionType::GatherResources => write!(f, "Gather Resources"),
            ActionType::BuildStructure => write!(f, "Build Structure"),
            ActionType::Rest => write!(f, "Rest"),
            ActionType::Explore => write!(f, "Explore"),
            ActionType::UseItem => write!(f, "Use Item"),
            ActionType::Trade => write!(f, "Trade"),
            ActionType::Combat => write!(f, "Combat"),
            ActionType::Craft => write!(f, "Craft"),
            ActionType::Research => write!(f, "Research"),
        }
    }
}

/// Summary information for display
#[derive(Debug, Clone, PartialEq)]
pub struct GameSessionSummary {
    pub session_name: String,
    pub turn_number: u32,
    pub current_phase: GamePhase,
    pub game_time: GameTime,
    pub difficulty_level: DifficultyLevel,
    pub total_actions: u32,
    pub session_duration: chrono::Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_session_creation() {
        let player_id = EntityId::generate();
        let map_id = EntityId::generate();
        let boundaries = WorldBoundaries::standard();

        let session = GameSession::new(
            "Test Session".to_string(),
            player_id,
            map_id,
            DifficultyLevel::Normal,
            boundaries,
        )
        .unwrap();

        assert_eq!(session.session_name(), "Test Session");
        assert_eq!(session.turn_number(), 1);
        assert!(session.is_player_turn());
        assert!(!session.is_paused());
        assert!(!session.is_game_over());
    }

    #[test]
    fn game_phase_progression() {
        let player_id = EntityId::generate();
        let map_id = EntityId::generate();
        let boundaries = WorldBoundaries::standard();

        let mut session = GameSession::new(
            "Test".to_string(),
            player_id,
            map_id,
            DifficultyLevel::Normal,
            boundaries,
        )
        .unwrap();

        assert_eq!(session.current_phase(), GamePhase::PlayerTurn);

        let next = session.advance_phase().unwrap();
        assert_eq!(next, GamePhase::Processing);
        assert_eq!(session.current_phase(), GamePhase::Processing);

        session.advance_phase().unwrap(); // EventPhase
        session.advance_phase().unwrap(); // EndTurn
        let next = session.advance_phase().unwrap(); // Back to PlayerTurn

        assert_eq!(next, GamePhase::PlayerTurn);
        assert_eq!(session.turn_number(), 2); // Should have incremented
    }

    #[test]
    fn game_pause_resume() {
        let player_id = EntityId::generate();
        let map_id = EntityId::generate();
        let boundaries = WorldBoundaries::standard();

        let mut session = GameSession::new(
            "Test".to_string(),
            player_id,
            map_id,
            DifficultyLevel::Normal,
            boundaries,
        )
        .unwrap();

        session.pause().unwrap();
        assert!(session.is_paused());

        // Cannot advance while paused
        assert!(session.advance_phase().is_err());

        session.resume().unwrap();
        assert!(session.is_player_turn());
        assert!(!session.is_paused());
    }

    #[test]
    fn game_statistics_recording() {
        let mut stats = GameStatistics::new();
        assert_eq!(stats.total_actions(), 0);

        stats.record_action(ActionType::Move);
        stats.record_action(ActionType::Move);
        stats.record_action(ActionType::GatherResources);

        assert_eq!(stats.total_actions(), 3);
        assert_eq!(*stats.actions_taken.get(&ActionType::Move).unwrap(), 2);
        assert_eq!(
            *stats
                .actions_taken
                .get(&ActionType::GatherResources)
                .unwrap(),
            1
        );
    }

    #[test]
    fn difficulty_level_modifiers() {
        assert_eq!(DifficultyLevel::Easy.experience_multiplier(), 0.75);
        assert_eq!(DifficultyLevel::Nightmare.experience_multiplier(), 2.0);

        assert!(DifficultyLevel::Easy.resource_scarcity_multiplier() > 1.0);
        assert!(DifficultyLevel::Nightmare.resource_scarcity_multiplier() < 1.0);

        assert!(DifficultyLevel::Easy.danger_multiplier() < 1.0);
        assert!(DifficultyLevel::Nightmare.danger_multiplier() > 1.0);
    }

    #[test]
    fn world_boundaries_validation() {
        let player_id = EntityId::generate();
        let map_id = EntityId::generate();
        let boundaries = WorldBoundaries::standard();

        let session = GameSession::new(
            "Test".to_string(),
            player_id,
            map_id,
            DifficultyLevel::Normal,
            boundaries,
        )
        .unwrap();

        assert!(session.is_position_valid(&Position3D::new(0, 0, 0)));
        assert!(!session.is_position_valid(&Position3D::new(1000, 1000, 1000)));

        let clamped = session.clamp_position(Position3D::new(1000, 1000, 1000));
        assert!(session.is_position_valid(&clamped));
    }

    #[test]
    fn invalid_session_creation() {
        let player_id = EntityId::generate();
        let map_id = EntityId::generate();
        let boundaries = WorldBoundaries::standard();

        let result = GameSession::new(
            "".to_string(), // Empty name
            player_id,
            map_id,
            DifficultyLevel::Normal,
            boundaries,
        );
        assert!(result.is_err());
    }

    #[test]
    fn game_time_progression() {
        let player_id = EntityId::generate();
        let map_id = EntityId::generate();
        let boundaries = WorldBoundaries::standard();

        let mut session = GameSession::new(
            "Test".to_string(),
            player_id,
            map_id,
            DifficultyLevel::Normal,
            boundaries,
        )
        .unwrap();

        let initial_time = session.game_time();

        // Complete a full turn cycle
        session.advance_phase().unwrap(); // Processing
        session.advance_phase().unwrap(); // EventPhase
        session.advance_phase().unwrap(); // EndTurn
        session.advance_phase().unwrap(); // PlayerTurn (new turn)

        let new_time = session.game_time();
        assert!(new_time.seconds() > initial_time.seconds());
    }
}
