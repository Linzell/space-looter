//! Input Handling - User Input Processing and Mapping
//!
//! This module handles user input processing, key mapping, and translates
//! user actions into game events. It provides a clean interface between
//! raw input events and game logic.

use bevy::prelude::*;
use std::collections::HashMap;

/// Game actions that can be triggered by user input
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameAction {
    /// Move player up
    MoveUp,
    /// Move player down
    MoveDown,
    /// Move player left
    MoveLeft,
    /// Move player right
    MoveRight,
    /// Pause/unpause the game
    TogglePause,
    /// Start a new game
    StartGame,
    /// Restart current game
    RestartGame,
    /// Return to main menu
    ReturnToMenu,
    /// Enter settings
    EnterSettings,
    /// Exit settings
    ExitSettings,
    /// Quit application
    QuitApp,
    /// Confirm action (Enter/Space)
    Confirm,
    /// Cancel action (Escape)
    Cancel,
}

/// Input mapper for converting key presses to game actions
#[derive(Resource, Debug, Clone)]
pub struct InputMapper {
    /// Key bindings map
    key_bindings: HashMap<KeyCode, GameAction>,
    /// Mouse sensitivity
    pub mouse_sensitivity: f32,
    /// Whether input is enabled
    pub input_enabled: bool,
}

impl InputMapper {
    /// Create a new input mapper with default key bindings
    pub fn new() -> Self {
        let mut key_bindings = HashMap::new();

        // Movement keys
        key_bindings.insert(KeyCode::KeyW, GameAction::MoveUp);
        key_bindings.insert(KeyCode::ArrowUp, GameAction::MoveUp);
        key_bindings.insert(KeyCode::KeyS, GameAction::MoveDown);
        key_bindings.insert(KeyCode::ArrowDown, GameAction::MoveDown);
        key_bindings.insert(KeyCode::KeyA, GameAction::MoveLeft);
        key_bindings.insert(KeyCode::ArrowLeft, GameAction::MoveLeft);
        key_bindings.insert(KeyCode::KeyD, GameAction::MoveRight);
        key_bindings.insert(KeyCode::ArrowRight, GameAction::MoveRight);

        // Action keys
        key_bindings.insert(KeyCode::Escape, GameAction::TogglePause);
        key_bindings.insert(KeyCode::KeyP, GameAction::TogglePause);
        key_bindings.insert(KeyCode::Space, GameAction::Confirm);
        key_bindings.insert(KeyCode::Enter, GameAction::Confirm);
        key_bindings.insert(KeyCode::KeyR, GameAction::RestartGame);
        key_bindings.insert(KeyCode::KeyM, GameAction::ReturnToMenu);
        key_bindings.insert(KeyCode::KeyS, GameAction::EnterSettings);
        key_bindings.insert(KeyCode::KeyQ, GameAction::QuitApp);

        Self {
            key_bindings,
            mouse_sensitivity: 1.0,
            input_enabled: true,
        }
    }

    /// Get game action for a key code
    pub fn get_action(&self, key_code: &KeyCode) -> Option<&GameAction> {
        self.key_bindings.get(key_code)
    }

    /// Get all keys bound to an action
    pub fn get_keys_for_action(&self, action: &GameAction) -> Vec<KeyCode> {
        self.key_bindings
            .iter()
            .filter_map(|(key, act)| if act == action { Some(*key) } else { None })
            .collect()
    }

    /// Bind a key to an action
    pub fn bind_key(&mut self, key_code: KeyCode, action: GameAction) {
        self.key_bindings.insert(key_code, action);
    }

    /// Unbind a key
    pub fn unbind_key(&mut self, key_code: &KeyCode) {
        self.key_bindings.remove(key_code);
    }

    /// Clear all bindings for an action
    pub fn clear_action_bindings(&mut self, action: &GameAction) {
        self.key_bindings.retain(|_, act| act != action);
    }

    /// Check if a key is bound to any action
    pub fn is_key_bound(&self, key_code: &KeyCode) -> bool {
        self.key_bindings.contains_key(key_code)
    }

    /// Get movement vector from current input state
    pub fn get_movement_vector(&self, keyboard_input: &ButtonInput<KeyCode>) -> Vec2 {
        if !self.input_enabled {
            return Vec2::ZERO;
        }

        let mut movement = Vec2::ZERO;

        // Check movement keys
        for (key, action) in &self.key_bindings {
            if keyboard_input.pressed(*key) {
                match action {
                    GameAction::MoveUp => movement.y += 1.0,
                    GameAction::MoveDown => movement.y -= 1.0,
                    GameAction::MoveLeft => movement.x -= 1.0,
                    GameAction::MoveRight => movement.x += 1.0,
                    _ => {}
                }
            }
        }

        // Normalize diagonal movement
        if movement != Vec2::ZERO {
            movement = movement.normalize();
        }

        movement
    }

