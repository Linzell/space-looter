//! Value Objects - Immutable Domain Concepts
//!
//! This module contains immutable value objects that represent core domain concepts
//! in the 3D isometric RPG system. All value objects are immutable and contain
//! validation logic to ensure domain invariants.

use crate::domain::{DomainError, DomainResult};
use serde::{Deserialize, Serialize};
use std::fmt;

// Module declarations
pub mod dice;
pub mod position;
pub mod resources;
pub mod terrain;

// Re-export all value objects for convenience
pub use dice::{DiceModifier, DiceResult, DiceRoll, DiceType};
pub use position::{Position3D, TileCoordinate};
pub use resources::{ResourceAmount, ResourceCollection, ResourceType};
pub use terrain::TerrainType;

/// Unique identifier for game entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub u64);

impl EntityId {
    /// Create a new entity ID
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Generate a new unique ID based on current timestamp
    pub fn generate() -> Self {
        use crate::infrastructure::time::TimeService;
        let timestamp = TimeService::now_millis().unwrap_or(0) as u128 * 1_000_000; // Convert millis to nanos
        Self(timestamp as u64)
    }

    /// Get the raw ID value
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ID({})", self.0)
    }
}

/// Player experience points and level calculation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Experience {
    points: u32,
    level: u32,
}

impl Experience {
    /// Create new experience with validation
    pub fn new(points: u32) -> DomainResult<Self> {
        let level = Self::calculate_level(points);
        if level > crate::domain::constants::MAX_PLAYER_LEVEL {
            return Err(DomainError::ValidationError(
                "Experience points exceed maximum level".to_string(),
            ));
        }
        Ok(Self { points, level })
    }

    /// Create experience for a specific level
    pub fn for_level(level: u32) -> DomainResult<Self> {
        if level > crate::domain::constants::MAX_PLAYER_LEVEL {
            return Err(DomainError::ValidationError(format!(
                "Level {} exceeds maximum level {}",
                level,
                crate::domain::constants::MAX_PLAYER_LEVEL
            )));
        }
        let points = Self::points_required_for_level(level);
        Ok(Self { points, level })
    }

    /// Get current experience points
    pub fn points(&self) -> u32 {
        self.points
    }

    /// Get current level
    pub fn level(&self) -> u32 {
        self.level
    }

    /// Add experience points and recalculate level
    pub fn add_points(&self, additional_points: u32) -> DomainResult<Self> {
        let new_points = self.points.saturating_add(additional_points);
        Self::new(new_points)
    }

    /// Calculate level based on experience points
    fn calculate_level(points: u32) -> u32 {
        let mut level = 1;
        let mut required_points = crate::domain::constants::BASE_EXPERIENCE_REQUIREMENT;
        let multiplier = crate::domain::constants::EXPERIENCE_MULTIPLIER;

        while points >= required_points && level < crate::domain::constants::MAX_PLAYER_LEVEL {
            level += 1;
            required_points = (required_points as f32 * multiplier) as u32;
        }

        level
    }

    /// Calculate points required for a specific level
    fn points_required_for_level(target_level: u32) -> u32 {
        if target_level <= 1 {
            return 0;
        }

        let mut total_points = 0;
        let mut level_requirement = crate::domain::constants::BASE_EXPERIENCE_REQUIREMENT;
        let multiplier = crate::domain::constants::EXPERIENCE_MULTIPLIER;

        for _ in 2..=target_level {
            total_points += level_requirement;
            level_requirement = (level_requirement as f32 * multiplier) as u32;
        }

        total_points
    }

    /// Points needed for next level
    pub fn points_to_next_level(&self) -> u32 {
        if self.level >= crate::domain::constants::MAX_PLAYER_LEVEL {
            return 0;
        }

        let next_level_points = Self::points_required_for_level(self.level + 1);
        next_level_points.saturating_sub(self.points)
    }

