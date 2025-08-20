//! Application Layer - Use Cases and Application Services
//!
//! This layer orchestrates the domain logic and manages application workflows.
//! It contains use cases that represent specific business operations and
//! application services that coordinate between the domain and infrastructure layers.
//!
//! ## Architecture
//! - **Use Cases**: Specific business operations (move player, spawn enemy, etc.)
//! - **Services**: Application-level services that coordinate domain operations
//! - **DTOs**: Data Transfer Objects for communication between layers
//!
//! ## Rules
//! - Can depend on domain layer
//! - Cannot depend on infrastructure or presentation layers
//! - Orchestrates domain entities and services
//! - Handles application-level business logic

pub mod services;
pub mod use_cases;

// Re-export common application types
pub use services::{GameSessionService, InputHandlerService};
pub use use_cases::{
    HandleCollisionUseCase, MovePlayerUseCase, SpawnEnemiesUseCase, UpdateScoreUseCase,
};

/// Application-specific error types
#[derive(Debug, Clone, PartialEq)]
pub enum ApplicationError {
    /// Domain layer error
    DomainError(crate::domain::DomainError),
    /// Use case execution failed
    UseCaseError(String),
    /// Service operation failed
    ServiceError(String),
    /// Invalid input provided
    InvalidInput(String),
    /// Game session not found or invalid
    InvalidSession(String),
}

impl std::fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApplicationError::DomainError(err) => write!(f, "Domain error: {}", err),
            ApplicationError::UseCaseError(msg) => write!(f, "Use case error: {}", msg),
            ApplicationError::ServiceError(msg) => write!(f, "Service error: {}", msg),
            ApplicationError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            ApplicationError::InvalidSession(msg) => write!(f, "Invalid session: {}", msg),
        }
    }
}

impl std::error::Error for ApplicationError {}

impl From<crate::domain::DomainError> for ApplicationError {
    fn from(error: crate::domain::DomainError) -> Self {
        ApplicationError::DomainError(error)
    }
}

/// Common result type for application operations
pub type ApplicationResult<T> = Result<T, ApplicationError>;

/// Input/Output DTOs for application layer
pub mod dto {
    use crate::domain::{Position, Score, Velocity};

    /// Input for moving a player
    #[derive(Debug, Clone)]
    pub struct MovePlayerInput {
        pub player_id: String,
        pub direction_x: f32,
        pub direction_y: f32,
        pub speed: f32,
        pub delta_time: f32,
    }

    /// Output from moving a player
    #[derive(Debug, Clone)]
    pub struct MovePlayerOutput {
        pub player_id: String,
        pub new_position: Position,
        pub new_velocity: Velocity,
    }

    /// Input for spawning enemies
    #[derive(Debug, Clone)]
    pub struct SpawnEnemiesInput {
        pub spawn_position: Position,
        pub enemy_type: String,
        pub velocity: Velocity,
    }

    /// Output from spawning enemies
    #[derive(Debug, Clone)]
    pub struct SpawnEnemiesOutput {
        pub enemy_id: String,
        pub position: Position,
        pub velocity: Velocity,
    }

    /// Input for collision handling
    #[derive(Debug, Clone)]
    pub struct HandleCollisionInput {
        pub entity1_id: String,
        pub entity1_position: Position,
        pub entity2_id: String,
        pub entity2_position: Position,
    }

    /// Output from collision handling
    #[derive(Debug, Clone)]
    pub struct HandleCollisionOutput {
        pub collision_detected: bool,
        pub score_change: Option<u32>,
        pub entities_to_remove: Vec<String>,
    }

    /// Input for score updates
    #[derive(Debug, Clone)]
    pub struct UpdateScoreInput {
        pub session_id: String,
        pub points_to_add: u32,
    }

    /// Output from score updates
    #[derive(Debug, Clone)]
    pub struct UpdateScoreOutput {
        pub session_id: String,
        pub old_score: Score,
        pub new_score: Score,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn application_error_from_domain_error() {
        let domain_error = crate::domain::DomainError::PlayerError("test".to_string());
        let app_error = ApplicationError::from(domain_error);

        if let ApplicationError::DomainError(_) = app_error {
            // Success
        } else {
            panic!("Expected DomainError variant");
        }
    }

    #[test]
    fn application_error_display() {
        let error = ApplicationError::UseCaseError("test error".to_string());
        assert_eq!(error.to_string(), "Use case error: test error");
    }
}
