//! Quest Entity - Objectives and missions for the player
//!
//! This entity represents quests that guide player exploration and provide
//! structured gameplay objectives with rewards.

use crate::domain::value_objects::resources::ResourceCollection;
use crate::domain::value_objects::{EntityId, Position3D};
use crate::domain::{DomainError, DomainResult};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// A quest entity
#[derive(Debug, Clone, PartialEq)]
pub struct Quest {
    id: EntityId,
    title: String,
    description: String,
    quest_type: QuestType,
    status: QuestStatus,
    objectives: Vec<QuestObjective>,
    rewards: QuestRewards,
    prerequisites: Vec<QuestPrerequisite>,
    created_at: DateTime<Utc>,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    version: u64,
}

impl Quest {
    /// Create a new quest
    pub fn new(
        title: String,
        description: String,
        quest_type: QuestType,
        objectives: Vec<QuestObjective>,
        rewards: QuestRewards,
    ) -> DomainResult<Self> {
        if title.is_empty() || title.len() > 100 {
            return Err(DomainError::ValidationError(
                "Quest title must be between 1 and 100 characters".to_string(),
            ));
        }

        if description.is_empty() || description.len() > 500 {
            return Err(DomainError::ValidationError(
                "Quest description must be between 1 and 500 characters".to_string(),
            ));
        }

        if objectives.is_empty() {
            return Err(DomainError::QuestError(
                "Quest must have at least one objective".to_string(),
            ));
        }

        Ok(Self {
            id: EntityId::generate(),
            title,
            description,
            quest_type,
            status: QuestStatus::Available,
            objectives,
            rewards,
            prerequisites: Vec::new(),
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            version: 1,
        })
    }

    /// Get quest ID
    pub fn id(&self) -> &EntityId {
        &self.id
    }

    /// Get quest title
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Get quest description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Get quest type
    pub fn quest_type(&self) -> QuestType {
        self.quest_type
    }

    /// Get quest status
    pub fn status(&self) -> QuestStatus {
        self.status
    }

    /// Get quest objectives
    pub fn objectives(&self) -> &[QuestObjective] {
        &self.objectives
    }

    /// Get quest rewards
    pub fn rewards(&self) -> &QuestRewards {
        &self.rewards
    }

    /// Start the quest
    pub fn start(&mut self) -> DomainResult<()> {
        match self.status {
            QuestStatus::Available => {
                self.status = QuestStatus::Active;
                self.started_at = Some(Utc::now());
                self.version += 1;
                Ok(())
            }
            _ => Err(DomainError::QuestError(
                "Quest can only be started if it's available".to_string(),
            )),
        }
    }

    /// Update objective progress
    pub fn update_objective_progress(
        &mut self,
        objective_id: &EntityId,
        progress: u32,
    ) -> DomainResult<bool> {
        if self.status != QuestStatus::Active {
            return Err(DomainError::QuestError(
                "Quest must be active to update progress".to_string(),
            ));
        }

        let mut quest_completed = false;

        for objective in &mut self.objectives {
            if objective.id == *objective_id {
                objective.current_progress = objective.current_progress.max(progress);

                if objective.is_completed() {
                    objective.completed_at = Some(Utc::now());
                }
                break;
            }
        }

        // Check if all objectives are completed
        if self.objectives.iter().all(|obj| obj.is_completed()) {
            self.complete()?;
            quest_completed = true;
        }

        self.version += 1;
        Ok(quest_completed)
    }

    /// Complete the quest
    fn complete(&mut self) -> DomainResult<()> {
        if self.status != QuestStatus::Active {
            return Err(DomainError::QuestError(
                "Quest must be active to complete".to_string(),
            ));
        }

        self.status = QuestStatus::Completed;
        self.completed_at = Some(Utc::now());
        Ok(())
    }

