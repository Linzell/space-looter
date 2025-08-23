//! Terrain system for 3D isometric world generation
//!
//! This module defines different terrain types, their properties, and
//! how they affect gameplay mechanics like movement, resource gathering,
//! and event triggers.

use crate::domain::value_objects::{
    resources::{
        RegenerationRate, ResourceAccessibility, ResourceNodeProperties, ResourceRichness,
    },
    ResourceType,
};
use crate::domain::{DomainError, DomainResult};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Different types of terrain that can exist in the world
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TerrainType {
    /// Open grassland - easy movement, basic resources
    Plains,
    /// Dense woodland - moderate movement, organics and food
    Forest,
    /// Rocky terrain - slow movement, metal and stone resources
    Mountains,
    /// Sandy terrain - moderate movement, hidden resources
    Desert,
    /// Frozen terrain - difficult movement, unique resources
    Tundra,
    /// Wet terrain - slow movement, organics and energy
    Swamp,
    /// Deep water - requires special movement, energy resources
    Ocean,
    /// Molten terrain - dangerous, rare resources
    Volcanic,
    /// Corrupted terrain - very dangerous, exotic matter
    Anomaly,
    /// Artificial terrain - base areas, no natural resources
    Constructed,
    /// Underground passages - hidden areas, rich resources
    Cave,
    /// Crystalline formations - energy and technology resources
    Crystal,
}

impl TerrainType {
    /// Get all terrain types
    pub fn all() -> Vec<TerrainType> {
        vec![
            TerrainType::Plains,
            TerrainType::Forest,
            TerrainType::Mountains,
            TerrainType::Desert,
            TerrainType::Tundra,
            TerrainType::Swamp,
            TerrainType::Ocean,
            TerrainType::Volcanic,
            TerrainType::Anomaly,
            TerrainType::Constructed,
            TerrainType::Cave,
            TerrainType::Crystal,
        ]
    }

    /// Get natural terrain types (excluding constructed/artificial)
    pub fn natural() -> Vec<TerrainType> {
        vec![
            TerrainType::Plains,
            TerrainType::Forest,
            TerrainType::Mountains,
            TerrainType::Desert,
            TerrainType::Tundra,
            TerrainType::Swamp,
            TerrainType::Ocean,
            TerrainType::Volcanic,
            TerrainType::Cave,
            TerrainType::Crystal,
        ]
    }

    /// Get safe terrain types (low danger)
    pub fn safe() -> Vec<TerrainType> {
        vec![
            TerrainType::Plains,
            TerrainType::Forest,
            TerrainType::Desert,
            TerrainType::Constructed,
        ]
    }

    /// Get dangerous terrain types
    pub fn dangerous() -> Vec<TerrainType> {
        vec![
            TerrainType::Volcanic,
            TerrainType::Anomaly,
            TerrainType::Ocean,
        ]
    }

    /// Get movement cost modifier (higher = slower movement)
    pub fn movement_cost(&self) -> u8 {
        match self {
            TerrainType::Plains => 1,
            TerrainType::Forest => 2,
            TerrainType::Mountains => 3,
            TerrainType::Desert => 2,
            TerrainType::Tundra => 3,
            TerrainType::Swamp => 4,
            TerrainType::Ocean => 5, // Requires special equipment
            TerrainType::Volcanic => 4,
            TerrainType::Anomaly => 3,
            TerrainType::Constructed => 1,
            TerrainType::Cave => 2,
            TerrainType::Crystal => 2,
        }
    }

    /// Get danger level (1-10, higher is more dangerous)
    pub fn danger_level(&self) -> u8 {
        match self {
            TerrainType::Plains => 1,
            TerrainType::Forest => 2,
            TerrainType::Mountains => 4,
            TerrainType::Desert => 3,
            TerrainType::Tundra => 5,
            TerrainType::Swamp => 4,
            TerrainType::Ocean => 7,
            TerrainType::Volcanic => 8,
            TerrainType::Anomaly => 10,
            TerrainType::Constructed => 1,
            TerrainType::Cave => 6,
            TerrainType::Crystal => 3,
        }
    }

    /// Check if this terrain can be traversed without special equipment
    pub fn is_passable(&self) -> bool {
        !matches!(self, TerrainType::Ocean)
    }

    /// Check if this terrain provides natural cover/concealment
    pub fn provides_cover(&self) -> bool {
        matches!(
            self,
            TerrainType::Forest | TerrainType::Mountains | TerrainType::Cave | TerrainType::Swamp
        )
    }

