//! Move Player Use Case - Player Movement Logic
//!
//! Handles player movement requests with business rule validation
//! and coordinate with domain entities.

use crate::application::{dto::MovePlayerInput, dto::MovePlayerOutput, ApplicationResult};
use crate::domain::{CollisionService, Player, Position, Velocity};

/// Use case for handling player movement operations
pub struct MovePlayerUseCase {
    collision_service: CollisionService,
}

impl MovePlayerUseCase {
    /// Create a new move player use case
    pub fn new(collision_service: CollisionService) -> Self {
        Self { collision_service }
    }

    /// Execute player movement with business rules
    pub fn execute(&self, input: MovePlayerInput) -> ApplicationResult<MovePlayerOutput> {
        // Create velocity from input direction and speed
        let velocity =
            Velocity::from_direction((input.direction_x, input.direction_y), input.speed)
                .map_err(|e| crate::application::ApplicationError::DomainError(e))?;

        // For now, just return the calculated values
        // In a real implementation, this would update the player entity
        let new_position = Position::new(0, 0, 0);

        Ok(MovePlayerOutput {
            player_id: input.player_id,
            new_position,
            new_velocity: velocity,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_player_use_case_creation() {
        // This is a stub test for compilation
    }
}
