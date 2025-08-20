//! Bevy Resources - Global Game State Management
//!
//! This module contains Bevy resources that manage global game state,
//! providing shared access to domain entities and services across systems.

use crate::domain::{GameBoundaries, GameSession, Score};
use bevy::prelude::*;

/// Bevy resource wrapper for Score domain value object
#[derive(Resource, Debug, Clone)]
pub struct ScoreResource {
    pub score: Score,
}

impl ScoreResource {
    /// Create a new score resource
    pub fn new(score: Score) -> Self {
        Self { score }
    }

    /// Create score resource starting at zero
    pub fn zero() -> Self {
        Self {
            score: Score::zero(),
        }
    }

    /// Update the score
    pub fn update_score(&mut self, new_score: Score) {
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
        let session = GameSession::new(session_id)?;
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
            session.start()?;
        }
        Ok(())
    }

    /// End the current session and return final score
    pub fn end_session(&mut self) -> Result<Option<Score>, crate::domain::DomainError> {
        if let Some(session) = &mut self.session {
            session.end()?;
            let final_score = *session.score();
            self.session = None;
            Ok(Some(final_score))
        } else {
            Ok(None)
        }
    }
}

impl Default for GameSessionResource {
    fn default() -> Self {
        Self::new()
    }
}

/// Bevy resource for game boundaries
#[derive(Resource, Debug, Clone)]
pub struct GameBoundariesResource {
    pub boundaries: GameBoundaries,
}

impl GameBoundariesResource {
    /// Create new game boundaries resource
    pub fn new(boundaries: GameBoundaries) -> Self {
        Self { boundaries }
    }

    /// Create resource with standard boundaries
    pub fn standard() -> Self {
        Self {
            boundaries: GameBoundaries::standard(),
        }
    }

    /// Check if position is within boundaries
    pub fn contains(&self, position: &crate::domain::Position) -> bool {
        self.boundaries.contains(position)
    }

    /// Clamp position to boundaries
    pub fn clamp(&self, position: crate::domain::Position) -> crate::domain::Position {
        self.boundaries.clamp(position)
    }
}

impl Default for GameBoundariesResource {
    fn default() -> Self {
        Self::standard()
    }
}

/// Resource for tracking game timing
#[derive(Resource, Debug, Clone)]
pub struct GameTimerResource {
    pub game_time: f32,
    pub last_enemy_spawn: f32,
    pub paused_time: f32,
}

impl GameTimerResource {
    /// Create new game timer resource
    pub fn new() -> Self {
        Self {
            game_time: 0.0,
            last_enemy_spawn: 0.0,
            paused_time: 0.0,
        }
    }

    /// Update game time
    pub fn update(&mut self, delta_time: f32) {
        self.game_time += delta_time;
    }

    /// Record enemy spawn time
    pub fn record_enemy_spawn(&mut self) {
        self.last_enemy_spawn = self.game_time;
    }

    /// Check if enough time has passed since last enemy spawn
    pub fn should_spawn_enemy(&self) -> bool {
        self.game_time - self.last_enemy_spawn >= crate::domain::constants::ENEMY_SPAWN_INTERVAL
    }

    /// Pause timer
    pub fn pause(&mut self) {
        self.paused_time = self.game_time;
    }

    /// Resume timer
    pub fn resume(&mut self) {
        // In a real implementation, we'd calculate pause duration
        // For now, just continue from where we left off
    }

    /// Reset timer
    pub fn reset(&mut self) {
        self.game_time = 0.0;
        self.last_enemy_spawn = 0.0;
        self.paused_time = 0.0;
    }
}

impl Default for GameTimerResource {
    fn default() -> Self {
        Self::new()
    }
}

/// Resource for tracking game statistics
#[derive(Resource, Debug, Clone)]
pub struct GameStatsResource {
    pub enemies_spawned: u32,
    pub enemies_destroyed: u32,
    pub collisions: u32,
    pub game_duration: f32,
}

impl GameStatsResource {
    /// Create new game stats resource
    pub fn new() -> Self {
        Self {
            enemies_spawned: 0,
            enemies_destroyed: 0,
            collisions: 0,
            game_duration: 0.0,
        }
    }

    /// Record enemy spawn
    pub fn record_enemy_spawn(&mut self) {
        self.enemies_spawned += 1;
    }

    /// Record enemy destruction
    pub fn record_enemy_destroy(&mut self) {
        self.enemies_destroyed += 1;
    }

    /// Record collision
    pub fn record_collision(&mut self) {
        self.collisions += 1;
    }

    /// Update game duration
    pub fn update_duration(&mut self, delta_time: f32) {
        self.game_duration += delta_time;
    }

    /// Reset all statistics
    pub fn reset(&mut self) {
        self.enemies_spawned = 0;
        self.enemies_destroyed = 0;
        self.collisions = 0;
        self.game_duration = 0.0;
    }

    /// Calculate accuracy percentage
    pub fn accuracy(&self) -> f32 {
        if self.enemies_spawned == 0 {
            0.0
        } else {
            (self.enemies_destroyed as f32 / self.enemies_spawned as f32) * 100.0
        }
    }
}

impl Default for GameStatsResource {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_resource_creation() {
        let resource = ScoreResource::zero();
        assert_eq!(resource.value(), 0);
    }

    #[test]
    fn score_resource_operations() {
        let mut resource = ScoreResource::zero();

        assert!(resource.add_points(100).is_ok());
        assert_eq!(resource.value(), 100);

        let formatted = resource.formatted();
        assert_eq!(formatted, "100");
    }

    #[test]
    fn game_session_resource_lifecycle() {
        let mut resource = GameSessionResource::new();

        assert!(!resource.has_active_session());

        assert!(resource.create_session("test_session".to_string()).is_ok());
        assert!(resource.start_session().is_ok());
        assert!(resource.has_active_session());

        let final_score = resource.end_session().unwrap();
        assert!(final_score.is_some());
        assert!(!resource.has_active_session());
    }

    #[test]
    fn game_boundaries_resource() {
        let resource = GameBoundariesResource::standard();
        let center_pos = crate::domain::Position::new(0.0, 0.0).unwrap();
        let outside_pos = crate::domain::Position::new(1000.0, 1000.0).unwrap();

        assert!(resource.contains(&center_pos));
        assert!(!resource.contains(&outside_pos));

        let clamped = resource.clamp(outside_pos);
        assert!(resource.contains(&clamped));
    }

    #[test]
    fn game_timer_resource_functionality() {
        let mut timer = GameTimerResource::new();

        timer.update(1.0);
        assert_eq!(timer.game_time, 1.0);

        timer.record_enemy_spawn();
        assert_eq!(timer.last_enemy_spawn, 1.0);

        timer.update(3.0); // Total time now 4.0
        assert!(timer.should_spawn_enemy()); // 3 seconds since last spawn (> 2.0 interval)
    }

    #[test]
    fn game_stats_resource_tracking() {
        let mut stats = GameStatsResource::new();

        stats.record_enemy_spawn();
        stats.record_enemy_spawn();
        stats.record_enemy_destroy();

        assert_eq!(stats.enemies_spawned, 2);
        assert_eq!(stats.enemies_destroyed, 1);
        assert_eq!(stats.accuracy(), 50.0);

        stats.reset();
        assert_eq!(stats.enemies_spawned, 0);
        assert_eq!(stats.accuracy(), 0.0);
    }
}
