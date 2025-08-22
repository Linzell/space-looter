//! Rendering Coordination - Visual Presentation Management
//!
//! This module coordinates the visual presentation of the game,
//! managing rendering systems, visual effects, and display coordination.
//! It acts as a bridge between the game logic and visual output.

use crate::domain::{Position3D, Score};
use bevy::prelude::*;
use std::time::Duration;

/// Rendering configuration and settings
#[derive(Resource, Debug, Clone)]
pub struct RenderingConfig {
    /// Target frame rate
    pub target_fps: Option<u32>,
    /// VSync enabled
    pub vsync: bool,
    /// Anti-aliasing level
    pub msaa_samples: u32,
    /// Window clear color
    pub clear_color: Color,
    /// Whether to show debug information
    pub show_debug: bool,
    /// UI scale factor
    pub ui_scale: f32,
}

impl Default for RenderingConfig {
    fn default() -> Self {
        Self {
            target_fps: Some(60),
            vsync: true,
            msaa_samples: 4,
            clear_color: Color::srgb(0.1, 0.1, 0.2),
            show_debug: false,
            ui_scale: 1.0,
        }
    }
}

impl RenderingConfig {
    /// Create configuration optimized for web
    pub fn web_optimized() -> Self {
        Self {
            target_fps: Some(60),
            vsync: true,
            msaa_samples: 2, // Lower MSAA for web performance
            clear_color: Color::srgb(0.1, 0.1, 0.2),
            show_debug: false,
            ui_scale: 1.0,
        }
    }

    /// Create configuration for high-performance native builds
    pub fn native_performance() -> Self {
        Self {
            target_fps: None, // Unlimited
            vsync: false,
            msaa_samples: 8,
            clear_color: Color::srgb(0.1, 0.1, 0.2),
            show_debug: false,
            ui_scale: 1.0,
        }
    }

    /// Toggle debug display
    pub fn toggle_debug(&mut self) {
        self.show_debug = !self.show_debug;
    }
}

/// Visual effects data for entities
#[derive(Component, Debug, Clone)]
pub struct VisualEffects {
    /// Scale animation progress
    pub scale_animation: f32,
    /// Rotation animation progress
    pub rotation_animation: f32,
    /// Color tint
    pub color_tint: Color,
    /// Flash effect intensity
    pub flash_intensity: f32,
    /// Trail effect enabled
    pub trail_enabled: bool,
}

impl Default for VisualEffects {
    fn default() -> Self {
        Self {
            scale_animation: 1.0,
            rotation_animation: 0.0,
            color_tint: Color::WHITE,
            flash_intensity: 0.0,
            trail_enabled: false,
        }
    }
}

impl VisualEffects {
    /// Create with flash effect
    pub fn with_flash(intensity: f32) -> Self {
        Self {
            flash_intensity: intensity,
            ..default()
        }
    }

    /// Create with color tint
    pub fn with_tint(color: Color) -> Self {
        Self {
            color_tint: color,
            ..default()
        }
    }

    /// Enable trail effect
    pub fn with_trail() -> Self {
        Self {
            trail_enabled: true,
            ..default()
        }
    }
}

/// Camera follow settings
#[derive(Component, Debug, Clone)]
pub struct CameraFollow {
    /// Target entity to follow
    pub target: Option<Entity>,
    /// Follow smoothing factor
    pub smoothing: f32,
    /// Offset from target
    pub offset: Vec3,
    /// Whether to follow on X axis
    pub follow_x: bool,
    /// Whether to follow on Y axis
    pub follow_y: bool,
}

impl Default for CameraFollow {
    fn default() -> Self {
        Self {
            target: None,
            smoothing: 5.0,
            offset: Vec3::ZERO,
            follow_x: true,
            follow_y: true,
        }
    }
}

/// UI rendering state
#[derive(Resource, Debug, Clone)]
pub struct UIRenderState {
    /// Current score display text
    pub score_text: String,
    /// Current time display text
    pub time_text: String,
    /// Health bar percentage (0.0 to 1.0)
    pub health_percentage: f32,
    /// Whether UI is visible
    pub ui_visible: bool,
    /// UI animation state
    pub animation_state: f32,
    /// Debug info display
    pub debug_text: Vec<String>,
}

