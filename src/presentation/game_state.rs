//! RPG Game State Management - 3D Isometric RPG State Handling
//!
//! This module manages the different states of the 3D isometric dice RPG,
//! handling transitions between exploration, combat, base management,
//! and other RPG-specific states.

use crate::domain::{
    entities::{Base, Player, Quest},
    value_objects::{dice::DiceResult, Position3D, ResourceCollection},
};
use bevy::prelude::*;
use std::collections::HashMap;

/// RPG application states
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum RpgAppState {
    #[default]
    Loading,
    MainMenu,
    CharacterCreation,
    Exploration,
    Combat,
    BaseManagement,
    QuestLog,
    Inventory,
    Settings,
    Paused,
    GameOver,
}

/// RPG-specific state transition events
#[derive(Event, Debug, Clone)]
pub enum RpgStateTransitionEvent {
    /// Start character creation
    CreateCharacter,
    /// Begin exploration with created character
    StartExploration,
    /// Enter combat encounter
    EnterCombat {
        encounter_type: String,
        difficulty: u8,
    },
    /// Enter base management mode
    EnterBase,
    /// Open quest log
    OpenQuestLog,
    /// Open inventory management
    OpenInventory,
    /// Pause the game
    PauseGame,
    /// Resume from pause
    ResumeGame,
    /// End game session
    EndGame {
        experience_gained: u32,
        resources_collected: ResourceCollection,
    },
    /// Return to main menu
    ReturnToMenu,
    /// Enter settings
    EnterSettings,
    /// Exit settings back to previous state
    ExitSettings { previous_state: RpgAppState },
    /// Quit application
    QuitApp,
}

/// RPG game session data
#[derive(Resource, Debug, Clone)]
pub struct RpgGameSession {
    /// Current player character
    pub player: Player,
    /// Player's base
    pub base: Base,
    /// Active quests
    pub active_quests: Vec<Quest>,
    /// Completed quests
    pub completed_quests: Vec<Quest>,
    /// Current exploration position
    pub current_position: Position3D,
    /// Session start time
    pub session_start: std::time::Instant,
    /// Total play time in seconds
    pub total_play_time: u32,
    /// Last save timestamp
    pub last_save: Option<std::time::Instant>,
}

impl RpgGameSession {
    /// Create a new game session with a player
    pub fn new(player: Player, base: Base) -> Self {
        Self {
            current_position: *player.position(),
            player,
            base,
            active_quests: Vec::new(),
            completed_quests: Vec::new(),
            session_start: std::time::Instant::now(),
            total_play_time: 0,
            last_save: None,
        }
    }

    /// Get current player level
    pub fn player_level(&self) -> u32 {
        self.player.level()
    }

    /// Get total resources value
    pub fn total_resources_value(&self) -> u32 {
        self.player.resources().total_value() + self.base.resources().total_value()
    }

    /// Get active quest count
    pub fn active_quest_count(&self) -> usize {
        self.active_quests.len()
    }

    /// Get completed quest count
    pub fn completed_quest_count(&self) -> usize {
        self.completed_quests.len()
    }

    /// Check if player can afford base upgrade
    pub fn can_afford_base_upgrade(&self) -> bool {
        // Placeholder cost calculation for base upgrade
        let upgrade_cost = self.base.level() as u32 * 1000;
        self.player.resources().total_value() >= upgrade_cost
    }

    /// Update play time
    pub fn update_play_time(&mut self) {
        let elapsed = self.session_start.elapsed().as_secs() as u32;
        self.total_play_time = elapsed;
    }

    /// Mark as saved
    pub fn mark_saved(&mut self) {
        self.last_save = Some(std::time::Instant::now());
    }

    /// Check if needs saving (more than 5 minutes since last save)
    pub fn needs_save(&self) -> bool {
        match self.last_save {
            Some(last) => last.elapsed().as_secs() > 300, // 5 minutes
            None => true,                                 // Never saved
        }
    }
}

