//! Game Event Logger - Integration layer between game events and UI logging
//!
//! This module provides systems that listen to game events and convert them
//! into properly formatted log messages for the UI display. It acts as a bridge
//! between the domain services and the presentation layer.

use crate::domain::services::game_log_service::{GameLogService, GameLogType};
use crate::domain::services::resting_service::{NightEventType, RestCycleResult, RestOutcome};
use crate::domain::services::tile_movement::{MovementDiceResult, MovementResult};
use crate::domain::value_objects::{Position3D, ResourceType};
use crate::infrastructure::bevy::resources::{GameStatsResource, PlayerResource};
use bevy::prelude::*;

/// Plugin for game event logging functionality
pub struct GameEventLoggerPlugin;

impl Plugin for GameEventLoggerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EventLoggerState>()
            .add_event::<MovementAttemptEvent>()
            .add_event::<RestCompletedEvent>()
            .add_event::<ResourceChangedEvent>()
            .add_event::<DiscoveryEvent>()
            .add_event::<GameSystemEvent>()
            .add_systems(
                Update,
                (
                    log_movement_events,
                    log_rest_events,
                    log_resource_events,
                    log_discovery_events,
                    log_system_events,
                ),
            );
    }
}

/// State resource to track logging status
#[derive(Resource, Default)]
pub struct EventLoggerState {
    pub last_position: Option<Position3D>,
    /// Session start time (milliseconds since epoch)
    pub session_start: Option<u64>,
}

/// Event fired when a movement attempt occurs
#[derive(Event, Debug, Clone)]
pub struct MovementAttemptEvent {
    pub from_position: Position3D,
    pub to_position: Position3D,
    pub success: bool,
    pub movement_result: Option<MovementResult>,
    pub dice_result: Option<MovementDiceResult>,
}

/// Event fired when a rest cycle completes
#[derive(Event, Debug, Clone)]
pub struct RestCompletedEvent {
    pub result: RestCycleResult,
    pub position: Position3D,
}

/// Event fired when resources change
#[derive(Event, Debug, Clone)]
pub struct ResourceChangedEvent {
    pub resource_type: ResourceType,
    pub amount_changed: i32,
    pub new_total: u32,
    pub reason: String,
}

/// Event fired when something is discovered
#[derive(Event, Debug, Clone)]
pub struct DiscoveryEvent {
    pub discovery_type: DiscoveryType,
    pub description: String,
    pub position: Position3D,
}

/// Event fired for general system messages
#[derive(Event, Debug, Clone)]
pub struct GameSystemEvent {
    pub event_type: SystemEventType,
    pub message: String,
    pub severity: SystemEventSeverity,
}

/// Types of discoveries that can be made
#[derive(Debug, Clone)]
pub enum DiscoveryType {
    NewTerrain(String),
    RandomEvent(String),
    Resource(String),
    Location(String),
    Anomaly(String),
}

/// Types of system events
#[derive(Debug, Clone)]
pub enum SystemEventType {
    MovementPointsRestored,
    RestPeriodStarted,
    RestPeriodEnded,
    InsufficientResources,
    ExplorationExhaustion,
    GameStateChange,
}

/// Severity levels for system events
#[derive(Debug, Clone)]
pub enum SystemEventSeverity {
    Info,
    Warning,
    Critical,
}

/// System to handle movement event logging
fn log_movement_events(
    mut movement_events: EventReader<MovementAttemptEvent>,
    mut game_log: ResMut<GameLogService>,
    mut logger_state: ResMut<EventLoggerState>,
) {
    for event in movement_events.read() {
        // Update last known position
        if event.success {
            logger_state.last_position = Some(event.to_position);
        }

        // Log the movement attempt
        if let Some(dice_result) = &event.dice_result {
            game_log.log_movement_attempt(
                event.from_position,
                event.to_position,
                event.success,
                Some(dice_result.base_roll),
            );

            // Log dice details
            game_log.log_dice_roll(
                dice_result.base_roll,
                dice_result.total_modifier,
                dice_result.final_result,
                &dice_result.outcome_category().to_string(),
            );
        } else {
            game_log.log_movement_attempt(
                event.from_position,
                event.to_position,
                event.success,
                None,
            );
        }

        // Log movement result details if available
        if let Some(movement_result) = &event.movement_result {
            if let Some(triggered_event) = &movement_result.triggered_event {
                game_log.log_event(triggered_event.title(), triggered_event.description());
            }

            // Log movement points gained
            if movement_result.success {
                game_log.log_message(
                    format!("Gained {} movement points from successful exploration", 2),
                    GameLogType::Resources,
                );
            }
        }
    }
}

