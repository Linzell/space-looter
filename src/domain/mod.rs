//! Domain Layer - 3D Isometric RPG with Dice Mechanics
//!
//! This module contains the pure business logic of the Space Looter RPG game.
//! It is framework-agnostic and contains no dependencies on Bevy or other infrastructure.
//!
//! ## Game Concept
//! A 3D isometric RPG that uses dice rolls for all actions and events:
//! - Dice-based mechanics for actions, events, and resource gathering
//! - 3D isometric world with procedural generation
//! - Base building and evolution system
//! - Random encounters during exploration
//! - Risk/reward exploration mechanics
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
//! - All mechanics based on dice rolls

pub mod constants;
pub mod entities;
pub mod services;
pub mod value_objects;

// Re-export common domain types for convenience
pub use entities::{Base, Event, GameSession, Map, Player, Quest, Resource};
pub use value_objects::{
    dice::{DiceModifier, DiceResult, DiceRoll, DiceType},
    position::{Position3D, TileCoordinate},
    resources::{ResourceAmount, ResourceCollection, ResourceType},
    terrain::TerrainType,
    EntityId, Experience, GameTime, PlayerStats, StatType,
};

// Backward compatibility aliases (to be removed in future versions)
pub use Position3D as Position;
pub use WorldBoundaries as GameBoundaries;

// Backward compatibility placeholder types (to be removed)
/// Placeholder Score type for backward compatibility
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Score {
    value: u32,
}

impl Score {
    pub fn zero() -> Self {
        Self { value: 0 }
    }

    pub fn new(value: u32) -> Result<Self, DomainError> {
        Ok(Self { value })
    }

    pub fn add(&self, points: u32) -> Result<Self, DomainError> {
        Ok(Self {
            value: self.value.saturating_add(points),
        })
    }

    pub fn value(&self) -> u32 {
        self.value
    }

    pub fn formatted(&self) -> String {
        format!("{}", self.value)
    }

    pub fn add_enemy_points(&mut self) -> Result<(), DomainError> {
        self.value = self.value.saturating_add(100);
        Ok(())
    }
}

impl Default for Score {
    fn default() -> Self {
        Self::zero()
    }
}

impl std::fmt::Display for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Placeholder Velocity type for backward compatibility
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Velocity {
    dx: f32,
    dy: f32,
}

impl Velocity {
    pub fn new(dx: f32, dy: f32) -> Result<Self, DomainError> {
        Ok(Self { dx, dy })
    }

    pub fn zero() -> Self {
        Self { dx: 0.0, dy: 0.0 }
    }

    pub fn from_direction(direction: (f32, f32), speed: f32) -> Result<Self, DomainError> {
        let magnitude = (direction.0 * direction.0 + direction.1 * direction.1).sqrt();
        if magnitude == 0.0 {
            return Ok(Self::zero());
        }
        Ok(Self {
            dx: (direction.0 / magnitude) * speed,
            dy: (direction.1 / magnitude) * speed,
        })
    }

    pub fn dx(&self) -> f32 {
        self.dx
    }

    pub fn dy(&self) -> f32 {
        self.dy
    }

    pub fn magnitude(&self) -> f32 {
        (self.dx * self.dx + self.dy * self.dy).sqrt()
    }
}

/// Placeholder Enemy type for backward compatibility
#[derive(Debug, Clone, PartialEq)]
pub struct Enemy {
    id: EntityId,
    position: Position3D,
    enemy_type: EnemyType,
}

impl Enemy {
    pub fn new(
        id: String,
        position: Position3D,
        _velocity: Velocity,
        enemy_type: EnemyType,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            id: EntityId::new(id.len() as u64),
            position,
            enemy_type,
        })
    }

    pub fn id(&self) -> &EntityId {
        &self.id
    }

    pub fn position(&self) -> Position3D {
        self.position
    }

    pub fn enemy_type(&self) -> EnemyType {
        self.enemy_type
    }
}

/// Placeholder EnemyType for backward compatibility
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyType {
    Basic,
    Fast,
    Heavy,
}

/// Placeholder CollisionService for backward compatibility
pub struct CollisionService;

impl CollisionService {
    pub fn new() -> Self {
        Self
    }

    pub fn check_collision(&self, pos1: &Position3D, pos2: &Position3D, _radius: f32) -> bool {
        // Simple distance-based collision detection
        pos1.distance_to(pos2) < constants::COLLISION_RADIUS
    }

