//! Infrastructure Layer - External Concerns and Framework Integration
//!
//! This layer handles all external concerns such as framework integration,
//! I/O operations, and platform-specific implementations. It adapts external
//! libraries and frameworks to work with our domain model.
//!
//! ## Architecture
//! - **Bevy Integration**: ECS components, systems, and resources
//! - **Random Generation**: Platform-specific random number generation
//! - **Web Integration**: WebAssembly bindings and web-specific code
//!
//! ## Rules
//! - Can depend on all other layers
//! - Contains framework-specific code (Bevy, web APIs, etc.)
//! - Implements adapter patterns for external services
//! - Handles platform-specific implementations

pub mod bevy;
pub mod random;
pub mod time;
pub mod web;

// Re-export common infrastructure types
pub use bevy::{BevyGamePlugin, BevySystemsPlugin};
pub use random::RandomNumberGenerator;
pub use time::TimeService;

/// Infrastructure-specific error types
#[derive(Debug, Clone, PartialEq)]
pub enum InfrastructureError {
    /// Bevy ECS operation failed
    BevyError(String),
    /// Random number generation failed
    RandomError(String),
    /// Time operation error
    TimeError(String),
    /// Web platform error
    WebError(String),
    /// External service error
    ExternalServiceError(String),
    /// 3D coordinate conversion error
    CoordinateConversionError(String),
    /// Isometric projection error
    IsometricProjectionError(String),
}

impl std::fmt::Display for InfrastructureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InfrastructureError::BevyError(msg) => write!(f, "Bevy error: {}", msg),
            InfrastructureError::RandomError(msg) => write!(f, "Random error: {}", msg),
            InfrastructureError::TimeError(msg) => write!(f, "Time error: {}", msg),
            InfrastructureError::WebError(msg) => write!(f, "Web error: {}", msg),
            InfrastructureError::ExternalServiceError(msg) => {
                write!(f, "External service error: {}", msg)
            }
            InfrastructureError::CoordinateConversionError(msg) => {
                write!(f, "Coordinate conversion error: {}", msg)
            }
            InfrastructureError::IsometricProjectionError(msg) => {
                write!(f, "Isometric projection error: {}", msg)
            }
        }
    }
}

impl std::error::Error for InfrastructureError {}

/// Common result type for infrastructure operations
pub type InfrastructureResult<T> = Result<T, InfrastructureError>;

/// Traits for infrastructure services
pub mod traits {
    use crate::domain::{DiceRoll, DiceType, Position3D, ResourceType, TerrainType};

    /// Trait for random number generation adapted for RPG mechanics
    pub trait RandomService: Send + Sync {
        /// Generate random f32 between 0.0 and 1.0
        fn random_f32(&self) -> f32;

        /// Generate random f32 in range [min, max]
        fn random_range(&self, min: f32, max: f32) -> f32;

        /// Generate random i32 in range [min, max] (inclusive)
        fn random_range_i32(&self, min: i32, max: i32) -> i32;

        /// Generate random 3D position within boundaries
        fn random_position_3d(
            &self,
            min_x: i32,
            max_x: i32,
            min_y: i32,
            max_y: i32,
            min_z: i32,
            max_z: i32,
        ) -> Position3D;

        /// Perform a dice roll
        fn roll_dice(&self, dice_type: DiceType, count: u8) -> DiceRoll;

        /// Generate random resource type for discovery
        fn random_resource_type(&self) -> ResourceType;

        /// Generate random terrain type for map generation
        fn random_terrain_type(&self) -> TerrainType;

        /// Generate random bool with given probability (0.0 to 1.0)
        fn random_bool(&self, probability: f32) -> bool;
    }

    /// Trait for time-related operations
    pub trait TimeService: Send + Sync {
        /// Get current time as seconds since start
        fn current_time(&self) -> f32;

        /// Get delta time since last frame
        fn delta_time(&self) -> f32;

        /// Get game time in turns/rounds
        fn game_turn(&self) -> u32;

        /// Check if enough time has passed for an action (in seconds)
        fn time_elapsed_since(&self, last_time: f32, duration: f32) -> bool;
    }

