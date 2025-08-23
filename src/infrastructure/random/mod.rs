//! Random Number Generation Infrastructure
//!
//! This module provides random number generation services for the RPG game,
//! with support for dice rolling and 3D coordinate generation.

pub mod generator;

// Re-export the main generator
pub use generator::RandomNumberGenerator;

use crate::domain::{DiceRoll, DiceType, Position3D, ResourceType, TerrainType};
use crate::infrastructure::traits::{MouseButton, RandomService};
use crate::infrastructure::InfrastructureResult;

/// Web-compatible random number generator for RPG mechanics
#[derive(Debug)]
pub struct WebRandomGenerator {
    state: std::sync::Mutex<u64>,
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
    pub fn new(seed: u64) -> Self {
        Self {
            state: std::sync::Mutex::new(seed),
        }
    }

    /// Create new random generator with default seed
    pub fn default_seed() -> Self {
        Self::new(1234567890)
    }

    /// Create new random generator with time-based seed
    #[cfg(not(target_arch = "wasm32"))]
    pub fn time_seed() -> Self {
        use crate::infrastructure::time::TimeService;
        let seed = TimeService::now_millis().unwrap_or(0) * 1_000_000; // Convert millis to nanos
        Self::new(seed)
    }

    /// Create new random generator with web-compatible time seed
    #[cfg(target_arch = "wasm32")]
    pub fn time_seed() -> Self {
        // Use a simple time-based seed for web
        let seed = js_sys::Date::now() as u64;
        Self::new(seed)
    }

    /// Linear congruential generator (LCG) implementation
    fn next_u64(&self) -> u64 {
        let mut state = self.state.lock().unwrap();
        // Constants from Numerical Recipes
        *state = state.wrapping_mul(1664525).wrapping_add(1013904223);
        *state
    }
}

impl RandomService for WebRandomGenerator {
    fn random_f32(&self) -> f32 {
        let val = self.next_u64();
        (val as f32) / (u64::MAX as f32)
    }

    fn random_range(&self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.random_f32()
    }

    fn random_range_i32(&self, min: i32, max: i32) -> i32 {
        if min >= max {
            return min;
        }
        let range = (max - min + 1) as u64;
        let val = self.next_u64() % range;
        min + val as i32
    }

    fn random_position_3d(
        &self,
        min_x: i32,
        max_x: i32,
        min_y: i32,
        max_y: i32,
        min_z: i32,
        max_z: i32,
    ) -> Position3D {
        let x = self.random_range_i32(min_x, max_x);
        let y = self.random_range_i32(min_y, max_y);
        let z = self.random_range_i32(min_z, max_z);
        Position3D::new(x, y, z)
    }

    fn roll_dice(&self, dice_type: DiceType, count: u8) -> DiceRoll {
        if count == 0 {
            return DiceRoll::from_rolls(dice_type, vec![]).unwrap_or_default();
        }

        let sides = dice_type.sides();
        let mut rolls = Vec::with_capacity(count as usize);

        for _ in 0..count {
            let roll = self.random_range_i32(1, sides as i32) as u8;
            rolls.push(roll);
        }

        DiceRoll::from_rolls(dice_type, rolls).unwrap_or_default()
    }

    fn random_resource_type(&self) -> ResourceType {
        let types = [
            ResourceType::Metal,
            ResourceType::Energy,
            ResourceType::Food,
            ResourceType::Technology,
            ResourceType::ExoticMatter,
            ResourceType::Alloys,
            ResourceType::Data,
            ResourceType::Organics,
        ];
        let index = self.random_range_i32(0, types.len() as i32 - 1) as usize;
        types[index]
    }

    fn random_terrain_type(&self) -> TerrainType {
        let types = [
            TerrainType::Plains,
            TerrainType::Forest,
            TerrainType::Mountains,
            TerrainType::Desert,
            TerrainType::Ocean,
            TerrainType::Swamp,
            TerrainType::Constructed,
        ];
        let index = self.random_range_i32(0, types.len() as i32 - 1) as usize;
        types[index]
    }

