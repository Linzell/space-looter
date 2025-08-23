//! Game Log Service - Centralized logging for game events and narrative
//!
//! This service provides a centralized way to capture, format, and categorize
//! game events for display in the UI. It handles different types of messages
//! with appropriate formatting and timestamps.

use crate::domain::{
    entities::{Event, Player},
    value_objects::{
        resources::{ResourceCollection, ResourceType},
        Position3D,
    },
    DomainResult,
};
use chrono::{DateTime, Utc};
use std::collections::VecDeque;

/// Service for managing game log messages and events
#[derive(Debug, bevy::prelude::Resource)]
pub struct GameLogService {
    /// Queue of recent log messages
    messages: VecDeque<GameLogMessage>,
    /// Maximum number of messages to keep
    max_messages: usize,
    /// Whether to include timestamps in messages
    include_timestamps: bool,
}

/// Individual log message with metadata
#[derive(Debug, Clone)]
pub struct GameLogMessage {
    /// The formatted message text
    pub message: String,
    /// Type/category of the message
    pub log_type: GameLogType,
    /// When the message was created
    pub timestamp: DateTime<Utc>,
    /// Priority level for display ordering
    pub priority: LogPriority,
}

/// Categories of game log messages
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameLogType {
    /// Player movement and exploration
    Movement,
    /// Combat encounters and battles
    Combat,
    /// Discoveries and findings
    Discovery,
    /// Rest events and recovery
    Rest,
    /// Resource management and changes
    Resources,
    /// Random events and encounters
    Event,
    /// System messages and notifications
    System,
    /// Warnings and cautions
    Warning,
    /// Critical failures or important events
    Critical,
    /// Narrative and atmospheric text
    Narrative,
}

/// Priority levels for log messages
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogPriority {
    /// Low priority, background information
    Low = 1,
    /// Normal priority, standard game events
    Normal = 2,
    /// High priority, important events
    High = 3,
    /// Critical priority, urgent notifications
    Critical = 4,
}

impl GameLogService {
    /// Create a new game log service
    pub fn new() -> Self {
        Self {
            messages: VecDeque::new(),
            max_messages: 100,
            include_timestamps: false,
        }
    }

    /// Add a simple message to the log
    pub fn log_message(&mut self, message: String, log_type: GameLogType) {
        self.log_message_with_priority(message, log_type, LogPriority::Normal);
    }

    /// Add a message with specific priority
    pub fn log_message_with_priority(
        &mut self,
        message: String,
        log_type: GameLogType,
        priority: LogPriority,
    ) {
        let log_message = GameLogMessage {
            message,
            log_type,
            timestamp: Utc::now(),
            priority,
        };

        self.messages.push_back(log_message);

        // Keep only the most recent messages
        while self.messages.len() > self.max_messages {
            self.messages.pop_front();
        }
    }

    /// Log a movement action
    pub fn log_movement_attempt(
        &mut self,
        from: Position3D,
        to: Position3D,
        success: bool,
        dice_roll: Option<u8>,
    ) {
        let message = if success {
            match dice_roll {
                Some(roll) => format!(
                    "Moved from ({}, {}) to ({}, {}) - Roll: {}",
                    from.x, from.y, to.x, to.y, roll
                ),
                None => format!(
                    "Moved from ({}, {}) to ({}, {})",
                    from.x, from.y, to.x, to.y
                ),
            }
        } else {
            format!(
                "Failed to move to ({}, {}) - insufficient resources",
                to.x, to.y
            )
        };

        let log_type = if success {
            GameLogType::Movement
        } else {
            GameLogType::Warning
        };

        self.log_message(message, log_type);
    }

    /// Log a rest event
    pub fn log_rest_event(
        &mut self,
        night_event: &str,
        rest_quality: &str,
        resources_gained: &ResourceCollection,
        description: &str,
        dice_roll: u8,
    ) {
        // Log the main rest event
        self.log_message(
            format!("Night Roll: {} - {}", dice_roll, night_event),
            GameLogType::Rest,
        );

        self.log_message(format!("Rest Quality: {}", rest_quality), GameLogType::Rest);

        // Log the narrative description
        self.log_message(description.to_string(), GameLogType::Narrative);

        // Log resources gained if any
        if !resources_gained.is_empty() {
            let resource_text = format_resource_collection(resources_gained);
            self.log_message(
                format!("Resources gained during rest: {}", resource_text),
                GameLogType::Resources,
            );
        }
    }

    /// Log movement points restoration
    pub fn log_movement_restoration(&mut self, points: u8, rest_duration_seconds: u64) {
        self.log_message(
            format!(
                "Movement points restored: {} - resting for {} seconds",
                points, rest_duration_seconds
            ),
            GameLogType::System,
        );
    }

    /// Log resource changes
    pub fn log_resource_change(&mut self, resource_type: ResourceType, amount: i32, reason: &str) {
        let message = if amount > 0 {
            format!("Gained {} {} from {}", amount, resource_type, reason)
        } else {
            format!("Lost {} {} from {}", -amount, resource_type, reason)
        };

        self.log_message(message, GameLogType::Resources);
    }

    /// Log a discovery
    pub fn log_discovery(&mut self, discovery: &str, location: Position3D) {
        self.log_message(
            format!(
                "Discovered {} at ({}, {})",
                discovery, location.x, location.y
            ),
            GameLogType::Discovery,
        );
    }

