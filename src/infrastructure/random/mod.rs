//! Random Number Generation Infrastructure
//!
//! This module provides random number generation services for the game,
//! with different implementations for native and web platforms.

pub mod generator;

// Re-export the main generator
pub use generator::RandomNumberGenerator;

use crate::infrastructure::InfrastructureResult;

/// Trait for random number generation services
pub trait RandomService: Send + Sync {
    /// Generate random f32 between 0.0 and 1.0
    fn random_f32(&self) -> f32;

    /// Generate random f32 in range [min, max]
    fn random_range(&self, min: f32, max: f32) -> f32;

    /// Generate random position within boundaries
    fn random_position(
        &self,
        min_x: f32,
        max_x: f32,
        min_y: f32,
        max_y: f32,
    ) -> InfrastructureResult<crate::domain::Position>;

    /// Generate random velocity with magnitude
    fn random_velocity(&self, max_speed: f32) -> InfrastructureResult<crate::domain::Velocity>;
}

/// Web-compatible random number generator
#[derive(Debug)]
pub struct WebRandomGenerator {
    state: std::sync::Mutex<u32>,
}

impl Clone for WebRandomGenerator {
    fn clone(&self) -> Self {
        let state_value = *self.state.lock().unwrap();
        Self {
            state: std::sync::Mutex::new(state_value),
        }
    }
}

impl WebRandomGenerator {
    /// Create new random generator with seed
    pub fn new(seed: u32) -> Self {
        Self {
            state: std::sync::Mutex::new(seed),
        }
    }

    /// Create new random generator with default seed
    pub fn default_seed() -> Self {
        Self::new(1234567890)
    }
}

impl RandomService for WebRandomGenerator {
    fn random_f32(&self) -> f32 {
        let mut state = self.state.lock().unwrap();
        *state = state.wrapping_mul(1103515245).wrapping_add(12345);
        (*state >> 16) as f32 / 65536.0
    }

    fn random_range(&self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.random_f32()
    }

    fn random_position(
        &self,
        min_x: f32,
        max_x: f32,
        min_y: f32,
        max_y: f32,
    ) -> InfrastructureResult<crate::domain::Position> {
        let x = self.random_range(min_x, max_x);
        let y = self.random_range(min_y, max_y);

        crate::domain::Position::new(x, y)
            .map_err(|e| crate::infrastructure::InfrastructureError::RandomError(e.to_string()))
    }

    fn random_velocity(&self, max_speed: f32) -> InfrastructureResult<crate::domain::Velocity> {
        let angle = self.random_f32() * 2.0 * std::f32::consts::PI;
        let speed = self.random_f32() * max_speed;

        let dx = angle.cos() * speed;
        let dy = angle.sin() * speed;

        crate::domain::Velocity::new(dx, dy)
            .map_err(|e| crate::infrastructure::InfrastructureError::RandomError(e.to_string()))
    }
}

impl Default for WebRandomGenerator {
    fn default() -> Self {
        Self::default_seed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn web_random_generator_creation() {
        let generator = WebRandomGenerator::new(12345);
        let val = generator.random_f32();
        assert!(val >= 0.0 && val <= 1.0);
    }

    #[test]
    fn random_range_generation() {
        let generator = WebRandomGenerator::default_seed();
        let val = generator.random_range(10.0, 20.0);
        assert!(val >= 10.0 && val <= 20.0);
    }

    #[test]
    fn random_position_generation() {
        let generator = WebRandomGenerator::default_seed();
        let pos = generator
            .random_position(-100.0, 100.0, -50.0, 50.0)
            .unwrap();
        assert!(pos.x() >= -100.0 && pos.x() <= 100.0);
        assert!(pos.y() >= -50.0 && pos.y() <= 50.0);
    }

    #[test]
    fn random_velocity_generation() {
        let generator = WebRandomGenerator::default_seed();
        let vel = generator.random_velocity(10.0).unwrap();
        assert!(vel.magnitude() <= 10.0);
    }
}
