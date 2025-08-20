//! Bevy Components - ECS Component Wrappers for Domain Entities
//!
//! This module contains Bevy ECS components that wrap our domain entities,
//! providing the bridge between our pure domain model and Bevy's ECS system.

use crate::domain::{Enemy, Player, Score, Velocity};
use bevy::prelude::*;

/// Bevy component wrapper for Player domain entity
#[derive(Component, Debug, Clone)]
pub struct PlayerComponent {
    player: Player,
}

impl PlayerComponent {
    /// Create a new player component
    pub fn new(player: Player) -> Self {
        Self { player }
    }

    /// Get reference to the player entity
    pub fn player(&self) -> &Player {
        &self.player
    }

    /// Get mutable reference to the player entity
    pub fn player_mut(&mut self) -> &mut Player {
        &mut self.player
    }

    /// Update the wrapped player entity
    pub fn update_player(&mut self, player: Player) {
        self.player = player;
    }
}

/// Bevy component wrapper for Enemy domain entity
#[derive(Component, Debug, Clone)]
pub struct EnemyComponent {
    enemy: Enemy,
}

impl EnemyComponent {
    /// Create a new enemy component
    pub fn new(enemy: Enemy) -> Self {
        Self { enemy }
    }

    /// Get reference to the enemy entity
    pub fn enemy(&self) -> &Enemy {
        &self.enemy
    }

    /// Get mutable reference to the enemy entity
    pub fn enemy_mut(&mut self) -> &mut Enemy {
        &mut self.enemy
    }

    /// Update the wrapped enemy entity
    pub fn update_enemy(&mut self, enemy: Enemy) {
        self.enemy = enemy;
    }
}

/// Bevy component wrapper for Velocity domain value object
#[derive(Component, Debug, Clone)]
pub struct VelocityComponent {
    velocity: Velocity,
}

impl VelocityComponent {
    /// Create a new velocity component
    pub fn new(velocity: Velocity) -> Self {
        Self { velocity }
    }

    /// Get reference to the velocity
    pub fn velocity(&self) -> &Velocity {
        &self.velocity
    }

    /// Get mutable reference to the velocity
    pub fn velocity_mut(&mut self) -> &mut Velocity {
        &mut self.velocity
    }

    /// Update the velocity
    pub fn update_velocity(&mut self, velocity: Velocity) {
        self.velocity = velocity;
    }
}

/// Marker component for score display UI elements
#[derive(Component, Debug, Clone)]
pub struct ScoreDisplayComponent;

/// Marker component for game boundary entities
#[derive(Component, Debug, Clone)]
pub struct BoundaryComponent;

/// Marker component for UI elements
#[derive(Component, Debug, Clone)]
pub struct UIComponent;

/// Component for entities that should be cleaned up when off-screen
#[derive(Component, Debug, Clone)]
pub struct CleanupComponent {
    pub cleanup_y: f32, // Y coordinate below which entity should be cleaned up
}

impl CleanupComponent {
    /// Create new cleanup component with default cleanup boundary
    pub fn new() -> Self {
        Self { cleanup_y: -350.0 }
    }

    /// Create cleanup component with custom boundary
    pub fn with_boundary(cleanup_y: f32) -> Self {
        Self { cleanup_y }
    }

    /// Check if entity should be cleaned up based on transform
    pub fn should_cleanup(&self, transform: &Transform) -> bool {
        transform.translation.y < self.cleanup_y
    }
}

impl Default for CleanupComponent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{EnemyType, Position};

    fn create_test_player() -> Player {
        let position = Position::new(0.0, 0.0).unwrap();
        let velocity = Velocity::zero();
        Player::new("test_player".to_string(), position, velocity, 200.0).unwrap()
    }

    fn create_test_enemy() -> Enemy {
        let position = Position::new(0.0, 100.0).unwrap();
        let velocity = Velocity::new(0.0, -50.0).unwrap();
        Enemy::new(
            "test_enemy".to_string(),
            position,
            velocity,
            EnemyType::Basic,
        )
        .unwrap()
    }

    #[test]
    fn player_component_creation() {
        let player = create_test_player();
        let component = PlayerComponent::new(player.clone());
        assert_eq!(component.player().id().value(), "test_player");
    }

    #[test]
    fn enemy_component_creation() {
        let enemy = create_test_enemy();
        let component = EnemyComponent::new(enemy.clone());
        assert_eq!(component.enemy().id().value(), "test_enemy");
    }

    #[test]
    fn velocity_component_creation() {
        let velocity = Velocity::new(5.0, -3.0).unwrap();
        let component = VelocityComponent::new(velocity);
        assert_eq!(component.velocity().dx(), 5.0);
        assert_eq!(component.velocity().dy(), -3.0);
    }

    #[test]
    fn cleanup_component_functionality() {
        let cleanup = CleanupComponent::new();
        let transform_above = Transform::from_translation(Vec3::new(0.0, 0.0, 0.0));
        let transform_below = Transform::from_translation(Vec3::new(0.0, -400.0, 0.0));

        assert!(!cleanup.should_cleanup(&transform_above));
        assert!(cleanup.should_cleanup(&transform_below));
    }

    #[test]
    fn cleanup_component_custom_boundary() {
        let cleanup = CleanupComponent::with_boundary(-100.0);
        let transform = Transform::from_translation(Vec3::new(0.0, -150.0, 0.0));
        assert!(cleanup.should_cleanup(&transform));
    }
}