    /// Check if this terrain is underground
    pub fn is_underground(&self) -> bool {
        matches!(self, TerrainType::Cave)
    }

    /// Check if this terrain is artificial
    pub fn is_artificial(&self) -> bool {
        matches!(self, TerrainType::Constructed)
    }

    /// Get primary resource types this terrain can contain
    pub fn primary_resources(&self) -> Vec<ResourceType> {
        match self {
            TerrainType::Plains => vec![ResourceType::Food, ResourceType::Organics],
            TerrainType::Forest => vec![
                ResourceType::Food,
                ResourceType::Organics,
                ResourceType::Data,
            ],
            TerrainType::Mountains => vec![ResourceType::Metal, ResourceType::Alloys],
            TerrainType::Desert => vec![ResourceType::Metal, ResourceType::Technology],
            TerrainType::Tundra => vec![ResourceType::Energy, ResourceType::Data],
            TerrainType::Swamp => vec![ResourceType::Organics, ResourceType::Energy],
            TerrainType::Ocean => vec![ResourceType::Energy, ResourceType::Organics],
            TerrainType::Volcanic => vec![ResourceType::ExoticMatter, ResourceType::Alloys],
            TerrainType::Anomaly => vec![ResourceType::ExoticMatter, ResourceType::Technology],
            TerrainType::Constructed => vec![], // No natural resources
            TerrainType::Cave => vec![
                ResourceType::Metal,
                ResourceType::ExoticMatter,
                ResourceType::Technology,
            ],
            TerrainType::Crystal => vec![
                ResourceType::Energy,
                ResourceType::Technology,
                ResourceType::ExoticMatter,
            ],
        }
    }

    /// Get secondary resource types (less common)
    pub fn secondary_resources(&self) -> Vec<ResourceType> {
        match self {
            TerrainType::Plains => vec![ResourceType::Metal],
            TerrainType::Forest => vec![ResourceType::Energy],
            TerrainType::Mountains => vec![ResourceType::ExoticMatter, ResourceType::Technology],
            TerrainType::Desert => vec![ResourceType::ExoticMatter],
            TerrainType::Tundra => vec![ResourceType::Metal],
            TerrainType::Swamp => vec![ResourceType::Data],
            TerrainType::Ocean => vec![ResourceType::Technology],
            TerrainType::Volcanic => vec![ResourceType::Energy],
            TerrainType::Anomaly => vec![ResourceType::Data],
            TerrainType::Constructed => vec![],
            TerrainType::Cave => vec![ResourceType::Energy, ResourceType::Organics],
            TerrainType::Crystal => vec![ResourceType::Data],
        }
    }

    /// Generate random resource node properties for this terrain
    pub fn generate_resource_node(
        &self,
        resource_type: ResourceType,
    ) -> Option<ResourceNodeProperties> {
        let primary_resources = self.primary_resources();
        let secondary_resources = self.secondary_resources();

        // Check if this terrain can have this resource
        if !primary_resources.contains(&resource_type)
            && !secondary_resources.contains(&resource_type)
        {
            return None;
        }

        let is_primary = primary_resources.contains(&resource_type);

        // Determine richness based on terrain and whether it's a primary resource
        let richness = if is_primary {
            match self.danger_level() {
                1..=2 => ResourceRichness::Average,
                3..=5 => ResourceRichness::Rich,
                6..=10 => ResourceRichness::Abundant,
                _ => ResourceRichness::Average,
            }
        } else {
            ResourceRichness::Poor
        };

        // Determine accessibility based on terrain danger and movement cost
        let accessibility = match (self.danger_level(), self.movement_cost()) {
            (1..=2, 1..=2) => ResourceAccessibility::Easy,
            (1..=4, 1..=3) => ResourceAccessibility::Moderate,
            (1..=6, _) | (_, 1..=4) => ResourceAccessibility::Hard,
            _ => ResourceAccessibility::Dangerous,
        };

        // Determine regeneration rate
        let regeneration_rate = match self {
            TerrainType::Plains | TerrainType::Forest => {
                if resource_type == ResourceType::Food || resource_type == ResourceType::Organics {
                    RegenerationRate::Fast
                } else {
                    RegenerationRate::None
                }
            }
            TerrainType::Ocean | TerrainType::Swamp => {
                if resource_type == ResourceType::Organics || resource_type == ResourceType::Energy
                {
                    RegenerationRate::Moderate
                } else {
                    RegenerationRate::Slow
                }
            }
            TerrainType::Crystal => {
                if resource_type == ResourceType::Energy {
                    RegenerationRate::Slow
                } else {
                    RegenerationRate::None
                }
            }
            _ => RegenerationRate::None,
        };

        Some(ResourceNodeProperties::new(
            resource_type,
            richness,
            accessibility,
            regeneration_rate,
        ))
    }

