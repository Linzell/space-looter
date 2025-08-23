//! Resource Entity - Represents resource instances in the game world
//!
//! This entity represents actual resource instances that exist in the world,
//! separate from the ResourceAmount value object which represents quantities.

use crate::domain::value_objects::{EntityId, Position3D, ResourceType};
use crate::domain::{DomainError, DomainResult};
use chrono::{DateTime, Utc};

/// A resource entity instance
#[derive(Debug, Clone, PartialEq)]
pub struct Resource {
    id: EntityId,
    resource_type: ResourceType,
    amount: u32,
    quality: ResourceQuality,
    position: Option<Position3D>,
    discovered_at: DateTime<Utc>,
    last_updated: DateTime<Utc>,
    version: u64,
}

impl Resource {
    /// Create a new resource instance
    pub fn new(
        resource_type: ResourceType,
        amount: u32,
        quality: ResourceQuality,
        position: Option<Position3D>,
    ) -> DomainResult<Self> {
        if amount == 0 {
            return Err(DomainError::InvalidResourceAmount(0));
        }

        if amount > 1_000_000 {
            return Err(DomainError::InvalidResourceAmount(amount as i32));
        }

        let now = Utc::now();
        Ok(Self {
            id: EntityId::generate(),
            resource_type,
            amount,
            quality,
            position,
            discovered_at: now,
            last_updated: now,
            version: 1,
        })
    }

    /// Get resource ID
    pub fn id(&self) -> &EntityId {
        &self.id
    }

    /// Get resource type
    pub fn resource_type(&self) -> ResourceType {
        self.resource_type
    }

    /// Get amount
    pub fn amount(&self) -> u32 {
        self.amount
    }

    /// Get quality
    pub fn quality(&self) -> ResourceQuality {
        self.quality
    }

    /// Get position
    pub fn position(&self) -> Option<&Position3D> {
        self.position.as_ref()
    }

    /// Set position
    pub fn set_position(&mut self, position: Option<Position3D>) {
        self.position = position;
        self.update_timestamp();
    }

    /// Get discovered time
    pub fn discovered_at(&self) -> DateTime<Utc> {
        self.discovered_at
    }

    /// Consume some of this resource
    pub fn consume(&mut self, amount_to_consume: u32) -> DomainResult<u32> {
        if amount_to_consume == 0 {
            return Ok(0);
        }

        let consumed = amount_to_consume.min(self.amount);
        self.amount -= consumed;
        self.update_timestamp();
        Ok(consumed)
    }

    /// Add more to this resource
    pub fn add(&mut self, amount_to_add: u32) -> DomainResult<()> {
        if amount_to_add == 0 {
            return Ok(());
        }

        let new_total = self.amount.saturating_add(amount_to_add);
        if new_total > 1_000_000 {
            return Err(DomainError::InvalidResourceAmount(new_total as i32));
        }

        self.amount = new_total;
        self.update_timestamp();
        Ok(())
    }

    /// Check if resource is depleted
    pub fn is_depleted(&self) -> bool {
        self.amount == 0
    }

    /// Get effective value considering quality
    pub fn effective_value(&self) -> u32 {
        let base_value = self.amount * self.resource_type.base_value();
        (base_value as f32 * self.quality.value_multiplier()) as u32
    }

    /// Get version
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Update timestamp and increment version
    fn update_timestamp(&mut self) {
        self.last_updated = Utc::now();
        self.version += 1;
    }

    /// Split this resource into two parts
    pub fn split(&mut self, split_amount: u32) -> DomainResult<Option<Resource>> {
        if split_amount == 0 {
            return Ok(None);
        }

        if split_amount >= self.amount {
            return Err(DomainError::InvalidResourceAmount(split_amount as i32));
        }

        let remaining = self.amount - split_amount;
        self.amount = remaining;
        self.update_timestamp();

        let new_resource = Resource::new(
            self.resource_type,
            split_amount,
            self.quality,
            self.position,
        )?;

        Ok(Some(new_resource))
    }

    /// Merge another resource of the same type into this one
    pub fn merge(&mut self, other: Resource) -> DomainResult<()> {
        if self.resource_type != other.resource_type {
            return Err(DomainError::InvalidResourceType(
                "Cannot merge different resource types".to_string(),
            ));
        }

        // Use the better quality when merging
        if other.quality.value_multiplier() > self.quality.value_multiplier() {
            self.quality = other.quality;
        }

        self.add(other.amount)?;
        Ok(())
    }
}

