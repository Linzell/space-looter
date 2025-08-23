//! Application Services - Application-Level Coordination
//!
//! This module contains application services that coordinate between
//! use cases, manage application state, and handle cross-cutting concerns
//! at the application layer.
//!
//! ## Architecture
//! - **Game Session Service**: Manages game session lifecycle and state
//! - **Input Handler Service**: Processes and validates user input
//!
//! ## Rules
//! - Coordinate between use cases and domain services
//! - Manage application-level state and workflows
//! - Handle application-specific business logic
//! - No direct infrastructure dependencies

pub mod game_session;
pub mod input_handler;

// Re-export services for convenience
pub use game_session::GameSessionService;
pub use input_handler::InputHandlerService;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn services_can_be_imported() {
        // Compilation test to ensure all services are properly exported
        // Individual service tests are in their respective modules
    }
}
