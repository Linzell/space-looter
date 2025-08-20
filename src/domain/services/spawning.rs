//! Spawning Service - Domain Logic for Enemy Spawning
//!
//! Handles enemy spawning rules, timing, and positioning logic
//! according to game business rules.

use crate::domain::{DomainError, DomainResult, Position, Velocity};

/// Service for managing enemy spawning logic
pub struct SpawningService;

impl SpawningService {
    /// Create a new spawning service
    pub fn new() -> Self {
        Self
    }

    /// Calculate spawn position for enemy at top of screen
    pub fn calculate_spawn_position(&self, x_offset: f32) -> DomainResult<Position> {
        let boundaries = crate::domain::GameBoundaries::standard();
        let x = x_offset.clamp(boundaries.min_x, boundaries.max_x);
        let y = boundaries.max_y;
        Position::new(x, y)
    }

    /// Calculate spawn velocity for downward movement
    pub fn calculate_spawn_velocity(&self, speed: f32) -> DomainResult<Velocity> {
        if speed <= 0.0 {
            return Err(DomainError::InvalidVelocity(0.0, -speed));
        }
        Velocity::new(0.0, -speed)
    }

    /// Check if it's time to spawn a new enemy
    pub fn should_spawn(&self, elapsed_time: f32, last_spawn_time: f32) -> bool {
        elapsed_time - last_spawn_time >= crate::domain::constants::ENEMY_SPAWN_INTERVAL
    }
}

impl Default for SpawningService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawning_service_creation() {
        let service = SpawningService::new();
        let _ = service;
    }

    #[test]
    fn spawn_position_calculation() {
        let service = SpawningService::new();
        let position = service.calculate_spawn_position(100.0).unwrap();
        assert_eq!(position.x(), 100.0);
    }

    #[test]
    fn spawn_velocity_calculation() {
        let service = SpawningService::new();
        let velocity = service.calculate_spawn_velocity(50.0).unwrap();
        assert_eq!(velocity.dy(), -50.0);
    }

    #[test]
    fn spawn_timing() {
        let service = SpawningService::new();
        assert!(service.should_spawn(5.0, 0.0));
        assert!(!service.should_spawn(1.0, 0.0));
    }
}