    fn random_bool(&self, probability: f32) -> bool {
        self.random_f32() < probability.clamp(0.0, 1.0)
    }
}

impl Default for WebRandomGenerator {
    fn default() -> Self {
        Self::time_seed()
    }
}

/// Native random number generator using system RNG
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
pub struct NativeRandomGenerator {
    rng: std::sync::Mutex<fastrand::Rng>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Clone for NativeRandomGenerator {
    fn clone(&self) -> Self {
        let seed = self.rng.lock().unwrap().get_seed();
        Self::new(seed)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl NativeRandomGenerator {
    /// Create new random generator with seed
    pub fn new(seed: u64) -> Self {
        Self {
            rng: std::sync::Mutex::new(fastrand::Rng::with_seed(seed)),
        }
    }

    /// Create new random generator with system entropy
    pub fn system_seed() -> Self {
        Self {
            rng: std::sync::Mutex::new(fastrand::Rng::new()),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl RandomService for NativeRandomGenerator {
    fn random_f32(&self) -> f32 {
        self.rng.lock().unwrap().f32()
    }

    fn random_range(&self, min: f32, max: f32) -> f32 {
        self.rng.lock().unwrap().f32() * (max - min) + min
    }

    fn random_range_i32(&self, min: i32, max: i32) -> i32 {
        if min >= max {
            return min;
        }
        self.rng.lock().unwrap().i32(min..=max)
    }

    fn random_position_3d(
        &self,
        min_x: i32,
        max_x: i32,
        min_y: i32,
        max_y: i32,
        min_z: i32,
        max_z: i32,
    ) -> Position3D {
        let mut rng = self.rng.lock().unwrap();
        let x = if min_x >= max_x {
            min_x
        } else {
            rng.i32(min_x..=max_x)
        };
        let y = if min_y >= max_y {
            min_y
        } else {
            rng.i32(min_y..=max_y)
        };
        let z = if min_z >= max_z {
            min_z
        } else {
            rng.i32(min_z..=max_z)
        };
        Position3D::new(x, y, z)
    }

    fn roll_dice(&self, dice_type: DiceType, count: u8) -> DiceRoll {
        if count == 0 {
            return DiceRoll::from_rolls(dice_type, vec![]).unwrap_or_default();
        }

        let sides = dice_type.sides();
        let mut rolls = Vec::with_capacity(count as usize);
        let mut rng = self.rng.lock().unwrap();

        for _ in 0..count {
            let roll = rng.u8(1..=sides);
            rolls.push(roll);
        }

        DiceRoll::from_rolls(dice_type, rolls).unwrap_or_default()
    }

    fn random_resource_type(&self) -> ResourceType {
        let types = [
            ResourceType::Metal,
            ResourceType::Energy,
            ResourceType::Food,
            ResourceType::Technology,
            ResourceType::ExoticMatter,
            ResourceType::Alloys,
            ResourceType::Data,
            ResourceType::Organics,
        ];
        let index = self.rng.lock().unwrap().usize(0..types.len());
        types[index]
    }

    fn random_terrain_type(&self) -> TerrainType {
        let types = [
            TerrainType::Plains,
            TerrainType::Forest,
            TerrainType::Mountains,
            TerrainType::Desert,
            TerrainType::Tundra,
            TerrainType::Volcanic,
            TerrainType::Anomaly,
            TerrainType::Ocean,
            TerrainType::Swamp,
            TerrainType::Constructed,
            TerrainType::Cave,
            TerrainType::Crystal,
        ];
        let index = self.random_range_i32(0, types.len() as i32 - 1) as usize;
        types[index]
    }

    fn random_bool(&self, probability: f32) -> bool {
        self.rng.lock().unwrap().f32() < probability.clamp(0.0, 1.0)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for NativeRandomGenerator {
    fn default() -> Self {
        Self::system_seed()
    }
}

/// Convenience function to create the appropriate random generator for the platform
pub fn create_random_generator() -> Box<dyn RandomService> {
    #[cfg(target_arch = "wasm32")]
    {
        Box::new(WebRandomGenerator::default())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Box::new(NativeRandomGenerator::default())
    }
}

/// Convenience function to create a seeded random generator
pub fn create_seeded_generator(seed: u64) -> Box<dyn RandomService> {
    #[cfg(target_arch = "wasm32")]
    {
        Box::new(WebRandomGenerator::new(seed))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Box::new(NativeRandomGenerator::new(seed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{DiceType, Position3D, ResourceType, TerrainType};

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
    fn random_range_i32_generation() {
        let generator = WebRandomGenerator::default_seed();
        let val = generator.random_range_i32(5, 15);
        assert!(val >= 5 && val <= 15);
    }

    #[test]
    fn random_position_3d_generation() {
        let generator = WebRandomGenerator::default_seed();
        let pos = generator.random_position_3d(-10, 10, -5, 5, 0, 3);
        assert!(pos.x >= -10 && pos.x <= 10);
        assert!(pos.y >= -5 && pos.y <= 5);
        assert!(pos.z >= 0 && pos.z <= 3);
    }

    #[test]
    fn dice_roll_generation() {
        let generator = WebRandomGenerator::default_seed();
        let roll = generator.roll_dice(DiceType::D6, 2);
        assert_eq!(roll.dice_type, DiceType::D6);
        assert_eq!(roll.rolls().len(), 2);
        for roll_val in &roll.rolls() {
            assert!(*roll_val >= 1 && *roll_val <= 6);
        }
    }

    #[test]
    fn random_resource_type_generation() {
        let generator = WebRandomGenerator::default_seed();
        let resource = generator.random_resource_type();
        // Should be one of the valid resource types
        match resource {
            ResourceType::Metal
            | ResourceType::Energy
            | ResourceType::Food
            | ResourceType::Technology
            | ResourceType::ExoticMatter
            | ResourceType::Alloys
            | ResourceType::Data
            | ResourceType::Organics => {}
        }
    }

    #[test]
    fn random_terrain_type_generation() {
        let generator = WebRandomGenerator::default_seed();
        let terrain = generator.random_terrain_type();
        // Should be one of the valid terrain types
        match terrain {
            TerrainType::Plains
            | TerrainType::Forest
            | TerrainType::Mountains
            | TerrainType::Desert
            | TerrainType::Tundra
            | TerrainType::Volcanic
            | TerrainType::Anomaly
            | TerrainType::Ocean
            | TerrainType::Swamp
            | TerrainType::Constructed
            | TerrainType::Cave
            | TerrainType::Crystal => {}
        }
    }

    #[test]
    fn random_bool_generation() {
        let generator = WebRandomGenerator::default_seed();

        // Test with 0.0 probability (should always be false)
        for _ in 0..10 {
            assert!(!generator.random_bool(0.0));
        }

        // Test with 1.0 probability (should always be true)
        for _ in 0..10 {
            assert!(generator.random_bool(1.0));
        }
    }

    #[test]
    fn edge_case_random_range_i32() {
        let generator = WebRandomGenerator::default_seed();

        // Same min and max should return that value
        let val = generator.random_range_i32(5, 5);
        assert_eq!(val, 5);

        // Min > max should return min
        let val = generator.random_range_i32(10, 5);
        assert_eq!(val, 10);
    }

    #[test]
    fn zero_dice_count() {
        let generator = WebRandomGenerator::default_seed();
        let roll = generator.roll_dice(DiceType::D20, 0);
        assert_eq!(roll.rolls().len(), 0);
        assert_eq!(roll.total(), 0);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_random_generator() {
        let generator = NativeRandomGenerator::system_seed();
        let val = generator.random_f32();
        assert!(val >= 0.0 && val <= 1.0);

        let roll = generator.roll_dice(DiceType::D20, 1);
        assert_eq!(roll.rolls().len(), 1);
        assert!(roll.rolls()[0] >= 1 && roll.rolls()[0] <= 20);
    }

    #[test]
    fn convenience_functions() {
        let _gen1 = create_random_generator();
        let _gen2 = create_seeded_generator(12345);
    }
}