impl Default for UIRenderState {
    fn default() -> Self {
        Self {
            score_text: "Score: 0".to_string(),
            time_text: "Time: 00:00".to_string(),
            health_percentage: 1.0,
            ui_visible: true,
            animation_state: 0.0,
            debug_text: Vec::new(),
        }
    }
}

impl UIRenderState {
    /// Update score display
    pub fn update_score(&mut self, score: &Score) {
        self.score_text = format!("Score: {}", score);
    }

    /// Update time display
    pub fn update_time(&mut self, time: Duration) {
        let total_seconds = time.as_secs();
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        self.time_text = format!("Time: {:02}:{:02}", minutes, seconds);
    }

    /// Update health display
    pub fn update_health(&mut self, current: u32, max: u32) {
        self.health_percentage = if max > 0 {
            current as f32 / max as f32
        } else {
            0.0
        };
    }

    /// Add debug information
    pub fn add_debug_info(&mut self, info: String) {
        self.debug_text.push(info);
        // Keep only last 10 debug lines
        if self.debug_text.len() > 10 {
            self.debug_text.remove(0);
        }
    }

    /// Clear debug information
    pub fn clear_debug_info(&mut self) {
        self.debug_text.clear();
    }

    /// Show or hide UI
    pub fn set_ui_visible(&mut self, visible: bool) {
        self.ui_visible = visible;
    }
}

/// System for applying visual effects to entities
pub fn apply_visual_effects_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Sprite, &mut VisualEffects)>,
) {
    let dt = time.delta_secs();

    for (mut transform, mut sprite, mut effects) in query.iter_mut() {
        // Apply scale animation
        if effects.scale_animation != 1.0 {
            let target_scale = effects.scale_animation;
            let current_scale = transform.scale.x;
            let new_scale = current_scale + (target_scale - current_scale) * dt * 5.0;
            transform.scale = Vec3::splat(new_scale);

            // Reset if close enough
            if (new_scale - target_scale).abs() < 0.01 {
                effects.scale_animation = 1.0;
            }
        }

        // Apply rotation animation
        if effects.rotation_animation != 0.0 {
            transform.rotation *= Quat::from_rotation_z(effects.rotation_animation * dt);
        }

        // Apply flash effect
        if effects.flash_intensity > 0.0 {
            let flash_color = Color::WHITE.with_alpha(effects.flash_intensity);
            sprite.color = sprite.color.mix(&flash_color, effects.flash_intensity);

            // Decay flash intensity
            effects.flash_intensity = (effects.flash_intensity - dt * 2.0).max(0.0);
        }

        // Apply color tint
        if effects.color_tint != Color::WHITE {
            sprite.color = sprite.color.mix(&effects.color_tint, 0.5);
        }
    }
}

/// System for camera following
pub fn camera_follow_system(
    time: Res<Time>,
    mut camera_query: Query<
        (&mut Transform, &CameraFollow),
        (
            With<Camera>,
            Without<crate::infrastructure::bevy::components::PlayerComponent>,
        ),
    >,
    target_query: Query<
        &Transform,
        (
            Without<Camera>,
            With<crate::infrastructure::bevy::components::PlayerComponent>,
        ),
    >,
) {
    let dt = time.delta_secs();

    for (mut camera_transform, follow) in camera_query.iter_mut() {
        if let Some(target_entity) = follow.target {
            if let Ok(target_transform) = target_query.get(target_entity) {
                let target_pos = target_transform.translation + follow.offset;
                let current_pos = camera_transform.translation;

                let mut new_pos = current_pos;

                if follow.follow_x {
                    new_pos.x += (target_pos.x - current_pos.x) * follow.smoothing * dt;
                }

                if follow.follow_y {
                    new_pos.y += (target_pos.y - current_pos.y) * follow.smoothing * dt;
                }

                camera_transform.translation = new_pos;
            }
        }
    }
}