    /// Get visibility range modifier (affects how far you can see)
    pub fn visibility_modifier(&self) -> f32 {
        match self {
            TerrainType::Plains => 1.5,
            TerrainType::Forest => 0.5,
            TerrainType::Mountains => 1.2,
            TerrainType::Desert => 1.3,
            TerrainType::Tundra => 1.1,
            TerrainType::Swamp => 0.6,
            TerrainType::Ocean => 1.0,
            TerrainType::Volcanic => 0.7,
            TerrainType::Anomaly => 0.3,
            TerrainType::Constructed => 1.0,
            TerrainType::Cave => 0.2,
            TerrainType::Crystal => 0.8,
        }
    }

    /// Get event trigger probability modifier
    pub fn event_probability_modifier(&self) -> f32 {
        match self {
            TerrainType::Plains => 0.8,
            TerrainType::Forest => 1.2,
            TerrainType::Mountains => 1.0,
            TerrainType::Desert => 0.9,
            TerrainType::Tundra => 1.1,
            TerrainType::Swamp => 1.4,
            TerrainType::Ocean => 1.3,
            TerrainType::Volcanic => 2.0,
            TerrainType::Anomaly => 3.0,
            TerrainType::Constructed => 0.1,
            TerrainType::Cave => 1.5,
            TerrainType::Crystal => 1.2,
        }
    }

    /// Get description of the terrain
    pub fn description(&self) -> &'static str {
        match self {
            TerrainType::Plains => {
                "Open grasslands with gentle rolling hills and scattered vegetation"
            }
            TerrainType::Forest => "Dense woodlands with towering trees and rich biodiversity",
            TerrainType::Mountains => "Rocky peaks and steep cliffs rich in mineral deposits",
            TerrainType::Desert => "Vast sandy expanses with hidden treasures beneath",
            TerrainType::Tundra => "Frozen wastelands with harsh conditions and unique resources",
            TerrainType::Swamp => "Wetlands teeming with life and mysterious energies",
            TerrainType::Ocean => "Deep waters requiring special equipment to traverse",
            TerrainType::Volcanic => {
                "Active volcanic regions with dangerous but valuable materials"
            }
            TerrainType::Anomaly => "Corrupted landscapes with reality-bending properties",
            TerrainType::Constructed => "Artificial terrain created for bases and settlements",
            TerrainType::Cave => "Underground networks hiding rare resources and secrets",
            TerrainType::Crystal => "Crystalline formations that resonate with energy",
        }
    }

    /// Get color representation for map display
    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            TerrainType::Plains => (102, 153, 51),       // Green
            TerrainType::Forest => (34, 102, 34),        // Dark Green
            TerrainType::Mountains => (102, 102, 102),   // Gray
            TerrainType::Desert => (255, 204, 102),      // Sandy
            TerrainType::Tundra => (204, 255, 255),      // Light Blue
            TerrainType::Swamp => (102, 153, 102),       // Dark Green-Gray
            TerrainType::Ocean => (51, 102, 204),        // Blue
            TerrainType::Volcanic => (204, 51, 51),      // Red
            TerrainType::Anomaly => (153, 51, 204),      // Purple
            TerrainType::Constructed => (153, 153, 153), // Light Gray
            TerrainType::Cave => (51, 51, 51),           // Very Dark Gray
            TerrainType::Crystal => (204, 204, 255),     // Light Purple
        }
    }

    /// Get terrain icon for UI display
    pub fn icon(&self) -> char {
        match self {
            TerrainType::Plains => '🌾',
            TerrainType::Forest => '🌲',
            TerrainType::Mountains => '⛰',
            TerrainType::Desert => '🏜',
            TerrainType::Tundra => '🧊',
            TerrainType::Swamp => '🐸',
            TerrainType::Ocean => '🌊',
            TerrainType::Volcanic => '🌋',
            TerrainType::Anomaly => '👁',
            TerrainType::Constructed => '🏗',
            TerrainType::Cave => '🕳',
            TerrainType::Crystal => '💎',
        }
    }

    /// Check if this terrain type is compatible with adjacent terrain
    /// Some terrain types naturally occur together, others don't
    pub fn is_compatible_with(&self, other: &TerrainType) -> bool {
        match (self, other) {
            // Plains are compatible with most terrain
            (TerrainType::Plains, _) | (_, TerrainType::Plains) => true,

            // Forest compatibility
            (TerrainType::Forest, TerrainType::Swamp)
            | (TerrainType::Swamp, TerrainType::Forest) => true,
            (TerrainType::Forest, TerrainType::Mountains)
            | (TerrainType::Mountains, TerrainType::Forest) => true,

            // Mountain compatibility
            (TerrainType::Mountains, TerrainType::Desert)
            | (TerrainType::Desert, TerrainType::Mountains) => true,
            (TerrainType::Mountains, TerrainType::Tundra)
            | (TerrainType::Tundra, TerrainType::Mountains) => true,
            (TerrainType::Mountains, TerrainType::Cave)
            | (TerrainType::Cave, TerrainType::Mountains) => true,

            // Desert and tundra are opposites
            (TerrainType::Desert, TerrainType::Tundra)
            | (TerrainType::Tundra, TerrainType::Desert) => false,

            // Ocean is only compatible with itself and constructed
            (TerrainType::Ocean, TerrainType::Ocean) => true,
            (TerrainType::Ocean, TerrainType::Constructed)
            | (TerrainType::Constructed, TerrainType::Ocean) => true,
            (TerrainType::Ocean, _) | (_, TerrainType::Ocean) => false,

            // Anomalies can appear anywhere
            (TerrainType::Anomaly, _) | (_, TerrainType::Anomaly) => true,

            // Volcanic is mostly incompatible except with mountains
            (TerrainType::Volcanic, TerrainType::Mountains)
            | (TerrainType::Mountains, TerrainType::Volcanic) => true,
            (TerrainType::Volcanic, TerrainType::Desert)
            | (TerrainType::Desert, TerrainType::Volcanic) => true,
            (TerrainType::Volcanic, _) | (_, TerrainType::Volcanic) => false,

            // Constructed can be placed anywhere
            (TerrainType::Constructed, _) | (_, TerrainType::Constructed) => true,

            // Cave is compatible with mountains and itself
            (TerrainType::Cave, TerrainType::Cave) => true,

            // Crystal formations are special
            (TerrainType::Crystal, TerrainType::Mountains)
            | (TerrainType::Mountains, TerrainType::Crystal) => true,
            (TerrainType::Crystal, TerrainType::Cave)
            | (TerrainType::Cave, TerrainType::Crystal) => true,
            (TerrainType::Crystal, _) | (_, TerrainType::Crystal) => false,

            // Default compatibility for remaining cases
            _ => true,
        }
    }
}

