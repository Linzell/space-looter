//! Resource Rewards Service - Dice-based resource calculation system
//!
//! This service calculates resource rewards based on dice roll outcomes, event types,
//! player progression, and various modifiers. It follows DDD principles and provides
//! consistent, balanced resource rewards for different game events.

use crate::domain::entities::event::{EventOutcome, EventType, OutcomeType};
use crate::domain::value_objects::{PlayerStats, ResourceCollection, ResourceType, StatType};
use crate::domain::{DomainError, DomainResult};
use rand::prelude::*;
use std::collections::HashMap;

/// Service for calculating resource rewards based on dice rolls and events
#[derive(Debug)]
pub struct ResourceRewardService {
    base_reward_tables: HashMap<EventType, BaseRewardTable>,
    tier_multipliers: HashMap<RewardTier, TierMultipliers>,
    level_scaling: LevelScaling,
}

impl ResourceRewardService {
    /// Create a new resource reward service
    pub fn new() -> Self {
        let mut service = Self {
            base_reward_tables: HashMap::new(),
            tier_multipliers: HashMap::new(),
            level_scaling: LevelScaling::default(),
        };

        service.initialize_reward_tables();
        service.initialize_tier_multipliers();
        service
    }

    /// Calculate resources gained from an event based on dice roll
    pub fn calculate_event_rewards(
        &self,
        event_type: EventType,
        dice_roll: u8,
        player_level: u32,
        player_stats: &PlayerStats,
    ) -> DomainResult<EventRewardResult> {
        // Determine reward tier based on dice roll
        let tier = self.determine_reward_tier(dice_roll)?;

        // Get base rewards for this event type
        let base_table = self.base_reward_tables.get(&event_type).ok_or_else(|| {
            DomainError::ValidationError(format!("No reward table for event type: {}", event_type))
        })?;

        // Calculate base resources
        let mut resource_collection = self.calculate_base_resources(base_table, &tier)?;

        // Apply level scaling
        resource_collection = self.apply_level_scaling(resource_collection, player_level)?;

        // Apply stat modifiers
        resource_collection =
            self.apply_stat_modifiers(resource_collection, event_type, player_stats)?;

        // Apply random variance for excitement
        resource_collection = self.apply_random_variance(resource_collection, &tier)?;

        // Calculate experience reward
        let experience_reward =
            self.calculate_experience_reward(event_type, &tier, dice_roll, player_level)?;

        // Create outcome based on tier
        let outcome_type = match tier {
            RewardTier::CriticalFailure => OutcomeType::Failure,
            RewardTier::Failure => OutcomeType::Failure,
            RewardTier::Neutral => OutcomeType::Neutral,
            RewardTier::Success => OutcomeType::Success,
            RewardTier::GreatSuccess => OutcomeType::Success,
            RewardTier::CriticalSuccess => OutcomeType::Success,
        };

        // Generate outcome description
        let description = self.generate_outcome_description(
            event_type,
            &tier,
            &resource_collection,
            experience_reward,
        );

        Ok(EventRewardResult {
            resources: resource_collection.clone(),
            experience: experience_reward,
            tier,
            outcome: EventOutcome::new(
                outcome_type,
                Some(resource_collection),
                None, // No resources lost for successful events
                experience_reward,
                description,
            ),
        })
    }

    /// Calculate resource losses for negative events
    pub fn calculate_event_penalties(
        &self,
        event_type: EventType,
        dice_roll: u8,
        _player_level: u32,
        current_resources: &ResourceCollection,
    ) -> DomainResult<EventRewardResult> {
        let tier = self.determine_reward_tier(dice_roll)?;

        // Only apply penalties for dangerous events and bad rolls
        if !event_type.is_dangerous()
            || !matches!(tier, RewardTier::CriticalFailure | RewardTier::Failure)
        {
            return Ok(EventRewardResult {
                resources: ResourceCollection::new(),
                experience: 1, // Minimal experience for surviving
                tier,
                outcome: EventOutcome::neutral("No significant impact".to_string()),
            });
        }

        // Calculate resource losses based on severity
        let loss_percentage = match tier {
            RewardTier::CriticalFailure => 0.15, // Lose 15% of resources
            RewardTier::Failure => 0.05,         // Lose 5% of resources
            _ => 0.0,
        };

        let mut lost_resources = ResourceCollection::new();
        for resource_type in ResourceType::all() {
            let current_amount = current_resources.get_amount(resource_type);
            let loss_amount = ((current_amount as f32) * loss_percentage) as u32;
            if loss_amount > 0 {
                lost_resources.set_amount(resource_type, loss_amount);
            }
        }

        let description = match tier {
            RewardTier::CriticalFailure => format!(
                "Catastrophic failure! Lost {} resources due to {}",
                lost_resources.total_value(),
                event_type
            ),
            RewardTier::Failure => format!(
                "Setback encountered. Lost {} resources from {}",
                lost_resources.total_value(),
                event_type
            ),
            _ => "No significant losses".to_string(),
        };

        Ok(EventRewardResult {
            resources: ResourceCollection::new(),
            experience: 1,
            tier,
            outcome: EventOutcome::new(
                OutcomeType::Failure,
                None,
                Some(lost_resources),
                1,
                description,
            ),
        })
    }

