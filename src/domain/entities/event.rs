//! Event Entity - Random events and encounters in the game world
//!
//! This entity represents random events that can occur during exploration,
//! resource gathering, and other game activities.

use crate::domain::value_objects::resources::ResourceCollection;
use crate::domain::value_objects::{EntityId, GameTime, Position3D};
use crate::domain::{DomainError, DomainResult};
use chrono::{DateTime, Utc};

/// A game event entity
#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    id: EntityId,
    event_type: EventType,
    title: String,
    description: String,
    location: Option<Position3D>,
    triggered_at: DateTime<Utc>,
    duration: Option<GameTime>,
    outcomes: Vec<EventOutcome>,
    is_resolved: bool,
    version: u64,
}

impl Event {
    /// Create a new event
    pub fn new(
        event_type: EventType,
        title: String,
        description: String,
        location: Option<Position3D>,
    ) -> DomainResult<Self> {
        if title.is_empty() || title.len() > 100 {
            return Err(DomainError::ValidationError(
                "Event title must be between 1 and 100 characters".to_string(),
            ));
        }

        if description.is_empty() || description.len() > 500 {
            return Err(DomainError::ValidationError(
                "Event description must be between 1 and 500 characters".to_string(),
            ));
        }

        Ok(Self {
            id: EntityId::generate(),
            event_type,
            title,
            description,
            location,
            triggered_at: Utc::now(),
            duration: None,
            outcomes: Vec::new(),
            is_resolved: false,
            version: 1,
        })
    }

    /// Get event ID
    pub fn id(&self) -> &EntityId {
        &self.id
    }

    /// Get event type
    pub fn event_type(&self) -> EventType {
        self.event_type
    }

    /// Get event title
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Get event description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Get event location
    pub fn location(&self) -> Option<&Position3D> {
        self.location.as_ref()
    }

    /// Get when event was triggered
    pub fn triggered_at(&self) -> DateTime<Utc> {
        self.triggered_at
    }

    /// Check if event is resolved
    pub fn is_resolved(&self) -> bool {
        self.is_resolved
    }

    /// Get event outcomes
    pub fn outcomes(&self) -> &[EventOutcome] {
        &self.outcomes
    }

    /// Resolve the event with an outcome
    pub fn resolve(&mut self, outcome: EventOutcome) -> DomainResult<()> {
        if self.is_resolved {
            return Err(DomainError::EventTriggerError(
                "Event is already resolved".to_string(),
            ));
        }

        self.outcomes.push(outcome);
        self.is_resolved = true;
        self.version += 1;
        Ok(())
    }

    /// Check if event has expired (for timed events)
    pub fn is_expired(&self, current_time: GameTime) -> bool {
        if let Some(duration) = &self.duration {
            let elapsed = current_time
                .seconds()
                .saturating_sub(self.triggered_at.timestamp() as u32);
            elapsed >= duration.seconds()
        } else {
            false
        }
    }
}

/// Types of events that can occur
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    /// Resource discovery event
    ResourceDiscovery,
    /// Hostile encounter
    Combat,
    /// Trading opportunity
    Trade,
    /// Environmental hazard
    Hazard,
    /// Mysterious phenomenon
    Mystery,
    /// Equipment malfunction
    Malfunction,
    /// Beneficial discovery
    Boon,
    /// Story/lore event
    Narrative,
    /// Base-related event
    BaseEvent,
}

impl EventType {
    /// Get all event types
    pub fn all() -> Vec<EventType> {
        vec![
            EventType::ResourceDiscovery,
            EventType::Combat,
            EventType::Trade,
            EventType::Hazard,
            EventType::Mystery,
            EventType::Malfunction,
            EventType::Boon,
            EventType::Narrative,
            EventType::BaseEvent,
        ]
    }

    /// Get base probability for this event type (0.0 to 1.0)
    pub fn base_probability(&self) -> f32 {
        match self {
            EventType::ResourceDiscovery => 0.25,
            EventType::Combat => 0.15,
            EventType::Trade => 0.10,
            EventType::Hazard => 0.20,
            EventType::Mystery => 0.05,
            EventType::Malfunction => 0.10,
            EventType::Boon => 0.08,
            EventType::Narrative => 0.05,
            EventType::BaseEvent => 0.02,
        }
    }

    /// Check if this event type is dangerous
    pub fn is_dangerous(&self) -> bool {
        matches!(
            self,
            EventType::Combat | EventType::Hazard | EventType::Malfunction
        )
    }

