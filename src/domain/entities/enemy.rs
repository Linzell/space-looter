//! Enemy Entity - Basic Enemy Domain Logic
//!
//! Represents enemies in the Space Looter game with movement and behavior.

use crate::domain::{DomainError, DomainResult, Position, Velocity};

/// Types of enemies in the game
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnemyType {
    Basic,
}

/// Unique identifier for an enemy
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnemyId(String);

impl EnemyId {
    pub fn new(id: String) -> DomainResult<Self> {
        if id.is_empty() {
            return Err(DomainError::EnemyError(
                "Enemy ID cannot be empty".to_string(),
            ));
        }
        Ok(EnemyId(id))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

/// Enemy entity with movement and behavior
#[derive(Debug, Clone, PartialEq)]
pub struct Enemy {
    id: EnemyId,
    position: Position,
    velocity: Velocity,
    enemy_type: EnemyType,
}

impl Enemy {
    /// Create a new enemy
    pub fn new(
        id: String,
        position: Position,
        velocity: Velocity,
        enemy_type: EnemyType,
    ) -> DomainResult<Self> {
        let enemy_id = EnemyId::new(id)?;

        Ok(Enemy {
            id: enemy_id,
            position,
            velocity,
            enemy_type,
        })
    }

    /// Get enemy ID
    pub fn id(&self) -> &EnemyId {
        &self.id
    }

    /// Get current position
    pub fn position(&self) -> &Position {
        &self.position
    }

    /// Get current velocity
    pub fn velocity(&self) -> &Velocity {
        &self.velocity
    }

    /// Get enemy type
    pub fn enemy_type(&self) -> &EnemyType {
        &self.enemy_type
    }

    /// Update position based on velocity and delta time
    pub fn update_position(&mut self, delta_time: f32) -> DomainResult<()> {
        let new_position = self.position.move_by_velocity(&self.velocity, delta_time)?;
        self.position = new_position;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enemy_creation() {
        let position = Position::new(0.0, 100.0).unwrap();
        let velocity = Velocity::new(0.0, -50.0).unwrap();
        let enemy =
            Enemy::new("enemy_1".to_string(), position, velocity, EnemyType::Basic).unwrap();

        assert_eq!(enemy.id().value(), "enemy_1");
        assert_eq!(enemy.position(), &position);
        assert_eq!(enemy.velocity(), &velocity);
        assert_eq!(enemy.enemy_type(), &EnemyType::Basic);
    }
}