    /// Determine reward tier based on dice roll (1-20 scale)
    fn determine_reward_tier(&self, dice_roll: u8) -> DomainResult<RewardTier> {
        let tier = match dice_roll {
            1..=3 => RewardTier::CriticalFailure,
            4..=7 => RewardTier::Failure,
            8..=12 => RewardTier::Neutral,
            13..=16 => RewardTier::Success,
            17..=19 => RewardTier::GreatSuccess,
            20..=255 => RewardTier::CriticalSuccess,
            0 => RewardTier::CriticalFailure,
        };
        Ok(tier)
    }

    /// Calculate base resources from reward table
    fn calculate_base_resources(
        &self,
        base_table: &BaseRewardTable,
        tier: &RewardTier,
    ) -> DomainResult<ResourceCollection> {
        let tier_multipliers = self
            .tier_multipliers
            .get(tier)
            .ok_or_else(|| DomainError::ValidationError("Invalid reward tier".to_string()))?;

        let mut collection = ResourceCollection::new();

        for (resource_type, base_amount) in &base_table.base_amounts {
            let multiplier = match resource_type {
                ResourceType::Metal => tier_multipliers.common_multiplier,
                ResourceType::Energy => tier_multipliers.common_multiplier,
                ResourceType::Food => tier_multipliers.common_multiplier,
                ResourceType::Technology => tier_multipliers.rare_multiplier,
                ResourceType::ExoticMatter => tier_multipliers.exotic_multiplier,
                ResourceType::Alloys => tier_multipliers.rare_multiplier,
                ResourceType::Data => tier_multipliers.rare_multiplier,
                ResourceType::Organics => tier_multipliers.common_multiplier,
            };

            let final_amount = ((*base_amount as f32) * multiplier).max(1.0) as u32;
            collection.set_amount(*resource_type, final_amount);
        }

        Ok(collection)
    }

    /// Apply level-based scaling to rewards
    fn apply_level_scaling(
        &self,
        mut collection: ResourceCollection,
        player_level: u32,
    ) -> DomainResult<ResourceCollection> {
        let scale_factor = self.level_scaling.calculate_multiplier(player_level);

        for resource_type in ResourceType::all() {
            let current_amount = collection.get_amount(resource_type);
            if current_amount > 0 {
                let scaled_amount = ((current_amount as f32) * scale_factor) as u32;
                collection.set_amount(resource_type, scaled_amount.max(1));
            }
        }

        Ok(collection)
    }

    /// Apply stat-based modifiers
    fn apply_stat_modifiers(
        &self,
        mut collection: ResourceCollection,
        event_type: EventType,
        player_stats: &PlayerStats,
    ) -> DomainResult<ResourceCollection> {
        // Different stats affect different resource types and events
        let (primary_stat, secondary_stat) = match event_type {
            EventType::ResourceDiscovery => (StatType::Intelligence, StatType::Luck),
            EventType::Combat => (StatType::Strength, StatType::Dexterity),
            EventType::Trade => (StatType::Charisma, StatType::Intelligence),
            EventType::Boon => (StatType::Luck, StatType::Charisma),
            EventType::Mystery => (StatType::Intelligence, StatType::Luck),
            EventType::Narrative => (StatType::Charisma, StatType::Intelligence),
            _ => (StatType::Luck, StatType::Endurance),
        };

        let primary_modifier = player_stats.get_modifier(primary_stat) as f32 * 0.05; // 5% per modifier point
        let secondary_modifier = player_stats.get_modifier(secondary_stat) as f32 * 0.025; // 2.5% per modifier point
        let total_modifier = 1.0 + primary_modifier + secondary_modifier;

        for resource_type in ResourceType::all() {
            let current_amount = collection.get_amount(resource_type);
            if current_amount > 0 {
                let modified_amount =
                    ((current_amount as f32) * total_modifier.max(0.1)).max(1.0) as u32;
                collection.set_amount(resource_type, modified_amount);
            }
        }

        Ok(collection)
    }

