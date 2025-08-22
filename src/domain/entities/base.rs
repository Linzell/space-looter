//! Base Entity - Player's home base that evolves over time
//!
//! This entity represents the player's base which can be upgraded with resources
//! and provides various benefits and capabilities.

use crate::domain::value_objects::resources::ResourceCollection;
use crate::domain::value_objects::{EntityId, Position3D, ResourceType};
use crate::domain::{DomainError, DomainResult};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// The player's base entity
#[derive(Debug, Clone, PartialEq)]
pub struct Base {
    id: EntityId,
    name: String,
    position: Position3D,
    level: BaseLevel,
    buildings: Vec<BaseBuilding>,
    resources_stored: ResourceCollection,
    storage_capacity: u32,
    created_at: DateTime<Utc>,
    last_updated: DateTime<Utc>,
    version: u64,
}

impl Base {
    /// Create a new base
    pub fn new(id: EntityId, name: String, position: Position3D) -> DomainResult<Self> {
        if name.is_empty() || name.len() > 50 {
            return Err(DomainError::ValidationError(
                "Base name must be between 1 and 50 characters".to_string(),
            ));
        }

        let now = Utc::now();
        Ok(Self {
            id,
            name,
            position,
            level: BaseLevel::Level1,
            buildings: Vec::new(),
            resources_stored: ResourceCollection::new(),
            storage_capacity: 1000,
            created_at: now,
            last_updated: now,
            version: 1,
        })
    }

    /// Get base ID
    pub fn id(&self) -> &EntityId {
        &self.id
    }

    /// Get base name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get base position
    pub fn position(&self) -> &Position3D {
        &self.position
    }

    /// Get base level
    pub fn level(&self) -> BaseLevel {
        self.level
    }

    /// Get buildings
    pub fn buildings(&self) -> &[BaseBuilding] {
        &self.buildings
    }

    /// Get stored resources
    pub fn resources(&self) -> &ResourceCollection {
        &self.resources_stored
    }

    /// Get storage capacity
    pub fn storage_capacity(&self) -> u32 {
        self.storage_capacity
    }
}

/// Base development levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseLevel {
    Level1,
    Level2,
    Level3,
    Level4,
    Level5,
}

impl BaseLevel {
    /// Get the storage capacity for this level
    pub fn storage_capacity(&self) -> u32 {
        match self {
            BaseLevel::Level1 => 1000,
            BaseLevel::Level2 => 2500,
            BaseLevel::Level3 => 5000,
            BaseLevel::Level4 => 10000,
            BaseLevel::Level5 => 25000,
        }
    }
}

/// Buildings that can be constructed in the base
#[derive(Debug, Clone, PartialEq)]
pub struct BaseBuilding {
    pub id: EntityId,
    pub building_type: BuildingType,
    pub name: String,
    pub level: u8,
    pub position_in_base: (i32, i32),
    pub constructed_at: DateTime<Utc>,
}

impl BaseBuilding {
    /// Create a new building
    pub fn new(building_type: BuildingType, name: String, position_in_base: (i32, i32)) -> Self {
        Self {
            id: EntityId::generate(),
            building_type,
            name,
            level: 1,
            position_in_base,
            constructed_at: Utc::now(),
        }
    }
}

/// Types of buildings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildingType {
    ResourceStorage,
    Workshop,
    Laboratory,
    PowerPlant,
    LivingQuarters,
    DefenseSystem,
}

impl BuildingType {
    /// Get the resource cost to build this building at level 1
    pub fn build_cost(&self) -> HashMap<ResourceType, u32> {
        let mut cost = HashMap::new();
        match self {
            BuildingType::ResourceStorage => {
                cost.insert(ResourceType::Metal, 50);
                cost.insert(ResourceType::Energy, 20);
            }
            BuildingType::Workshop => {
                cost.insert(ResourceType::Metal, 75);
                cost.insert(ResourceType::Technology, 10);
            }
            BuildingType::Laboratory => {
                cost.insert(ResourceType::Technology, 25);
                cost.insert(ResourceType::Energy, 50);
            }
            BuildingType::PowerPlant => {
                cost.insert(ResourceType::Metal, 100);
                cost.insert(ResourceType::Technology, 15);
            }
            BuildingType::LivingQuarters => {
                cost.insert(ResourceType::Metal, 40);
                cost.insert(ResourceType::Food, 30);
            }
            BuildingType::DefenseSystem => {
                cost.insert(ResourceType::Metal, 150);
                cost.insert(ResourceType::Technology, 30);
                cost.insert(ResourceType::Energy, 75);
            }
        }
        cost
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_creation() {
        let id = EntityId::generate();
        let name = "Test Base".to_string();
        let position = Position3D::origin();

        let base = Base::new(id, name.clone(), position).unwrap();
        assert_eq!(base.name(), &name);
        assert_eq!(base.position(), &position);
        assert_eq!(base.level(), BaseLevel::Level1);
    }

    #[test]
    fn building_creation() {
        let building =
            BaseBuilding::new(BuildingType::Workshop, "Main Workshop".to_string(), (0, 0));
        assert_eq!(building.building_type, BuildingType::Workshop);
        assert_eq!(building.level, 1);
    }

    #[test]
    fn building_costs() {
        let cost = BuildingType::Workshop.build_cost();
        assert!(cost.contains_key(&ResourceType::Metal));
        assert!(cost.contains_key(&ResourceType::Technology));
    }
}
