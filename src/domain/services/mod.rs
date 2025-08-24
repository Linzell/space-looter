//! Domain Services - Complex Business Logic
//!
//! This module contains domain services that encapsulate complex business logic
//! that doesn't naturally belong to a single entity. Domain services coordinate
//! between entities and implement cross-cutting business rules.
//!
//! ## Architecture
//! - **Collision Service**: Handles collision detection between entities
//! - **Spawning Service**: Manages enemy spawning rules and logic
//!
//! ## Rules
//! - No infrastructure dependencies
//! - Pure business logic only
//! - Stateless services (or explicitly managed state)
//! - Clear single responsibility

pub mod audio_service;
pub mod collision;
pub mod font_service;
pub mod game_log_service;
pub mod map_service;
pub mod resting_service;
pub mod spawning;
pub mod tile_cache_service;
pub mod tile_movement;
pub mod visibility_service;

// Re-export services for convenience
pub use audio_service::{AudioService, AudioServiceError, DefaultAudioService};
pub use collision::CollisionService;
pub use font_service::{FontConfig, FontService, FontSize, FontType, FontWeight};
pub use game_log_service::{GameLogMessage, GameLogService, GameLogType, LogPriority};
pub use map_service::{BiomeStats, BiomeType, GenerationStats, MapService};
pub use resting_service::RestingService;
pub use spawning::SpawningService;
pub use tile_cache_service::{CacheStats, TileCacheService};
pub use tile_movement::TileMovementService;
pub use visibility_service::{VisibilityLevel, VisibilityService};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn services_can_be_imported() {
        // Compilation test to ensure all services are properly exported
        // Individual service tests are in their respective modules
    }
}
