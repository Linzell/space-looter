//! Game State Management - Application State Handling
//!
//! This module manages the different states of the game application,
//! handling transitions between states and coordinating state-specific
//! systems and UI elements.

use crate::domain::Score;
use bevy::prelude::*;
use std::time::Duration;

/// Application states for the game
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Loading,
    MainMenu,
    Playing,
    Paused,
    GameOver,
    Settings,
}

/// State transition events
#[derive(Event, Debug, Clone)]
pub enum StateTransitionEvent {
    /// Start a new game
    StartGame,
    /// Pause the current game
    PauseGame,
    /// Resume paused game
    ResumeGame,
    /// End current game and go to game over
    EndGame { final_score: Score },
    /// Return to main menu
    ReturnToMenu,
    /// Restart the current game
    RestartGame,
    /// Enter settings
    EnterSettings,
    /// Exit settings
    ExitSettings,
    /// Quit the application
    QuitApp,
}

/// Game state data that persists across state transitions
#[derive(Resource, Debug, Clone)]
pub struct GameStateData {
    /// Current score
    pub current_score: Score,
    /// Best score achieved
    pub best_score: Score,
    /// Time spent in current session
    pub session_time: Duration,
    /// Game statistics
    pub stats: GameStats,
    /// Whether this is the first time playing
    pub first_time: bool,
}

impl Default for GameStateData {
    fn default() -> Self {
        Self {
            current_score: Score::zero(),
            best_score: Score::zero(),
            session_time: Duration::new(0, 0),
            stats: GameStats::default(),
            first_time: true,
        }
    }
}

/// Game statistics
#[derive(Debug, Clone, Default)]
pub struct GameStats {
    /// Total games played
    pub games_played: u32,
    /// Total enemies destroyed
    pub total_enemies_destroyed: u32,
    /// Total time played
    pub total_time_played: Duration,
    /// Highest score achieved
    pub highest_score: Score,
    /// Current session stats
    pub session_stats: SessionStats,
}

/// Statistics for current session
#[derive(Debug, Clone, Default)]
pub struct SessionStats {
    /// Enemies destroyed this session
    pub enemies_destroyed: u32,
    /// Collisions this session
    pub collisions: u32,
    /// Time played this session
    pub time_played: Duration,
}

impl GameStateData {
    /// Create new game state data
    pub fn new() -> Self {
        Self::default()
    }

    /// Update current score
    pub fn update_score(&mut self, new_score: Score) {
        self.current_score = new_score;

        // Update best score if needed
        if new_score.is_higher_than(&self.best_score) {
            self.best_score = new_score;
            self.stats.highest_score = new_score;
        }
    }

    /// Start a new game session
    pub fn start_new_game(&mut self) {
        self.current_score = Score::zero();
        self.session_time = Duration::new(0, 0);
        self.stats.session_stats = SessionStats::default();
        self.stats.games_played += 1;
        self.first_time = false;
    }

    /// End current game session
    pub fn end_game(&mut self, final_score: Score) {
        self.current_score = final_score;

        // Update statistics
        if final_score.is_higher_than(&self.best_score) {
            self.best_score = final_score;
            self.stats.highest_score = final_score;
        }

        // Add session stats to totals
        self.stats.total_enemies_destroyed += self.stats.session_stats.enemies_destroyed;
        self.stats.total_time_played += self.stats.session_stats.time_played;
    }

    /// Update session time
    pub fn update_session_time(&mut self, delta_time: Duration) {
        self.session_time += delta_time;
        self.stats.session_stats.time_played += delta_time;
    }

    /// Record enemy destruction
    pub fn record_enemy_destroyed(&mut self) {
        self.stats.session_stats.enemies_destroyed += 1;
        self.stats.session_stats.collisions += 1;
    }

