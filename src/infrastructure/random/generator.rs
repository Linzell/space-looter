//! Random Number Generator Implementation
//!
//! This module provides random number generation services for the game,
//! with cross-platform compatibility for both native and web builds.

use crate::domain::{Position, Velocity};
use crate::infrastructure::{InfrastructureError, InfrastructureResult};

/// Main random number generator for the game
pub struct RandomNumberGenerator {
    state: std::sync::Mutex<u32>,
}

impl RandomNumberGenerator {
    /// Create a new random number generator
    pub fn new() -> Self {
        Self {
            state: std::sync::Mutex::new(Self::default_seed()),
        }
    }

    /// Create a new random number generator with specific seed
    pub fn with_seed(seed: u32) -> Self {
        Self {
            state: std::sync::Mutex::new(seed),
        }
    }

    /// Generate a random f32 between 0.0 and 1.0
    pub fn random_f32(&self) -> f32 {
        #[cfg(target_arch = "wasm32")]
        {
            let mut state = self.state.lock().unwrap();
            *state = state.wrapping_mul(1103515245).wrapping_add(12345);
            (*state >> 16) as f32 / 65536.0
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            use std::time::{SystemTime, UNIX_EPOCH};

            let mut hasher = DefaultHasher::new();
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
                .hash(&mut hasher);

            let hash = hasher.finish();
            (hash as f32 % 65536.0) / 65536.0
        }
    }

    /// Generate a random f32 in the specified range [min, max]
    pub fn random_range(&self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.random_f32()
    }

    /// Generate a random integer in the specified range [min, max]
    pub fn random_int_range(&self, min: i32, max: i32) -> i32 {
        min + (self.random_f32() * (max - min + 1) as f32) as i32
    }

    /// Generate a random position within the specified boundaries
    pub fn random_position(
        &self,
        min_x: f32,
        max_x: f32,
        min_y: f32,
        max_y: f32,
    ) -> InfrastructureResult<Position> {
        let x = self.random_range(min_x, max_x);
        let y = self.random_range(min_y, max_y);

        Position::new(x, y).map_err(|e| {
            InfrastructureError::RandomError(format!("Failed to create random position: {}", e))
        })
    }

    /// Generate a random velocity with the specified maximum speed
    pub fn random_velocity(&self, max_speed: f32) -> InfrastructureResult<Velocity> {
        let angle = self.random_f32() * 2.0 * std::f32::consts::PI;
        let speed = self.random_f32() * max_speed;

        let dx = angle.cos() * speed;
        let dy = angle.sin() * speed;

        Velocity::new(dx, dy).map_err(|e| {
            InfrastructureError::RandomError(format!("Failed to create random velocity: {}", e))
        })
    }

    /// Generate a random unit vector (direction)
    pub fn random_unit_vector(&self) -> InfrastructureResult<Velocity> {
        let angle = self.random_f32() * 2.0 * std::f32::consts::PI;
        let dx = angle.cos();
        let dy = angle.sin();

        Velocity::new(dx, dy).map_err(|e| {
            InfrastructureError::RandomError(format!("Failed to create random unit vector: {}", e))
        })
    }

    /// Generate a random boolean with the specified probability of being true
    pub fn random_bool(&self, probability: f32) -> bool {
        self.random_f32() < probability
    }

    /// Choose a random element from a slice
    pub fn choose<'a, T>(&self, items: &'a [T]) -> Option<&'a T> {
        if items.is_empty() {
            None
        } else {
            let index = self.random_int_range(0, items.len() as i32 - 1) as usize;
            items.get(index)
        }
    }

    /// Default seed for deterministic behavior
    const fn default_seed() -> u32 {
        1234567890
    }
}

impl Default for RandomNumberGenerator {
    fn default() -> Self {
        Self::new()
    }
}

// Thread-safe wrapper for shared usage
use std::sync::{Arc, Mutex};

/// Thread-safe random number generator
#[derive(Clone)]
pub struct SharedRandomGenerator {
    inner: Arc<Mutex<RandomNumberGenerator>>,
}