    pub fn resolve_player_enemy_collision(
        &self,
        _player_pos: &Position3D,
        _enemy_pos: &Position3D,
    ) -> DomainResult<Score> {
        // Placeholder: award points for collision
        Score::new(constants::POINTS_PER_ENEMY)
    }
}

/// Placeholder SpawningService for backward compatibility
pub struct SpawningService;

impl SpawningService {
    pub fn new() -> Self {
        Self
    }

    pub fn should_spawn_enemy(&self, _current_time: f32) -> bool {
        // Placeholder: always allow spawning
        true
    }

    pub fn calculate_spawn_position(
        &self,
        _boundaries: &WorldBoundaries,
    ) -> DomainResult<Position3D> {
        // Placeholder: spawn at origin
        Ok(Position3D::new(0, 0, 0))
    }

    pub fn create_enemy_velocity(&self, _speed: f32) -> DomainResult<Velocity> {
        // Placeholder: downward velocity
        Velocity::new(0.0, -100.0)
    }
}

/// Domain-specific error types for the RPG system
#[derive(Debug, Clone, PartialEq)]
pub enum DomainError {
    // Player-related errors
    PlayerError(String),
    InvalidPlayerStats(String),
    InsufficientPlayerLevel(u32),

    // Dice-related errors
    InvalidDiceRoll(String),
    DiceModifierError(String),
    InvalidDiceType(String),

    // Map and world errors
    MapGenerationError(String),
    InvalidMapCoordinates(i32, i32, i32),
    TileNotAccessible(i32, i32, i32),
    MapBoundaryExceeded(String),

    // Resource and base errors
    InsufficientResources(String),
    InvalidResourceType(String),
    InvalidResourceAmount(i32),
    BaseUpgradeError(String),
    BuildingRequirementsNotMet(String),

    // Event and quest errors
    EventTriggerError(String),
    InvalidEventType(String),
    QuestError(String),
    QuestRequirementsNotMet(String),

    // Game session errors
    GameSessionError(String),
    InvalidGameState(String),

    // General validation errors
    ValidationError(String),
    ConfigurationError(String),

    // Service errors
    ServiceError { service: String, reason: String },

    // Backward compatibility error types
    CollisionError(String),
    InvalidVelocity(f32, f32),
}

impl std::fmt::Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DomainError::PlayerError(msg) => write!(f, "Player error: {}", msg),
            DomainError::InvalidPlayerStats(msg) => write!(f, "Invalid player stats: {}", msg),
            DomainError::InsufficientPlayerLevel(level) => {
                write!(f, "Insufficient player level: {}", level)
            }
            DomainError::InvalidDiceRoll(msg) => write!(f, "Invalid dice roll: {}", msg),
            DomainError::DiceModifierError(msg) => write!(f, "Dice modifier error: {}", msg),
            DomainError::InvalidDiceType(msg) => write!(f, "Invalid dice type: {}", msg),
            DomainError::MapGenerationError(msg) => write!(f, "Map generation error: {}", msg),
            DomainError::InvalidMapCoordinates(x, y, z) => {
                write!(f, "Invalid map coordinates: ({}, {}, {})", x, y, z)
            }
            DomainError::TileNotAccessible(x, y, z) => {
                write!(f, "Tile not accessible: ({}, {}, {})", x, y, z)
            }
            DomainError::MapBoundaryExceeded(msg) => write!(f, "Map boundary exceeded: {}", msg),
            DomainError::InsufficientResources(msg) => {
                write!(f, "Insufficient resources: {}", msg)
            }
            DomainError::InvalidResourceType(msg) => write!(f, "Invalid resource type: {}", msg),
            DomainError::InvalidResourceAmount(amount) => {
                write!(f, "Invalid resource amount: {}", amount)
            }
            DomainError::BaseUpgradeError(msg) => write!(f, "Base upgrade error: {}", msg),
            DomainError::BuildingRequirementsNotMet(msg) => {
                write!(f, "Building requirements not met: {}", msg)
            }
            DomainError::EventTriggerError(msg) => write!(f, "Event trigger error: {}", msg),
            DomainError::ServiceError { service, reason } => {
                write!(f, "{} service error: {}", service, reason)
            }
            DomainError::InvalidEventType(msg) => write!(f, "Invalid event type: {}", msg),
            DomainError::QuestError(msg) => write!(f, "Quest error: {}", msg),
            DomainError::QuestRequirementsNotMet(msg) => {
                write!(f, "Quest requirements not met: {}", msg)
            }
            DomainError::GameSessionError(msg) => write!(f, "Game session error: {}", msg),
            DomainError::InvalidGameState(msg) => write!(f, "Invalid game state: {}", msg),
            DomainError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            DomainError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            DomainError::CollisionError(msg) => write!(f, "Collision error: {}", msg),
            DomainError::InvalidVelocity(dx, dy) => {
                write!(f, "Invalid velocity: dx={}, dy={}", dx, dy)
            }
        }
    }
}

