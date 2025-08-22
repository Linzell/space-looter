//! Domain Entities - Core Business Objects with Identity and Behavior
//!
//! This module contains the entities that form the core of the 3D isometric RPG
//! domain. All entities have identity and encapsulate business logic related
//! to their behavior and state transitions.

pub mod base;
pub mod event;
pub mod game;
pub mod map;
pub mod player;
pub mod quest;
pub mod resource;

// Re-export all entity types for convenience
pub use base::{Base, BaseBuilding, BaseLevel};
pub use event::{Event, EventType};
pub use game::GameSession;
pub use map::{Map, MapTile, ResourceNode};
pub use player::Player;
pub use quest::{Quest, QuestObjective, QuestStatus};
pub use resource::Resource;

/// Common trait for all domain entities
pub trait Entity {
    /// Get the unique identifier for this entity
    fn id(&self) -> &crate::domain::value_objects::EntityId;

    /// Check if this entity is valid (satisfies all business rules)
    fn is_valid(&self) -> bool;

    /// Get the version/timestamp of this entity for optimistic concurrency
    fn version(&self) -> u64;
}

/// Trait for entities that can be serialized for persistence
pub trait Persistable: Entity {
    /// Get the entity type name for storage identification
    fn entity_type() -> &'static str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_trait_is_object_safe() {
        // This test ensures the Entity trait can be used as a trait object
        fn _test_trait_object(_entity: &dyn Entity) {}
    }
}