    /// Log a random event
    pub fn log_event(&mut self, event_title: &str, event_description: &str) {
        self.log_message(
            format!("Event: {} - {}", event_title, event_description),
            GameLogType::Event,
        );
    }

    /// Log insufficient resources warning
    pub fn log_insufficient_resources(&mut self, action: &str, needed: u8, available: u8) {
        self.log_message(
            format!(
                "Cannot {}: Need {} movement points, have {}",
                action, needed, available
            ),
            GameLogType::Warning,
        );
    }

    /// Log exploration exhaustion
    pub fn log_exploration_exhaustion(&mut self) {
        self.log_message(
            "You are exhausted and must rest for the night".to_string(),
            GameLogType::System,
        );
    }

    /// Log rest completion
    pub fn log_rest_completion(&mut self) {
        self.log_message(
            "Rest period complete, you can now move again".to_string(),
            GameLogType::System,
        );
    }

    /// Log dice roll results
    pub fn log_dice_roll(&mut self, base_roll: u8, modifiers: i8, final_result: u8, outcome: &str) {
        let modifier_text = if modifiers >= 0 {
            format!("+{}", modifiers)
        } else {
            modifiers.to_string()
        };

        self.log_message(
            format!(
                "Rolled {} {} = {} - Outcome: {}",
                base_roll, modifier_text, final_result, outcome
            ),
            GameLogType::System,
        );
    }

    /// Get recent messages for display (oldest first, so newest appears at bottom)
    pub fn get_recent_messages(&self, count: usize) -> Vec<&GameLogMessage> {
        let total_messages = self.messages.len();
        if total_messages <= count {
            self.messages.iter().collect()
        } else {
            self.messages.iter().skip(total_messages - count).collect()
        }
    }

    /// Get all messages
    pub fn get_all_messages(&self) -> Vec<&GameLogMessage> {
        self.messages.iter().collect()
    }

    /// Clear all messages
    pub fn clear_messages(&mut self) {
        self.messages.clear();
    }

    /// Get message count
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }
}

impl Default for GameLogService {
    fn default() -> Self {
        Self::new()
    }
}

/// Format a resource collection into a readable string
fn format_resource_collection(resources: &ResourceCollection) -> String {
    let mut parts = Vec::new();

    for resource_type in [
        ResourceType::Food,
        ResourceType::Energy,
        ResourceType::Data,
        ResourceType::Technology,
    ] {
        let amount = resources.get_amount(resource_type);
        if amount > 0 {
            parts.push(format!("{}: {}", resource_type, amount));
        }
    }

    if parts.is_empty() {
        "None".to_string()
    } else {
        parts.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::resources::ResourceCollection;

    #[test]
    fn test_service_creation() {
        let service = GameLogService::new();
        assert_eq!(service.message_count(), 0);
        assert!(service.get_recent_messages(10).is_empty());
    }

    #[test]
    fn test_add_message() {
        let mut service = GameLogService::new();
        service.log_message("Test message".to_string(), GameLogType::System);

        assert_eq!(service.message_count(), 1);
        let messages = service.get_recent_messages(1);
        assert_eq!(messages[0].message, "Test message");
        assert_eq!(messages[0].log_type, GameLogType::System);
    }

    #[test]
    fn test_message_limit() {
        let mut service = GameLogService::new();
        service.max_messages = 3;

        for i in 0..5 {
            service.log_message(format!("Message {}", i), GameLogType::System);
        }

        assert_eq!(service.message_count(), 3);
        let messages = service.get_recent_messages(3);
        assert_eq!(messages[0].message, "Message 4");
        assert_eq!(messages[1].message, "Message 3");
        assert_eq!(messages[2].message, "Message 2");
    }

    #[test]
    fn test_movement_logging() {
        let mut service = GameLogService::new();
        let from = Position3D::new(0, 0, 0);
        let to = Position3D::new(1, 0, 0);

        service.log_movement_attempt(from, to, true, Some(15));

        assert_eq!(service.message_count(), 1);
        let messages = service.get_recent_messages(1);
        assert!(messages[0].message.contains("Moved from (0, 0) to (1, 0)"));
        assert!(messages[0].message.contains("Roll: 15"));
        assert_eq!(messages[0].log_type, GameLogType::Movement);
    }

    #[test]
    fn test_rest_logging() {
        let mut service = GameLogService::new();
        let mut resources = ResourceCollection::new();
        resources.set_amount(ResourceType::Food, 5);

        service.log_rest_event(
            "Peaceful Rest",
            "Normal Rest",
            &resources,
            "A peaceful night under the stars",
            12,
        );

        assert_eq!(service.message_count(), 3); // Night event, quality, description
        let messages = service.get_recent_messages(3);

        // Check that all expected messages are present
        let message_texts: Vec<&str> = messages.iter().map(|m| m.message.as_str()).collect();
        assert!(message_texts.iter().any(|m| m.contains("Night Roll: 12")));
        assert!(message_texts
            .iter()
            .any(|m| m.contains("Rest Quality: Normal Rest")));
        assert!(message_texts.iter().any(|m| m.contains("peaceful night")));
    }

    #[test]
    fn test_resource_formatting() {
        let mut resources = ResourceCollection::new();
        resources.set_amount(ResourceType::Food, 10);
        resources.set_amount(ResourceType::Energy, 5);

        let formatted = format_resource_collection(&resources);
        assert!(formatted.contains("Food: 10"));
        assert!(formatted.contains("Energy: 5"));
    }
}
