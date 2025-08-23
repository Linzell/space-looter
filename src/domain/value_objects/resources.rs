//! Resource system for base building and crafting
//!
//! This module defines the various resource types, quantities, and operations
//! used throughout the game for base construction, upgrades, and trading.

use crate::domain::{DomainError, DomainResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Types of resources available in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    /// Basic construction material from mining
    Metal,
    /// Power source for base operations
    Energy,
    /// Sustenance for survival and population
    Food,
    /// Advanced materials for high-tech upgrades
    Technology,
    /// Rare materials found in dangerous areas
    ExoticMatter,
    /// Refined materials from processing
    Alloys,
    /// Information and data resources
    Data,
    /// Biological samples and materials
    Organics,
}

impl ResourceType {
    /// Get all available resource types
    pub fn all() -> Vec<ResourceType> {
        vec![
            ResourceType::Metal,
            ResourceType::Energy,
            ResourceType::Food,
            ResourceType::Technology,
            ResourceType::ExoticMatter,
            ResourceType::Alloys,
            ResourceType::Data,
            ResourceType::Organics,
        ]
    }

    /// Get basic resource types (common resources)
    pub fn basic() -> Vec<ResourceType> {
        vec![
            ResourceType::Metal,
            ResourceType::Energy,
            ResourceType::Food,
        ]
    }

    /// Get advanced resource types (rare/processed resources)
    pub fn advanced() -> Vec<ResourceType> {
        vec![
            ResourceType::Technology,
            ResourceType::ExoticMatter,
            ResourceType::Alloys,
            ResourceType::Data,
            ResourceType::Organics,
        ]
    }

    /// Check if this is a basic resource
    pub fn is_basic(&self) -> bool {
        matches!(
            self,
            ResourceType::Metal | ResourceType::Energy | ResourceType::Food
        )
    }

    /// Check if this is an advanced resource
    pub fn is_advanced(&self) -> bool {
        !self.is_basic()
    }

    /// Get the base value per unit for trading
    pub fn base_value(&self) -> u32 {
        match self {
            ResourceType::Metal => 1,
            ResourceType::Energy => 2,
            ResourceType::Food => 1,
            ResourceType::Technology => 10,
            ResourceType::ExoticMatter => 50,
            ResourceType::Alloys => 5,
            ResourceType::Data => 3,
            ResourceType::Organics => 4,
        }
    }

    /// Get the rarity level (1-10, higher is rarer)
    pub fn rarity(&self) -> u8 {
        match self {
            ResourceType::Metal => 2,
            ResourceType::Energy => 3,
            ResourceType::Food => 1,
            ResourceType::Technology => 6,
            ResourceType::ExoticMatter => 9,
            ResourceType::Alloys => 5,
            ResourceType::Data => 4,
            ResourceType::Organics => 3,
        }
    }

    /// Get typical gathering rate per successful dice roll
    pub fn base_gathering_rate(&self) -> u32 {
        match self {
            ResourceType::Metal => 5,
            ResourceType::Energy => 3,
            ResourceType::Food => 4,
            ResourceType::Technology => 1,
            ResourceType::ExoticMatter => 1,
            ResourceType::Alloys => 2,
            ResourceType::Data => 2,
            ResourceType::Organics => 3,
        }
    }

    /// Get description of the resource
    pub fn description(&self) -> &'static str {
        match self {
            ResourceType::Metal => "Basic construction material obtained from mining operations",
            ResourceType::Energy => "Power cells and energy sources for base operations",
            ResourceType::Food => "Sustenance required for survival and population growth",
            ResourceType::Technology => "Advanced components and research materials",
            ResourceType::ExoticMatter => "Rare quantum materials with unique properties",
            ResourceType::Alloys => "Processed metals with enhanced properties",
            ResourceType::Data => "Information, blueprints, and digital assets",
            ResourceType::Organics => "Biological samples and living materials",
        }
    }

    /// Get icon symbol for UI display
    pub fn icon(&self) -> char {
        match self {
            ResourceType::Metal => '⚒',
            ResourceType::Energy => '⚡',
            ResourceType::Food => '🍎',
            ResourceType::Technology => '🔬',
            ResourceType::ExoticMatter => '💎',
            ResourceType::Alloys => '🔩',
            ResourceType::Data => '💾',
            ResourceType::Organics => '🧬',
        }
    }
}

