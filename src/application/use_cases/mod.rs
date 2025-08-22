//! Use Cases - Application Business Operations
//!
//! This module contains use cases that represent specific business operations
//! in the Space Looter game. Each use case encapsulates a single business
//! workflow and coordinates between domain entities and services.
//!
//! ## Architecture
//! - **Move Player**: Handle player movement with business rules
//! - **Spawn Enemies**: Manage enemy creation and placement
//! - **Handle Collision**: Process entity collisions and effects
//! - **Update Score**: Manage score changes and validation
//!
//! ## Rules
//! - Single responsibility per use case
//! - Clear input/output contracts
//! - Domain entity coordination
//! - Business rule enforcement

pub mod handle_collision;
pub mod move_player;
pub mod spawn_enemies;
pub mod update_score;

// Re-export use cases for convenience
pub use handle_collision::HandleEncounterUseCase;
pub use move_player::MovePlayerUseCase;
pub use spawn_enemies::SpawnEnemiesUseCase;
pub use update_score::UpdateScoreUseCase;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn use_cases_can_be_imported() {
        // Compilation test to ensure all use cases are properly exported
        // Individual use case tests are in their respective modules
    }
}
