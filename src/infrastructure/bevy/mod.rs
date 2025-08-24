//! Bevy Infrastructure - ECS Integration
//!
//! This module provides the integration between our domain model and Bevy's
//! Entity Component System (ECS). It contains components, systems, resources,
//! and plugins that adapt our domain entities to work with Bevy.
//!
//! ## Architecture
//! - **Components**: Bevy ECS components wrapping domain entities
//! - **Resources**: Bevy resources for global game state
//! - **Systems**: Bevy systems that execute use cases
//! - **Plugins**: Bevy plugins for organizing functionality

pub mod audio;
pub mod components;
pub mod font_service;
pub mod resources;
pub mod systems;

// Re-export common Bevy integration types
pub use audio::{AudioPlaybacks, BevyAudioAdapter, SpaceLooterAudioPlugin};
pub use components::{EnemyComponent, PlayerComponent, ScoreDisplayComponent, VelocityComponent};
pub use font_service::{BevyFontService, FontPlugin};
pub use resources::{GameBoundariesResource, GameSessionResource, ScoreResource};
pub use systems::*;

use crate::infrastructure::InfrastructureError;
use bevy::prelude::*;

/// Main plugin for Bevy game integration
pub struct BevyGamePlugin;

impl Plugin for BevyGamePlugin {
    fn build(&self, app: &mut App) {
        info!("Initializing BevyGamePlugin");

        // Add resources
        app.insert_resource(ScoreResource::default())
            .insert_resource(GameSessionResource::default());

        // Add startup systems
        app.add_systems(
            Startup,
            (
                systems::setup_camera,
                systems::setup_ui,
                systems::spawn_player,
            ),
        );

        // Add update systems
        app.add_systems(
            Update,
            (
                systems::player_input_system,
                systems::movement_system,
                systems::enemy_spawning_system,
                systems::collision_system,
                systems::cleanup_system,
            )
                .chain(),
        ); // Chain to ensure proper execution order

        info!("BevyGamePlugin initialized successfully");
    }
}

/// Plugin for organizing Bevy systems by responsibility
pub struct BevySystemsPlugin;

impl Plugin for BevySystemsPlugin {
    fn build(&self, app: &mut App) {
        info!("Initializing BevySystemsPlugin");

        // Configure system sets for better organization
        app.configure_sets(
            Update,
            (
                BevySystemSet::Input,
                BevySystemSet::Logic,
                BevySystemSet::Physics,
                BevySystemSet::Rendering,
            )
                .chain(),
        );

        // Add systems to appropriate sets
        app.add_systems(
            Update,
            (
                systems::player_input_system.in_set(BevySystemSet::Input),
                systems::movement_system.in_set(BevySystemSet::Physics),
                systems::collision_system.in_set(BevySystemSet::Logic),
                systems::cleanup_system.in_set(BevySystemSet::Logic),
            ),
        );

        info!("BevySystemsPlugin initialized successfully");
    }
}

/// System sets for organizing Bevy systems
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum BevySystemSet {
    /// Input processing systems
    Input,
    /// Game logic systems
    Logic,
    /// Physics and movement systems
    Physics,
    /// Rendering and visual systems
    Rendering,
}

/// Helper functions for Bevy integration
pub mod helpers {
    use super::*;
    use crate::domain::{Position3D, Velocity};

    /// Convert domain Position3D to Bevy Transform
    pub fn position_to_transform(position: &Position3D) -> Transform {
        Transform::from_translation(Vec3::new(
            position.x as f32,
            position.y as f32,
            position.z as f32,
        ))
    }

    /// Convert Bevy Transform to domain Position3D
    pub fn transform_to_position(transform: &Transform) -> Result<Position3D, InfrastructureError> {
        Ok(Position3D::new(
            transform.translation.x as i32,
            transform.translation.y as i32,
            transform.translation.z as i32,
        ))
    }

    /// Convert domain Velocity to Bevy Vec3
    pub fn velocity_to_vec3(velocity: &Velocity) -> Vec3 {
        Vec3::new(velocity.dx(), velocity.dy(), 0.0)
    }

    /// Convert Bevy Vec3 to domain Velocity
    pub fn vec3_to_velocity(vec3: &Vec3) -> Result<Velocity, InfrastructureError> {
        Velocity::new(vec3.x, vec3.y)
            .map_err(|e| InfrastructureError::BevyError(format!("Vec3 conversion failed: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::helpers::*;
    use super::*;
    use crate::domain::{Position3D, Velocity};

    #[test]
    fn position_transform_conversion() {
        let position = Position3D::new(10, 20, 5);
        let transform = position_to_transform(&position);

        assert_eq!(transform.translation.x, 10.0);
        assert_eq!(transform.translation.y, 20.0);
        assert_eq!(transform.translation.z, 5.0);

        let converted_back = transform_to_position(&transform).unwrap();
        assert_eq!(converted_back, position);
    }

    #[test]
    fn velocity_vec3_conversion() {
        let velocity = Velocity::new(5.0, -3.0).unwrap();
        let vec3 = velocity_to_vec3(&velocity);

        assert_eq!(vec3.x, 5.0);
        assert_eq!(vec3.y, -3.0);
        assert_eq!(vec3.z, 0.0);

        let converted_back = vec3_to_velocity(&vec3).unwrap();
        assert_eq!(converted_back, velocity);
    }

    #[test]
    fn bevy_plugin_can_be_created() {
        // Test that plugins can be instantiated
        let _game_plugin = BevyGamePlugin;
        let _systems_plugin = BevySystemsPlugin;
    }
}