    /// Abandon the quest
    pub fn abandon(&mut self) -> DomainResult<()> {
        match self.status {
            QuestStatus::Active => {
                self.status = QuestStatus::Abandoned;
                self.version += 1;
                Ok(())
            }
            _ => Err(DomainError::QuestError(
                "Only active quests can be abandoned".to_string(),
            )),
        }
    }

    /// Check if quest is completed
    pub fn is_completed(&self) -> bool {
        self.status == QuestStatus::Completed
    }

    /// Check if quest is active
    pub fn is_active(&self) -> bool {
        self.status == QuestStatus::Active
    }

    /// Get completion percentage (0-100)
    pub fn completion_percentage(&self) -> u8 {
        if self.objectives.is_empty() {
            return 0;
        }

        let completed_objectives = self
            .objectives
            .iter()
            .filter(|obj| obj.is_completed())
            .count();
        ((completed_objectives as f32 / self.objectives.len() as f32) * 100.0) as u8
    }
}

/// Types of quests
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuestType {
    /// Main story quest
    Main,
    /// Side quest
    Side,
    /// Daily quest
    Daily,
    /// Exploration quest
    Exploration,
    /// Resource gathering quest
    Gathering,
    /// Base building quest
    Construction,
    /// Combat quest
    Combat,
}

impl std::fmt::Display for QuestType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuestType::Main => write!(f, "Main Quest"),
            QuestType::Side => write!(f, "Side Quest"),
            QuestType::Daily => write!(f, "Daily Quest"),
            QuestType::Exploration => write!(f, "Exploration Quest"),
            QuestType::Gathering => write!(f, "Gathering Quest"),
            QuestType::Construction => write!(f, "Construction Quest"),
            QuestType::Combat => write!(f, "Combat Quest"),
        }
    }
}

/// Quest status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestStatus {
    Available,
    Active,
    Completed,
    Failed,
    Abandoned,
}

impl std::fmt::Display for QuestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuestStatus::Available => write!(f, "Available"),
            QuestStatus::Active => write!(f, "Active"),
            QuestStatus::Completed => write!(f, "Completed"),
            QuestStatus::Failed => write!(f, "Failed"),
            QuestStatus::Abandoned => write!(f, "Abandoned"),
        }
    }
}

/// A quest objective
#[derive(Debug, Clone, PartialEq)]
pub struct QuestObjective {
    pub id: EntityId,
    pub objective_type: ObjectiveType,
    pub description: String,
    pub target_amount: u32,
    pub current_progress: u32,
    pub completed_at: Option<DateTime<Utc>>,
}

impl QuestObjective {
    /// Create a new quest objective
    pub fn new(objective_type: ObjectiveType, description: String, target_amount: u32) -> Self {
        Self {
            id: EntityId::generate(),
            objective_type,
            description,
            target_amount,
            current_progress: 0,
            completed_at: None,
        }
    }

    /// Check if objective is completed
    pub fn is_completed(&self) -> bool {
        self.current_progress >= self.target_amount
    }

    /// Get completion percentage
    pub fn completion_percentage(&self) -> u8 {
        if self.target_amount == 0 {
            return 100;
        }
        ((self.current_progress as f32 / self.target_amount as f32) * 100.0).min(100.0) as u8
    }
}

/// Types of quest objectives
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectiveType {
    /// Visit specific locations
    VisitLocation(Vec<Position3D>),
    /// Collect resources
    CollectResources(HashMap<crate::domain::value_objects::ResourceType, u32>),
    /// Build structures
    BuildStructures(u32), // Number of any structures
    /// Explore tiles
    ExploreTiles(u32),
    /// Survive for duration
    Survive(u32), // seconds
    /// Reach player level
    ReachLevel(u32),
    /// Defeat enemies
    DefeatEnemies(u32),
}

/// Quest rewards
#[derive(Debug, Clone, PartialEq)]
pub struct QuestRewards {
    pub experience: u32,
    pub resources: ResourceCollection,
    pub unlocks: Vec<String>, // Features or content unlocked
}