impl std::error::Error for DomainError {}

/// Common result type for domain operations
pub type DomainResult<T> = Result<T, DomainError>;

/// Game world boundaries for 3D isometric space
#[derive(Debug, Clone, PartialEq)]
pub struct WorldBoundaries {
    pub min_x: i32,
    pub max_x: i32,
    pub min_y: i32,
    pub max_y: i32,
    pub min_z: i32,
    pub max_z: i32,
}

impl WorldBoundaries {
    /// Create new world boundaries
    pub fn new(min_x: i32, max_x: i32, min_y: i32, max_y: i32, min_z: i32, max_z: i32) -> Self {
        Self {
            min_x,
            max_x,
            min_y,
            max_y,
            min_z,
            max_z,
        }
    }

    /// Standard world boundaries for initial game area
    pub fn standard() -> Self {
        Self::new(-50, 50, -50, 50, 0, 10)
    }

    /// Large world boundaries for expanded gameplay
    pub fn large() -> Self {
        Self::new(-200, 200, -200, 200, 0, 20)
    }

    /// Check if position is within boundaries
    pub fn contains(&self, position: &Position3D) -> bool {
        position.x >= self.min_x
            && position.x <= self.max_x
            && position.y >= self.min_y
            && position.y <= self.max_y
            && position.z >= self.min_z
            && position.z <= self.max_z
    }

    /// Clamp position to boundaries
    pub fn clamp(&self, position: Position3D) -> Position3D {
        Position3D {
            x: position.x.clamp(self.min_x, self.max_x),
            y: position.y.clamp(self.min_y, self.max_y),
            z: position.z.clamp(self.min_z, self.max_z),
        }
    }

    /// Get the size of the world in each dimension
    pub fn size(&self) -> (i32, i32, i32) {
        (
            self.max_x - self.min_x + 1,
            self.max_y - self.min_y + 1,
            self.max_z - self.min_z + 1,
        )
    }
}

/// Game state tracking for turn-based mechanics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    /// Player's turn - can move and take actions
    PlayerTurn,
    /// Processing phase - dice rolls, events, resource gathering
    Processing,
    /// Event phase - random encounters and events
    EventPhase,
    /// End turn phase - cleanup and preparation for next turn
    EndTurn,
    /// Paused state
    Paused,
    /// Game over state
    GameOver,
}

impl Default for GamePhase {
    fn default() -> Self {
        GamePhase::PlayerTurn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_boundaries_contains_position() {
        let boundaries = WorldBoundaries::standard();
        let position = Position3D::new(0, 0, 5);
        assert!(boundaries.contains(&position));
    }

    #[test]
    fn world_boundaries_clamps_position() {
        let boundaries = WorldBoundaries::standard();
        let position = Position3D::new(1000, 1000, 1000);
        let clamped = boundaries.clamp(position);
        assert_eq!(clamped.x, boundaries.max_x);
        assert_eq!(clamped.y, boundaries.max_y);
        assert_eq!(clamped.z, boundaries.max_z);
    }

    #[test]
    fn world_boundaries_size_calculation() {
        let boundaries = WorldBoundaries::standard();
        let size = boundaries.size();
        assert_eq!(size, (101, 101, 11)); // -50 to 50 = 101 tiles, 0 to 10 = 11 levels
    }

    #[test]
    fn domain_error_display() {
        let error = DomainError::PlayerError("test error".to_string());
        assert_eq!(error.to_string(), "Player error: test error");
    }

    #[test]
    fn game_phase_default() {
        let phase = GamePhase::default();
        assert_eq!(phase, GamePhase::PlayerTurn);
    }
}