    /// Apply random variance to keep rewards exciting
    fn apply_random_variance(
        &self,
        mut collection: ResourceCollection,
        tier: &RewardTier,
    ) -> DomainResult<ResourceCollection> {
        let mut rng = rand::thread_rng();

        // Higher tiers get more variance for excitement
        let variance_range = match tier {
            RewardTier::CriticalFailure => 0.1,
            RewardTier::Failure => 0.15,
            RewardTier::Neutral => 0.2,
            RewardTier::Success => 0.25,
            RewardTier::GreatSuccess => 0.35,
            RewardTier::CriticalSuccess => 0.5,
        };

        for resource_type in ResourceType::all() {
            let current_amount = collection.get_amount(resource_type);
            if current_amount > 0 {
                let variance = rng.gen_range(-variance_range..=variance_range);
                let varied_amount = ((current_amount as f32) * (1.0 + variance)).max(1.0) as u32;
                collection.set_amount(resource_type, varied_amount);
            }
        }

        Ok(collection)
    }

    /// Calculate experience reward
    fn calculate_experience_reward(
        &self,
        event_type: EventType,
        tier: &RewardTier,
        dice_roll: u8,
        player_level: u32,
    ) -> DomainResult<u32> {
        let base_exp = match event_type {
            EventType::Combat => 15,
            EventType::ResourceDiscovery => 10,
            EventType::Trade => 8,
            EventType::Mystery => 12,
            EventType::Hazard => 10,
            EventType::Boon => 8,
            EventType::Narrative => 5,
            _ => 5,
        };

        let tier_multiplier = match tier {
            RewardTier::CriticalFailure => 0.5,
            RewardTier::Failure => 0.75,
            RewardTier::Neutral => 1.0,
            RewardTier::Success => 1.25,
            RewardTier::GreatSuccess => 1.75,
            RewardTier::CriticalSuccess => 2.5,
        };

        // Bonus for exceptional rolls
        let roll_bonus = if dice_roll >= 18 { 1.2 } else { 1.0 };

        // Slight scaling with level (diminishing returns)
        let level_factor = 1.0 + (player_level as f32 * 0.05).min(2.0);

        let total_exp = ((base_exp as f32) * tier_multiplier * roll_bonus * level_factor) as u32;
        Ok(total_exp.max(1))
    }

    /// Generate descriptive outcome text
    fn generate_outcome_description(
        &self,
        event_type: EventType,
        tier: &RewardTier,
        resources: &ResourceCollection,
        experience: u32,
    ) -> String {
        let resource_summary = if !resources.is_empty() {
            format!(" Gained: {}", self.format_resource_summary(resources))
        } else {
            String::new()
        };

        let tier_desc = match tier {
            RewardTier::CriticalSuccess => "Incredible success!",
            RewardTier::GreatSuccess => "Great success!",
            RewardTier::Success => "Success!",
            RewardTier::Neutral => "Neutral outcome.",
            RewardTier::Failure => "Minor setback.",
            RewardTier::CriticalFailure => "Critical failure!",
        };

        format!(
            "{} {} (+{} XP){}",
            event_type, tier_desc, experience, resource_summary
        )
    }

    /// Format resource summary for descriptions
    fn format_resource_summary(&self, resources: &ResourceCollection) -> String {
        let mut parts = Vec::new();

        for resource_type in ResourceType::all() {
            let amount = resources.get_amount(resource_type);
            if amount > 0 {
                parts.push(format!("{}{}", amount, resource_type.icon()));
            }
        }

        if parts.is_empty() {
            "Nothing".to_string()
        } else {
            parts.join(" ")
        }
    }

