//! Presentation Layer - User Interface and Input Handling
//!
//! This layer manages user interface, input handling, and game state presentation.
//! It coordinates between the user interactions and the application layer,
//! translating user inputs into application commands and presenting game state
//! to the user.
//!
//! ## Architecture
//! - **Game State**: Application state management and transitions
//! - **Input Handling**: User input processing and mapping
//! - **Rendering**: Coordination of visual presentation
//!
//! ## Rules
//! - Can depend on application and domain layers
//! - Handles UI state and user interactions
//! - Translates between user actions and application commands
//! - Manages presentation logic (not business logic)

pub mod audio_integration;
pub mod game_event_logger;
pub mod game_log_integration;
pub mod game_state;
pub mod game_ui;
pub mod input;
pub mod log_interceptor;
pub mod map_renderer;
pub mod rendering;

// Re-export common presentation types
pub use audio_integration::{AudioAssets, AudioEventIntegrationPlugin};
pub use game_state::RpgAppState;
pub use input::{GameAction, InputMapper};

use bevy::prelude::*;

/// Presentation-specific error types
#[derive(Debug, Clone, PartialEq)]
pub enum PresentationError {
    /// Input processing error
    InputError(String),
    /// Rendering error
    RenderError(String),
    /// State transition error
    StateTransitionError(String),
    /// UI component error
    UIError(String),
}

impl std::fmt::Display for PresentationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PresentationError::InputError(msg) => write!(f, "Input error: {}", msg),
            PresentationError::RenderError(msg) => write!(f, "Render error: {}", msg),
            PresentationError::StateTransitionError(msg) => {
                write!(f, "State transition error: {}", msg)
            }
            PresentationError::UIError(msg) => write!(f, "UI error: {}", msg),
        }
    }
}

impl std::error::Error for PresentationError {}

/// Common result type for presentation operations
pub type PresentationResult<T> = Result<T, PresentationError>;

/// Events for state transitions
#[derive(Event, Debug, Clone)]
pub enum StateTransitionEvent {
    /// Start the game from menu
    StartGame,
    /// Pause the current game
    PauseGame,
    /// Resume paused game
    ResumeGame,
    /// End current game and go to game over
    EndGame,
    /// Return to main menu
    ReturnToMenu,
    /// Restart the current game
    RestartGame,
}

/// Events for game actions
#[derive(Event, Debug, Clone)]
pub enum GameActionEvent {
    /// Player movement
    PlayerMove { direction_x: f32, direction_y: f32 },
    /// Game pause toggle
    TogglePause,
    /// Exit game
    ExitGame,
}

/// UI state for different screens
#[derive(Resource, Debug, Clone)]
pub struct UIState {
    /// Whether UI is visible
    pub visible: bool,
    /// Current score display
    pub score_text: String,
    /// Game time display
    pub time_text: String,
    /// Whether pause menu is open
    pub pause_menu_open: bool,
}

impl Default for UIState {
    fn default() -> Self {
        Self {
            visible: true,
            score_text: "Score: 0".to_string(),
            time_text: "Time: 00:00".to_string(),
            pause_menu_open: false,
        }
    }
}

/// Input configuration for different controls
#[derive(Resource, Debug, Clone)]
pub struct InputConfig {
    /// Movement keys
    pub move_up: Vec<KeyCode>,
    pub move_down: Vec<KeyCode>,
    pub move_left: Vec<KeyCode>,
    pub move_right: Vec<KeyCode>,
    /// Action keys
    pub pause: Vec<KeyCode>,
    pub exit: Vec<KeyCode>,
    /// Mouse sensitivity
    pub mouse_sensitivity: f32,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            move_up: vec![KeyCode::KeyW, KeyCode::ArrowUp],
            move_down: vec![KeyCode::KeyS, KeyCode::ArrowDown],
            move_left: vec![KeyCode::KeyA, KeyCode::ArrowLeft],
            move_right: vec![KeyCode::KeyD, KeyCode::ArrowRight],
            pause: vec![KeyCode::Escape, KeyCode::KeyP],
            exit: vec![KeyCode::KeyQ],
            mouse_sensitivity: 1.0,
        }
    }
}

/// System sets for organizing presentation systems
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum PresentationSystemSet {
    /// Input processing systems
    Input,
    /// UI update systems
    UIUpdate,
    /// Rendering coordination systems
    Rendering,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_state_transitions() {
        // Test that states can be created and compared
        assert_eq!(RpgAppState::default(), RpgAppState::Loading);
        assert_ne!(RpgAppState::Loading, RpgAppState::Exploration);
    }

    #[test]
    fn presentation_error_display() {
        let error = PresentationError::InputError("test error".to_string());
        assert_eq!(error.to_string(), "Input error: test error");
    }

    #[test]
    fn ui_state_default() {
        let ui_state = UIState::default();
        assert!(ui_state.visible);
        assert_eq!(ui_state.score_text, "Score: 0");
        assert!(!ui_state.pause_menu_open);
    }

    #[test]
    fn input_config_default() {
        let config = InputConfig::default();
        assert!(config.move_up.contains(&KeyCode::KeyW));
        assert!(config.move_up.contains(&KeyCode::ArrowUp));
        assert_eq!(config.mouse_sensitivity, 1.0);
    }

    #[test]
    fn state_transition_events() {
        let event = StateTransitionEvent::StartGame;
        // Test that event can be created (compilation test)
        match event {
            StateTransitionEvent::StartGame => (),
            _ => panic!("Wrong event variant"),
        }
    }
}
