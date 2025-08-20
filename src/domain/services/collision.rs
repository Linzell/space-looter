//! Collision Service - Domain Logic for Entity Collisions
//!
//! Handles collision detection and resolution between game entities
//! following business rules for the Space Looter game.

use crate::domain::{DomainError, DomainResult, Position};

/// Service for handling collision detection and resolution
pub struct CollisionService;

impl CollisionService {
    /// Create a new collision service
    pub fn new() -> Self {
        Self
    }

    /// Check if two positions are within collision distance
    pub fn check_collision(
        &self,
        pos1: &Position,
        pos2: &Position,
        collision_radius: f32,
    ) -> DomainResult<bool> {
        if collision_radius <= 0.0 {
            return Err(DomainError::CollisionError(
                "Collision radius must be positive".to_string(),
            ));
        }

        let distance = pos1.distance_to(pos2);
        Ok(distance <= collision_radius)
    }

    /// Check collision between player and enemy
    pub fn check_player_enemy_collision(
        &self,
        player_pos: &Position,
        enemy_pos: &Position,
    ) -> DomainResult<bool> {
        self.check_collision(
            player_pos,
            enemy_pos,
            crate::domain::constants::COLLISION_RADIUS,
        )
    }

    /// Check if entity is within game boundaries
    pub fn check_boundary_collision(
        &self,
        position: &Position,
        boundaries: &crate::domain::GameBoundaries,
    ) -> bool {
        !boundaries.contains(position)
    }

    /// Get collision resolution for player-enemy collision
    pub fn resolve_player_enemy_collision(
        &self,
        player_id: &str,
        enemy_id: &str,
    ) -> CollisionResolution {
        CollisionResolution {
            collision_type: CollisionType::PlayerEnemy,
            entities_to_remove: vec![enemy_id.to_string()],
            score_change: Some(crate::domain::constants::POINTS_PER_ENEMY),
            player_effects: vec![PlayerEffect::ScoreIncrease],
        }
    }
}

impl Default for CollisionService {
    fn default() -> Self {
        Self::new()
    }
}

/// Types of collisions that can occur
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollisionType {
    PlayerEnemy,
    EnemyBoundary,
    PlayerBoundary,
}

/// Effects that can be applied to the player
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerEffect {
    ScoreIncrease,
    TakeDamage,
    Heal,
    SpeedBoost,
}

/// Result of collision resolution
#[derive(Debug, Clone, PartialEq)]
pub struct CollisionResolution {
    pub collision_type: CollisionType,
    pub entities_to_remove: Vec<String>,
    pub score_change: Option<u32>,
    pub player_effects: Vec<PlayerEffect>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collision_service() -> CollisionService {
        CollisionService::new()
    }

    #[test]
    fn collision_service_creation() {
        let service = collision_service();
        // Just test that it can be created
        let _ = service;
    }

    #[test]
    fn check_collision_within_radius() {
        let service = collision_service();
        let pos1 = Position::new(0.0, 0.0).unwrap();
        let pos2 = Position::new(5.0, 0.0).unwrap();

        let result = service.check_collision(&pos1, &pos2, 10.0).unwrap();
        assert!(result);

        let result = service.check_collision(&pos1, &pos2, 3.0).unwrap();
        assert!(!result);
    }

    #[test]
    fn check_collision_invalid_radius() {
        let service = collision_service();
        let pos1 = Position::new(0.0, 0.0).unwrap();
        let pos2 = Position::new(5.0, 0.0).unwrap();

        assert!(service.check_collision(&pos1, &pos2, 0.0).is_err());
        assert!(service.check_collision(&pos1, &pos2, -5.0).is_err());
    }

    #[test]
    fn player_enemy_collision_check() {
        let service = collision_service();
        let player_pos = Position::new(0.0, 0.0).unwrap();
        let enemy_pos =
            Position::new(crate::domain::constants::COLLISION_RADIUS / 2.0, 0.0).unwrap();

        let result = service
            .check_player_enemy_collision(&player_pos, &enemy_pos)
            .unwrap();
        assert!(result);
    }

    #[test]
    fn boundary_collision_check() {
        let service = collision_service();
        let boundaries = crate::domain::GameBoundaries::standard();

        let inside_pos = Position::new(0.0, 0.0).unwrap();
        let outside_pos = Position::new(1000.0, 1000.0).unwrap();

        assert!(!service.check_boundary_collision(&inside_pos, &boundaries));
        assert!(service.check_boundary_collision(&outside_pos, &boundaries));
    }

    #[test]
    fn collision_resolution() {
        let service = collision_service();
        let resolution = service.resolve_player_enemy_collision("player1", "enemy1");

        assert_eq!(resolution.collision_type, CollisionType::PlayerEnemy);
        assert_eq!(resolution.entities_to_remove, vec!["enemy1"]);
        assert_eq!(
            resolution.score_change,
            Some(crate::domain::constants::POINTS_PER_ENEMY)
        );
        assert_eq!(resolution.player_effects, vec![PlayerEffect::ScoreIncrease]);
    }
}