    /// Trait for input handling adapted for isometric RPG
    pub trait InputService: Send + Sync {
        /// Check if key is currently pressed
        fn is_key_pressed(&self, key: &str) -> bool;

        /// Check if key was just pressed this frame
        fn is_key_just_pressed(&self, key: &str) -> bool;

        /// Get movement input as tile-based direction (-1, 0, 1 for each axis)
        fn get_movement_input(&self) -> (i32, i32);

        /// Get mouse position in screen coordinates
        fn get_mouse_position(&self) -> Option<(f32, f32)>;

        /// Check if mouse button was just pressed
        fn is_mouse_just_pressed(&self, button: MouseButton) -> bool;

        /// Convert screen coordinates to world tile coordinates
        fn screen_to_tile(&self, screen_x: f32, screen_y: f32) -> Option<Position3D>;
    }

    /// Mouse button types for input handling
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum MouseButton {
        Left,
        Right,
        Middle,
    }

    /// Trait for rendering services adapted for isometric view
    pub trait RenderService: Send + Sync {
        /// Convert 3D world position to 2D screen coordinates for isometric view
        fn world_to_screen(&self, world_pos: &Position3D) -> (f32, f32);

        /// Convert 2D screen coordinates to 3D world position
        fn screen_to_world(&self, screen_x: f32, screen_y: f32) -> Option<Position3D>;

        /// Get the current camera position
        fn get_camera_position(&self) -> Position3D;

        /// Set the camera to follow a target position
        fn set_camera_target(&self, target: Position3D);

        /// Check if a position is visible on screen
        fn is_visible(&self, world_pos: &Position3D) -> bool;
    }

    /// Trait for audio services for RPG feedback
    pub trait AudioService: Send + Sync {
        /// Play a sound effect
        fn play_sound(&self, sound_name: &str);

        /// Play background music
        fn play_music(&self, music_name: &str);

        /// Stop all sounds
        fn stop_all_sounds(&self);

        /// Set master volume (0.0 to 1.0)
        fn set_volume(&self, volume: f32);
    }

    /// Trait for persistence services
    pub trait PersistenceService: Send + Sync {
        /// Save game data
        fn save_game(&self, data: &str) -> Result<(), String>;

        /// Load game data
        fn load_game(&self) -> Result<String, String>;

        /// Check if save data exists
        fn has_save_data(&self) -> bool;

        /// Delete save data
        fn delete_save_data(&self) -> Result<(), String>;
    }
}

/// Configuration for infrastructure services
#[derive(Debug, Clone)]
pub struct InfrastructureConfig {
    /// Enable debug logging
    pub debug_logging: bool,
    /// Random seed for deterministic behavior (None for random)
    pub random_seed: Option<u64>,
    /// Web-specific configurations
    pub web_config: WebConfig,
    /// Rendering configurations
    pub render_config: RenderConfig,
    /// Audio configurations
    pub audio_config: AudioConfig,
}

#[derive(Debug, Clone)]
pub struct WebConfig {
    /// Canvas element ID for web deployment
    pub canvas_id: String,
    /// Enable web-specific optimizations
    pub optimizations: bool,
    /// Maximum frame rate for web
    pub max_fps: Option<u32>,
    /// Enable touch controls for mobile
    pub touch_controls: bool,
}

#[derive(Debug, Clone)]
pub struct RenderConfig {
    /// Isometric tile size in pixels
    pub tile_size: f32,
    /// Isometric angle in degrees (typically 30)
    pub iso_angle: f32,
    /// Camera zoom level
    pub zoom_level: f32,
    /// Enable shadows
    pub shadows: bool,
    /// Enable anti-aliasing
    pub anti_aliasing: bool,
}

#[derive(Debug, Clone)]
pub struct AudioConfig {
    /// Master volume (0.0 to 1.0)
    pub master_volume: f32,
    /// Sound effects volume (0.0 to 1.0)
    pub sfx_volume: f32,
    /// Music volume (0.0 to 1.0)
    pub music_volume: f32,
    /// Enable audio
    pub enabled: bool,
}

