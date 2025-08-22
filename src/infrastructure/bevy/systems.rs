//! Bevy Systems - Game Logic Implementation
//!
//! This module contains Bevy systems that implement the game logic by
//! executing domain use cases and coordinating between ECS components.

use crate::domain::constants;
use crate::infrastructure::bevy::{components::*, resources::*};
use bevy::prelude::*;

/// Setup the game camera
pub fn setup_camera(mut commands: Commands) {
    info!("Setting up game camera");
    commands.spawn(Camera2d);
}

/// Setup the game UI (placeholder)
pub fn setup_ui(mut commands: Commands) {
    info!("Setting up game UI");

    // Spawn UI root node
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Start,
            ..default()
        })
        .with_children(|parent| {
            // Score display
            parent.spawn((
                Text::new("Score: 0"),
                TextColor(Color::WHITE),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                ScoreDisplayComponent,
            ));
        });
}

/// Spawn the player entity
pub fn spawn_player(mut commands: Commands) {
    info!("Spawning player");

    // Create domain player entity
    let position = crate::domain::Position3D::new(0, -250, 0);
    let entity_id = crate::domain::EntityId::generate();
    let starting_stats = crate::domain::PlayerStats::new(10, 10, 10, 10, 10, 10).unwrap();

    let player =
        crate::domain::Player::new(entity_id, "player_1".to_string(), position, starting_stats)
            .expect("Failed to create player");

    // Create velocity for compatibility
    let velocity = crate::domain::Velocity::new(0.0, 0.0).unwrap();

    // Spawn Bevy entity with components
    commands.spawn((
        Sprite {
            color: Color::srgb(0.0, 0.8, 0.0),
            custom_size: Some(Vec2::new(
                constants::PLAYER_SIZE.0,
                constants::PLAYER_SIZE.1,
            )),
            ..default()
        },
        Transform::from_translation(Vec3::new(
            position.x() as f32,
            position.y() as f32,
            position.z() as f32,
        )),
        PlayerComponent::new(player),
        VelocityComponent::new(velocity),
    ));
}

/// Handle player input and movement
pub fn player_input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut PlayerComponent, &mut VelocityComponent)>,
) {
    for (_player_comp, mut velocity_comp) in query.iter_mut() {
        let mut direction_x = 0.0;
        let mut direction_y = 0.0;

        // Process keyboard input
        if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
            direction_x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
            direction_x += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
            direction_y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
            direction_y -= 1.0;
        }

        // Update velocity based on input
        if let Ok(new_velocity) =
            crate::domain::Velocity::from_direction((direction_x, direction_y), 200.0)
        {
            *velocity_comp.velocity_mut() = new_velocity;
        }
    }
}

/// Update entity positions based on velocity
pub fn movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &VelocityComponent)>,
    game_boundaries: Option<Res<GameBoundariesResource>>,
) {
    let default_boundaries = crate::domain::GameBoundaries::standard();
    let boundaries = if let Some(res) = game_boundaries.as_ref() {
        &res.boundaries
    } else {
        &default_boundaries
    };

    for (mut transform, velocity_comp) in query.iter_mut() {
        // Convert current transform to domain position
        let current_position = crate::domain::Position3D::new(
            transform.translation.x as i32,
            transform.translation.y as i32,
            transform.translation.z as i32,
        );

        // Calculate new position with simple velocity integration
        let velocity = velocity_comp.velocity();
        let new_x = current_position.x() as f32 + velocity.dx() * time.delta_secs();
        let new_y = current_position.y() as f32 + velocity.dy() * time.delta_secs();

        let new_position =
            crate::domain::Position3D::new(new_x as i32, new_y as i32, current_position.z());

        // Clamp to game boundaries
        let clamped_position = boundaries.clamp(new_position);

        // Update transform
        transform.translation.x = clamped_position.x() as f32;
        transform.translation.y = clamped_position.y() as f32;
    }
}