    /// Check if this event type is beneficial
    pub fn is_beneficial(&self) -> bool {
        matches!(
            self,
            EventType::ResourceDiscovery | EventType::Trade | EventType::Boon
        )
    }
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::ResourceDiscovery => write!(f, "Resource Discovery"),
            EventType::Combat => write!(f, "Combat"),
            EventType::Trade => write!(f, "Trade"),
            EventType::Hazard => write!(f, "Hazard"),
            EventType::Mystery => write!(f, "Mystery"),
            EventType::Malfunction => write!(f, "Malfunction"),
            EventType::Boon => write!(f, "Boon"),
            EventType::Narrative => write!(f, "Narrative"),
            EventType::BaseEvent => write!(f, "Base Event"),
        }
    }
}

/// Outcome of an event after resolution
#[derive(Debug, Clone, PartialEq)]
pub struct EventOutcome {
    pub outcome_type: OutcomeType,
    pub resources_gained: Option<ResourceCollection>,
    pub resources_lost: Option<ResourceCollection>,
    pub experience_gained: u32,
    pub description: String,
}

impl EventOutcome {
    /// Create a new event outcome
    pub fn new(
        outcome_type: OutcomeType,
        resources_gained: Option<ResourceCollection>,
        resources_lost: Option<ResourceCollection>,
        experience_gained: u32,
        description: String,
    ) -> Self {
        Self {
            outcome_type,
            resources_gained,
            resources_lost,
            experience_gained,
            description,
        }
    }

    /// Create a successful outcome
    pub fn success(
        resources_gained: ResourceCollection,
        experience_gained: u32,
        description: String,
    ) -> Self {
        Self::new(
            OutcomeType::Success,
            Some(resources_gained),
            None,
            experience_gained,
            description,
        )
    }

    /// Create a failure outcome
    pub fn failure(resources_lost: ResourceCollection, description: String) -> Self {
        Self::new(
            OutcomeType::Failure,
            None,
            Some(resources_lost),
            0,
            description,
        )
    }

    /// Create a neutral outcome
    pub fn neutral(description: String) -> Self {
        Self::new(OutcomeType::Neutral, None, None, 0, description)
    }
}

/// Types of event outcomes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutcomeType {
    Success,
    Failure,
    Neutral,
    Mixed,
}

impl std::fmt::Display for OutcomeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutcomeType::Success => write!(f, "Success"),
            OutcomeType::Failure => write!(f, "Failure"),
            OutcomeType::Neutral => write!(f, "Neutral"),
            OutcomeType::Mixed => write!(f, "Mixed"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_creation() {
        let event = Event::new(
            EventType::ResourceDiscovery,
            "Found Resources".to_string(),
            "You discovered a cache of metal ore".to_string(),
            Some(Position3D::new(5, 5, 0)),
        )
        .unwrap();

        assert_eq!(event.event_type(), EventType::ResourceDiscovery);
        assert_eq!(event.title(), "Found Resources");
        assert!(!event.is_resolved());
    }

    #[test]
    fn event_resolution() {
        let mut event = Event::new(
            EventType::Trade,
            "Trading Post".to_string(),
            "A merchant offers to trade".to_string(),
            None,
        )
        .unwrap();

        let outcome = EventOutcome::neutral("You declined the trade".to_string());
        event.resolve(outcome).unwrap();

        assert!(event.is_resolved());
        assert_eq!(event.outcomes().len(), 1);

        // Cannot resolve again
        let another_outcome = EventOutcome::neutral("Another outcome".to_string());
        assert!(event.resolve(another_outcome).is_err());
    }

    #[test]
    fn event_type_properties() {
        assert!(EventType::Combat.is_dangerous());
        assert!(!EventType::Combat.is_beneficial());

        assert!(EventType::Boon.is_beneficial());
        assert!(!EventType::Boon.is_dangerous());

        assert!(EventType::ResourceDiscovery.base_probability() > 0.0);
    }

    #[test]
    fn event_outcome_creation() {
        let mut resources = ResourceCollection::new();
        resources.set_amount(crate::domain::value_objects::ResourceType::Metal, 50);

        let outcome =
            EventOutcome::success(resources, 25, "Successfully gathered resources".to_string());

        assert_eq!(outcome.outcome_type, OutcomeType::Success);
        assert_eq!(outcome.experience_gained, 25);
        assert!(outcome.resources_gained.is_some());
    }

    #[test]
    fn invalid_event_creation() {
        // Empty title
        let result = Event::new(
            EventType::Mystery,
            "".to_string(),
            "Description".to_string(),
            None,
        );
        assert!(result.is_err());

        // Empty description
        let result = Event::new(
            EventType::Mystery,
            "Title".to_string(),
            "".to_string(),
            None,
        );
        assert!(result.is_err());
    }
}