/// System to handle rest event logging
fn log_rest_events(
    mut rest_events: EventReader<RestCompletedEvent>,
    mut game_log: ResMut<GameLogService>,
) {
    for event in rest_events.read() {
        let result = &event.result;

        // Log the rest cycle details
        game_log.log_rest_event(
            &result.night_event.to_string(),
            &result.rest_outcome.to_string(),
            &result.resources_gained,
            &result.description,
            result.dice_roll,
        );

        // Log movement points restoration
        game_log.log_movement_restoration(
            result.movement_points_restored,
            calculate_rest_duration(&result.rest_outcome),
        );
    }
}

/// System to handle resource change events
fn log_resource_events(
    mut resource_events: EventReader<ResourceChangedEvent>,
    mut game_log: ResMut<GameLogService>,
) {
    for event in resource_events.read() {
        game_log.log_resource_change(event.resource_type, event.amount_changed, &event.reason);
    }
}

/// System to handle discovery events
fn log_discovery_events(
    mut discovery_events: EventReader<DiscoveryEvent>,
    mut game_log: ResMut<GameLogService>,
) {
    for event in discovery_events.read() {
        match &event.discovery_type {
            DiscoveryType::NewTerrain(terrain) => {
                game_log.log_discovery(&format!("new terrain: {}", terrain), event.position);
            }
            DiscoveryType::RandomEvent(event_name) => {
                game_log.log_discovery(&format!("event: {}", event_name), event.position);
            }
            DiscoveryType::Resource(resource) => {
                game_log.log_discovery(&format!("resource: {}", resource), event.position);
            }
            DiscoveryType::Location(location) => {
                game_log.log_discovery(&format!("location: {}", location), event.position);
            }
            DiscoveryType::Anomaly(anomaly) => {
                game_log.log_discovery(&format!("anomaly: {}", anomaly), event.position);
            }
        }

        // Log the detailed description if provided
        if !event.description.is_empty() {
            game_log.log_message(event.description.clone(), GameLogType::Narrative);
        }
    }
}

/// System to handle general system events
fn log_system_events(
    mut system_events: EventReader<GameSystemEvent>,
    mut game_log: ResMut<GameLogService>,
) {
    for event in system_events.read() {
        let log_type = match event.severity {
            SystemEventSeverity::Info => GameLogType::System,
            SystemEventSeverity::Warning => GameLogType::Warning,
            SystemEventSeverity::Critical => GameLogType::Critical,
        };

        game_log.log_message(event.message.clone(), log_type);
    }
}

/// Calculate rest duration based on rest outcome
fn calculate_rest_duration(rest_outcome: &RestOutcome) -> u64 {
    match rest_outcome {
        RestOutcome::PoorRest => 8,        // 8 seconds for poor rest
        RestOutcome::NormalRest => 6,      // 6 seconds for normal rest
        RestOutcome::GoodRest => 5,        // 5 seconds for good rest
        RestOutcome::GreatRest => 4,       // 4 seconds for great rest
        RestOutcome::ExceptionalRest => 3, // 3 seconds for exceptional rest
    }
}

/// Helper functions to create and send events
impl MovementAttemptEvent {
    pub fn new(
        from: Position3D,
        to: Position3D,
        success: bool,
        movement_result: Option<MovementResult>,
        dice_result: Option<MovementDiceResult>,
    ) -> Self {
        Self {
            from_position: from,
            to_position: to,
            success,
            movement_result,
            dice_result,
        }
    }
}