    /// Enable or disable input processing
    pub fn set_input_enabled(&mut self, enabled: bool) {
        self.input_enabled = enabled;
    }

    /// Reset to default key bindings
    pub fn reset_to_defaults(&mut self) {
        *self = Self::new();
    }
}

impl Default for InputMapper {
    fn default() -> Self {
        Self::new()
    }
}

/// Input events for game actions
#[derive(Event, Debug, Clone)]
pub struct GameActionEvent {
    /// The action that was triggered
    pub action: GameAction,
    /// Whether this was a press or release
    pub pressed: bool,
}

impl GameActionEvent {
    /// Create a new action event for a key press
    pub fn pressed(action: GameAction) -> Self {
        Self {
            action,
            pressed: true,
        }
    }

    /// Create a new action event for a key release
    pub fn released(action: GameAction) -> Self {
        Self {
            action,
            pressed: false,
        }
    }
}

/// Input context for different game states
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub enum InputContext {
    /// Main menu input context
    MainMenu,
    /// In-game input context
    InGame,
    /// Paused game input context
    Paused,
    /// Game over input context
    GameOver,
    /// Settings input context
    Settings,
    /// Loading screen (no input)
    Loading,
}

impl Default for InputContext {
    fn default() -> Self {
        Self::Loading
    }
}

/// System for processing keyboard input and generating action events
pub fn process_keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    input_mapper: Res<InputMapper>,
    input_context: Res<InputContext>,
    mut action_events: EventWriter<GameActionEvent>,
) {
    if !input_mapper.input_enabled {
        return;
    }

    // Process pressed keys
    for key in keyboard_input.get_just_pressed() {
        if let Some(action) = input_mapper.get_action(key) {
            // Filter actions based on context
            if is_action_valid_in_context(action, &input_context) {
                action_events.write(GameActionEvent::pressed(action.clone()));
            }
        }
    }

    // Process released keys
    for key in keyboard_input.get_just_released() {
        if let Some(action) = input_mapper.get_action(key) {
            if is_action_valid_in_context(action, &input_context) {
                action_events.write(GameActionEvent::released(action.clone()));
            }
        }
    }
}

/// Check if an action is valid in the current input context
fn is_action_valid_in_context(action: &GameAction, context: &InputContext) -> bool {
    match context {
        InputContext::Loading => false, // No input during loading
        InputContext::MainMenu => matches!(
            action,
            GameAction::Confirm
                | GameAction::EnterSettings
                | GameAction::QuitApp
                | GameAction::Cancel
        ),
        InputContext::InGame => matches!(
            action,
            GameAction::MoveUp
                | GameAction::MoveDown
                | GameAction::MoveLeft
                | GameAction::MoveRight
                | GameAction::TogglePause
        ),
        InputContext::Paused => matches!(
            action,
            GameAction::TogglePause
                | GameAction::RestartGame
                | GameAction::ReturnToMenu
                | GameAction::Cancel
        ),
        InputContext::GameOver => matches!(
            action,
            GameAction::Confirm
                | GameAction::RestartGame
                | GameAction::ReturnToMenu
                | GameAction::Cancel
        ),
        InputContext::Settings => matches!(
            action,
            GameAction::ExitSettings | GameAction::Cancel | GameAction::Confirm
        ),
    }
}

/// System for updating input context based on game state
pub fn update_input_context(
    mut input_context: ResMut<InputContext>,
    current_state: Res<State<crate::presentation::AppState>>,
) {
    let new_context = match current_state.get() {
        crate::presentation::AppState::Loading => InputContext::Loading,
        crate::presentation::AppState::MainMenu => InputContext::MainMenu,
        crate::presentation::AppState::Playing => InputContext::InGame,
        crate::presentation::AppState::Paused => InputContext::Paused,
        crate::presentation::AppState::GameOver => InputContext::GameOver,
        crate::presentation::AppState::Settings => InputContext::Settings,
    };

    if *input_context != new_context {
        *input_context = new_context;
    }
}

