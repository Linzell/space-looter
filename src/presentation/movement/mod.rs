//! Smooth Movement System - Tile-to-Tile Animation
//!
//! This module provides smooth tile-to-tile movement transitions with:
//! - Smooth interpolation between positions
//! - Movement state tracking to prevent input during transitions
//! - Camera following during movement
//! - Configurable animation timing
//! - Integration with dice-based movement system
//! - Support for both keyboard and click/touch input (mobile-friendly)
//!
//! ## Usage
//!
//! ### Basic Setup
//! ```rust
//! use crate::presentation::movement::{SmoothMovementPlugin, MovementConfig};
//!
//! // Add the plugin to your app
//! app.add_plugins(SmoothMovementPlugin);
//!
//! // Configure movement settings (optional - uses defaults if not provided)
//! app.insert_resource(MovementConfig {
//!     base_duration_ms: 1500,          // 1.5 seconds per tile
//!     enable_click_to_move: true,      // Enable mobile click/touch
//!     enable_keyboard_movement: true,  // Enable WASD/arrow keys
//!     block_input_during_movement: true, // Prevent input spam
//!     ..Default::default()
//! });
//! ```
//!
//! ### Mobile Configuration
//! ```rust
//! // For mobile-only games
//! app.insert_resource(MovementConfig {
//!     enable_click_to_move: true,
//!     enable_keyboard_movement: false,  // Disable for mobile-only
//!     allow_diagonal_click_movement: true, // Allow 8-direction click movement
//!     ..Default::default()
//! });
//! ```
//!
//! ### Desktop Configuration
//! ```rust
//! // For desktop-only games
//! app.insert_resource(MovementConfig {
//!     enable_click_to_move: false,     // Disable click movement
//!     enable_keyboard_movement: true,
//!     base_duration_ms: 800,           // Faster movement for responsive gameplay
//!     ..Default::default()
//! });
//! ```
//!
//! ### Player Entity Setup
//! Your player entity needs the `SmoothMovement` component:
//! ```rust
//! commands.spawn((
//!     // ... other components (mesh, material, etc.)
//!     SmoothMovement::new(Position3D::new(0, 0, 0)),
//!     PlayerMarker,
//!     Transform::from_translation(world_position),
//! ));
//! ```
//!
//! ## Input Methods
//!
//! ### Keyboard (Desktop)
//! - **WASD** or **Arrow Keys** for movement
//! - Only cardinal directions (North, South, East, West)
//! - Input is blocked during movement animations (prevents spam)
//!
//! ### Click/Touch (Mobile)
//! - **Left Click** or **Touch** on adjacent tiles
//! - Can be configured for cardinal-only or 8-direction movement
//! - Visual feedback available (configure `show_tile_highlights: true`)
//!
//! ## Integration
//! The system integrates with your existing RPG movement logic:
//! 1. Player inputs movement (keyboard/click)
//! 2. Animation starts immediately (visual feedback)
//! 3. When animation completes, RPG movement logic executes
//! 4. This ensures smooth visuals while maintaining game rules
//!

use crate::domain::value_objects::position::{Direction, Position3D};
use bevy::prelude::*;
use std::time::Duration;

/// Plugin for smooth movement system
pub struct SmoothMovementPlugin;

impl Plugin for SmoothMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_player_movement_input,
                handle_click_movement_input,
                update_movement_animations,
                start_movement_transitions,
                update_camera_following,
            )
                .chain(),
        )
        .add_event::<MovementStarted>()
        .add_event::<MovementCompleted>()
        .add_event::<ExecuteRpgMovement>()
        .add_event::<TileClickEvent>()
        .add_event::<RestingTriggered>()
        .init_resource::<MovementConfig>()
        .add_systems(
            Update,
            check_for_zero_movement_points
                .run_if(any_with_component::<crate::presentation::map_renderer::PlayerMarker>),
        );
    }
}

/// Component for entities that can move smoothly between tiles
#[derive(Component, Debug, Clone)]
pub struct SmoothMovement {
    /// Current world position (may be between tiles during animation)
    pub current_position: Vec3,
    /// Target tile position
    pub target_position: Position3D,
    /// Starting position for current animation
    pub start_position: Position3D,
    /// Animation progress (0.0 to 1.0)
    pub progress: f32,
    /// Whether currently animating
    pub is_moving: bool,
    /// Animation duration
    pub duration: Duration,
    /// Time elapsed in current animation
    pub elapsed: Duration,
    /// Movement speed modifier
    pub speed_multiplier: f32,
}