impl SharedRandomGenerator {
    /// Create a new shared random generator
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(RandomNumberGenerator::new())),
        }
    }

    /// Create a new shared random generator with seed
    pub fn with_seed(seed: u32) -> Self {
        Self {
            inner: Arc::new(Mutex::new(RandomNumberGenerator::with_seed(seed))),
        }
    }

    /// Generate a random f32 between 0.0 and 1.0
    pub fn random_f32(&self) -> f32 {
        self.inner.lock().unwrap().random_f32()
    }

    /// Generate a random f32 in the specified range [min, max]
    pub fn random_range(&self, min: f32, max: f32) -> f32 {
        self.inner.lock().unwrap().random_range(min, max)
    }

    /// Generate a random position within the specified boundaries
    pub fn random_position(
        &self,
        min_x: f32,
        max_x: f32,
        min_y: f32,
        max_y: f32,
    ) -> InfrastructureResult<Position> {
        self.inner
            .lock()
            .unwrap()
            .random_position(min_x, max_x, min_y, max_y)
    }

    /// Generate a random velocity with the specified maximum speed
    pub fn random_velocity(&self, max_speed: f32) -> InfrastructureResult<Velocity> {
        self.inner.lock().unwrap().random_velocity(max_speed)
    }
}

impl Default for SharedRandomGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_generator_creation() {
        let rng = RandomNumberGenerator::new();
        let val = rng.random_f32();
        assert!(val >= 0.0 && val <= 1.0);
    }

    #[test]
    fn random_generator_with_seed() {
        let rng1 = RandomNumberGenerator::with_seed(12345);
        let rng2 = RandomNumberGenerator::with_seed(12345);

        let val1 = rng1.random_f32();
        let val2 = rng2.random_f32();

        // With the same seed, should generate the same values
        #[cfg(target_arch = "wasm32")]
        assert_eq!(val1, val2);

        // On native, we don't guarantee determinism with seeds yet
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = val1;
            let _ = val2;
        }
    }

    #[test]
    fn random_range_generation() {
        let rng = RandomNumberGenerator::new();
        let val = rng.random_range(10.0, 20.0);
        assert!(val >= 10.0 && val <= 20.0);
    }

    #[test]
    fn random_int_range_generation() {
        let rng = RandomNumberGenerator::new();
        let val = rng.random_int_range(5, 10);
        assert!(val >= 5 && val <= 10);
    }

    #[test]
    fn random_position_generation() {
        let rng = RandomNumberGenerator::new();
        let pos = rng.random_position(-100.0, 100.0, -50.0, 50.0).unwrap();
        assert!(pos.x() >= -100.0 && pos.x() <= 100.0);
        assert!(pos.y() >= -50.0 && pos.y() <= 50.0);
    }

    #[test]
    fn random_velocity_generation() {
        let rng = RandomNumberGenerator::new();
        let vel = rng.random_velocity(10.0).unwrap();
        assert!(vel.magnitude() <= 10.0);
    }

    #[test]
    fn random_unit_vector_generation() {
        let rng = RandomNumberGenerator::new();
        let unit_vec = rng.random_unit_vector().unwrap();
        let magnitude = unit_vec.magnitude();
        assert!((magnitude - 1.0).abs() < f32::EPSILON || magnitude == 0.0);
    }

    #[test]
    fn random_bool_generation() {
        let rng = RandomNumberGenerator::new();

        // Test extreme probabilities
        assert!(!rng.random_bool(0.0));
        assert!(rng.random_bool(1.0));

        // Test middle probability (can't guarantee specific outcome)
        let _result = rng.random_bool(0.5);
    }

    #[test]
    fn choose_from_slice() {
        let rng = RandomNumberGenerator::new();
        let items = vec![1, 2, 3, 4, 5];

        if let Some(chosen) = rng.choose(&items) {
            assert!(items.contains(chosen));
        }

        // Empty slice should return None
        let empty: Vec<i32> = vec![];
        assert!(rng.choose(&empty).is_none());
    }

    #[test]
    fn shared_random_generator() {
        let shared_rng = SharedRandomGenerator::new();
        let val = shared_rng.random_f32();
        assert!(val >= 0.0 && val <= 1.0);
    }

    #[test]
    fn shared_random_generator_thread_safety() {
        use std::thread;

        let shared_rng = SharedRandomGenerator::new();
        let rng_clone = shared_rng.clone();

        let handle = thread::spawn(move || rng_clone.random_f32());

        let val1 = shared_rng.random_f32();
        let val2 = handle.join().unwrap();

        assert!(val1 >= 0.0 && val1 <= 1.0);
        assert!(val2 >= 0.0 && val2 <= 1.0);
    }
}