/// Quality levels for resources
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResourceQuality {
    Poor,
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl ResourceQuality {
    /// Get all quality levels
    pub fn all() -> Vec<ResourceQuality> {
        vec![
            ResourceQuality::Poor,
            ResourceQuality::Common,
            ResourceQuality::Uncommon,
            ResourceQuality::Rare,
            ResourceQuality::Epic,
            ResourceQuality::Legendary,
        ]
    }

    /// Get value multiplier for this quality
    pub fn value_multiplier(&self) -> f32 {
        match self {
            ResourceQuality::Poor => 0.5,
            ResourceQuality::Common => 1.0,
            ResourceQuality::Uncommon => 1.5,
            ResourceQuality::Rare => 2.5,
            ResourceQuality::Epic => 4.0,
            ResourceQuality::Legendary => 8.0,
        }
    }

    /// Get rarity percentage (chance of finding this quality)
    pub fn rarity_percentage(&self) -> f32 {
        match self {
            ResourceQuality::Poor => 15.0,
            ResourceQuality::Common => 60.0,
            ResourceQuality::Uncommon => 20.0,
            ResourceQuality::Rare => 4.0,
            ResourceQuality::Epic => 0.9,
            ResourceQuality::Legendary => 0.1,
        }
    }

    /// Get color code for UI display
    pub fn color_code(&self) -> (u8, u8, u8) {
        match self {
            ResourceQuality::Poor => (150, 150, 150),    // Gray
            ResourceQuality::Common => (255, 255, 255),  // White
            ResourceQuality::Uncommon => (30, 255, 30),  // Green
            ResourceQuality::Rare => (30, 100, 255),     // Blue
            ResourceQuality::Epic => (160, 32, 240),     // Purple
            ResourceQuality::Legendary => (255, 165, 0), // Orange/Gold
        }
    }
}

impl std::fmt::Display for ResourceQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceQuality::Poor => write!(f, "Poor"),
            ResourceQuality::Common => write!(f, "Common"),
            ResourceQuality::Uncommon => write!(f, "Uncommon"),
            ResourceQuality::Rare => write!(f, "Rare"),
            ResourceQuality::Epic => write!(f, "Epic"),
            ResourceQuality::Legendary => write!(f, "Legendary"),
        }
    }
}

impl Default for ResourceQuality {
    fn default() -> Self {
        ResourceQuality::Common
    }
}

/// Resource stack - represents a collection of the same resource type and quality
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceStack {
    resources: Vec<Resource>,
}

impl ResourceStack {
    /// Create a new empty resource stack
    pub fn new() -> Self {
        Self {
            resources: Vec::new(),
        }
    }

    /// Create a stack with a single resource
    pub fn from_resource(resource: Resource) -> Self {
        Self {
            resources: vec![resource],
        }
    }

    /// Add a resource to the stack
    pub fn add_resource(&mut self, resource: Resource) -> DomainResult<()> {
        // Check if we can merge with existing resources
        for existing in &mut self.resources {
            if existing.resource_type() == resource.resource_type()
                && existing.quality() == resource.quality()
            {
                existing.merge(resource)?;
                return Ok(());
            }
        }

        // If we can't merge, add as new resource
        self.resources.push(resource);
        Ok(())
    }

    /// Get total amount in stack
    pub fn total_amount(&self) -> u32 {
        self.resources.iter().map(|r| r.amount()).sum()
    }

    /// Get all resources in stack
    pub fn resources(&self) -> &[Resource] {
        &self.resources
    }

    /// Check if stack is empty
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty() || self.total_amount() == 0
    }

    /// Remove depleted resources from stack
    pub fn clean_depleted(&mut self) {
        self.resources.retain(|r| !r.is_depleted());
    }

    /// Consume amount from stack (best quality first)
    pub fn consume(&mut self, mut amount_to_consume: u32) -> u32 {
        let mut total_consumed = 0;

        // Sort by quality (best first) for consumption
        self.resources.sort_by(|a, b| b.quality().cmp(&a.quality()));

        for resource in &mut self.resources {
            if amount_to_consume == 0 {
                break;
            }

            let consumed = resource.consume(amount_to_consume).unwrap_or(0);
            total_consumed += consumed;
            amount_to_consume -= consumed;
        }

        self.clean_depleted();
        total_consumed
    }
}