/// System for updating UI render state
pub fn update_ui_render_state_system(
    time: Res<Time>,
    score_resource: Option<Res<crate::infrastructure::bevy::resources::ScoreResource>>,
    game_state: Option<Res<crate::presentation::game_state::RpgGameSession>>,
    config: Res<RenderingConfig>,
    mut ui_state: ResMut<UIRenderState>,
) {
    // Update animation state
    ui_state.animation_state += time.delta_secs();

    // Update score if available
    if let Some(score_res) = score_resource {
        ui_state.update_score(&score_res.score);
    }

    // Update time if available
    if let Some(game_data) = game_state {
        let duration = std::time::Duration::from_secs(game_data.total_play_time as u64);
        ui_state.update_time(duration);
    }

    // Update debug info if enabled
    if config.show_debug {
        ui_state.clear_debug_info();
        ui_state.add_debug_info(format!("FPS: {:.1}", 1.0 / time.delta_secs()));
        ui_state.add_debug_info(format!("Frame time: {:.2}ms", time.delta_secs() * 1000.0));
        ui_state.add_debug_info(format!("UI Scale: {:.1}x", config.ui_scale));
    } else {
        ui_state.clear_debug_info();
    }
}

/// System for handling window resizing and maintaining aspect ratio
pub fn handle_window_resize_system(mut resize_events: EventReader<bevy::window::WindowResized>) {
    for resize_event in resize_events.read() {
        let width = resize_event.width;
        let height = resize_event.height;
        let aspect_ratio = width / height;

        info!(
            "Window resized to {}x{}, aspect ratio: {:.2}",
            width, height, aspect_ratio
        );
    }
}

/// System for managing clear color updates
pub fn update_clear_color_system(
    config: Res<RenderingConfig>,
    mut clear_color: ResMut<ClearColor>,
) {
    if config.is_changed() {
        clear_color.0 = config.clear_color;
    }
}

/// Component marker for UI elements that should pulse
#[derive(Component)]
pub struct PulsingUI {
    pub speed: f32,
    pub min_alpha: f32,
    pub max_alpha: f32,
}

impl Default for PulsingUI {
    fn default() -> Self {
        Self {
            speed: 2.0,
            min_alpha: 0.5,
            max_alpha: 1.0,
        }
    }
}

/// System for pulsing UI elements
pub fn pulsing_ui_system(time: Res<Time>, mut query: Query<(&Text, &PulsingUI)>) {
    for (_text, pulsing) in query.iter_mut() {
        let pulse = (time.elapsed_secs() * pulsing.speed).sin() * 0.5 + 0.5;
        let alpha = pulsing.min_alpha + (pulsing.max_alpha - pulsing.min_alpha) * pulse;

        // Note: Text component structure has changed in Bevy 0.16+
        // This would need to be updated based on the new Text API
        // For now, we'll skip the color update
        let _ = alpha; // Suppress unused variable warning
    }
}

/// Plugin for rendering systems
pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add resources
            .init_resource::<RenderingConfig>()
            .init_resource::<UIRenderState>()
            // Add systems
            .add_systems(
                Update,
                (
                    apply_visual_effects_system,
                    camera_follow_system,
                    update_ui_render_state_system,
                    update_clear_color_system,
                    pulsing_ui_system,
                ),
            );
    }
}

/// Utility functions for rendering
pub mod utils {
    use super::*;

    /// Convert domain position to world coordinates
    pub fn domain_to_world_position(position: &Position3D) -> Vec3 {
        Vec3::new(
            position.x() as f32,
            position.y() as f32,
            position.z() as f32,
        )
    }

    /// Convert world coordinates to domain position
    pub fn world_to_domain_position(
        world_pos: &Vec3,
    ) -> Result<Position3D, crate::domain::DomainError> {
        Ok(Position3D::new(
            world_pos.x as i32,
            world_pos.y as i32,
            world_pos.z as i32,
        ))
    }

    /// Create a flash effect for collision
    pub fn create_collision_flash() -> VisualEffects {
        VisualEffects::with_flash(0.8)
    }

    /// Create a scale-up effect for spawn
    pub fn create_spawn_effect() -> VisualEffects {
        let mut effects = VisualEffects::default();
        effects.scale_animation = 1.2;
        effects
    }