    /// Initialize reward tables for different event types
    fn initialize_reward_tables(&mut self) {
        // Resource Discovery Events
        let mut resource_discovery = HashMap::new();
        resource_discovery.insert(ResourceType::Metal, 8);
        resource_discovery.insert(ResourceType::Energy, 5);
        resource_discovery.insert(ResourceType::Technology, 2);
        self.base_reward_tables.insert(
            EventType::ResourceDiscovery,
            BaseRewardTable {
                base_amounts: resource_discovery,
            },
        );

        // Trade Events
        let mut trade = HashMap::new();
        trade.insert(ResourceType::Metal, 4);
        trade.insert(ResourceType::Energy, 6);
        trade.insert(ResourceType::Food, 5);
        trade.insert(ResourceType::Data, 3);
        self.base_reward_tables.insert(
            EventType::Trade,
            BaseRewardTable {
                base_amounts: trade,
            },
        );

        // Boon Events
        let mut boon = HashMap::new();
        boon.insert(ResourceType::Technology, 4);
        boon.insert(ResourceType::ExoticMatter, 1);
        boon.insert(ResourceType::Data, 5);
        self.base_reward_tables
            .insert(EventType::Boon, BaseRewardTable { base_amounts: boon });

        // Mystery Events
        let mut mystery = HashMap::new();
        mystery.insert(ResourceType::Data, 8);
        mystery.insert(ResourceType::Technology, 3);
        mystery.insert(ResourceType::ExoticMatter, 1);
        self.base_reward_tables.insert(
            EventType::Mystery,
            BaseRewardTable {
                base_amounts: mystery,
            },
        );

        // Combat Events (rewards for victory)
        let mut combat = HashMap::new();
        combat.insert(ResourceType::Metal, 3);
        combat.insert(ResourceType::Organics, 4);
        combat.insert(ResourceType::Alloys, 2);
        self.base_reward_tables.insert(
            EventType::Combat,
            BaseRewardTable {
                base_amounts: combat,
            },
        );

        // Narrative Events (small but guaranteed rewards)
        let mut narrative = HashMap::new();
        narrative.insert(ResourceType::Data, 3);
        narrative.insert(ResourceType::Energy, 2);
        self.base_reward_tables.insert(
            EventType::Narrative,
            BaseRewardTable {
                base_amounts: narrative,
            },
        );

        // Add empty tables for events that don't give resources
        for event_type in [
            EventType::Hazard,
            EventType::Malfunction,
            EventType::BaseEvent,
        ] {
            self.base_reward_tables.insert(
                event_type,
                BaseRewardTable {
                    base_amounts: HashMap::new(),
                },
            );
        }
    }

    /// Initialize tier multipliers
    fn initialize_tier_multipliers(&mut self) {
        self.tier_multipliers.insert(
            RewardTier::CriticalFailure,
            TierMultipliers {
                common_multiplier: 0.0,
                rare_multiplier: 0.0,
                exotic_multiplier: 0.0,
            },
        );

        self.tier_multipliers.insert(
            RewardTier::Failure,
            TierMultipliers {
                common_multiplier: 0.5,
                rare_multiplier: 0.1,
                exotic_multiplier: 0.0,
            },
        );

        self.tier_multipliers.insert(
            RewardTier::Neutral,
            TierMultipliers {
                common_multiplier: 1.0,
                rare_multiplier: 0.3,
                exotic_multiplier: 0.0,
            },
        );

        self.tier_multipliers.insert(
            RewardTier::Success,
            TierMultipliers {
                common_multiplier: 1.0,
                rare_multiplier: 0.5,
                exotic_multiplier: 0.1,
            },
        );

        self.tier_multipliers.insert(
            RewardTier::GreatSuccess,
            TierMultipliers {
                common_multiplier: 1.5,
                rare_multiplier: 1.0,
                exotic_multiplier: 0.3,
            },
        );

        self.tier_multipliers.insert(
            RewardTier::CriticalSuccess,
            TierMultipliers {
                common_multiplier: 2.5,
                rare_multiplier: 2.0,
                exotic_multiplier: 1.0,
            },
        );
    }
}

impl Default for ResourceRewardService {
    fn default() -> Self {
        Self::new()
    }
}

/// Reward tiers based on dice roll outcomes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RewardTier {
    CriticalFailure,
    Failure,
    Neutral,
    Success,
    GreatSuccess,
    CriticalSuccess,
}

/// Base reward amounts for different event types
#[derive(Debug, Clone)]
struct BaseRewardTable {
    base_amounts: HashMap<ResourceType, u32>,
}