/// Spawn enemies at regular intervals
pub fn enemy_spawning_system(mut commands: Commands, time: Res<Time>, mut timer: Local<Timer>) {
    // Initialize timer on first run
    if timer.duration().as_secs_f32() == 0.0 {
        *timer = Timer::from_seconds(constants::ENEMY_SPAWN_INTERVAL, TimerMode::Repeating);
    }

    timer.tick(time.delta());

    if timer.just_finished() {
        // Generate random spawn position at top of screen
        let x_pos = (simple_random() - 0.5) * 600.0;
        let spawn_position = crate::domain::Position3D::new(x_pos as i32, 300, 0);

        let enemy_velocity = crate::domain::Velocity::new(0.0, -constants::DEFAULT_ENEMY_SPEED)
            .expect("Invalid enemy velocity");

        // Create domain enemy
        let enemy_id = format!("enemy_{}", time.elapsed_secs_f64());
        let spawn_position_3d = spawn_position;
        let enemy = crate::domain::Enemy::new(
            enemy_id,
            spawn_position_3d,
            enemy_velocity,
            crate::domain::EnemyType::Basic,
        )
        .expect("Failed to create enemy");

        // Spawn Bevy entity
        commands.spawn((
            Sprite {
                color: Color::srgb(0.8, 0.0, 0.0),
                custom_size: Some(Vec2::new(constants::ENEMY_SIZE.0, constants::ENEMY_SIZE.1)),
                ..default()
            },
            Transform::from_translation(Vec3::new(
                spawn_position_3d.x() as f32,
                spawn_position_3d.y() as f32,
                spawn_position_3d.z() as f32,
            )),
            EnemyComponent::new(enemy),
            VelocityComponent::new(enemy_velocity),
        ));

        debug!("Spawned enemy at position: {:?}", spawn_position);
    }
}

/// Handle collisions between entities
pub fn collision_system(
    mut commands: Commands,
    player_query: Query<(Entity, &Transform, &PlayerComponent), Without<EnemyComponent>>,
    enemy_query: Query<(Entity, &Transform, &EnemyComponent), Without<PlayerComponent>>,
    mut score_resource: ResMut<ScoreResource>,
) {
    for (_player_entity, player_transform, _player_comp) in player_query.iter() {
        for (enemy_entity, enemy_transform, _enemy_comp) in enemy_query.iter() {
            let distance = player_transform
                .translation
                .distance(enemy_transform.translation);

            if distance < constants::COLLISION_RADIUS {
                // Collision detected - remove enemy and update score
                commands.entity(enemy_entity).despawn();

                if let Ok(()) = score_resource.score.add_enemy_points() {
                    info!("Collision! New score: {}", score_resource.score);
                } else {
                    warn!("Failed to add enemy points to score");
                }
            }
        }
    }
}

/// Clean up off-screen entities
pub fn cleanup_system(
    mut commands: Commands,
    enemy_query: Query<(Entity, &Transform), With<EnemyComponent>>,
) {
    for (entity, transform) in enemy_query.iter() {
        // Remove enemies that have moved off the bottom of the screen
        if transform.translation.y < -350.0 {
            commands.entity(entity).despawn();
            debug!("Cleaned up off-screen enemy");
        }
    }
}

/// Update score display in UI
pub fn update_score_display_system(
    score_resource: Res<ScoreResource>,
    mut query: Query<&mut Text, With<ScoreDisplayComponent>>,
) {
    if score_resource.is_changed() {
        for mut text in query.iter_mut() {
            **text = format!("Score: {}", score_resource.score);
        }
    }
}

/// Placeholder system for testing
pub fn placeholder_system() {
    // This system does nothing but can be used for testing
    // It will be replaced with proper systems as development progresses
}

/// Simple random number generator for web compatibility
fn simple_random() -> f32 {
    use std::cell::RefCell;

    thread_local! {
        static RNG_STATE: RefCell<u32> = RefCell::new(1234567890);
    }

    RNG_STATE.with(|state| {
        let mut s = state.borrow_mut();
        *s = (*s).wrapping_mul(1103515245).wrapping_add(12345);
        (*s >> 16) as f32 / 65536.0
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_random_generates_values() {
        let val1 = simple_random();
        let val2 = simple_random();

        assert!(val1 >= 0.0 && val1 <= 1.0);
        assert!(val2 >= 0.0 && val2 <= 1.0);
        assert_ne!(val1, val2); // Should generate different values
    }

    #[test]
    fn systems_can_be_called() {
        // Test that systems can be instantiated (compilation test)
        placeholder_system();
    }
}