impl fmt::Display for TerrainType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TerrainType::Plains => write!(f, "Plains"),
            TerrainType::Forest => write!(f, "Forest"),
            TerrainType::Mountains => write!(f, "Mountains"),
            TerrainType::Desert => write!(f, "Desert"),
            TerrainType::Tundra => write!(f, "Tundra"),
            TerrainType::Swamp => write!(f, "Swamp"),
            TerrainType::Ocean => write!(f, "Ocean"),
            TerrainType::Volcanic => write!(f, "Volcanic"),
            TerrainType::Anomaly => write!(f, "Anomaly"),
            TerrainType::Constructed => write!(f, "Constructed"),
            TerrainType::Cave => write!(f, "Cave"),
            TerrainType::Crystal => write!(f, "Crystal"),
        }
    }
}

/// Elevation information for terrain tiles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Elevation {
    pub height: i32,
}

impl Elevation {
    /// Create new elevation
    pub fn new(height: i32) -> DomainResult<Self> {
        if height < -100 || height > 100 {
            return Err(DomainError::ValidationError(
                "Elevation must be between -100 and 100".to_string(),
            ));
        }
        Ok(Self { height })
    }

    /// Sea level elevation
    pub fn sea_level() -> Self {
        Self { height: 0 }
    }

    /// Check if this is above sea level
    pub fn is_above_sea_level(&self) -> bool {
        self.height > 0
    }

    /// Check if this is below sea level
    pub fn is_below_sea_level(&self) -> bool {
        self.height < 0
    }

    /// Get elevation category
    pub fn category(&self) -> ElevationCategory {
        match self.height {
            h if h < -20 => ElevationCategory::DeepUnderwater,
            h if h < 0 => ElevationCategory::Underwater,
            0 => ElevationCategory::SeaLevel,
            h if h < 10 => ElevationCategory::Lowlands,
            h if h < 30 => ElevationCategory::Hills,
            h if h < 60 => ElevationCategory::Mountains,
            _ => ElevationCategory::HighMountains,
        }
    }

