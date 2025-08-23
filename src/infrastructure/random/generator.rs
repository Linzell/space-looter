//! Random Number Generator Implementation for 3D Isometric RPG
//!
//! This module provides random number generation services optimized for RPG mechanics,
//! with cross-platform compatibility for both native and web builds.

use crate::domain::{DiceRoll, DiceType, Position3D, ResourceType, TerrainType};
use crate::infrastructure::traits::RandomService;
use crate::infrastructure::{InfrastructureError, InfrastructureResult};

/// Main random number generator for the game
pub struct RandomNumberGenerator {
    state: std::sync::Mutex<u64>,
}

impl RandomNumberGenerator {
    /// Create a new random number generator
    pub fn new() -> Self {
        Self {
            state: std::sync::Mutex::new(Self::default_seed()),
        }
    }

    /// Create a new random number generator with specific seed
    pub fn with_seed(seed: u64) -> Self {
        Self {
            state: std::sync::Mutex::new(seed),
        }
    }

    /// Get default seed value
    fn default_seed() -> u64 {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::infrastructure::time::TimeService;
            TimeService::now_millis().unwrap_or(0) * 1_000_000 // Convert millis to nanos
        }
        #[cfg(target_arch = "wasm32")]
        {
            1234567890u64
        }
    }

    /// Linear congruential generator implementation
    fn next_u64(&self) -> u64 {
        let mut state = self.state.lock().unwrap();
        // Using constants from Numerical Recipes
        *state = state.wrapping_mul(1664525).wrapping_add(1013904223);
        *state
    }
}

impl RandomService for RandomNumberGenerator {
    fn random_f32(&self) -> f32 {
        let val = self.next_u64();
        (val as f32) / (u64::MAX as f32)
    }

