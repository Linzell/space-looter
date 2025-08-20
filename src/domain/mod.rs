//! Domain Layer - Core Business Logic
//!
//! This module contains the pure business logic of the Space Looter game.
//! It is framework-agnostic and contains no dependencies on Bevy or other infrastructure.
//!
//! ## Architecture
//! - **Entities**: Core business objects with identity and behavior
//! - **Value Objects**: Immutable data structures representing domain concepts
//! - **Domain Services**: Complex business operations that don't belong to entities
//!
//! ## Rules
//! - No external framework dependencies (no `use bevy::*`)
//! - Pure Rust standard library only
//! - Rich domain model with business rules
//! - Comprehensive error handling

pub mod entities;
pub mod services;
pub mod value_objects;

// Re-export common domain types for convenience
pub use entities::{Enemy, EnemyType, GameSession, Player};
pub use services::{CollisionService, SpawningService};
pub use value_objects::{Position, Score, Velocity};

/// Domain-specific error types
#[derive(Debug, Clone, PartialEq)]
pub enum DomainError {
    /// Player operation failed
    PlayerError(String),
    /// Enemy operation failed
    EnemyError(String),
    /// Game session operation failed
    GameSessionError(String),
    /// Invalid position coordinates
    InvalidPosition(f32, f32),
    /// Invalid velocity values
    InvalidVelocity(f32, f32),
    /// Invalid score value
    InvalidScore(String),
    /// Collision detection failed
    CollisionError(String),
}

impl std::fmt::Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DomainError::PlayerError(msg) => write!(f, "Player error: {}", msg),
            DomainError::EnemyError(msg) => write!(f, "Enemy error: {}", msg),
            DomainError::GameSessionError(msg) => write!(f, "Game session error: {}", msg),
            DomainError::InvalidPosition(x, y) => {
                write!(f, "Invalid position: ({}, {})", x, y)
            }
            DomainError::InvalidVelocity(dx, dy) => {
                write!(f, "Invalid velocity: ({}, {})", dx, dy)
            }
            DomainError::InvalidScore(msg) => write!(f, "Invalid score: {}", msg),
            DomainError::CollisionError(msg) => write!(f, "Collision error: {}", msg),
        }
    }
}

impl std::error::Error for DomainError {}

/// Common result type for domain operations
pub type DomainResult<T> = Result<T, DomainError>;

/// Game boundaries defining the playable area
#[derive(Debug, Clone)]
pub struct GameBoundaries {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}

impl GameBoundaries {
    /// Create new game boundaries
    pub fn new(min_x: f32, max_x: f32, min_y: f32, max_y: f32) -> Self {
        Self {
            min_x,
            max_x,
            min_y,
            max_y,
        }
    }

    /// Standard game boundaries for 800x600 screen
    pub fn standard() -> Self {
        Self::new(-375.0, 375.0, -275.0, 275.0)
    }

    /// Check if position is within boundaries
    pub fn contains(&self, position: &Position) -> bool {
        position.x() >= self.min_x
            && position.x() <= self.max_x
            && position.y() >= self.min_y
            && position.y() <= self.max_y
    }

    /// Clamp position to boundaries
    pub fn clamp(&self, position: Position) -> Position {
        let x = position.x().clamp(self.min_x, self.max_x);
        let y = position.y().clamp(self.min_y, self.max_y);
        Position::new(x, y).unwrap_or(position)
    }
}

/// Game configuration constants
pub mod constants {
    /// Default player movement speed
    pub const DEFAULT_PLAYER_SPEED: f32 = 300.0;

    /// Default enemy movement speed
    pub const DEFAULT_ENEMY_SPEED: f32 = 100.0;

    /// Collision detection radius
    pub const COLLISION_RADIUS: f32 = 30.0;

    /// Points awarded per enemy collision
    pub const POINTS_PER_ENEMY: u32 = 10;

    /// Enemy spawn interval in seconds
    pub const ENEMY_SPAWN_INTERVAL: f32 = 2.0;

    /// Player size
    pub const PLAYER_SIZE: (f32, f32) = (50.0, 30.0);

    /// Enemy size
    pub const ENEMY_SIZE: (f32, f32) = (30.0, 30.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_boundaries_contains_position() {
        let boundaries = GameBoundaries::standard();
        let position = Position::new(0.0, 0.0).unwrap();
        assert!(boundaries.contains(&position));
    }

    #[test]
    fn game_boundaries_clamps_position() {
        let boundaries = GameBoundaries::standard();
        let position = Position::new(1000.0, 1000.0).unwrap();
        let clamped = boundaries.clamp(position);
        assert_eq!(clamped.x(), boundaries.max_x);
        assert_eq!(clamped.y(), boundaries.max_y);
    }

    #[test]
    fn domain_error_display() {
        let error = DomainError::PlayerError("test error".to_string());
        assert_eq!(error.to_string(), "Player error: test error");
    }
}
