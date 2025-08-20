//! Domain Services - Complex Business Logic
//!
//! This module contains domain services that encapsulate complex business logic
//! that doesn't naturally belong to a single entity. Domain services coordinate
//! between entities and implement cross-cutting business rules.
//!
//! ## Architecture
//! - **Collision Service**: Handles collision detection between entities
//! - **Spawning Service**: Manages enemy spawning rules and logic
//!
//! ## Rules
//! - No infrastructure dependencies
//! - Pure business logic only
//! - Stateless services (or explicitly managed state)
//! - Clear single responsibility

pub mod collision;
pub mod spawning;

// Re-export services for convenience
pub use collision::CollisionService;
pub use spawning::SpawningService;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn services_can_be_imported() {
        // Compilation test to ensure all services are properly exported
        // Individual service tests are in their respective modules
    }
}