impl Default for SmoothMovement {
    fn default() -> Self {
        Self {
            current_position: Vec3::ZERO,
            target_position: Position3D::origin(),
            start_position: Position3D::origin(),
            progress: 1.0,
            is_moving: false,
            duration: Duration::from_millis(1000),
            elapsed: Duration::ZERO,
            speed_multiplier: 1.0,
        }
    }
}

impl SmoothMovement {
    /// Create new smooth movement component at position
    pub fn new(position: Position3D) -> Self {
        let world_pos = tile_to_world_position(position);
        Self {
            current_position: world_pos,
            target_position: position,
            start_position: position,
            progress: 1.0,
            is_moving: false,
            duration: Duration::from_millis(1000),
            elapsed: Duration::ZERO,
            speed_multiplier: 1.0,
        }
    }

    /// Start movement to new position
    pub fn start_movement(&mut self, target: Position3D, config: &MovementConfig) {
        if self.is_moving {
            return; // Don't interrupt current movement
        }

        self.start_position = self.target_position;
        self.target_position = target;
        self.is_moving = true;
        self.progress = 0.0;
        self.elapsed = Duration::ZERO;
        self.duration =
            Duration::from_millis((config.base_duration_ms as f32 / self.speed_multiplier) as u64);
    }

    /// Check if movement is complete
    pub fn is_movement_complete(&self) -> bool {
        !self.is_moving || self.progress >= 1.0
    }

    /// Force complete current movement
    pub fn complete_movement(&mut self) {
        if self.is_moving {
            self.progress = 1.0;
            self.is_moving = false;
            self.current_position = tile_to_world_position(self.target_position);
            self.elapsed = Duration::ZERO;
        }
    }

    /// Update animation progress
    pub fn update(&mut self, delta_time: Duration) {
        if !self.is_moving {
            return;
        }

        self.elapsed += delta_time;
        self.progress =
            (self.elapsed.as_millis() as f32 / self.duration.as_millis() as f32).min(1.0);

        // Use easing function for smoother animation
        let eased_progress = ease_in_out_cubic(self.progress);

        let start_world = tile_to_world_position(self.start_position);
        let target_world = tile_to_world_position(self.target_position);

        self.current_position = start_world.lerp(target_world, eased_progress);

        if self.progress >= 1.0 {
            self.is_moving = false;
            self.current_position = target_world;
            self.elapsed = Duration::ZERO;
        }
    }

    /// Start movement to new position with custom duration
    pub fn start_movement_with_duration(&mut self, target: Position3D, duration_seconds: f32) {
        if self.is_moving {
            return; // Don't interrupt current movement
        }

        self.start_position = self.target_position;
        self.target_position = target;
        self.is_moving = true;
        self.progress = 0.0;
        self.elapsed = Duration::ZERO;
        self.duration = Duration::from_millis((duration_seconds * 1000.0) as u64);
    }

    /// Reset to a specific position (used when movement fails)
    pub fn reset_to_position(&mut self, position: Position3D) {
        self.target_position = position;
        self.start_position = position;
        self.current_position = tile_to_world_position(position);
        self.is_moving = false;
        self.progress = 1.0;
        self.elapsed = Duration::ZERO;
    }

    /// Set movement speed multiplier
    pub fn set_speed_multiplier(&mut self, multiplier: f32) {
        self.speed_multiplier = multiplier.max(0.1); // Minimum speed
    }
}

/// Configuration for movement animations
#[derive(Resource, Debug, Clone)]
pub struct MovementConfig {
    /// Base duration for tile-to-tile movement in milliseconds
    pub base_duration_ms: u32,
    /// Whether to use easing curves
    pub use_easing: bool,
    /// Camera follow speed
    pub camera_follow_speed: f32,
    /// Whether to prevent input during movement
    pub block_input_during_movement: bool,
    /// Enable click/touch to move for mobile
    pub enable_click_to_move: bool,
    /// Enable keyboard movement
    pub enable_keyboard_movement: bool,
    /// Show visual feedback for clickable tiles
    pub show_tile_highlights: bool,
    /// Allow diagonal movement via click (keyboard always cardinal only)
    pub allow_diagonal_click_movement: bool,
}