/// Current exploration context
#[derive(Resource, Debug, Clone)]
pub struct ExplorationContext {
    /// Current map location
    pub current_location: Position3D,
    /// Discovered locations
    pub discovered_locations: HashMap<Position3D, String>,
    /// Recent dice results
    pub recent_dice_results: Vec<DiceResult>,
    /// Movement points remaining
    pub movement_points_remaining: u8,
    /// Action points remaining
    pub action_points_remaining: u8,
    /// Current exploration bonus modifiers
    pub exploration_modifiers: HashMap<String, i8>,
}

impl ExplorationContext {
    /// Create new exploration context
    pub fn new(starting_position: Position3D) -> Self {
        Self {
            current_location: starting_position,
            discovered_locations: HashMap::new(),
            recent_dice_results: Vec::new(),
            movement_points_remaining: 3, // Default movement points per turn
            action_points_remaining: 2,   // Default action points per turn
            exploration_modifiers: HashMap::new(),
        }
    }

    /// Record a dice result
    pub fn add_dice_result(&mut self, result: DiceResult) {
        self.recent_dice_results.push(result);
        // Keep only last 10 results
        if self.recent_dice_results.len() > 10 {
            self.recent_dice_results.remove(0);
        }
    }

    /// Discover a new location
    pub fn discover_location(&mut self, position: Position3D, description: String) {
        self.discovered_locations.insert(position, description);
    }

    /// Use movement point
    pub fn use_movement_point(&mut self) -> bool {
        if self.movement_points_remaining > 0 {
            self.movement_points_remaining -= 1;
            true
        } else {
            false
        }
    }

    /// Use action point
    pub fn use_action_point(&mut self) -> bool {
        if self.action_points_remaining > 0 {
            self.action_points_remaining -= 1;
            true
        } else {
            false
        }
    }

    /// Reset points for new turn
    pub fn reset_turn_points(&mut self) {
        self.movement_points_remaining = 3;
        self.action_points_remaining = 2;
    }

    /// Get exploration summary
    pub fn exploration_summary(&self) -> ExplorationSummary {
        ExplorationSummary {
            current_location: self.current_location,
            locations_discovered: self.discovered_locations.len(),
            movement_points: self.movement_points_remaining,
            action_points: self.action_points_remaining,
            can_move: self.movement_points_remaining > 0,
            can_act: self.action_points_remaining > 0,
        }
    }
}

/// Summary of exploration state for UI display
#[derive(Debug, Clone)]
pub struct ExplorationSummary {
    pub current_location: Position3D,
    pub locations_discovered: usize,
    pub movement_points: u8,
    pub action_points: u8,
    pub can_move: bool,
    pub can_act: bool,
}

/// Combat encounter state
#[derive(Resource, Debug, Clone)]
pub struct CombatState {
    /// Type of encounter
    pub encounter_type: String,
    /// Combat difficulty (1-20)
    pub difficulty: u8,
    /// Player's combat modifiers
    pub player_modifiers: HashMap<String, i8>,
    /// Combat round counter
    pub current_round: u32,
    /// Combat log entries
    pub combat_log: Vec<String>,
    /// Whether combat is resolved
    pub is_resolved: bool,
    /// Combat outcome
    pub outcome: Option<CombatOutcome>,
}

impl CombatState {
    /// Create new combat state
    pub fn new(encounter_type: String, difficulty: u8) -> Self {
        Self {
            encounter_type,
            difficulty,
            player_modifiers: HashMap::new(),
            current_round: 1,
            combat_log: Vec::new(),
            is_resolved: false,
            outcome: None,
        }
    }

    /// Add log entry
    pub fn add_log_entry(&mut self, entry: String) {
        self.combat_log.push(entry);
        // Keep only last 20 entries
        if self.combat_log.len() > 20 {
            self.combat_log.remove(0);
        }
    }

    /// Advance to next round
    pub fn next_round(&mut self) {
        self.current_round += 1;
        self.add_log_entry(format!("--- Round {} ---", self.current_round));
    }

    /// Resolve combat with outcome
    pub fn resolve(&mut self, outcome: CombatOutcome) {
        self.outcome = Some(outcome.clone());
        self.is_resolved = true;
        self.add_log_entry(format!("Combat resolved: {:?}", outcome));
    }
}

