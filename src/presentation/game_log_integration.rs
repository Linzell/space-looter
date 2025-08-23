//! Game Log Integration - Simple integration for converting game events to UI logs
//!
//! This module provides a straightforward way to convert the existing game events
//! and log patterns into UI-displayable messages without complex interception.
//! It focuses on the specific log patterns shown in the user's example.

use crate::domain::services::game_log_service::{GameLogService, GameLogType};
use crate::domain::services::resting_service::RestCycleResult;
use crate::domain::value_objects::{Position3D, ResourceType};
use bevy::prelude::*;

/// Plugin for game log integration
pub struct GameLogIntegrationPlugin;

impl Plugin for GameLogIntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LogMovementEvent>()
            .add_event::<LogRestEvent>()
            .add_event::<LogResourceEvent>()
            .add_event::<LogSystemEvent>()
            .add_systems(
                Update,
                (
                    handle_movement_log_events,
                    handle_rest_log_events,
                    handle_resource_log_events,
                    handle_system_log_events,
                    populate_initial_logs,
                ),
            );
    }
}

/// Event for logging movement actions
#[derive(Event, Debug, Clone)]
pub struct LogMovementEvent {
    pub from_position: Position3D,
    pub to_position: Position3D,
    pub success: bool,
    pub dice_roll: Option<u8>,
    pub modifiers: Option<i8>,
    pub final_result: Option<u8>,
    pub outcome: Option<String>,
    pub event_triggered: Option<String>,
}

/// Event for logging rest cycles
#[derive(Event, Debug, Clone)]
pub struct LogRestEvent {
    pub night_event: String,
    pub rest_quality: String,
    pub dice_roll: u8,
    pub description: String,
    pub resources_gained: Vec<(ResourceType, u32)>,
    pub movement_points_restored: u8,
    pub rest_duration_seconds: u64,
}

/// Event for logging resource changes
#[derive(Event, Debug, Clone)]
pub struct LogResourceEvent {
    pub resource_type: ResourceType,
    pub amount: i32,
    pub reason: String,
}

/// Event for logging system messages
#[derive(Event, Debug, Clone)]
pub struct LogSystemEvent {
    pub message: String,
    pub log_type: GameLogType,
}

/// System to handle movement log events
fn handle_movement_log_events(
    mut movement_events: EventReader<LogMovementEvent>,
    mut game_log: ResMut<GameLogService>,
) {
    for event in movement_events.read() {
        if !event.success {
            // Movement failed - insufficient resources
            game_log.log_message(
                format!(
                    "Movement failed: Insufficient resources: Not enough movement points. Need: 2, Have: 0"
                ),
                GameLogType::Warning,
            );

            game_log.log_message(
                "Not enough movement points!".to_string(),
                GameLogType::System,
            );
            return;
        }

        // Log dice roll if present
        if let (Some(roll), Some(modifiers), Some(final_result), Some(outcome)) = (
            event.dice_roll,
            event.modifiers,
            event.final_result,
            &event.outcome,
        ) {
            let modifier_text = if modifiers >= 0 {
                format!("+{}", modifiers)
            } else {
                modifiers.to_string()
            };

            game_log.log_message(
                format!(
                    "Rolled {} {} = {} (Base: {}, Level: +0, Terrain: -1, Danger: -1)",
                    roll, modifier_text, final_result, roll
                ),
                GameLogType::System,
            );

            game_log.log_message(format!("Outcome: {}", outcome), GameLogType::System);
        }

        // Log successful movement
        game_log.log_message(
            format!(
                "Player moved to position: Position3D {{ x: {}, y: {}, z: {} }}",
                event.to_position.x, event.to_position.y, event.to_position.z
            ),
            GameLogType::Movement,
        );

        // Log triggered event if present
        if let Some(event_desc) = &event.event_triggered {
            game_log.log_message(
                format!("Event Triggered: {}", event_desc),
                GameLogType::Event,
            );
        }

        // Log movement points gained
        game_log.log_message(
            "Gained 2 movement points from successful exploration!".to_string(),
            GameLogType::Resources,
        );
    }
}