impl Default for MovementConfig {
    fn default() -> Self {
        Self {
            base_duration_ms: 1000, // 1 second per tile
            use_easing: true,
            camera_follow_speed: 2.0,
            block_input_during_movement: true,
            enable_click_to_move: true,
            enable_keyboard_movement: true,
            show_tile_highlights: false, // Disabled by default for performance
            allow_diagonal_click_movement: false, // Keep consistent with keyboard
        }
    }
}

/// Event triggered when movement starts
#[derive(Event, Debug, Clone)]
pub struct MovementStarted {
    pub entity: Entity,
    pub from: Position3D,
    pub to: Position3D,
}

/// Event triggered when movement completes
#[derive(Event, Debug, Clone)]
pub struct MovementCompleted {
    pub entity: Entity,
    pub final_position: Position3D,
}

/// Component to mark entities as movement blockers during animation
#[derive(Component, Debug)]
pub struct MovementBlocked {
    pub reason: String,
}

/// Component for camera that follows smooth movement
#[derive(Component, Debug)]
pub struct CameraFollowsMovement {
    pub target_entity: Option<Entity>,
    pub offset: Vec3,
    pub follow_speed: f32,
}

impl Default for CameraFollowsMovement {
    fn default() -> Self {
        Self {
            target_entity: None,
            offset: Vec3::new(0.0, 10.0, 10.0),
            follow_speed: 2.0,
        }
    }
}

/// Event to execute movement through RPG system after animation
#[derive(Event, Debug, Clone)]
pub struct ExecuteRpgMovement {
    pub direction: Direction,
    pub target_position: Position3D,
    pub entity: Entity,
}

/// Click input for movement
#[derive(Event, Debug, Clone)]
pub struct TileClickEvent {
    pub tile_position: Position3D,
    pub screen_position: Vec2,
}

/// Component for visual feedback when hovering/clicking tiles
#[derive(Component, Debug)]
pub struct TileHighlight {
    pub highlight_type: HighlightType,
    pub fade_timer: Timer,
}

/// Types of tile highlights
#[derive(Debug, Clone, PartialEq)]
pub enum HighlightType {
    Clickable,
    Invalid,
    Path,
}

/// Event triggered when player needs to rest (zero movement points)
#[derive(Event, Debug)]
pub struct RestingTriggered {
    pub player_position: Position3D,
    pub remaining_movement_points: u8,
}

/// System to update movement animations
pub fn update_movement_animations(
    time: Res<Time>,
    mut movement_query: Query<(Entity, &mut SmoothMovement, &mut Transform)>,
    mut movement_completed_events: EventWriter<MovementCompleted>,
) {
    for (entity, mut smooth_movement, mut transform) in movement_query.iter_mut() {
        let was_moving = smooth_movement.is_moving;
        smooth_movement.update(time.delta());

        // Update transform position
        transform.translation = smooth_movement.current_position;

        // Send completion event if movement just finished
        if was_moving && smooth_movement.is_movement_complete() {
            info!(
                "🎮 Smooth movement: Animation completed at {:?}",
                smooth_movement.target_position
            );
            movement_completed_events.write(MovementCompleted {
                entity,
                final_position: smooth_movement.target_position,
            });
        }
    }
}