/// Combat resolution outcomes
#[derive(Debug, Clone)]
pub enum CombatOutcome {
    Victory {
        experience_gained: u32,
        resources_gained: ResourceCollection,
    },
    Defeat {
        resources_lost: ResourceCollection,
        lesson_learned: String,
    },
    Escape {
        resources_spent: ResourceCollection,
    },
    Negotiation {
        deal_struck: String,
        resources_traded: ResourceCollection,
    },
}

/// State transition system
pub fn handle_rpg_state_transitions(
    mut next_state: ResMut<NextState<RpgAppState>>,
    mut transition_events: EventReader<RpgStateTransitionEvent>,
    _current_state: Res<State<RpgAppState>>,
) {
    for event in transition_events.read() {
        match event {
            RpgStateTransitionEvent::CreateCharacter => {
                next_state.set(RpgAppState::CharacterCreation);
            }
            RpgStateTransitionEvent::StartExploration => {
                next_state.set(RpgAppState::Exploration);
            }
            RpgStateTransitionEvent::EnterCombat { .. } => {
                next_state.set(RpgAppState::Combat);
            }
            RpgStateTransitionEvent::EnterBase => {
                next_state.set(RpgAppState::BaseManagement);
            }
            RpgStateTransitionEvent::OpenQuestLog => {
                next_state.set(RpgAppState::QuestLog);
            }
            RpgStateTransitionEvent::OpenInventory => {
                next_state.set(RpgAppState::Inventory);
            }
            RpgStateTransitionEvent::PauseGame => {
                next_state.set(RpgAppState::Paused);
            }
            RpgStateTransitionEvent::ResumeGame => {
                next_state.set(RpgAppState::Exploration);
            }
            RpgStateTransitionEvent::EndGame { .. } => {
                next_state.set(RpgAppState::GameOver);
            }
            RpgStateTransitionEvent::ReturnToMenu => {
                next_state.set(RpgAppState::MainMenu);
            }
            RpgStateTransitionEvent::EnterSettings => {
                next_state.set(RpgAppState::Settings);
            }
            RpgStateTransitionEvent::ExitSettings { previous_state } => {
                next_state.set(previous_state.clone());
            }
            RpgStateTransitionEvent::QuitApp => {
                // Handle application quit
                std::process::exit(0);
            }
        }
    }
}

/// Update game session data
pub fn update_game_session(mut game_session: ResMut<RpgGameSession>, _time: Res<Time>) {
    game_session.update_play_time();
}

/// RPG state management plugin
pub struct RpgStatePlugin;

impl Plugin for RpgStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<RpgAppState>()
            .add_event::<RpgStateTransitionEvent>()
            .add_systems(Update, (handle_rpg_state_transitions, update_game_session));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{EntityId, PlayerStats};

    #[test]
    fn rpg_game_session_creation() {
        let entity_id = EntityId::generate();
        let player_stats = PlayerStats::new(10, 10, 10, 10, 10, 10).unwrap();
        let player = Player::new(
            entity_id,
            "TestPlayer".to_string(),
            Position3D::origin(),
            player_stats,
        )
        .unwrap();

        let base_id = EntityId::generate();
        let base = Base::new(base_id, "TestBase".to_string(), Position3D::origin()).unwrap();

        let session = RpgGameSession::new(player, base);
        assert_eq!(session.player_level(), 1);
        assert_eq!(session.active_quest_count(), 0);
    }

    #[test]
    fn exploration_context_functionality() {
        let mut context = ExplorationContext::new(Position3D::origin());

        assert!(context.use_movement_point());
        assert_eq!(context.movement_points_remaining, 2);

        context.discover_location(Position3D::new(1, 0, 0), "Forest".to_string());
        assert_eq!(context.discovered_locations.len(), 1);
    }

    #[test]
    fn combat_state_management() {
        let mut combat = CombatState::new("Bandit".to_string(), 10);

        combat.add_log_entry("Combat begins!".to_string());
        assert_eq!(combat.combat_log.len(), 1);

        combat.next_round();
        assert_eq!(combat.current_round, 2);
        assert!(!combat.is_resolved);
    }
}