    /// Check if level up is possible
    pub fn can_level_up(&self) -> bool {
        self.level < crate::domain::constants::MAX_PLAYER_LEVEL
            && self.points >= Self::points_required_for_level(self.level + 1)
    }
}

/// Player statistics that affect dice rolls and actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerStats {
    pub strength: u8,     // Affects combat and heavy lifting
    pub dexterity: u8,    // Affects movement and precision
    pub intelligence: u8, // Affects technology and problem solving
    pub charisma: u8,     // Affects trading and negotiations
    pub luck: u8,         // Affects random events and critical hits
    pub endurance: u8,    // Affects action points and health
}

impl PlayerStats {
    /// Create new player stats with validation
    pub fn new(
        strength: u8,
        dexterity: u8,
        intelligence: u8,
        charisma: u8,
        luck: u8,
        endurance: u8,
    ) -> DomainResult<Self> {
        // Validate stat ranges (1-20 for RPG-like stats)
        for &stat in &[strength, dexterity, intelligence, charisma, luck, endurance] {
            if stat == 0 || stat > 20 {
                return Err(DomainError::InvalidPlayerStats(
                    "All stats must be between 1 and 20".to_string(),
                ));
            }
        }

        Ok(Self {
            strength,
            dexterity,
            intelligence,
            charisma,
            luck,
            endurance,
        })
    }

    /// Create default starting stats
    pub fn starting_stats() -> Self {
        Self {
            strength: 10,
            dexterity: 10,
            intelligence: 10,
            charisma: 10,
            luck: 10,
            endurance: 10,
        }
    }

    /// Get modifier for dice rolls based on stat
    pub fn get_modifier(&self, stat_type: StatType) -> i8 {
        let stat_value = match stat_type {
            StatType::Strength => self.strength,
            StatType::Dexterity => self.dexterity,
            StatType::Intelligence => self.intelligence,
            StatType::Charisma => self.charisma,
            StatType::Luck => self.luck,
            StatType::Endurance => self.endurance,
        };

        // Convert stat to D&D-like modifier (-5 to +5)
        ((stat_value as i8) - 10) / 2
    }

    /// Increase a stat by one point (with level-up restrictions)
    pub fn increase_stat(&self, stat_type: StatType) -> DomainResult<Self> {
        let mut new_stats = *self;
        let current_value = match stat_type {
            StatType::Strength => self.strength,
            StatType::Dexterity => self.dexterity,
            StatType::Intelligence => self.intelligence,
            StatType::Charisma => self.charisma,
            StatType::Luck => self.luck,
            StatType::Endurance => self.endurance,
        };

        if current_value >= 20 {
            return Err(DomainError::InvalidPlayerStats(
                "Cannot increase stat above 20".to_string(),
            ));
        }

        match stat_type {
            StatType::Strength => new_stats.strength += 1,
            StatType::Dexterity => new_stats.dexterity += 1,
            StatType::Intelligence => new_stats.intelligence += 1,
            StatType::Charisma => new_stats.charisma += 1,
            StatType::Luck => new_stats.luck += 1,
            StatType::Endurance => new_stats.endurance += 1,
        }

        Ok(new_stats)
    }
}

/// Types of player statistics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatType {
    Strength,
    Dexterity,
    Intelligence,
    Charisma,
    Luck,
    Endurance,
}

impl fmt::Display for StatType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StatType::Strength => write!(f, "Strength"),
            StatType::Dexterity => write!(f, "Dexterity"),
            StatType::Intelligence => write!(f, "Intelligence"),
            StatType::Charisma => write!(f, "Charisma"),
            StatType::Luck => write!(f, "Luck"),
            StatType::Endurance => write!(f, "Endurance"),
        }
    }
}

/// Time-based value object for tracking game time and durations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GameTime {
    seconds: u32,
}

impl GameTime {
    /// Create new game time
    pub fn new(seconds: u32) -> Self {
        Self { seconds }
    }