    /// Get color based on health percentage
    pub fn get_health_color(health_percentage: f32) -> Color {
        if health_percentage > 0.6 {
            Color::srgb(0.0, 1.0, 0.0) // Green
        } else if health_percentage > 0.3 {
            Color::srgb(1.0, 1.0, 0.0) // Yellow
        } else {
            Color::srgb(1.0, 0.0, 0.0) // Red
        }
    }

    /// Calculate UI position for different screen areas
    pub fn get_ui_position(area: UIArea, window_size: Vec2) -> Vec2 {
        match area {
            UIArea::TopLeft => Vec2::new(20.0, window_size.y - 20.0),
            UIArea::TopCenter => Vec2::new(window_size.x * 0.5, window_size.y - 20.0),
            UIArea::TopRight => Vec2::new(window_size.x - 20.0, window_size.y - 20.0),
            UIArea::Center => Vec2::new(window_size.x * 0.5, window_size.y * 0.5),
            UIArea::BottomLeft => Vec2::new(20.0, 20.0),
            UIArea::BottomCenter => Vec2::new(window_size.x * 0.5, 20.0),
            UIArea::BottomRight => Vec2::new(window_size.x - 20.0, 20.0),
        }
    }
}

/// UI positioning areas
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UIArea {
    TopLeft,
    TopCenter,
    TopRight,
    Center,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rendering_config_creation() {
        let config = RenderingConfig::default();
        assert_eq!(config.target_fps, Some(60));
        assert!(config.vsync);
        assert_eq!(config.msaa_samples, 4);
    }

    #[test]
    fn rendering_config_web_optimized() {
        let config = RenderingConfig::web_optimized();
        assert_eq!(config.msaa_samples, 2); // Lower for web performance
    }

    #[test]
    fn visual_effects_creation() {
        let effects = VisualEffects::default();
        assert_eq!(effects.scale_animation, 1.0);
        assert_eq!(effects.flash_intensity, 0.0);

        let flash_effects = VisualEffects::with_flash(0.5);
        assert_eq!(flash_effects.flash_intensity, 0.5);
    }

    #[test]
    fn ui_render_state_updates() {
        let mut ui_state = UIRenderState::default();
        let score = Score::new(1000).unwrap();

        ui_state.update_score(&score);
        assert_eq!(ui_state.score_text, "Score: 1,000");

        ui_state.update_time(Duration::from_secs(125));
        assert_eq!(ui_state.time_text, "Time: 02:05");

        ui_state.update_health(75, 100);
        assert_eq!(ui_state.health_percentage, 0.75);
    }

    #[test]
    fn ui_render_state_debug_info() {
        let mut ui_state = UIRenderState::default();

        ui_state.add_debug_info("Test info".to_string());
        assert_eq!(ui_state.debug_text.len(), 1);

        ui_state.clear_debug_info();
        assert_eq!(ui_state.debug_text.len(), 0);
    }

    #[test]
    fn camera_follow_default() {
        let follow = CameraFollow::default();
        assert_eq!(follow.smoothing, 5.0);
        assert!(follow.follow_x);
        assert!(follow.follow_y);
        assert_eq!(follow.offset, Vec3::ZERO);
    }

    #[test]
    fn pulsing_ui_default() {
        let pulsing = PulsingUI::default();
        assert_eq!(pulsing.speed, 2.0);
        assert_eq!(pulsing.min_alpha, 0.5);
        assert_eq!(pulsing.max_alpha, 1.0);
    }

    #[test]
    fn rendering_utils_position_conversion() {
        let domain_pos = Position3D::new(10, 20, 0);
        let world_pos = utils::domain_to_world_position(&domain_pos);

        assert_eq!(world_pos, Vec3::new(10.0, 20.0, 0.0));

        let converted_back = utils::world_to_domain_position(&world_pos).unwrap();
        assert_eq!(converted_back, domain_pos);
    }

    #[test]
    fn rendering_utils_health_color() {
        assert_eq!(utils::get_health_color(1.0), Color::srgb(0.0, 1.0, 0.0)); // Green
        assert_eq!(utils::get_health_color(0.5), Color::srgb(1.0, 1.0, 0.0)); // Yellow
        assert_eq!(utils::get_health_color(0.2), Color::srgb(1.0, 0.0, 0.0)); // Red
    }
}