/// System to handle player movement input with blocking during animations
/// This system runs BEFORE the RPG exploration system to intercept and control movement
pub fn handle_player_movement_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<
        (&mut SmoothMovement, Entity),
        With<crate::presentation::map_renderer::PlayerMarker>,
    >,
    mut player_resource: ResMut<crate::infrastructure::bevy::resources::PlayerResource>,
    map_resource: Res<crate::infrastructure::bevy::resources::MapResource>,
    config: Res<MovementConfig>,
    mut movement_started_events: EventWriter<MovementStarted>,
    mut execute_rpg_events: EventWriter<ExecuteRpgMovement>,
) {
    if !player_resource.has_player() || !config.enable_keyboard_movement {
        return;
    }

    // Check if movement is blocked
    if let Ok((smooth_movement, _)) = player_query.single() {
        if config.block_input_during_movement && smooth_movement.is_moving {
            return; // Block input during movement
        }
    }

    // Check if player has movement points
    if let Some(player) = player_resource.get_player() {
        if !player.can_move() {
            return; // No movement points available
        }
    }

    let mut movement_direction: Option<Direction> = None;

    // Check for movement input (only process one direction at a time for tile-based movement)
    if keyboard_input.just_pressed(KeyCode::KeyW) || keyboard_input.just_pressed(KeyCode::ArrowUp) {
        movement_direction = Some(Direction::South);
    } else if keyboard_input.just_pressed(KeyCode::KeyS)
        || keyboard_input.just_pressed(KeyCode::ArrowDown)
    {
        movement_direction = Some(Direction::North);
    } else if keyboard_input.just_pressed(KeyCode::KeyA)
        || keyboard_input.just_pressed(KeyCode::ArrowLeft)
    {
        movement_direction = Some(Direction::West);
    } else if keyboard_input.just_pressed(KeyCode::KeyD)
        || keyboard_input.just_pressed(KeyCode::ArrowRight)
    {
        movement_direction = Some(Direction::East);
    }

    if let Some(direction) = movement_direction {
        if let Ok((mut smooth_movement, entity)) = player_query.single_mut() {
            // Calculate new target position
            let current_tile = smooth_movement.target_position;
            let new_target = current_tile.move_direction(direction, 1);

            // Pre-validate movement before starting animation
            let current_position = player_resource.player_position().unwrap_or_default();
            let map = map_resource.current_map();
            let movement_cost = if let Some(map) = map {
                map.movement_cost(&new_target)
            } else {
                crate::domain::constants::BASE_MOVEMENT_COST
            };

            // Check if player has enough movement points for this specific movement
            if let Some(player) = player_resource.get_player() {
                if player.movement_points() < movement_cost {
                    info!(
                        "⚡ Movement blocked! Need: {} points, Have: {}",
                        movement_cost,
                        player.movement_points()
                    );
                    return;
                }
            }

            // Check basic adjacency (RPG system will do full validation)
            if !is_adjacent_tile(current_tile, new_target) {
                info!("🚫 Movement blocked! Target not adjacent to current position");
                return;
            }

            info!(
                "🎮 Smooth movement: Starting animation from {:?} to {:?}",
                current_tile, new_target
            );

            // Start movement animation with default duration - will be updated after RPG validation
            smooth_movement.start_movement(new_target, &config);

            // Send movement started event
            movement_started_events.write(MovementStarted {
                entity,
                from: current_tile,
                to: new_target,
            });

            // Send RPG movement event immediately
            execute_rpg_events.write(ExecuteRpgMovement {
                direction,
                target_position: new_target,
                entity,
            });

            info!(
                "🎮 Smooth movement: Sent RPG movement event for {:?}",
                new_target
            );
        }
    }
}