    /// Create game time from minutes
    pub fn from_minutes(minutes: u32) -> Self {
        Self {
            seconds: minutes * 60,
        }
    }

    /// Create game time from hours
    pub fn from_hours(hours: u32) -> Self {
        Self {
            seconds: hours * 3600,
        }
    }

    /// Get seconds
    pub fn seconds(&self) -> u32 {
        self.seconds
    }

    /// Get minutes
    pub fn minutes(&self) -> u32 {
        self.seconds / 60
    }

    /// Get hours
    pub fn hours(&self) -> u32 {
        self.seconds / 3600
    }

    /// Add time duration
    pub fn add(&self, duration: GameTime) -> Self {
        Self {
            seconds: self.seconds.saturating_add(duration.seconds),
        }
    }

    /// Subtract time duration
    pub fn subtract(&self, duration: GameTime) -> Self {
        Self {
            seconds: self.seconds.saturating_sub(duration.seconds),
        }
    }

    /// Check if enough time has passed
    pub fn has_elapsed(&self, since: GameTime) -> bool {
        self.seconds >= since.seconds
    }

    /// Advance time by seconds
    pub fn advance_by_seconds(&self, seconds: u32) -> Self {
        Self {
            seconds: self.seconds.saturating_add(seconds),
        }
    }

    /// Advance time by turns (each turn = 1 minute)
    pub fn advance_by_turns(&self, turns: u32) -> Self {
        Self {
            seconds: self.seconds.saturating_add(turns * 60),
        }
    }

    /// Create start of game time
    pub fn start_of_game() -> Self {
        Self::new(0)
    }
}

impl Default for GameTime {
    fn default() -> Self {
        Self::new(0)
    }
}

impl fmt::Display for GameTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hours = self.hours();
        let minutes = (self.seconds % 3600) / 60;
        let seconds = self.seconds % 60;

        if hours > 0 {
            write!(f, "{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            write!(f, "{}m {}s", minutes, seconds)
        } else {
            write!(f, "{}s", seconds)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_id_generation() {
        let id1 = EntityId::generate();
        let id2 = EntityId::generate();
        assert_ne!(id1, id2);
    }

    #[test]
    fn experience_level_calculation() {
        let exp = Experience::new(0).unwrap();
        assert_eq!(exp.level(), 1);

        let exp = Experience::new(100).unwrap();
        assert_eq!(exp.level(), 2);
    }

    #[test]
    fn player_stats_validation() {
        let stats = PlayerStats::new(10, 10, 10, 10, 10, 10);
        assert!(stats.is_ok());

        let invalid_stats = PlayerStats::new(0, 10, 10, 10, 10, 10);
        assert!(invalid_stats.is_err());

        let invalid_stats = PlayerStats::new(21, 10, 10, 10, 10, 10);
        assert!(invalid_stats.is_err());
    }

    #[test]
    fn player_stats_modifiers() {
        let stats = PlayerStats::starting_stats();
        assert_eq!(stats.get_modifier(StatType::Strength), 0); // 10 -> modifier 0

        let high_stats = PlayerStats::new(18, 6, 10, 10, 10, 10).unwrap();
        assert_eq!(high_stats.get_modifier(StatType::Strength), 4); // 18 -> modifier +4
        assert_eq!(high_stats.get_modifier(StatType::Dexterity), -2); // 6 -> modifier -2
    }

    #[test]
    fn game_time_operations() {
        let time1 = GameTime::new(60);
        let time2 = GameTime::new(30);

        assert_eq!(time1.minutes(), 1);
        assert_eq!(time1.add(time2).seconds(), 90);
        assert_eq!(time1.subtract(time2).seconds(), 30);
    }

    #[test]
    fn game_time_display() {
        let time = GameTime::from_hours(1)
            .add(GameTime::from_minutes(30))
            .add(GameTime::new(45));
        assert_eq!(time.to_string(), "1h 30m 45s");
    }
}