impl fmt::Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceType::Metal => write!(f, "Metal"),
            ResourceType::Energy => write!(f, "Energy"),
            ResourceType::Food => write!(f, "Food"),
            ResourceType::Technology => write!(f, "Technology"),
            ResourceType::ExoticMatter => write!(f, "Exotic Matter"),
            ResourceType::Alloys => write!(f, "Alloys"),
            ResourceType::Data => write!(f, "Data"),
            ResourceType::Organics => write!(f, "Organics"),
        }
    }
}

/// A specific amount of a resource type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceAmount {
    pub resource_type: ResourceType,
    pub amount: u32,
}

impl ResourceAmount {
    /// Create a new resource amount
    pub fn new(resource_type: ResourceType, amount: u32) -> DomainResult<Self> {
        if amount > 1_000_000 {
            return Err(DomainError::InvalidResourceAmount(amount as i32));
        }

        Ok(Self {
            resource_type,
            amount,
        })
    }

    /// Create a zero amount of a resource
    pub fn zero(resource_type: ResourceType) -> Self {
        Self {
            resource_type,
            amount: 0,
        }
    }

    /// Check if this amount is zero
    pub fn is_zero(&self) -> bool {
        self.amount == 0
    }

    /// Check if this amount is non-zero
    pub fn is_positive(&self) -> bool {
        self.amount > 0
    }

    /// Add another amount of the same resource
    pub fn add(&self, other: &ResourceAmount) -> DomainResult<Self> {
        if self.resource_type != other.resource_type {
            return Err(DomainError::InvalidResourceType(
                "Cannot add different resource types".to_string(),
            ));
        }

        let new_amount = self.amount.saturating_add(other.amount);
        Self::new(self.resource_type, new_amount)
    }

    /// Subtract another amount of the same resource
    pub fn subtract(&self, other: &ResourceAmount) -> DomainResult<Self> {
        if self.resource_type != other.resource_type {
            return Err(DomainError::InvalidResourceType(
                "Cannot subtract different resource types".to_string(),
            ));
        }

        let new_amount = self.amount.saturating_sub(other.amount);
        Self::new(self.resource_type, new_amount)
    }

    /// Multiply amount by a factor
    pub fn multiply(&self, factor: u32) -> DomainResult<Self> {
        let new_amount = self.amount.saturating_mul(factor);
        Self::new(self.resource_type, new_amount)
    }

    /// Get the total trade value of this amount
    pub fn trade_value(&self) -> u32 {
        self.amount * self.resource_type.base_value()
    }

    /// Check if we have at least this amount
    pub fn can_afford(&self, required: &ResourceAmount) -> bool {
        self.resource_type == required.resource_type && self.amount >= required.amount
    }

    /// Get percentage of another amount
    pub fn percentage_of(&self, other: &ResourceAmount) -> Option<f32> {
        if self.resource_type != other.resource_type || other.amount == 0 {
            return None;
        }

        Some((self.amount as f32 / other.amount as f32) * 100.0)
    }
}

impl fmt::Display for ResourceAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.amount,
            self.resource_type.icon(),
            self.resource_type
        )
    }
}

/// A collection of different resource amounts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceCollection {
    resources: HashMap<ResourceType, u32>,
}

impl ResourceCollection {
    /// Create a new empty resource collection
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    /// Create a collection from a list of resource amounts
    pub fn from_amounts(amounts: Vec<ResourceAmount>) -> DomainResult<Self> {
        let mut collection = Self::new();
        for amount in amounts {
            collection.add_amount(&amount)?;
        }
        Ok(collection)
    }

    /// Create a collection from the starting resources
    pub fn starting_resources() -> Self {
        let mut collection = Self::new();
        for &(resource_type, amount) in &crate::domain::constants::STARTING_RESOURCES {
            collection.set_amount(resource_type, amount as u32);
        }
        collection
    }

    /// Get the amount of a specific resource
    pub fn get_amount(&self, resource_type: ResourceType) -> u32 {
        self.resources.get(&resource_type).copied().unwrap_or(0)
    }

    /// Set the amount of a specific resource
    pub fn set_amount(&mut self, resource_type: ResourceType, amount: u32) {
        if amount == 0 {
            self.resources.remove(&resource_type);
        } else {
            self.resources.insert(resource_type, amount);
        }
    }

    /// Add an amount of a resource
    pub fn add_amount(&mut self, amount: &ResourceAmount) -> DomainResult<()> {
        let current = self.get_amount(amount.resource_type);
        let new_amount = current.saturating_add(amount.amount);

        if new_amount > 1_000_000 {
            return Err(DomainError::InvalidResourceAmount(new_amount as i32));
        }

        self.set_amount(amount.resource_type, new_amount);
        Ok(())
    }

