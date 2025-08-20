//! Input Handler Service - Application-Level Input Processing
//!
//! Processes and validates user input, translates input events into
//! application commands, and coordinates with use cases for input handling.

use crate::application::{ApplicationError, ApplicationResult};
use std::collections::HashMap;

/// Types of input actions that can be performed
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputAction {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Pause,
    Resume,
    Exit,
}

/// Input state for tracking pressed keys
#[derive(Debug, Clone)]
pub struct InputState {
    pressed_keys: HashMap<String, bool>,
    movement_vector: (f32, f32),
}

impl InputState {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashMap::new(),
            movement_vector: (0.0, 0.0),
        }
    }

    pub fn is_key_pressed(&self, key: &str) -> bool {
        *self.pressed_keys.get(key).unwrap_or(&false)
    }

    pub fn movement_vector(&self) -> (f32, f32) {
        self.movement_vector
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

/// Service for handling input processing at the application level
pub struct InputHandlerService {
    input_state: InputState,
    key_mappings: HashMap<String, InputAction>,
}

impl InputHandlerService {
    /// Create a new input handler service
    pub fn new() -> Self {
        let mut service = Self {
            input_state: InputState::new(),
            key_mappings: HashMap::new(),
        };

        // Set up default key mappings
        service.setup_default_mappings();
        service
    }

    /// Set up default key mappings
    fn setup_default_mappings(&mut self) {
        self.key_mappings
            .insert("w".to_string(), InputAction::MoveUp);
        self.key_mappings
            .insert("s".to_string(), InputAction::MoveDown);
        self.key_mappings
            .insert("a".to_string(), InputAction::MoveLeft);
        self.key_mappings
            .insert("d".to_string(), InputAction::MoveRight);
        self.key_mappings
            .insert("ArrowUp".to_string(), InputAction::MoveUp);
        self.key_mappings
            .insert("ArrowDown".to_string(), InputAction::MoveDown);
        self.key_mappings
            .insert("ArrowLeft".to_string(), InputAction::MoveLeft);
        self.key_mappings
            .insert("ArrowRight".to_string(), InputAction::MoveRight);
        self.key_mappings
            .insert("p".to_string(), InputAction::Pause);
        self.key_mappings
            .insert("Escape".to_string(), InputAction::Pause);
        self.key_mappings.insert("q".to_string(), InputAction::Exit);
    }

    /// Update key state (pressed or released)
    pub fn update_key_state(&mut self, key: &str, pressed: bool) {
        self.input_state
            .pressed_keys
            .insert(key.to_string(), pressed);
        self.update_movement_vector();
    }

    /// Update movement vector based on currently pressed keys
    fn update_movement_vector(&mut self) {
        let mut x = 0.0;
        let mut y = 0.0;

        // Check movement keys
        for (key, action) in &self.key_mappings {
            if self.input_state.is_key_pressed(key) {
                match action {
                    InputAction::MoveLeft => x -= 1.0,
                    InputAction::MoveRight => x += 1.0,
                    InputAction::MoveUp => y += 1.0,
                    InputAction::MoveDown => y -= 1.0,
                    _ => {}
                }
            }
        }

        self.input_state.movement_vector = (x, y);
    }

    /// Get current movement vector
    pub fn get_movement_vector(&self) -> (f32, f32) {
        self.input_state.movement_vector()
    }

    /// Check if a specific action is currently active
    pub fn is_action_active(&self, action: &InputAction) -> bool {
        for (key, mapped_action) in &self.key_mappings {
            if mapped_action == action && self.input_state.is_key_pressed(key) {
                return true;
            }
        }
        false
    }

    /// Get all currently active actions
    pub fn get_active_actions(&self) -> Vec<InputAction> {
        let mut active_actions = Vec::new();

        for (key, action) in &self.key_mappings {
            if self.input_state.is_key_pressed(key) && !active_actions.contains(action) {
                active_actions.push(action.clone());
            }
        }

        active_actions
    }

    /// Map a key to an action
    pub fn map_key(&mut self, key: String, action: InputAction) {
        self.key_mappings.insert(key, action);
    }

    /// Remove key mapping
    pub fn unmap_key(&mut self, key: &str) {
        self.key_mappings.remove(key);
    }

    /// Get current input state (read-only)
    pub fn input_state(&self) -> &InputState {
        &self.input_state
    }

    /// Process input and return application commands
    pub fn process_input(&self) -> Vec<InputCommand> {
        let mut commands = Vec::new();

        // Check for movement
        let (x, y) = self.get_movement_vector();
        if x != 0.0 || y != 0.0 {
            commands.push(InputCommand::Move {
                direction_x: x,
                direction_y: y,
            });
        }

        // Check for other actions
        if self.is_action_active(&InputAction::Pause) {
            commands.push(InputCommand::Pause);
        }

        if self.is_action_active(&InputAction::Exit) {
            commands.push(InputCommand::Exit);
        }

        commands
    }
}

impl Default for InputHandlerService {
    fn default() -> Self {
        Self::new()
    }
}

/// Commands that can be generated from input processing
#[derive(Debug, Clone, PartialEq)]
pub enum InputCommand {
    Move { direction_x: f32, direction_y: f32 },
    Pause,
    Resume,
    Exit,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_handler_service_creation() {
        let service = InputHandlerService::new();
        assert_eq!(service.get_movement_vector(), (0.0, 0.0));
    }

    #[test]
    fn key_state_updates() {
        let mut service = InputHandlerService::new();

        // Press 'w' key (move up)
        service.update_key_state("w", true);
        assert!(service.is_action_active(&InputAction::MoveUp));
        assert_eq!(service.get_movement_vector(), (0.0, 1.0));

        // Release 'w' key
        service.update_key_state("w", false);
        assert!(!service.is_action_active(&InputAction::MoveUp));
        assert_eq!(service.get_movement_vector(), (0.0, 0.0));
    }

    #[test]
    fn movement_vector_calculation() {
        let mut service = InputHandlerService::new();

        // Press multiple movement keys
        service.update_key_state("w", true); // Up
        service.update_key_state("d", true); // Right

        let (x, y) = service.get_movement_vector();
        assert_eq!(x, 1.0);
        assert_eq!(y, 1.0);
    }

    #[test]
    fn input_command_generation() {
        let mut service = InputHandlerService::new();

        // Press movement key
        service.update_key_state("a", true);

        let commands = service.process_input();
        assert_eq!(commands.len(), 1);

        if let InputCommand::Move {
            direction_x,
            direction_y,
        } = &commands[0]
        {
            assert_eq!(*direction_x, -1.0);
            assert_eq!(*direction_y, 0.0);
        } else {
            panic!("Expected Move command");
        }
    }

    #[test]
    fn custom_key_mapping() {
        let mut service = InputHandlerService::new();

        // Map custom key
        service.map_key("space".to_string(), InputAction::Pause);
        service.update_key_state("space", true);

        assert!(service.is_action_active(&InputAction::Pause));
    }

    #[test]
    fn active_actions_tracking() {
        let mut service = InputHandlerService::new();

        service.update_key_state("w", true);
        service.update_key_state("p", true);

        let active_actions = service.get_active_actions();
        assert!(active_actions.contains(&InputAction::MoveUp));
        assert!(active_actions.contains(&InputAction::Pause));
    }
}