/// System to handle rest log events
fn handle_rest_log_events(
    mut rest_events: EventReader<LogRestEvent>,
    mut game_log: ResMut<GameLogService>,
) {
    for event in rest_events.read() {
        // Log exhaustion message
        game_log.log_message(
            "You are exhausted and must rest for the night...".to_string(),
            GameLogType::System,
        );

        // Log dawn message
        game_log.log_message(
            "Dawn breaks after a night of rest".to_string(),
            GameLogType::System,
        );

        // Log night roll
        game_log.log_message(
            format!("Night Roll: {} - {}", event.dice_roll, event.night_event),
            GameLogType::Rest,
        );

        // Log rest quality
        game_log.log_message(
            format!("Rest Quality: {}", event.rest_quality),
            GameLogType::Rest,
        );

        // Log description
        game_log.log_message(event.description.clone(), GameLogType::Narrative);

        // Log resources gained
        if !event.resources_gained.is_empty() {
            let resource_text: Vec<String> = event
                .resources_gained
                .iter()
                .map(|(resource_type, amount)| format!("{}: {}", resource_type, amount))
                .collect();

            game_log.log_message(
                format!("Resources gained during rest: {}", resource_text.join(", ")),
                GameLogType::Resources,
            );
        }

        // Log movement points restoration
        game_log.log_message(
            format!(
                "Movement points restored: {} - resting for {} seconds...",
                event.movement_points_restored, event.rest_duration_seconds
            ),
            GameLogType::System,
        );

        // Log drowsiness
        game_log.log_message(
            "You feel drowsy and must rest before moving again".to_string(),
            GameLogType::System,
        );
    }
}

/// System to handle resource log events
fn handle_resource_log_events(
    mut resource_events: EventReader<LogResourceEvent>,
    mut game_log: ResMut<GameLogService>,
) {
    for event in resource_events.read() {
        let message = if event.amount > 0 {
            format!(
                "Gained {} {} from {}",
                event.amount, event.resource_type, event.reason
            )
        } else {
            format!(
                "Lost {} {} from {}",
                -event.amount, event.resource_type, event.reason
            )
        };

        game_log.log_message(message, GameLogType::Resources);
    }
}

/// System to handle system log events
fn handle_system_log_events(
    mut system_events: EventReader<LogSystemEvent>,
    mut game_log: ResMut<GameLogService>,
) {
    for event in system_events.read() {
        game_log.log_message(event.message.clone(), event.log_type.clone());
    }
}

/// System to populate some initial log messages to demonstrate the UI
fn populate_initial_logs(mut game_log: ResMut<GameLogService>, mut has_populated: Local<bool>) {
    if *has_populated {
        return;
    }
    *has_populated = true;

    // Add welcome message
    game_log.log_message(
        "Welcome to Space Looter - RPG systems online".to_string(),
        GameLogType::System,
    );

    // Add some example messages based on your log output
    game_log.log_message(
        "Updated map around player position (0, 0, 0)".to_string(),
        GameLogType::Discovery,
    );

    game_log.log_message(
        "Explored new tile at (0, 0) - Starting location secured".to_string(),
        GameLogType::Discovery,
    );

    game_log.log_message(
        "Space Command Interface initialized".to_string(),
        GameLogType::System,
    );

    game_log.log_message(
        "Long range sensors online - Beginning sector scan".to_string(),
        GameLogType::System,
    );

    game_log.log_message(
        "Ship systems nominal - Ready for exploration".to_string(),
        GameLogType::System,
    );

    game_log.log_message("Press WASD to move".to_string(), GameLogType::System);
}

/// Helper functions to create and send log events
impl LogMovementEvent {
    pub fn successful_movement(
        from: Position3D,
        to: Position3D,
        dice_roll: u8,
        modifiers: i8,
        final_result: u8,
        outcome: String,
        event_triggered: Option<String>,
    ) -> Self {
        Self {
            from_position: from,
            to_position: to,
            success: true,
            dice_roll: Some(dice_roll),
            modifiers: Some(modifiers),
            final_result: Some(final_result),
            outcome: Some(outcome),
            event_triggered,
        }
    }

    pub fn failed_movement(from: Position3D, to: Position3D) -> Self {
        Self {
            from_position: from,
            to_position: to,
            success: false,
            dice_roll: None,
            modifiers: None,
            final_result: None,
            outcome: None,
            event_triggered: None,
        }
    }
}