impl Default for InfrastructureConfig {
    fn default() -> Self {
        Self {
            debug_logging: cfg!(debug_assertions),
            random_seed: None,
            web_config: WebConfig::default(),
            render_config: RenderConfig::default(),
            audio_config: AudioConfig::default(),
        }
    }
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            canvas_id: "bevy".to_string(),
            optimizations: true,
            max_fps: Some(60),
            touch_controls: cfg!(target_arch = "wasm32"),
        }
    }
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            tile_size: 32.0,
            iso_angle: 30.0,
            zoom_level: 1.0,
            shadows: true,
            anti_aliasing: true,
        }
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 0.8,
            sfx_volume: 1.0,
            music_volume: 0.6,
            enabled: true,
        }
    }
}

/// Helper functions for 3D isometric conversions
pub mod isometric {
    use crate::domain::Position3D;

    /// Convert 3D world coordinates to 2D isometric screen coordinates
    pub fn world_to_iso(world_pos: &Position3D, tile_size: f32) -> (f32, f32) {
        let x = (world_pos.x - world_pos.y) as f32 * tile_size * 0.5;
        let y = (world_pos.x + world_pos.y) as f32 * tile_size * 0.25
            - world_pos.z as f32 * tile_size * 0.5;
        (x, y)
    }

    /// Convert 2D isometric screen coordinates to 3D world coordinates (Z=0)
    pub fn iso_to_world(screen_x: f32, screen_y: f32, tile_size: f32) -> Position3D {
        let x = ((screen_x / (tile_size * 0.5)) + (screen_y / (tile_size * 0.25))) * 0.5;
        let y = ((screen_y / (tile_size * 0.25)) - (screen_x / (tile_size * 0.5))) * 0.5;
        Position3D::new(x.round() as i32, y.round() as i32, 0)
    }

    /// Convert 3D world coordinates to 2D isometric screen coordinates with elevation
    pub fn world_to_iso_with_elevation(
        world_pos: &Position3D,
        tile_size: f32,
        elevation_factor: f32,
    ) -> (f32, f32) {
        let x = (world_pos.x - world_pos.y) as f32 * tile_size * 0.5;
        let y = (world_pos.x + world_pos.y) as f32 * tile_size * 0.25
            - world_pos.z as f32 * tile_size * elevation_factor;
        (x, y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Position3D;

    #[test]
    fn infrastructure_error_display() {
        let error = InfrastructureError::BevyError("test error".to_string());
        assert_eq!(error.to_string(), "Bevy error: test error");
    }

    #[test]
    fn infrastructure_config_default() {
        let config = InfrastructureConfig::default();
        assert_eq!(config.web_config.canvas_id, "bevy");
        assert_eq!(config.web_config.max_fps, Some(60));
        assert_eq!(config.render_config.tile_size, 32.0);
        assert_eq!(config.audio_config.master_volume, 0.8);
    }

    #[test]
    fn isometric_conversion() {
        let world_pos = Position3D::new(5, 3, 0);
        let tile_size = 32.0;

        let (iso_x, iso_y) = isometric::world_to_iso(&world_pos, tile_size);
        let converted_back = isometric::iso_to_world(iso_x, iso_y, tile_size);

        assert_eq!(converted_back.x, world_pos.x);
        assert_eq!(converted_back.y, world_pos.y);
        assert_eq!(converted_back.z, 0); // Z is lost in 2D conversion
    }

    #[test]
    fn render_config_defaults() {
        let config = RenderConfig::default();
        assert_eq!(config.tile_size, 32.0);
        assert_eq!(config.iso_angle, 30.0);
        assert_eq!(config.zoom_level, 1.0);
        assert!(config.shadows);
        assert!(config.anti_aliasing);
    }

    #[test]
    fn audio_config_defaults() {
        let config = AudioConfig::default();
        assert_eq!(config.master_volume, 0.8);
        assert_eq!(config.sfx_volume, 1.0);
        assert_eq!(config.music_volume, 0.6);
        assert!(config.enabled);
    }
}