/// System for handling movement input specifically
pub fn handle_movement_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    input_mapper: Res<InputMapper>,
    input_context: Res<InputContext>,
    mut movement_events: EventWriter<crate::presentation::GameActionEvent>,
) {
    // Only process movement in game contexts
    if !matches!(*input_context, InputContext::InGame) {
        return;
    }

    let movement = input_mapper.get_movement_vector(&keyboard_input);

    if movement != Vec2::ZERO {
        movement_events.write(crate::presentation::GameActionEvent::PlayerMove {
            direction_x: movement.x,
            direction_y: movement.y,
        });
    }
}

/// Plugin for input handling
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add resources
            .init_resource::<InputMapper>()
            .init_resource::<InputContext>()
            // Add events
            .add_event::<GameActionEvent>()
            // Add systems
            .add_systems(
                Update,
                (
                    update_input_context,
                    process_keyboard_input,
                    handle_movement_input,
                )
                    .chain(),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_mapper_creation() {
        let mapper = InputMapper::new();
        assert!(mapper.input_enabled);
        assert_eq!(mapper.mouse_sensitivity, 1.0);
    }

    #[test]
    fn input_mapper_key_binding() {
        let mapper = InputMapper::new();
        assert_eq!(mapper.get_action(&KeyCode::KeyW), Some(&GameAction::MoveUp));
        assert_eq!(
            mapper.get_action(&KeyCode::ArrowUp),
            Some(&GameAction::MoveUp)
        );
    }

    #[test]
    fn input_mapper_get_keys_for_action() {
        let mapper = InputMapper::new();
        let move_up_keys = mapper.get_keys_for_action(&GameAction::MoveUp);
        assert!(move_up_keys.contains(&KeyCode::KeyW));
        assert!(move_up_keys.contains(&KeyCode::ArrowUp));
    }

    #[test]
    fn input_mapper_bind_unbind() {
        let mut mapper = InputMapper::new();

        mapper.bind_key(KeyCode::KeyZ, GameAction::MoveUp);
        assert_eq!(mapper.get_action(&KeyCode::KeyZ), Some(&GameAction::MoveUp));

        mapper.unbind_key(&KeyCode::KeyZ);
        assert_eq!(mapper.get_action(&KeyCode::KeyZ), None);
    }

    #[test]
    fn input_mapper_movement_vector() {
        let mapper = InputMapper::new();
        let mut keyboard_input = ButtonInput::<KeyCode>::default();

        // No keys pressed
        let movement = mapper.get_movement_vector(&keyboard_input);
        assert_eq!(movement, Vec2::ZERO);

        // Single direction
        keyboard_input.press(KeyCode::KeyW);
        let movement = mapper.get_movement_vector(&keyboard_input);
        assert_eq!(movement, Vec2::new(0.0, 1.0));

        // Diagonal movement should be normalized
        keyboard_input.press(KeyCode::KeyD);
        let movement = mapper.get_movement_vector(&keyboard_input);
        assert!((movement.length() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn input_mapper_disabled() {
        let mut mapper = InputMapper::new();
        let mut keyboard_input = ButtonInput::<KeyCode>::default();

        keyboard_input.press(KeyCode::KeyW);
        mapper.set_input_enabled(false);

        let movement = mapper.get_movement_vector(&keyboard_input);
        assert_eq!(movement, Vec2::ZERO);
    }

    #[test]
    fn game_action_event_creation() {
        let event = GameActionEvent::pressed(GameAction::MoveUp);
        assert_eq!(event.action, GameAction::MoveUp);
        assert!(event.pressed);

        let event = GameActionEvent::released(GameAction::MoveUp);
        assert!(!event.pressed);
    }

    #[test]
    fn input_context_validation() {
        assert!(is_action_valid_in_context(
            &GameAction::MoveUp,
            &InputContext::InGame
        ));
        assert!(!is_action_valid_in_context(
            &GameAction::MoveUp,
            &InputContext::MainMenu
        ));
        assert!(!is_action_valid_in_context(
            &GameAction::MoveUp,
            &InputContext::Loading
        ));
    }

    #[test]
    fn input_context_default() {
        let context = InputContext::default();
        assert_eq!(context, InputContext::Loading);
    }

    #[test]
    fn input_mapper_reset() {
        let mut mapper = InputMapper::new();
        mapper.mouse_sensitivity = 2.0;
        mapper.set_input_enabled(false);

        mapper.reset_to_defaults();

        assert_eq!(mapper.mouse_sensitivity, 1.0);
        assert!(mapper.input_enabled);
    }
}