impl Default for ResourceStack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_creation() {
        let resource = Resource::new(
            ResourceType::Metal,
            100,
            ResourceQuality::Common,
            Some(Position3D::new(5, 5, 0)),
        )
        .unwrap();

        assert_eq!(resource.resource_type(), ResourceType::Metal);
        assert_eq!(resource.amount(), 100);
        assert_eq!(resource.quality(), ResourceQuality::Common);
        assert!(resource.position().is_some());
    }

    #[test]
    fn resource_consumption() {
        let mut resource =
            Resource::new(ResourceType::Energy, 50, ResourceQuality::Common, None).unwrap();

        let consumed = resource.consume(20).unwrap();
        assert_eq!(consumed, 20);
        assert_eq!(resource.amount(), 30);

        let consumed = resource.consume(100).unwrap();
        assert_eq!(consumed, 30); // Only consumed what was available
        assert!(resource.is_depleted());
    }

    #[test]
    fn resource_splitting() {
        let mut resource =
            Resource::new(ResourceType::Food, 100, ResourceQuality::Rare, None).unwrap();

        let split_resource = resource.split(30).unwrap().unwrap();
        assert_eq!(resource.amount(), 70);
        assert_eq!(split_resource.amount(), 30);
        assert_eq!(split_resource.quality(), ResourceQuality::Rare);
    }

    #[test]
    fn resource_merging() {
        let mut resource1 =
            Resource::new(ResourceType::Metal, 50, ResourceQuality::Common, None).unwrap();

        let resource2 =
            Resource::new(ResourceType::Metal, 30, ResourceQuality::Rare, None).unwrap();

        resource1.merge(resource2).unwrap();
        assert_eq!(resource1.amount(), 80);
        assert_eq!(resource1.quality(), ResourceQuality::Rare); // Better quality wins
    }

    #[test]
    fn resource_quality_properties() {
        assert_eq!(ResourceQuality::Common.value_multiplier(), 1.0);
        assert_eq!(ResourceQuality::Legendary.value_multiplier(), 8.0);
        assert!(
            ResourceQuality::Common.rarity_percentage()
                > ResourceQuality::Legendary.rarity_percentage()
        );
    }

    #[test]
    fn resource_stack_operations() {
        let mut stack = ResourceStack::new();
        assert!(stack.is_empty());

        let resource1 =
            Resource::new(ResourceType::Metal, 50, ResourceQuality::Common, None).unwrap();

        let resource2 =
            Resource::new(ResourceType::Metal, 30, ResourceQuality::Common, None).unwrap();

        stack.add_resource(resource1).unwrap();
        stack.add_resource(resource2).unwrap();

        assert_eq!(stack.total_amount(), 80);
        assert_eq!(stack.resources().len(), 1); // Should have merged
    }

    #[test]
    fn resource_stack_consumption() {
        let mut stack = ResourceStack::new();

        let common_resource =
            Resource::new(ResourceType::Energy, 50, ResourceQuality::Common, None).unwrap();

        let rare_resource =
            Resource::new(ResourceType::Energy, 30, ResourceQuality::Rare, None).unwrap();

        stack.add_resource(common_resource).unwrap();
        stack.add_resource(rare_resource).unwrap();

        let consumed = stack.consume(40);
        assert_eq!(consumed, 40);

        // Should consume rare first (better quality)
        assert_eq!(stack.total_amount(), 40);
    }

    #[test]
    fn invalid_resource_creation() {
        let result = Resource::new(ResourceType::Metal, 0, ResourceQuality::Common, None);
        assert!(result.is_err());

        let result = Resource::new(
            ResourceType::Metal,
            2_000_000,
            ResourceQuality::Common,
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn resource_merging_different_types() {
        let mut metal_resource =
            Resource::new(ResourceType::Metal, 50, ResourceQuality::Common, None).unwrap();

        let energy_resource =
            Resource::new(ResourceType::Energy, 30, ResourceQuality::Common, None).unwrap();

        let result = metal_resource.merge(energy_resource);
        assert!(result.is_err());
    }
}