    /// Get formatted session time
    pub fn formatted_session_time(&self) -> String {
        let total_seconds = self.session_time.as_secs();
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

    /// Check if this is a new high score
    pub fn is_new_high_score(&self) -> bool {
        self.current_score.is_higher_than(&self.best_score)
    }

    /// Get accuracy percentage for current session
    pub fn session_accuracy(&self) -> f32 {
        let destroyed = self.stats.session_stats.enemies_destroyed;
        let collisions = self.stats.session_stats.collisions;

        if collisions == 0 {
            0.0
        } else {
            (destroyed as f32 / collisions as f32) * 100.0
        }
    }
}

/// System for handling state transitions
pub fn handle_state_transitions(
    mut next_state: ResMut<NextState<AppState>>,
    mut state_events: EventReader<StateTransitionEvent>,
    mut game_state_data: ResMut<GameStateData>,
    current_state: Res<State<AppState>>,
) {
    for event in state_events.read() {
        match event {
            StateTransitionEvent::StartGame => {
                info!("Starting new game");
                game_state_data.start_new_game();
                next_state.set(AppState::Playing);
            }
            StateTransitionEvent::PauseGame => {
                if *current_state.get() == AppState::Playing {
                    info!("Pausing game");
                    next_state.set(AppState::Paused);
                }
            }
            StateTransitionEvent::ResumeGame => {
                if *current_state.get() == AppState::Paused {
                    info!("Resuming game");
                    next_state.set(AppState::Playing);
                }
            }
            StateTransitionEvent::EndGame { final_score } => {
                info!("Ending game with score: {}", final_score);
                game_state_data.end_game(*final_score);
                next_state.set(AppState::GameOver);
            }
            StateTransitionEvent::ReturnToMenu => {
                info!("Returning to main menu");
                next_state.set(AppState::MainMenu);
            }
            StateTransitionEvent::RestartGame => {
                info!("Restarting game");
                game_state_data.start_new_game();
                next_state.set(AppState::Playing);
            }
            StateTransitionEvent::EnterSettings => {
                info!("Entering settings");
                next_state.set(AppState::Settings);
            }
            StateTransitionEvent::ExitSettings => {
                info!("Exiting settings");
                next_state.set(AppState::MainMenu);
            }
            StateTransitionEvent::QuitApp => {
                info!("Quitting application");
                // In web builds, this might not actually quit
                #[cfg(not(target_arch = "wasm32"))]
                std::process::exit(0);
            }
        }
    }
}

/// System for updating game session time
pub fn update_session_time_system(
    time: Res<Time>,
    mut game_state_data: ResMut<GameStateData>,
    current_state: Res<State<AppState>>,
) {
    // Only update time when actually playing
    if *current_state.get() == AppState::Playing {
        game_state_data.update_session_time(time.delta());
    }
}

/// System for handling keyboard shortcuts for state transitions
pub fn keyboard_state_shortcuts(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut state_events: EventWriter<StateTransitionEvent>,
    current_state: Res<State<AppState>>,
) {
    match current_state.get() {
        AppState::Playing => {
            if keyboard_input.just_pressed(KeyCode::Escape) {
                state_events.write(StateTransitionEvent::PauseGame);
            }
        }
        AppState::Paused => {
            if keyboard_input.just_pressed(KeyCode::Escape) {
                state_events.write(StateTransitionEvent::ResumeGame);
            }
            if keyboard_input.just_pressed(KeyCode::KeyR) {
                state_events.write(StateTransitionEvent::RestartGame);
            }
            if keyboard_input.just_pressed(KeyCode::KeyM) {
                state_events.write(StateTransitionEvent::ReturnToMenu);
            }
        }
        AppState::MainMenu => {
            if keyboard_input.just_pressed(KeyCode::Space) {
                state_events.write(StateTransitionEvent::StartGame);
            }
            if keyboard_input.just_pressed(KeyCode::KeyS) {
                state_events.write(StateTransitionEvent::EnterSettings);
            }
            if keyboard_input.just_pressed(KeyCode::KeyQ) {
                state_events.write(StateTransitionEvent::QuitApp);
            }
        }
        AppState::GameOver => {
            if keyboard_input.just_pressed(KeyCode::Space) {
                state_events.write(StateTransitionEvent::RestartGame);
            }
            if keyboard_input.just_pressed(KeyCode::KeyM) {
                state_events.write(StateTransitionEvent::ReturnToMenu);
            }
        }
        AppState::Settings => {
            if keyboard_input.just_pressed(KeyCode::Escape) {
                state_events.write(StateTransitionEvent::ExitSettings);
            }
        }
        AppState::Loading => {
            // No shortcuts during loading
        }
    }
}

/// Plugin for game state management
pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app
            // Initialize state
            .init_state::<AppState>()
            // Add resources
            .init_resource::<GameStateData>()
            // Add events
            .add_event::<StateTransitionEvent>()
            // Add systems
            .add_systems(
                Update,
                (
                    handle_state_transitions,
                    update_session_time_system,
                    keyboard_state_shortcuts,
                )
                    .chain(),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_state_data_creation() {
        let data = GameStateData::new();
        assert_eq!(data.current_score, Score::zero());
        assert_eq!(data.best_score, Score::zero());
        assert!(data.first_time);
    }

    #[test]
    fn game_state_data_score_update() {
        let mut data = GameStateData::new();
        let new_score = Score::from(100);

        data.update_score(new_score);

        assert_eq!(data.current_score, new_score);
        assert_eq!(data.best_score, new_score);
    }

    #[test]
    fn game_state_data_new_game() {
        let mut data = GameStateData::new();
        data.current_score = Score::from(100);

        data.start_new_game();

        assert_eq!(data.current_score, Score::zero());
        assert_eq!(data.stats.games_played, 1);
        assert!(!data.first_time);
    }

    #[test]
    fn game_state_data_enemy_tracking() {
        let mut data = GameStateData::new();

        data.record_enemy_destroyed();
        data.record_enemy_destroyed();

        assert_eq!(data.stats.session_stats.enemies_destroyed, 2);
        assert_eq!(data.stats.session_stats.collisions, 2);
    }

    #[test]
    fn game_state_data_time_formatting() {
        let mut data = GameStateData::new();
        data.session_time = Duration::from_secs(125); // 2:05

        let formatted = data.formatted_session_time();
        assert_eq!(formatted, "02:05");
    }

    #[test]
    fn game_state_data_accuracy_calculation() {
        let mut data = GameStateData::new();

        // No collisions yet
        assert_eq!(data.session_accuracy(), 0.0);

        // Add some collisions
        data.stats.session_stats.enemies_destroyed = 8;
        data.stats.session_stats.collisions = 10;

        assert_eq!(data.session_accuracy(), 80.0);
    }

    #[test]
    fn app_state_equality() {
        assert_eq!(AppState::default(), AppState::Loading);
        assert_ne!(AppState::Loading, AppState::Playing);
    }

    #[test]
    fn state_transition_event_creation() {
        let event = StateTransitionEvent::StartGame;
        match event {
            StateTransitionEvent::StartGame => (),
            _ => panic!("Wrong event variant"),
        }

        let score = Score::from(100);
        let end_event = StateTransitionEvent::EndGame { final_score: score };
        match end_event {
            StateTransitionEvent::EndGame { final_score } => {
                assert_eq!(final_score, score);
            }
            _ => panic!("Wrong event variant"),
        }
    }
}
