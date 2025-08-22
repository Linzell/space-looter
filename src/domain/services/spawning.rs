//! Spawning Service - Domain Logic for Enemy Spawning
//!
//! Handles enemy spawning rules, timing, and positioning logic
//! according to game business rules.

use crate::domain::{DomainError, DomainResult, Position3D, Velocity, WorldBoundaries};

/// Service for managing enemy spawning logic
pub struct SpawningService;

impl SpawningService {
    /// Create a new spawning service
    pub fn new() -> Self {
        Self
    }

    /// Calculate spawn position for enemy at edge of world
    pub fn calculate_spawn_position(
        &self,
        x_offset: f32,
        boundaries: &WorldBoundaries,
    ) -> DomainResult<Position3D> {
        let x = (x_offset.clamp(boundaries.min_x as f32, boundaries.max_x as f32)) as i32;
        let y = boundaries.max_y;
        let z = 0; // Spawn at ground level
        Ok(Position3D::new(x, y, z))
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

    /// Calculate random spawn position within boundaries
    pub fn random_spawn_position(
        &self,
        boundaries: &WorldBoundaries,
        rng: &dyn crate::infrastructure::traits::RandomService,
    ) -> Position3D {
        rng.random_position_3d(
            boundaries.min_x,
            boundaries.max_x,
            boundaries.min_y,
            boundaries.max_y,
            0,
            0, // Spawn at ground level
        )
    }

    /// Get spawn difficulty based on time elapsed
    pub fn calculate_spawn_difficulty(&self, game_time: f32) -> u8 {
        // Gradually increase difficulty over time
        let base_difficulty = 1u8;
        let time_multiplier = (game_time / 60.0) as u8; // Increase every minute
        (base_difficulty + time_multiplier).min(10)
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
    use crate::infrastructure::random::WebRandomGenerator;
    use crate::infrastructure::traits::RandomService;

    #[test]
    fn spawning_service_creation() {
        let service = SpawningService::new();
        let _ = service;
    }

    #[test]
    fn spawn_position_calculation() {
        let service = SpawningService::new();
        let boundaries = WorldBoundaries::standard();
        let position = service
            .calculate_spawn_position(100.0, &boundaries)
            .unwrap();
        assert!(boundaries.contains(&position));
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

    #[test]
    fn random_spawn_position() {
        let service = SpawningService::new();
        let boundaries = WorldBoundaries::standard();
        let rng = WebRandomGenerator::default();

        let position = service.random_spawn_position(&boundaries, &rng);
        assert!(boundaries.contains(&position));
    }

    #[test]
    fn spawn_difficulty_scaling() {
        let service = SpawningService::new();

        let early_difficulty = service.calculate_spawn_difficulty(30.0);
        let late_difficulty = service.calculate_spawn_difficulty(300.0);

        assert!(late_difficulty > early_difficulty);
        assert!(late_difficulty <= 10);
    }
}
