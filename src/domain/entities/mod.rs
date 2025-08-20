//! Domain Entities - Core Business Objects
//!
//! This module contains the domain entities that represent the core business
//! concepts in the Space Looter game. Each entity has identity, state, and
//! business behavior.

pub mod enemy;
pub mod game;
pub mod player;

// Re-export entities for convenience
pub use enemy::{Enemy, EnemyType};
pub use game::GameSession;
pub use player::Player;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entities_can_be_imported() {
        // Compilation test to ensure all entities are properly exported
        // Individual entity tests are in their respective modules
    }
}
