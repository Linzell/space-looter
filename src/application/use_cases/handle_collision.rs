//! Handle Collision Use Case - Collision Processing Logic
//!
//! Handles collision detection and resolution with business rule validation
//! and coordinates with domain entities and services.

use crate::application::{
    dto::HandleCollisionInput, dto::HandleCollisionOutput, ApplicationResult,
};
use crate::domain::CollisionService;
use std::sync::Arc;

/// Use case for handling collision detection and resolution
pub struct HandleCollisionUseCase {
    collision_service: Arc<CollisionService>,
}

impl HandleCollisionUseCase {
    /// Create a new handle collision use case
    pub fn new(collision_service: Arc<CollisionService>) -> Self {
        Self { collision_service }
    }

    /// Execute collision handling with business rules
    pub fn execute(&self, input: HandleCollisionInput) -> ApplicationResult<HandleCollisionOutput> {
        // Check if collision occurred between the two entities
        let collision_detected = self
            .collision_service
            .check_collision(
                &input.entity1_position,
                &input.entity2_position,
                crate::domain::constants::COLLISION_RADIUS,
            )
            .map_err(|e| crate::application::ApplicationError::DomainError(e))?;

        if collision_detected {
            // Resolve collision based on entity types
            // For now, assume player-enemy collision
            let resolution = self
                .collision_service
                .resolve_player_enemy_collision(&input.entity1_id, &input.entity2_id);

            Ok(HandleCollisionOutput {
                collision_detected: true,
                score_change: resolution.score_change,
                entities_to_remove: resolution.entities_to_remove,
            })
        } else {
            Ok(HandleCollisionOutput {
                collision_detected: false,
                score_change: None,
                entities_to_remove: vec![],
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_collision_use_case_creation() {
        let collision_service = Arc::new(CollisionService::new());
        let use_case = HandleCollisionUseCase::new(collision_service);
        let _ = use_case;
    }
}