    /// Remove an amount of a resource
    pub fn remove_amount(&mut self, amount: &ResourceAmount) -> DomainResult<()> {
        let current = self.get_amount(amount.resource_type);
        if current < amount.amount {
            return Err(DomainError::InsufficientResources(format!(
                "Need {} {}, have {}",
                amount.amount, amount.resource_type, current
            )));
        }

        let new_amount = current - amount.amount;
        self.set_amount(amount.resource_type, new_amount);
        Ok(())
    }

    /// Check if we can afford all resources in a cost
    pub fn can_afford(&self, cost: &ResourceCollection) -> bool {
        for (&resource_type, &required_amount) in &cost.resources {
            if self.get_amount(resource_type) < required_amount {
                return false;
            }
        }
        true
    }

    /// Pay a cost (remove resources)
    pub fn pay_cost(&mut self, cost: &ResourceCollection) -> DomainResult<()> {
        if !self.can_afford(cost) {
            return Err(DomainError::InsufficientResources(
                "Cannot afford this cost".to_string(),
            ));
        }

        for (&resource_type, &amount) in &cost.resources {
            let resource_amount = ResourceAmount::new(resource_type, amount)?;
            self.remove_amount(&resource_amount)?;
        }

        Ok(())
    }

    /// Add resources from another collection
    pub fn add_collection(&mut self, other: &ResourceCollection) -> DomainResult<()> {
        for (&resource_type, &amount) in &other.resources {
            let resource_amount = ResourceAmount::new(resource_type, amount)?;
            self.add_amount(&resource_amount)?;
        }
        Ok(())
    }

    /// Get total trade value of all resources
    pub fn total_value(&self) -> u32 {
        self.resources
            .iter()
            .map(|(&resource_type, &amount)| amount * resource_type.base_value())
            .sum()
    }

    /// Get all resource types that have positive amounts
    pub fn resource_types(&self) -> Vec<ResourceType> {
        self.resources.keys().copied().collect()
    }

    /// Get all resource amounts
    pub fn amounts(&self) -> Vec<ResourceAmount> {
        self.resources
            .iter()
            .map(|(&resource_type, &amount)| ResourceAmount::new(resource_type, amount).unwrap())
            .collect()
    }

    /// Check if collection is empty
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }

    /// Get count of different resource types
    pub fn type_count(&self) -> usize {
        self.resources.len()
    }

    /// Create a resource collection for a specific cost
    pub fn cost(costs: &[(ResourceType, u32)]) -> DomainResult<Self> {
        let mut collection = Self::new();
        for &(resource_type, amount) in costs {
            collection.set_amount(resource_type, amount);
        }
        Ok(collection)
    }

    /// Get missing resources compared to a required collection
    pub fn missing_resources(&self, required: &ResourceCollection) -> ResourceCollection {
        let mut missing = ResourceCollection::new();

        for (&resource_type, &required_amount) in &required.resources {
            let available = self.get_amount(resource_type);
            if available < required_amount {
                missing.set_amount(resource_type, required_amount - available);
            }
        }

        missing
    }

    /// Calculate storage requirement (simplified weight system)
    pub fn storage_requirement(&self) -> u32 {
        self.resources.values().sum()
    }
}

impl Default for ResourceCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ResourceCollection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.resources.is_empty() {
            write!(f, "No resources")
        } else {
            let amounts: Vec<String> = self
                .resources
                .iter()
                .map(|(&resource_type, &amount)| format!("{} {}", amount, resource_type.icon()))
                .collect();
            write!(f, "{}", amounts.join(", "))
        }
    }
}

/// Resource node properties for world generation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceNodeProperties {
    pub resource_type: ResourceType,
    pub richness: ResourceRichness,
    pub accessibility: ResourceAccessibility,
    pub regeneration_rate: RegenerationRate,
}

impl ResourceNodeProperties {
    /// Create new resource node properties
    pub fn new(
        resource_type: ResourceType,
        richness: ResourceRichness,
        accessibility: ResourceAccessibility,
        regeneration_rate: RegenerationRate,
    ) -> Self {
        Self {
            resource_type,
            richness,
            accessibility,
            regeneration_rate,
        }
    }

    /// Create common metal node
    pub fn metal_common() -> Self {
        Self::new(
            ResourceType::Metal,
            ResourceRichness::Average,
            ResourceAccessibility::Easy,
            RegenerationRate::None,
        )
    }

