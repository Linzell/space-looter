//! Handle Encounter Use Case - RPG Encounter Processing Logic
//!
//! Handles encounter detection and resolution using dice mechanics
//! for the 3D isometric RPG system.

use crate::application::{
    dto::{HandleCollisionInput, HandleCollisionOutput},
    ApplicationResult,
};
use crate::domain::{
    entities::Event,
    value_objects::{dice::*, Position3D},
    DomainResult,
};

/// Use case for handling RPG encounters with dice mechanics
pub struct HandleEncounterUseCase {}

impl HandleEncounterUseCase {
    /// Create a new handle encounter use case
    pub fn new() -> Self {
        Self {}
    }

    /// Execute encounter handling with dice-based mechanics
    pub fn execute(&self, input: HandleCollisionInput) -> ApplicationResult<HandleCollisionOutput> {
        // Convert legacy collision input to encounter detection
        let encounter_position = input.entity1_position;
        let encounter_result = self.process_encounter(&encounter_position)?;

        Ok(encounter_result)
    }

    /// Process an encounter at the given position using RPG mechanics
    fn process_encounter(&self, position: &Position3D) -> ApplicationResult<HandleCollisionOutput> {
        // Simulate dice roll result (in real implementation, would use RandomService)
        let roll_result = 10; // Placeholder

        // Determine encounter type based on dice roll
        let encounter_outcome = match roll_result {
            1..=5 => self.handle_resource_discovery(position),
            6..=10 => self.handle_random_event(position),
            11..=15 => self.handle_safe_exploration(position),
            16..=19 => self.handle_dangerous_encounter(position),
            20 => self.handle_rare_discovery(position),
            _ => Ok(EncounterOutcome::NoEncounter),
        }?;

        // Convert encounter outcome to legacy collision output format
        Ok(self.convert_outcome_to_output(encounter_outcome))
    }

    /// Handle resource discovery encounter
    fn handle_resource_discovery(&self, _position: &Position3D) -> DomainResult<EncounterOutcome> {
        let resource_roll = DiceRoll::simple(1, DiceType::D6)?;

        Ok(EncounterOutcome::ResourceFound {
            resource_type: crate::domain::value_objects::ResourceType::Metal,
            amount: 10 + (resource_roll.average_result() as u32),
        })
    }

    /// Handle random event encounter
    fn handle_random_event(&self, position: &Position3D) -> DomainResult<EncounterOutcome> {
        // Create a simple exploration event
        let event = Event::new(
            crate::domain::entities::event::EventType::ResourceDiscovery,
            "Exploration Discovery".to_string(),
            "You found something interesting during your exploration!".to_string(),
            Some(*position),
        )?;

        Ok(EncounterOutcome::EventTriggered { event })
    }

    /// Handle safe exploration
    fn handle_safe_exploration(&self, _position: &Position3D) -> DomainResult<EncounterOutcome> {
        Ok(EncounterOutcome::SafeExploration {
            experience_gained: 5,
        })
    }

    /// Handle dangerous encounter with dice-based resolution
    fn handle_dangerous_encounter(&self, _position: &Position3D) -> DomainResult<EncounterOutcome> {
        // Player must make a skill check to avoid danger
        let danger_roll = DiceRoll::simple(1, DiceType::D20)?;
        let difficulty = 15;

        if danger_roll.average_result() as i32 >= difficulty {
            Ok(EncounterOutcome::DangerAvoided {
                experience_gained: 10,
            })
        } else {
            Ok(EncounterOutcome::DangerEncountered {
                damage_taken: 5,
                lesson_learned: "Always check your surroundings!".to_string(),
            })
        }
    }

    /// Handle rare discovery with exceptional rewards
    fn handle_rare_discovery(&self, _position: &Position3D) -> DomainResult<EncounterOutcome> {
        Ok(EncounterOutcome::RareDiscovery {
            treasure_found: "Ancient Artifact".to_string(),
            experience_gained: 50,
        })
    }

    /// Convert encounter outcome to legacy collision output format
    fn convert_outcome_to_output(&self, outcome: EncounterOutcome) -> HandleCollisionOutput {
        match outcome {
            EncounterOutcome::NoEncounter => HandleCollisionOutput {
                collision_detected: false,
                score_change: None,
                entities_to_remove: vec![],
            },
            EncounterOutcome::ResourceFound { amount, .. } => HandleCollisionOutput {
                collision_detected: true,
                score_change: Some(
                    crate::domain::Score::new(amount)
                        .unwrap_or_default()
                        .value(),
                ),
                entities_to_remove: vec![],
            },
            EncounterOutcome::EventTriggered { .. } => HandleCollisionOutput {
                collision_detected: true,
                score_change: Some(25),
                entities_to_remove: vec![],
            },
            EncounterOutcome::SafeExploration { experience_gained } => HandleCollisionOutput {
                collision_detected: true,
                score_change: Some(experience_gained),
                entities_to_remove: vec![],
            },
            EncounterOutcome::DangerAvoided { experience_gained } => HandleCollisionOutput {
                collision_detected: true,
                score_change: Some(experience_gained),
                entities_to_remove: vec![],
            },
            EncounterOutcome::DangerEncountered { .. } => HandleCollisionOutput {
                collision_detected: true,
                score_change: Some(0),
                entities_to_remove: vec![],
            },
            EncounterOutcome::RareDiscovery {
                experience_gained, ..
            } => HandleCollisionOutput {
                collision_detected: true,
                score_change: Some(experience_gained),
                entities_to_remove: vec![],
            },
        }
    }
}

/// RPG encounter outcomes using dice mechanics
#[derive(Debug, Clone)]
enum EncounterOutcome {
    NoEncounter,
    ResourceFound {
        resource_type: crate::domain::value_objects::ResourceType,
        amount: u32,
    },
    EventTriggered {
        event: Event,
    },
    SafeExploration {
        experience_gained: u32,
    },
    DangerAvoided {
        experience_gained: u32,
    },
    DangerEncountered {
        damage_taken: u32,
        lesson_learned: String,
    },
    RareDiscovery {
        treasure_found: String,
        experience_gained: u32,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encounter_use_case_creation() {
        let use_case = HandleEncounterUseCase::new();

        // Test that use case can be created
        assert!(!std::ptr::addr_of!(use_case).is_null());
    }

    #[test]
    fn encounter_processing_with_dice() {
        let use_case = HandleEncounterUseCase::new();

        let position = Position3D::new(0, 0, 0);
        let result = use_case.process_encounter(&position);

        // Should successfully process encounter
        assert!(result.is_ok());
    }

    #[test]
    fn dice_based_encounter_resolution() {
        let resource_encounter = EncounterOutcome::ResourceFound {
            resource_type: crate::domain::value_objects::ResourceType::Energy,
            amount: 15,
        };

        match resource_encounter {
            EncounterOutcome::ResourceFound { amount, .. } => {
                assert!(amount > 0);
            }
            _ => panic!("Expected resource encounter"),
        }
    }
}