impl RestCompletedEvent {
    pub fn new(result: RestCycleResult, position: Position3D) -> Self {
        Self { result, position }
    }
}

impl ResourceChangedEvent {
    pub fn new(
        resource_type: ResourceType,
        amount_changed: i32,
        new_total: u32,
        reason: String,
    ) -> Self {
        Self {
            resource_type,
            amount_changed,
            new_total,
            reason,
        }
    }
}

impl DiscoveryEvent {
    pub fn new(discovery_type: DiscoveryType, description: String, position: Position3D) -> Self {
        Self {
            discovery_type,
            description,
            position,
        }
    }
}

impl GameSystemEvent {
    pub fn new(
        event_type: SystemEventType,
        message: String,
        severity: SystemEventSeverity,
    ) -> Self {
        Self {
            event_type,
            message,
            severity,
        }
    }

    pub fn insufficient_resources(needed: u8, available: u8) -> Self {
        Self::new(
            SystemEventType::InsufficientResources,
            format!(
                "Movement failed: Insufficient resources: Not enough movement points. Need: {}, Have: {}",
                needed, available
            ),
            SystemEventSeverity::Warning,
        )
    }

    pub fn exploration_exhaustion() -> Self {
        Self::new(
            SystemEventType::ExplorationExhaustion,
            "Not enough movement points!".to_string(),
            SystemEventSeverity::Info,
        )
    }

    pub fn rest_period_started() -> Self {
        Self::new(
            SystemEventType::RestPeriodStarted,
            "You are exhausted and must rest for the night...".to_string(),
            SystemEventSeverity::Info,
        )
    }

    pub fn rest_period_ended() -> Self {
        Self::new(
            SystemEventType::RestPeriodEnded,
            "Dawn breaks after a night of rest".to_string(),
            SystemEventSeverity::Info,
        )
    }

    pub fn rest_drowsiness() -> Self {
        Self::new(
            SystemEventType::RestPeriodStarted,
            "You feel drowsy and must rest before moving again".to_string(),
            SystemEventSeverity::Info,
        )
    }
}

/// Convert outcome category to string for display
impl std::fmt::Display for crate::domain::services::tile_movement::EventCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            crate::domain::services::tile_movement::EventCategory::CriticalFailure => {
                write!(f, "Critical Failure")
            }
            crate::domain::services::tile_movement::EventCategory::Failure => write!(f, "Failure"),
            crate::domain::services::tile_movement::EventCategory::Neutral => write!(f, "Neutral"),
            crate::domain::services::tile_movement::EventCategory::Success => write!(f, "Success"),
            crate::domain::services::tile_movement::EventCategory::GreatSuccess => {
                write!(f, "Great Success")
            }
            crate::domain::services::tile_movement::EventCategory::CriticalSuccess => {
                write!(f, "Critical Success")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::{EntityId, PlayerStats};

    #[test]
    fn test_movement_event_creation() {
        let from = Position3D::new(0, 0, 0);
        let to = Position3D::new(1, 0, 0);

        let event = MovementAttemptEvent::new(from, to, true, None, None);

        assert_eq!(event.from_position, from);
        assert_eq!(event.to_position, to);
        assert!(event.success);
    }

    #[test]
    fn test_system_event_creation() {
        let event = GameSystemEvent::insufficient_resources(2, 0);

        assert!(event.message.contains("Need: 2"));
        assert!(event.message.contains("Have: 0"));
        matches!(event.severity, SystemEventSeverity::Warning);
    }

    #[test]
    fn test_rest_duration_calculation() {
        assert_eq!(calculate_rest_duration(&RestOutcome::PoorRest), 8);
        assert_eq!(calculate_rest_duration(&RestOutcome::NormalRest), 6);
        assert_eq!(calculate_rest_duration(&RestOutcome::ExceptionalRest), 3);
    }
}