    /// Create rare exotic matter node
    pub fn exotic_rare() -> Self {
        Self::new(
            ResourceType::ExoticMatter,
            ResourceRichness::Rich,
            ResourceAccessibility::Dangerous,
            RegenerationRate::Slow,
        )
    }

    /// Get gathering difficulty modifier
    pub fn gathering_difficulty(&self) -> u8 {
        let base = match self.accessibility {
            ResourceAccessibility::Easy => 8,
            ResourceAccessibility::Moderate => 12,
            ResourceAccessibility::Hard => 15,
            ResourceAccessibility::Dangerous => 18,
        };

        // Rarer resources are harder to extract
        base + (self.resource_type.rarity() / 2)
    }

    /// Get potential yield per successful gathering
    pub fn potential_yield(&self) -> u32 {
        let base = self.resource_type.base_gathering_rate();
        let richness_multiplier = self.richness.yield_multiplier();
        (base as f32 * richness_multiplier) as u32
    }
}

/// How rich a resource node is
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceRichness {
    Poor,
    Average,
    Rich,
    Abundant,
}

impl ResourceRichness {
    /// Get yield multiplier for this richness level
    pub fn yield_multiplier(&self) -> f32 {
        match self {
            ResourceRichness::Poor => 0.5,
            ResourceRichness::Average => 1.0,
            ResourceRichness::Rich => 2.0,
            ResourceRichness::Abundant => 3.0,
        }
    }
}

impl fmt::Display for ResourceRichness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceRichness::Poor => write!(f, "Poor"),
            ResourceRichness::Average => write!(f, "Average"),
            ResourceRichness::Rich => write!(f, "Rich"),
            ResourceRichness::Abundant => write!(f, "Abundant"),
        }
    }
}

/// How accessible a resource node is
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceAccessibility {
    Easy,
    Moderate,
    Hard,
    Dangerous,
}

impl fmt::Display for ResourceAccessibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceAccessibility::Easy => write!(f, "Easy"),
            ResourceAccessibility::Moderate => write!(f, "Moderate"),
            ResourceAccessibility::Hard => write!(f, "Hard"),
            ResourceAccessibility::Dangerous => write!(f, "Dangerous"),
        }
    }
}

/// How fast resources regenerate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegenerationRate {
    None,
    Slow,
    Moderate,
    Fast,
}

impl RegenerationRate {
    /// Get minutes between regeneration ticks
    pub fn regeneration_interval_minutes(&self) -> Option<u32> {
        match self {
            RegenerationRate::None => None,
            RegenerationRate::Slow => Some(60),     // 1 hour
            RegenerationRate::Moderate => Some(30), // 30 minutes
            RegenerationRate::Fast => Some(15),     // 15 minutes
        }
    }

    /// Get percentage of max resources regenerated per tick
    pub fn regeneration_percentage(&self) -> f32 {
        match self {
            RegenerationRate::None => 0.0,
            RegenerationRate::Slow => 0.05,     // 5%
            RegenerationRate::Moderate => 0.10, // 10%
            RegenerationRate::Fast => 0.20,     // 20%
        }
    }
}