/// Multipliers for different reward tiers
#[derive(Debug, Clone)]
struct TierMultipliers {
    common_multiplier: f32,
    rare_multiplier: f32,
    exotic_multiplier: f32,
}

/// Level-based scaling configuration
#[derive(Debug, Clone)]
struct LevelScaling {
    base_multiplier: f32,
    level_increment: f32,
    max_multiplier: f32,
}

impl LevelScaling {
    fn calculate_multiplier(&self, player_level: u32) -> f32 {
        let level_bonus = (player_level.saturating_sub(1) as f32) * self.level_increment;
        (self.base_multiplier + level_bonus).min(self.max_multiplier)
    }
}

impl Default for LevelScaling {
    fn default() -> Self {
        Self {
            base_multiplier: 1.0,
            level_increment: 0.1, // 10% increase per level
            max_multiplier: 5.0,  // Cap at 5x multiplier
        }
    }
}

/// Result of event reward calculation
#[derive(Debug, Clone, PartialEq)]
pub struct EventRewardResult {
    pub resources: ResourceCollection,
    pub experience: u32,
    pub tier: RewardTier,
    pub outcome: EventOutcome,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reward_tier_determination() {
        let service = ResourceRewardService::new();

        assert_eq!(
            service.determine_reward_tier(1).unwrap(),
            RewardTier::CriticalFailure
        );
        assert_eq!(
            service.determine_reward_tier(5).unwrap(),
            RewardTier::Failure
        );
        assert_eq!(
            service.determine_reward_tier(10).unwrap(),
            RewardTier::Neutral
        );
        assert_eq!(
            service.determine_reward_tier(15).unwrap(),
            RewardTier::Success
        );
        assert_eq!(
            service.determine_reward_tier(18).unwrap(),
            RewardTier::GreatSuccess
        );
        assert_eq!(
            service.determine_reward_tier(20).unwrap(),
            RewardTier::CriticalSuccess
        );
    }

    #[test]
    fn test_resource_discovery_rewards() {
        let service = ResourceRewardService::new();
        let stats = PlayerStats::starting_stats();

        let result = service
            .calculate_event_rewards(
                EventType::ResourceDiscovery,
                20, // Critical success
                5,  // Level 5
                &stats,
            )
            .unwrap();

        assert_eq!(result.tier, RewardTier::CriticalSuccess);
        assert!(result.resources.get_amount(ResourceType::Metal) > 0);
        assert!(result.experience > 10);
    }

    #[test]
    fn test_level_scaling() {
        let service = ResourceRewardService::new();
        let stats = PlayerStats::starting_stats();

        // Compare level 1 vs level 10 rewards
        let level1_result = service
            .calculate_event_rewards(EventType::ResourceDiscovery, 15, 1, &stats)
            .unwrap();

        let level10_result = service
            .calculate_event_rewards(EventType::ResourceDiscovery, 15, 10, &stats)
            .unwrap();

        assert!(level10_result.resources.total_value() > level1_result.resources.total_value());
    }

    #[test]
    fn test_event_penalties() {
        let service = ResourceRewardService::new();
        let mut current_resources = ResourceCollection::new();
        current_resources.set_amount(ResourceType::Metal, 100);

        let result = service
            .calculate_event_penalties(
                EventType::Combat,
                2, // Critical failure
                5,
                &current_resources,
            )
            .unwrap();

        assert_eq!(result.tier, RewardTier::CriticalFailure);
        if let Some(lost_resources) = &result.outcome.resources_lost {
            assert!(lost_resources.get_amount(ResourceType::Metal) > 0);
        }
    }

    #[test]
    fn test_stat_modifiers() {
        let service = ResourceRewardService::new();

        // High intelligence character
        let high_int_stats = PlayerStats::new(10, 10, 18, 10, 10, 10).unwrap();
        let result_high_int = service
            .calculate_event_rewards(
                EventType::ResourceDiscovery, // Benefits from Intelligence
                15,
                5,
                &high_int_stats,
            )
            .unwrap();

        // Average stats character
        let avg_stats = PlayerStats::starting_stats();
        let result_avg = service
            .calculate_event_rewards(EventType::ResourceDiscovery, 15, 5, &avg_stats)
            .unwrap();

        assert!(result_high_int.resources.total_value() > result_avg.resources.total_value());
    }
}