    /// Get movement difficulty modifier based on elevation change
    pub fn movement_difficulty(&self, from: &Elevation) -> f32 {
        let elevation_change = (self.height - from.height).abs();
        match elevation_change {
            0 => 1.0,
            1..=5 => 1.2,
            6..=15 => 1.5,
            16..=30 => 2.0,
            _ => 3.0,
        }
    }
}

impl fmt::Display for Elevation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.height, self.category())
    }
}

/// Categories of elevation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElevationCategory {
    DeepUnderwater,
    Underwater,
    SeaLevel,
    Lowlands,
    Hills,
    Mountains,
    HighMountains,
}

impl fmt::Display for ElevationCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ElevationCategory::DeepUnderwater => write!(f, "Deep Underwater"),
            ElevationCategory::Underwater => write!(f, "Underwater"),
            ElevationCategory::SeaLevel => write!(f, "Sea Level"),
            ElevationCategory::Lowlands => write!(f, "Lowlands"),
            ElevationCategory::Hills => write!(f, "Hills"),
            ElevationCategory::Mountains => write!(f, "Mountains"),
            ElevationCategory::HighMountains => write!(f, "High Mountains"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terrain_type_properties() {
        assert_eq!(TerrainType::Plains.movement_cost(), 1);
        assert_eq!(TerrainType::Mountains.movement_cost(), 3);
        assert_eq!(TerrainType::Anomaly.danger_level(), 10);
        assert!(TerrainType::Plains.is_passable());
        assert!(!TerrainType::Ocean.is_passable());
    }

    #[test]
    fn terrain_resources() {
        let plains_resources = TerrainType::Plains.primary_resources();
        assert!(plains_resources.contains(&ResourceType::Food));
        assert!(plains_resources.contains(&ResourceType::Organics));

        let mountain_resources = TerrainType::Mountains.primary_resources();
        assert!(mountain_resources.contains(&ResourceType::Metal));
    }

    #[test]
    fn terrain_resource_node_generation() {
        let node = TerrainType::Mountains.generate_resource_node(ResourceType::Metal);
        assert!(node.is_some());
        let node = node.unwrap();
        assert_eq!(node.resource_type, ResourceType::Metal);

        // Plains shouldn't generate metal nodes as primary resource
        let invalid_node = TerrainType::Plains.generate_resource_node(ResourceType::ExoticMatter);
        assert!(invalid_node.is_none());
    }

    #[test]
    fn terrain_compatibility() {
        assert!(TerrainType::Plains.is_compatible_with(&TerrainType::Forest));
        assert!(TerrainType::Mountains.is_compatible_with(&TerrainType::Cave));
        assert!(!TerrainType::Desert.is_compatible_with(&TerrainType::Tundra));
        assert!(!TerrainType::Ocean.is_compatible_with(&TerrainType::Desert));
    }

    #[test]
    fn terrain_properties() {
        assert!(TerrainType::Forest.provides_cover());
        assert!(!TerrainType::Plains.provides_cover());
        assert!(TerrainType::Cave.is_underground());
        assert!(TerrainType::Constructed.is_artificial());
    }

    #[test]
    fn elevation_creation() {
        let elevation = Elevation::new(25).unwrap();
        assert_eq!(elevation.height, 25);
        assert!(elevation.is_above_sea_level());
        assert_eq!(elevation.category(), ElevationCategory::Hills);

        // Invalid elevation
        assert!(Elevation::new(150).is_err());
    }

    #[test]
    fn elevation_movement_difficulty() {
        let low = Elevation::new(5).unwrap();
        let high = Elevation::new(25).unwrap();

        let difficulty = high.movement_difficulty(&low);
        assert!(difficulty > 1.0); // Should be more difficult to climb
    }

    #[test]
    fn elevation_categories() {
        let deep = Elevation::new(-25).unwrap();
        assert_eq!(deep.category(), ElevationCategory::DeepUnderwater);

        let sea = Elevation::sea_level();
        assert_eq!(sea.category(), ElevationCategory::SeaLevel);
        assert!(!sea.is_above_sea_level());
        assert!(!sea.is_below_sea_level());

        let mountain = Elevation::new(45).unwrap();
        assert_eq!(mountain.category(), ElevationCategory::Mountains);
    }

    #[test]
    fn terrain_modifiers() {
        assert!(TerrainType::Plains.visibility_modifier() > 1.0);
        assert!(TerrainType::Forest.visibility_modifier() < 1.0);
        assert!(TerrainType::Anomaly.event_probability_modifier() > 2.0);
        assert!(TerrainType::Constructed.event_probability_modifier() < 0.5);
    }
}