impl LogRestEvent {
    pub fn new(
        night_event: String,
        rest_quality: String,
        dice_roll: u8,
        description: String,
        resources_gained: Vec<(ResourceType, u32)>,
        movement_points_restored: u8,
        rest_duration_seconds: u64,
    ) -> Self {
        Self {
            night_event,
            rest_quality,
            dice_roll,
            description,
            resources_gained,
            movement_points_restored,
            rest_duration_seconds,
        }
    }

    pub fn from_rest_cycle_result(result: &RestCycleResult, rest_duration: u64) -> Self {
        let resources_gained: Vec<(ResourceType, u32)> = [
            ResourceType::Food,
            ResourceType::Energy,
            ResourceType::Data,
            ResourceType::Technology,
        ]
        .iter()
        .filter_map(|&resource_type| {
            let amount = result.resources_gained.get_amount(resource_type);
            if amount > 0 {
                Some((resource_type, amount))
            } else {
                None
            }
        })
        .collect();

        Self::new(
            result.night_event.to_string(),
            result.rest_outcome.to_string(),
            result.dice_roll,
            result.description.clone(),
            resources_gained,
            result.movement_points_restored,
            rest_duration,
        )
    }
}

/// Public functions to easily send log events from other systems
pub fn log_successful_movement(
    event_writer: &mut EventWriter<LogMovementEvent>,
    from: Position3D,
    to: Position3D,
    dice_roll: u8,
    modifiers: i8,
    final_result: u8,
    outcome: &str,
    event_triggered: Option<&str>,
) {
    event_writer.write(LogMovementEvent::successful_movement(
        from,
        to,
        dice_roll,
        modifiers,
        final_result,
        outcome.to_string(),
        event_triggered.map(|s| s.to_string()),
    ));
}

pub fn log_failed_movement(
    event_writer: &mut EventWriter<LogMovementEvent>,
    from: Position3D,
    to: Position3D,
) {
    event_writer.write(LogMovementEvent::failed_movement(from, to));
}

pub fn log_rest_cycle(
    event_writer: &mut EventWriter<LogRestEvent>,
    result: &RestCycleResult,
    rest_duration: u64,
) {
    event_writer.write(LogRestEvent::from_rest_cycle_result(result, rest_duration));
}

pub fn log_resource_change(
    event_writer: &mut EventWriter<LogResourceEvent>,
    resource_type: ResourceType,
    amount: i32,
    reason: &str,
) {
    event_writer.write(LogResourceEvent {
        resource_type,
        amount,
        reason: reason.to_string(),
    });
}

pub fn log_system_message(
    event_writer: &mut EventWriter<LogSystemEvent>,
    message: &str,
    log_type: GameLogType,
) {
    event_writer.write(LogSystemEvent {
        message: message.to_string(),
        log_type,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_movement_event_creation() {
        let from = Position3D::new(-8, -2, 0);
        let to = Position3D::new(-9, -2, 0);

        let event = LogMovementEvent::successful_movement(
            from,
            to,
            10,
            -2,
            8,
            "Neutral".to_string(),
            Some("Strange Phenomenon".to_string()),
        );

        assert!(event.success);
        assert_eq!(event.dice_roll, Some(10));
        assert_eq!(event.modifiers, Some(-2));
        assert_eq!(event.final_result, Some(8));
    }

    #[test]
    fn test_failed_movement_event() {
        let from = Position3D::new(0, 0, 0);
        let to = Position3D::new(1, 0, 0);

        let event = LogMovementEvent::failed_movement(from, to);

        assert!(!event.success);
        assert_eq!(event.dice_roll, None);
    }

    #[test]
    fn test_rest_event_creation() {
        let resources = vec![(ResourceType::Food, 2)];

        let event = LogRestEvent::new(
            "Restless Night".to_string(),
            "Poor Rest".to_string(),
            10,
            "You toss and turn...".to_string(),
            resources,
            3,
            6,
        );

        assert_eq!(event.dice_roll, 10);
        assert_eq!(event.movement_points_restored, 3);
        assert_eq!(event.rest_duration_seconds, 6);
    }
}