/// System to handle click/touch movement input for mobile
/// This system also runs BEFORE the RPG exploration system
pub fn handle_click_movement_input(
    mut mouse_button_input: ResMut<ButtonInput<MouseButton>>,
    mut touch_events: EventReader<bevy::input::touch::TouchInput>,
    windows: Query<&Window>,
    camera_query: Query<
        (&Camera, &GlobalTransform),
        With<crate::presentation::map_renderer::IsometricCamera>,
    >,
    mut player_query: Query<
        (&mut SmoothMovement, Entity),
        With<crate::presentation::map_renderer::PlayerMarker>,
    >,
    mut player_resource: ResMut<crate::infrastructure::bevy::resources::PlayerResource>,
    map_resource: Res<crate::infrastructure::bevy::resources::MapResource>,
    config: Res<MovementConfig>,
    mut movement_started_events: EventWriter<MovementStarted>,
    mut execute_rpg_events: EventWriter<ExecuteRpgMovement>,
) {
    if !player_resource.has_player() || !config.enable_click_to_move {
        return;
    }

    // Check if movement is blocked
    if let Ok((smooth_movement, _)) = player_query.single() {
        if config.block_input_during_movement && smooth_movement.is_moving {
            return; // Block input during movement
        }
    }

    // Check if player has movement points
    if let Some(player) = player_resource.get_player() {
        if !player.can_move() {
            return; // No movement points available
        }
    }

    let mut click_position: Option<Vec2> = None;

    // Handle mouse clicks
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Ok(window) = windows.single() {
            if let Some(cursor_pos) = window.cursor_position() {
                click_position = Some(cursor_pos);
                mouse_button_input.clear_just_pressed(MouseButton::Left);
                // Prevent other systems from processing
            }
        }
    }

    // Handle touch input
    for touch in touch_events.read() {
        if matches!(touch.phase, bevy::input::touch::TouchPhase::Started) {
            click_position = Some(touch.position);
        }
    }

    if let Some(screen_pos) = click_position {
        if let Ok((camera, camera_transform)) = camera_query.single() {
            if let Ok((mut smooth_movement, entity)) = player_query.single_mut() {
                // Convert screen position to world position and then to tile coordinates
                if let Some(clicked_tile) =
                    screen_to_tile_position(screen_pos, camera, camera_transform)
                {
                    let current_tile = smooth_movement.target_position;

                    info!("📱 Click detected at tile: {:?}", clicked_tile);

                    // Check if movement is allowed (adjacent for cardinal, or diagonal if enabled)
                    if is_valid_click_movement(current_tile, clicked_tile, &config) {
                        // Pre-validate movement before starting animation
                        let current_position =
                            player_resource.player_position().unwrap_or_default();
                        let map = map_resource.current_map();
                        let movement_cost = if let Some(map) = map {
                            map.movement_cost(&clicked_tile)
                        } else {
                            crate::domain::constants::BASE_MOVEMENT_COST
                        };

                        // Check if player has enough movement points for this specific movement
                        if let Some(player) = player_resource.get_player() {
                            if player.movement_points() < movement_cost {
                                info!(
                                    "⚡ Click movement blocked! Need: {} points, Have: {}",
                                    movement_cost,
                                    player.movement_points()
                                );
                                return;
                            }
                        }

                        let direction = calculate_direction(current_tile, clicked_tile)
                            .unwrap_or(Direction::North);

                        info!(
                            "🖱️ Click movement: Starting animation from {:?} to {:?}",
                            current_tile, clicked_tile
                        );

                        // Start movement animation with default duration - will be updated after RPG validation
                        smooth_movement.start_movement(clicked_tile, &config);

                        // Send movement started event
                        movement_started_events.write(MovementStarted {
                            entity,
                            from: current_tile,
                            to: clicked_tile,
                        });

                        // Send RPG movement event immediately
                        execute_rpg_events.write(ExecuteRpgMovement {
                            direction,
                            target_position: clicked_tile,
                            entity,
                        });

                        info!(
                            "🖱️ Click movement: Sent RPG movement event for {:?}",
                            clicked_tile
                        );
                    } else {
                        info!("🖱️ Click movement blocked! Target not adjacent or not allowed");
                    }
                }
            }
        }
    }
}

/// System to start movement transitions based on position changes
pub fn start_movement_transitions() {
    // This system would be triggered by movement commands
    // For now, it's a placeholder that would integrate with your input system
}

/// System to update camera following smooth movement
pub fn update_camera_following(
    time: Res<Time>,
    movement_query: Query<&SmoothMovement>,
    mut camera_query: Query<(&mut Transform, &CameraFollowsMovement), Without<SmoothMovement>>,
) {
    for (mut camera_transform, camera_follow) in camera_query.iter_mut() {
        if let Some(target_entity) = camera_follow.target_entity {
            if let Ok(smooth_movement) = movement_query.get(target_entity) {
                let target_position = smooth_movement.current_position + camera_follow.offset;
                let follow_speed = camera_follow.follow_speed;

                camera_transform.translation = camera_transform
                    .translation
                    .lerp(target_position, follow_speed * time.delta().as_secs_f32());
            }
        }
    }
}

/// Convert tile position to world position
pub fn tile_to_world_position(tile_pos: Position3D) -> Vec3 {
    // Use the same coordinate conversion as the existing tile_to_world_position in map_renderer
    let x = (tile_pos.x as f32) * 2.2; // Proper spacing for 2.0 unit wide tiles
    let z = (tile_pos.y as f32) * 2.2;
    let y = (tile_pos.z as f32) * 0.3; // Layer height

    Vec3::new(x, y, z)
}

/// Cubic ease-in-out function for smooth animation
fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

/// Helper function to check if any entity with smooth movement is currently moving
pub fn any_entity_moving(movement_query: &Query<&SmoothMovement>) -> bool {
    movement_query.iter().any(|sm| sm.is_moving)
}