impl QuestRewards {
    /// Create new quest rewards
    pub fn new(experience: u32, resources: ResourceCollection, unlocks: Vec<String>) -> Self {
        Self {
            experience,
            resources,
            unlocks,
        }
    }

    /// Create simple experience reward
    pub fn experience_only(experience: u32) -> Self {
        Self {
            experience,
            resources: ResourceCollection::new(),
            unlocks: Vec::new(),
        }
    }
}

/// Quest prerequisites
#[derive(Debug, Clone, PartialEq)]
pub enum QuestPrerequisite {
    /// Must have completed another quest
    CompletedQuest(EntityId),
    /// Must be at least this level
    MinimumLevel(u32),
    /// Must have visited a location
    VisitedLocation(Position3D),
    /// Must have resources
    HasResources(ResourceCollection),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quest_creation() {
        let objectives = vec![QuestObjective::new(
            ObjectiveType::ExploreTiles(10),
            "Explore 10 tiles".to_string(),
            10,
        )];

        let rewards = QuestRewards::experience_only(100);

        let quest = Quest::new(
            "First Steps".to_string(),
            "Explore the world around you".to_string(),
            QuestType::Main,
            objectives,
            rewards,
        )
        .unwrap();

        assert_eq!(quest.title(), "First Steps");
        assert_eq!(quest.status(), QuestStatus::Available);
        assert_eq!(quest.objectives().len(), 1);
    }

    #[test]
    fn quest_lifecycle() {
        let objectives = vec![QuestObjective::new(
            ObjectiveType::ExploreTiles(5),
            "Explore 5 tiles".to_string(),
            5,
        )];

        let rewards = QuestRewards::experience_only(50);

        let mut quest = Quest::new(
            "Explorer".to_string(),
            "Start exploring".to_string(),
            QuestType::Side,
            objectives,
            rewards,
        )
        .unwrap();

        // Start quest
        assert!(quest.start().is_ok());
        assert!(quest.is_active());
        assert!(quest.started_at.is_some());

        // Update progress
        let obj_id = quest.objectives()[0].id;
        let completed = quest.update_objective_progress(&obj_id, 5).unwrap();
        assert!(completed);
        assert!(quest.is_completed());
    }

    #[test]
    fn quest_objective_progress() {
        let mut objective = QuestObjective::new(
            ObjectiveType::ExploreTiles(10),
            "Explore tiles".to_string(),
            10,
        );

        assert!(!objective.is_completed());
        assert_eq!(objective.completion_percentage(), 0);

        objective.current_progress = 5;
        assert_eq!(objective.completion_percentage(), 50);

        objective.current_progress = 10;
        assert!(objective.is_completed());
        assert_eq!(objective.completion_percentage(), 100);
    }

    #[test]
    fn invalid_quest_creation() {
        let objectives = vec![QuestObjective::new(
            ObjectiveType::ExploreTiles(1),
            "Test".to_string(),
            1,
        )];

        let rewards = QuestRewards::experience_only(10);

        // Empty title
        let result = Quest::new(
            "".to_string(),
            "Description".to_string(),
            QuestType::Main,
            objectives.clone(),
            rewards.clone(),
        );
        assert!(result.is_err());

        // Empty objectives
        let result = Quest::new(
            "Title".to_string(),
            "Description".to_string(),
            QuestType::Main,
            Vec::new(),
            rewards,
        );
        assert!(result.is_err());
    }

    #[test]
    fn quest_abandonment() {
        let objectives = vec![QuestObjective::new(
            ObjectiveType::ExploreTiles(1),
            "Test".to_string(),
            1,
        )];

        let rewards = QuestRewards::experience_only(10);

        let mut quest = Quest::new(
            "Test Quest".to_string(),
            "Test Description".to_string(),
            QuestType::Side,
            objectives,
            rewards,
        )
        .unwrap();

        // Can't abandon before starting
        assert!(quest.abandon().is_err());

        quest.start().unwrap();
        assert!(quest.abandon().is_ok());
        assert_eq!(quest.status(), QuestStatus::Abandoned);
    }
}
