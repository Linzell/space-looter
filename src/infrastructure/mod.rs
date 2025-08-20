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
pub mod web;

// Re-export common infrastructure types
pub use bevy::{BevyGamePlugin, BevySystemsPlugin};
pub use random::RandomNumberGenerator;

/// Infrastructure-specific error types
#[derive(Debug, Clone, PartialEq)]
pub enum InfrastructureError {
    /// Bevy ECS operation failed
    BevyError(String),
    /// Random number generation failed
    RandomError(String),
    /// Web platform error
    WebError(String),
    /// External service error
    ExternalServiceError(String),
}

impl std::fmt::Display for InfrastructureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InfrastructureError::BevyError(msg) => write!(f, "Bevy error: {}", msg),
            InfrastructureError::RandomError(msg) => write!(f, "Random error: {}", msg),
            InfrastructureError::WebError(msg) => write!(f, "Web error: {}", msg),
            InfrastructureError::ExternalServiceError(msg) => {
                write!(f, "External service error: {}", msg)
            }
        }
    }
}

impl std::error::Error for InfrastructureError {}

/// Common result type for infrastructure operations
pub type InfrastructureResult<T> = Result<T, InfrastructureError>;

/// Traits for infrastructure services
pub mod traits {
    use crate::domain::{Position, Velocity};

    /// Trait for random number generation
    pub trait RandomService: Send + Sync {
        /// Generate random f32 between 0.0 and 1.0
        fn random_f32(&self) -> f32;

        /// Generate random f32 in range [min, max]
        fn random_range(&self, min: f32, max: f32) -> f32;

        /// Generate random position within boundaries
        fn random_position(&self, min_x: f32, max_x: f32, min_y: f32, max_y: f32) -> Position;

        /// Generate random velocity with magnitude
        fn random_velocity(&self, max_speed: f32) -> Velocity;
    }

    /// Trait for time-related operations
    pub trait TimeService: Send + Sync {
        /// Get current time as seconds since start
        fn current_time(&self) -> f32;

        /// Get delta time since last frame
        fn delta_time(&self) -> f32;
    }

    /// Trait for input handling
    pub trait InputService: Send + Sync {
        /// Check if key is currently pressed
        fn is_key_pressed(&self, key: &str) -> bool;

        /// Get movement input as normalized direction vector
        fn get_movement_input(&self) -> (f32, f32);
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
}

#[derive(Debug, Clone)]
pub struct WebConfig {
    /// Canvas element ID for web deployment
    pub canvas_id: String,
    /// Enable web-specific optimizations
    pub optimizations: bool,
    /// Maximum frame rate for web
    pub max_fps: Option<u32>,
}

impl Default for InfrastructureConfig {
    fn default() -> Self {
        Self {
            debug_logging: cfg!(debug_assertions),
            random_seed: None,
            web_config: WebConfig::default(),
        }
    }
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            canvas_id: "bevy".to_string(),
            optimizations: true,
            max_fps: Some(60),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    }

    #[test]
    fn web_config_default() {
        let config = WebConfig::default();
        assert_eq!(config.canvas_id, "bevy");
        assert!(config.optimizations);
        assert_eq!(config.max_fps, Some(60));
    }
}