impl fmt::Display for RegenerationRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegenerationRate::None => write!(f, "No Regeneration"),
            RegenerationRate::Slow => write!(f, "Slow Regeneration"),
            RegenerationRate::Moderate => write!(f, "Moderate Regeneration"),
            RegenerationRate::Fast => write!(f, "Fast Regeneration"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_type_properties() {
        assert_eq!(ResourceType::Metal.base_value(), 1);
        assert_eq!(ResourceType::ExoticMatter.base_value(), 50);
        assert!(ResourceType::Metal.is_basic());
        assert!(ResourceType::ExoticMatter.is_advanced());
        assert_eq!(ResourceType::ExoticMatter.rarity(), 9);
    }

    #[test]
    fn resource_amount_creation() {
        let amount = ResourceAmount::new(ResourceType::Metal, 100).unwrap();
        assert_eq!(amount.resource_type, ResourceType::Metal);
        assert_eq!(amount.amount, 100);
        assert!(!amount.is_zero());
        assert!(amount.is_positive());
    }

    #[test]
    fn resource_amount_operations() {
        let amount1 = ResourceAmount::new(ResourceType::Metal, 50).unwrap();
        let amount2 = ResourceAmount::new(ResourceType::Metal, 30).unwrap();

        let sum = amount1.add(&amount2).unwrap();
        assert_eq!(sum.amount, 80);

        let diff = amount1.subtract(&amount2).unwrap();
        assert_eq!(diff.amount, 20);

        let multiplied = amount1.multiply(3).unwrap();
        assert_eq!(multiplied.amount, 150);
    }

    #[test]
    fn resource_amount_invalid_operations() {
        let metal = ResourceAmount::new(ResourceType::Metal, 50).unwrap();
        let energy = ResourceAmount::new(ResourceType::Energy, 30).unwrap();

        // Cannot add different resource types
        assert!(metal.add(&energy).is_err());
        assert!(metal.subtract(&energy).is_err());
    }

    #[test]
    fn resource_collection_basic_operations() {
        let mut collection = ResourceCollection::new();
        assert!(collection.is_empty());
        assert_eq!(collection.get_amount(ResourceType::Metal), 0);

        collection.set_amount(ResourceType::Metal, 100);
        assert_eq!(collection.get_amount(ResourceType::Metal), 100);
        assert!(!collection.is_empty());
        assert_eq!(collection.type_count(), 1);
    }

    #[test]
    fn resource_collection_cost_payment() {
        let mut collection = ResourceCollection::new();
        collection.set_amount(ResourceType::Metal, 100);
        collection.set_amount(ResourceType::Energy, 50);

        let cost =
            ResourceCollection::cost(&[(ResourceType::Metal, 30), (ResourceType::Energy, 20)])
                .unwrap();

        assert!(collection.can_afford(&cost));
        collection.pay_cost(&cost).unwrap();

        assert_eq!(collection.get_amount(ResourceType::Metal), 70);
        assert_eq!(collection.get_amount(ResourceType::Energy), 30);
    }

    #[test]
    fn resource_collection_insufficient_funds() {
        let mut collection = ResourceCollection::new();
        collection.set_amount(ResourceType::Metal, 10);

        let cost = ResourceCollection::cost(&[(ResourceType::Metal, 50)]).unwrap();

        assert!(!collection.can_afford(&cost));
        assert!(collection.pay_cost(&cost).is_err());
    }

    #[test]
    fn resource_collection_trade_value() {
        let mut collection = ResourceCollection::new();
        collection.set_amount(ResourceType::Metal, 10); // 10 * 1 = 10
        collection.set_amount(ResourceType::Energy, 5); // 5 * 2 = 10
        collection.set_amount(ResourceType::Technology, 2); // 2 * 10 = 20

        assert_eq!(collection.total_value(), 40);
    }

    #[test]
    fn resource_node_properties() {
        let node = ResourceNodeProperties::metal_common();
        assert_eq!(node.resource_type, ResourceType::Metal);
        assert_eq!(node.richness, ResourceRichness::Average);

        let difficulty = node.gathering_difficulty();
        assert!(difficulty >= 8); // Easy accessibility gives base 8

        let potential_yield = node.potential_yield();
        assert!(potential_yield > 0);
    }

    #[test]
    fn resource_richness_multipliers() {
        assert_eq!(ResourceRichness::Poor.yield_multiplier(), 0.5);
        assert_eq!(ResourceRichness::Average.yield_multiplier(), 1.0);
        assert_eq!(ResourceRichness::Rich.yield_multiplier(), 2.0);
        assert_eq!(ResourceRichness::Abundant.yield_multiplier(), 3.0);
    }

    #[test]
    fn regeneration_rate_properties() {
        assert_eq!(RegenerationRate::None.regeneration_interval_minutes(), None);
        assert_eq!(
            RegenerationRate::Slow.regeneration_interval_minutes(),
            Some(60)
        );
        assert_eq!(RegenerationRate::Fast.regeneration_percentage(), 0.20);
    }

    #[test]
    fn starting_resources() {
        let collection = ResourceCollection::starting_resources();
        assert!(collection.get_amount(ResourceType::Metal) > 0);
        assert!(collection.get_amount(ResourceType::Energy) > 0);
        assert!(collection.get_amount(ResourceType::Food) > 0);
        assert!(collection.get_amount(ResourceType::Technology) > 0);
    }

    #[test]
    fn missing_resources_calculation() {
        let mut have = ResourceCollection::new();
        have.set_amount(ResourceType::Metal, 50);
        have.set_amount(ResourceType::Energy, 10);

        let need = ResourceCollection::cost(&[
            (ResourceType::Metal, 100),
            (ResourceType::Energy, 5),
            (ResourceType::Technology, 2),
        ])
        .unwrap();

        let missing = have.missing_resources(&need);
        assert_eq!(missing.get_amount(ResourceType::Metal), 50);
        assert_eq!(missing.get_amount(ResourceType::Energy), 0); // We have enough
        assert_eq!(missing.get_amount(ResourceType::Technology), 2);
    }
}