/// Helper function to block movement input during transitions
pub fn should_block_movement_input(
    movement_query: &Query<&SmoothMovement>,
    config: &MovementConfig,
) -> bool {
    config.block_input_during_movement && any_entity_moving(movement_query)
}

/// Convert screen position to tile position
fn screen_to_tile_position(
    screen_pos: Vec2,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Position3D> {
    // Cast ray from screen position to world
    if let Ok(ray) = camera.viewport_to_world(camera_transform, screen_pos) {
        // Cast ray to ground plane (y = 0)
        let ground_y = 0.0;
        let ray_origin = ray.origin;
        let ray_direction = ray.direction;

        if ray_direction.y.abs() > 0.001 {
            let t = (ground_y - ray_origin.y) / ray_direction.y;
            if t > 0.0 {
                let world_pos = ray_origin + ray_direction * t;

                // Convert world position back to tile coordinates
                // This is the inverse of the existing tile_to_world_position function
                // x = tile_x * 2.2, z = tile_y * 2.2
                let tile_x = (world_pos.x / 2.2).round() as i32;
                let tile_y = (world_pos.z / 2.2).round() as i32;
                let tile_z = (world_pos.y / 0.3).round() as i32;

                return Some(Position3D::new(tile_x, tile_y, tile_z));
            }
        }
    }
    None
}

/// Check if click movement is valid based on configuration
fn is_valid_click_movement(from: Position3D, to: Position3D, config: &MovementConfig) -> bool {
    if config.allow_diagonal_click_movement {
        is_adjacent_tile_including_diagonals(from, to)
    } else {
        is_adjacent_tile(from, to)
    }
}

/// Check if two tiles are adjacent including diagonals
fn is_adjacent_tile_including_diagonals(from: Position3D, to: Position3D) -> bool {
    let dx = (to.x - from.x).abs();
    let dy = (to.y - from.y).abs();
    let dz = (to.z - from.z).abs();

    // Adjacent tiles are within 1 step in any direction (including diagonals)
    dx <= 1 && dy <= 1 && dz <= 1 && (dx + dy + dz > 0)
}

/// Check if two tiles are adjacent (only cardinal directions for movement)
fn is_adjacent_tile(from: Position3D, to: Position3D) -> bool {
    let dx = (to.x - from.x).abs();
    let dy = (to.y - from.y).abs();
    let dz = (to.z - from.z).abs();

    // Only allow cardinal directions (no diagonals) for consistent movement
    match (dx, dy, dz) {
        (1, 0, 0) | (0, 1, 0) | (0, 0, 1) => true,
        _ => false,
    }
}

/// System to check if player has zero movement points and trigger resting
pub fn check_for_zero_movement_points(
    player_resource: Res<crate::infrastructure::bevy::resources::PlayerResource>,
    mut resting_events: EventWriter<RestingTriggered>,
    mut already_triggered: Local<bool>,
) {
    if let Some(player) = player_resource.get_player() {
        if player.movement_points() == 0 && !*already_triggered {
            let current_position = player_resource.player_position().unwrap_or_default();

            info!(
                "⚡ Movement points exhausted! Triggering automatic rest at {:?}",
                current_position
            );

            resting_events.write(RestingTriggered {
                player_position: current_position,
                remaining_movement_points: player.movement_points(),
            });

            *already_triggered = true;
        } else if player.movement_points() > 0 {
            *already_triggered = false;
        }
    }
}

/// Calculate direction from one tile to adjacent tile
fn calculate_direction(from: Position3D, to: Position3D) -> Option<Direction> {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let dz = to.z - from.z;

    // Handle cardinal directions (always supported)
    if dz == 0 {
        match (dx, dy) {
            (0, 1) => Some(Direction::North),
            (0, -1) => Some(Direction::South),
            (1, 0) => Some(Direction::East),
            (-1, 0) => Some(Direction::West),
            // For diagonal movement, pick the primary direction
            // This is a simplified approach - you might want more sophisticated logic
            (1, 1) => Some(Direction::North),   // NE -> N
            (1, -1) => Some(Direction::South),  // SE -> S
            (-1, 1) => Some(Direction::North),  // NW -> N
            (-1, -1) => Some(Direction::South), // SW -> S
            _ => None,
        }
    } else if dx == 0 && dy == 0 {
        match dz {
            1 => Some(Direction::Up),
            -1 => Some(Direction::Down),
            _ => None,
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smooth_movement_creation() {
        let pos = Position3D::new(1, 2, 0);
        let smooth = SmoothMovement::new(pos);

        assert_eq!(smooth.target_position, pos);
        assert!(!smooth.is_moving);
        assert_eq!(smooth.progress, 1.0);
    }

    #[test]
    fn smooth_movement_start() {
        let mut smooth = SmoothMovement::new(Position3D::origin());
        let target = Position3D::new(1, 0, 0);
        let config = MovementConfig::default();

        smooth.start_movement(target, &config);

        assert!(smooth.is_moving);
        assert_eq!(smooth.progress, 0.0);
        assert_eq!(smooth.target_position, target);
    }

    #[test]
    fn smooth_movement_update() {
        let mut smooth = SmoothMovement::new(Position3D::origin());
        let target = Position3D::new(1, 0, 0);
        let config = MovementConfig::default();

        smooth.start_movement(target, &config);

        // Update with half the duration
        smooth.update(Duration::from_millis(500));

        assert!(smooth.is_moving);
        assert!(smooth.progress > 0.0 && smooth.progress < 1.0);
    }

    #[test]
    fn smooth_movement_completion() {
        let mut smooth = SmoothMovement::new(Position3D::origin());
        let target = Position3D::new(1, 0, 0);
        let config = MovementConfig::default();

        smooth.start_movement(target, &config);

        // Update with full duration
        smooth.update(Duration::from_millis(1000));

        assert!(!smooth.is_moving);
        assert_eq!(smooth.progress, 1.0);
    }

    #[test]
    fn easing_function() {
        assert_eq!(ease_in_out_cubic(0.0), 0.0);
        assert_eq!(ease_in_out_cubic(1.0), 1.0);

        let mid = ease_in_out_cubic(0.5);
        assert!(mid > 0.4 && mid < 0.6); // Should be around 0.5
    }

    #[test]
    fn tile_to_world_conversion() {
        let tile_pos = Position3D::new(1, 1, 0);
        let world_pos = tile_to_world_position(tile_pos);

        // Basic sanity check
        assert!(world_pos.x != 0.0 || world_pos.z != 0.0);
    }

    #[test]
    fn movement_config_default() {
        let config = MovementConfig::default();

        assert_eq!(config.base_duration_ms, 1000);
        assert!(config.use_easing);
        assert!(config.block_input_during_movement);
        assert!(config.enable_click_to_move);
        assert!(config.enable_keyboard_movement);
    }

    #[test]
    fn speed_multiplier() {
        let mut smooth = SmoothMovement::new(Position3D::origin());

        smooth.set_speed_multiplier(2.0);
        assert_eq!(smooth.speed_multiplier, 2.0);

        // Test minimum speed
        smooth.set_speed_multiplier(0.05);
        assert_eq!(smooth.speed_multiplier, 0.1);
    }

    #[test]
    fn movement_blocking() {
        let mut smooth = SmoothMovement::new(Position3D::origin());
        let target1 = Position3D::new(1, 0, 0);
        let target2 = Position3D::new(2, 0, 0);
        let config = MovementConfig::default();

        // Start first movement
        smooth.start_movement(target1, &config);
        assert!(smooth.is_moving);

        // Try to start second movement - should be blocked
        smooth.start_movement(target2, &config);
        assert_eq!(smooth.target_position, target1); // Should still be moving to target1
    }

    #[test]
    fn smooth_movement_integration() {
        use bevy::app::App;

        let mut app = App::new();
        app.add_plugins(SmoothMovementPlugin);

        // Test that the plugin adds systems
        {
            let world = app.world();
            assert!(world.contains_resource::<MovementConfig>());
        }

        // Create a test entity with smooth movement
        let entity = app
            .world_mut()
            .spawn(SmoothMovement::new(Position3D::origin()))
            .id();

        // Simulate one update
        app.update();

        // Verify entity still exists and has correct component
        let world = app.world();
        let smooth_movement = world.get::<SmoothMovement>(entity).unwrap();
        assert!(!smooth_movement.is_moving);
        assert_eq!(smooth_movement.target_position, Position3D::origin());
    }

    #[test]
    fn movement_animation_completes() {
        let mut smooth = SmoothMovement::new(Position3D::origin());
        let target = Position3D::new(1, 0, 0);
        let config = MovementConfig {
            base_duration_ms: 100, // Short duration for test
            ..Default::default()
        };

        smooth.start_movement(target, &config);
        assert!(smooth.is_moving);

        // Update with full duration should complete movement
        smooth.update(Duration::from_millis(100));

        assert!(!smooth.is_moving);
        assert_eq!(smooth.progress, 1.0);
        assert_eq!(smooth.target_position, target);
    }

    #[test]
    fn camera_follows_movement_setup() {
        let camera_follow = CameraFollowsMovement::default();

        assert!(camera_follow.target_entity.is_none());
        assert_eq!(camera_follow.offset, Vec3::new(0.0, 10.0, 10.0));
        assert_eq!(camera_follow.follow_speed, 2.0);
    }

    #[test]
    fn movement_events() {
        let entity = Entity::from_raw(42);
        let from_pos = Position3D::origin();
        let to_pos = Position3D::new(1, 0, 0);

        let start_event = MovementStarted {
            entity,
            from: from_pos,
            to: to_pos,
        };

        let complete_event = MovementCompleted {
            entity,
            final_position: to_pos,
        };

        assert_eq!(start_event.entity, entity);
        assert_eq!(start_event.from, from_pos);
        assert_eq!(start_event.to, to_pos);

        assert_eq!(complete_event.entity, entity);
        assert_eq!(complete_event.final_position, to_pos);
    }

    #[test]
    fn adjacent_tile_detection() {
        let origin = Position3D::origin();

        // Test adjacent tiles
        assert!(is_adjacent_tile(origin, Position3D::new(1, 0, 0)));
        assert!(is_adjacent_tile(origin, Position3D::new(0, 1, 0)));
        assert!(is_adjacent_tile(origin, Position3D::new(-1, 0, 0)));
        assert!(is_adjacent_tile(origin, Position3D::new(0, -1, 0)));

        // Test non-adjacent tiles
        assert!(!is_adjacent_tile(origin, Position3D::new(2, 0, 0)));
        assert!(!is_adjacent_tile(origin, Position3D::new(0, 2, 0)));

        // Test diagonal tiles (not adjacent for cardinal-only movement)
        assert!(!is_adjacent_tile(origin, Position3D::new(1, 1, 0)));

        // Test same tile
        assert!(!is_adjacent_tile(origin, origin));
    }

    #[test]
    fn adjacent_tile_detection_with_diagonals() {
        let origin = Position3D::origin();

        // Test adjacent tiles including diagonals
        assert!(is_adjacent_tile_including_diagonals(
            origin,
            Position3D::new(1, 0, 0)
        ));
        assert!(is_adjacent_tile_including_diagonals(
            origin,
            Position3D::new(1, 1, 0)
        ));
        assert!(is_adjacent_tile_including_diagonals(
            origin,
            Position3D::new(0, 1, 0)
        ));
        assert!(is_adjacent_tile_including_diagonals(
            origin,
            Position3D::new(-1, 1, 0)
        ));

        // Test non-adjacent tiles
        assert!(!is_adjacent_tile_including_diagonals(
            origin,
            Position3D::new(2, 0, 0)
        ));
        assert!(!is_adjacent_tile_including_diagonals(
            origin,
            Position3D::new(2, 2, 0)
        ));

        // Test same tile
        assert!(!is_adjacent_tile_including_diagonals(origin, origin));
    }

    #[test]
    fn direction_calculation() {
        let origin = Position3D::origin();

        assert_eq!(
            calculate_direction(origin, Position3D::new(0, 1, 0)),
            Some(Direction::North)
        );
        assert_eq!(
            calculate_direction(origin, Position3D::new(0, -1, 0)),
            Some(Direction::South)
        );
        assert_eq!(
            calculate_direction(origin, Position3D::new(1, 0, 0)),
            Some(Direction::East)
        );
        assert_eq!(
            calculate_direction(origin, Position3D::new(-1, 0, 0)),
            Some(Direction::West)
        );
        assert_eq!(
            calculate_direction(origin, Position3D::new(0, 0, 1)),
            Some(Direction::Up)
        );
        assert_eq!(
            calculate_direction(origin, Position3D::new(0, 0, -1)),
            Some(Direction::Down)
        );

        // Test diagonal (should return None for now)
        assert_eq!(calculate_direction(origin, Position3D::new(1, 1, 0)), None);
    }
}