    fn random_range(&self, min: f32, max: f32) -> f32 {
        if min >= max {
            return min;
        }
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

impl Clone for RandomNumberGenerator {
    fn clone(&self) -> Self {
        let seed = *self.state.lock().unwrap();
        Self::with_seed(seed)
    }
}

impl Default for RandomNumberGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for common random operations
impl RandomNumberGenerator {
    /// Generate a random position within world boundaries
    pub fn random_world_position(&self, boundaries: &crate::domain::WorldBoundaries) -> Position3D {
        self.random_position_3d(
            boundaries.min_x,
            boundaries.max_x,
            boundaries.min_y,
            boundaries.max_y,
            boundaries.min_z,
            boundaries.max_z,
        )
    }

    /// Generate random position around a center point
    pub fn random_position_around(&self, center: Position3D, radius: i32) -> Position3D {
        let x = self.random_range_i32(center.x - radius, center.x + radius);
        let y = self.random_range_i32(center.y - radius, center.y + radius);
        let z = self.random_range_i32((center.z - 1).max(0), center.z + 2);
        Position3D::new(x, y, z)
    }

    /// Roll dice with advantage (roll twice, take higher)
    pub fn roll_with_advantage(&self, dice_type: DiceType) -> DiceRoll {
        let roll1 = self.roll_dice(dice_type, 1);
        let roll2 = self.roll_dice(dice_type, 1);

        if roll1.total() >= roll2.total() {
            roll1
        } else {
            roll2
        }
    }

    /// Roll dice with disadvantage (roll twice, take lower)
    pub fn roll_with_disadvantage(&self, dice_type: DiceType) -> DiceRoll {
        let roll1 = self.roll_dice(dice_type, 1);
        let roll2 = self.roll_dice(dice_type, 1);

        if roll1.total() <= roll2.total() {
            roll1
        } else {
            roll2
        }
    }

    /// Generate a random element from a slice
    pub fn choose<T: Clone>(&self, items: &[T]) -> Option<T> {
        if items.is_empty() {
            return None;
        }
        let index = self.random_range_i32(0, items.len() as i32 - 1) as usize;
        Some(items[index].clone())
    }

    /// Shuffle a vector in place
    pub fn shuffle<T>(&self, items: &mut Vec<T>) {
        for i in (1..items.len()).rev() {
            let j = self.random_range_i32(0, i as i32) as usize;
            items.swap(i, j);
        }
    }

    /// Generate weighted random choice
    pub fn weighted_choice<T: Clone>(&self, items: &[(T, f32)]) -> Option<T> {
        if items.is_empty() {
            return None;
        }

        let total_weight: f32 = items.iter().map(|(_, weight)| weight).sum();
        if total_weight <= 0.0 {
            return None;
        }

        let mut random_value = self.random_f32() * total_weight;

        for (item, weight) in items {
            if random_value <= *weight {
                return Some(item.clone());
            }
            random_value -= weight;
        }

        // Fallback to last item in case of floating point precision issues
        items.last().map(|(item, _)| item.clone())
    }

    /// Generate random resource amounts for gathering
    pub fn random_resource_amount(&self, base_amount: i32, variance: f32) -> i32 {
        let min_amount = ((base_amount as f32) * (1.0 - variance)) as i32;
        let max_amount = ((base_amount as f32) * (1.0 + variance)) as i32;
        self.random_range_i32(min_amount.max(1), max_amount)
    }

    /// Check for random event trigger
    pub fn check_event_trigger(&self, probability: f32) -> bool {
        self.random_bool(probability)
    }

    /// Generate random exploration encounter
    pub fn generate_encounter_type(&self) -> EncounterType {
        let encounters = [
            (EncounterType::ResourceNode, 0.4),
            (EncounterType::RandomEvent, 0.3),
            (EncounterType::QuestGiver, 0.15),
            (EncounterType::Treasure, 0.1),
            (EncounterType::Danger, 0.05),
        ];

        self.weighted_choice(&encounters)
            .unwrap_or(EncounterType::ResourceNode)
    }
}

/// Types of encounters that can be generated during exploration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncounterType {
    /// A resource gathering location
    ResourceNode,
    /// A random event trigger
    RandomEvent,
    /// An NPC with a quest
    QuestGiver,
    /// A treasure chest or valuable item
    Treasure,
    /// A dangerous situation requiring skill checks
    Danger,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_generator_creation() {
        let generator = RandomNumberGenerator::new();
        let val = generator.random_f32();
        assert!(val >= 0.0 && val <= 1.0);
    }

    #[test]
    fn seeded_generator_consistency() {
        let generator1 = RandomNumberGenerator::with_seed(12345);
        let generator2 = RandomNumberGenerator::with_seed(12345);

        let val1 = generator1.random_f32();
        let val2 = generator2.random_f32();

        // First values should be the same with same seed
        assert_eq!(val1, val2);
    }

    #[test]
    fn random_range_generation() {
        let generator = RandomNumberGenerator::new();

        for _ in 0..100 {
            let val = generator.random_range(10.0, 20.0);
            assert!(val >= 10.0 && val <= 20.0);
        }
    }

    #[test]
    fn random_range_i32_generation() {
        let generator = RandomNumberGenerator::new();

        for _ in 0..100 {
            let val = generator.random_range_i32(5, 15);
            assert!(val >= 5 && val <= 15);
        }
    }

    #[test]
    fn random_position_3d_generation() {
        let generator = RandomNumberGenerator::new();
        let pos = generator.random_position_3d(-10, 10, -5, 5, 0, 3);

        assert!(pos.x >= -10 && pos.x <= 10);
        assert!(pos.y >= -5 && pos.y <= 5);
        assert!(pos.z >= 0 && pos.z <= 3);
    }

    #[test]
    fn dice_roll_generation() {
        let generator = RandomNumberGenerator::new();
        let roll = generator.roll_dice(DiceType::D6, 2);

        assert_eq!(roll.dice_type, DiceType::D6);
        assert_eq!(roll.rolls().len(), 2);

        for roll_val in &roll.rolls() {
            assert!(*roll_val >= 1 && *roll_val <= 6);
        }
    }

    #[test]
    fn advantage_disadvantage_rolls() {
        let generator = RandomNumberGenerator::new();

        // Test that advantage and disadvantage work
        let _advantage_roll = generator.roll_with_advantage(DiceType::D20);
        let _disadvantage_roll = generator.roll_with_disadvantage(DiceType::D20);

        // We can't test that advantage is actually higher without many samples
        // but we can test that they return valid rolls
    }

    #[test]
    fn choose_from_slice() {
        let generator = RandomNumberGenerator::new();
        let items = vec!["apple", "banana", "cherry"];

        let chosen = generator.choose(&items);
        assert!(chosen.is_some());
        assert!(items.contains(&chosen.unwrap()));

        // Empty slice should return None
        let empty: Vec<&str> = vec![];
        assert!(generator.choose(&empty).is_none());
    }

    #[test]
    fn weighted_choice_generation() {
        let generator = RandomNumberGenerator::new();
        let items = vec![("common", 0.7), ("uncommon", 0.2), ("rare", 0.1)];

        let chosen = generator.weighted_choice(&items);
        assert!(chosen.is_some());

        let result = chosen.unwrap();
        assert!(items.iter().any(|(item, _)| *item == result));
    }

    #[test]
    fn shuffle_vector() {
        let generator = RandomNumberGenerator::new();
        let mut items = vec![1, 2, 3, 4, 5];
        let original = items.clone();

        generator.shuffle(&mut items);

        // Items should have same elements but likely different order
        assert_eq!(items.len(), original.len());
        for item in &original {
            assert!(items.contains(item));
        }
    }

    #[test]
    fn resource_type_generation() {
        let generator = RandomNumberGenerator::new();
        let resource_type = generator.random_resource_type();

        // Should be one of the valid resource types
        match resource_type {
            ResourceType::Metal
            | ResourceType::Energy
            | ResourceType::Food
            | ResourceType::Technology => {}
            _ => panic!("Unexpected resource type"),
        }
    }

    #[test]
    fn terrain_type_generation() {
        let generator = RandomNumberGenerator::new();
        let terrain_type = generator.random_terrain_type();

        // Should be one of the valid terrain types
        match terrain_type {
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
    fn encounter_type_generation() {
        let generator = RandomNumberGenerator::new();
        let encounter = generator.generate_encounter_type();

        // Should be one of the valid encounter types
        match encounter {
            EncounterType::ResourceNode
            | EncounterType::RandomEvent
            | EncounterType::QuestGiver
            | EncounterType::Treasure
            | EncounterType::Danger => {}
        }
    }

    #[test]
    fn edge_cases() {
        let generator = RandomNumberGenerator::new();

        // Same min and max should return that value
        assert_eq!(generator.random_range_i32(5, 5), 5);

        // Min > max should return min
        assert_eq!(generator.random_range_i32(10, 5), 10);

        // Zero dice count should return empty roll
        let roll = generator.roll_dice(DiceType::D6, 0);
        assert_eq!(roll.rolls().len(), 0);
        assert_eq!(roll.total(), 0);

        // Random bool with 0.0 probability should always be false
        assert!(!generator.random_bool(0.0));

        // Random bool with 1.0 probability should always be true
        assert!(generator.random_bool(1.0));
    }
}
