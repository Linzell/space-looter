//! Player Entity - The main character controlled by the user
//!
//! This entity represents the player character with RPG statistics,
//! progression system, inventory, and all player-related game state.

use crate::domain::value_objects::{
    resources::ResourceCollection, EntityId, Experience, GameTime, PlayerStats, Position3D,
    StatType,
};
use crate::domain::{DomainError, DomainResult};
use bevy::log::warn;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The player character entity
#[derive(Debug, Clone, PartialEq)]
pub struct Player {
    id: EntityId,
    name: String,
    position: Position3D,
    stats: PlayerStats,
    experience: Experience,
    resources: ResourceCollection,
    movement_points: u8,
    action_points: u8,
    max_movement_points: u8,
    max_action_points: u8,
    equipment: PlayerEquipment,
    status_effects: Vec<StatusEffect>,
    exploration_data: ExplorationData,
    created_at: DateTime<Utc>,
    last_updated: DateTime<Utc>,
    version: u64,
}

impl Player {
    /// Create a new player character
    pub fn new(
        id: EntityId,
        name: String,
        starting_position: Position3D,
        starting_stats: PlayerStats,
    ) -> DomainResult<Self> {
        if name.is_empty() || name.len() > 50 {
            return Err(DomainError::PlayerError(
                "Player name must be between 1 and 50 characters".to_string(),
            ));
        }

        let experience = Experience::new(0)?;
        let resources = ResourceCollection::starting_resources();
        let now = Utc::now();

        Ok(Self {
            id,
            name,
            position: starting_position,
            stats: starting_stats,
            experience,
            resources,
            movement_points: crate::domain::constants::BASE_MOVEMENT_POINTS,
            action_points: crate::domain::constants::BASE_ACTION_POINTS,
            max_movement_points: crate::domain::constants::BASE_MOVEMENT_POINTS,
            max_action_points: crate::domain::constants::BASE_ACTION_POINTS,
            equipment: PlayerEquipment::new(),
            status_effects: Vec::new(),
            exploration_data: ExplorationData::new(),
            created_at: now,
            last_updated: now,
            version: 1,
        })
    }

    /// Create a new player with starting stats
    pub fn create_new_character(name: String, starting_position: Position3D) -> DomainResult<Self> {
        let id = EntityId::generate();
        let stats = PlayerStats::starting_stats();
        Self::new(id, name, starting_position, stats)
    }

    /// Get player ID
    pub fn id(&self) -> &EntityId {
        &self.id
    }

    /// Get player name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get current position
    pub fn position(&self) -> &Position3D {
        &self.position
    }

    /// Get player statistics
    pub fn stats(&self) -> &PlayerStats {
        &self.stats
    }

    /// Get current experience
    pub fn experience(&self) -> &Experience {
        &self.experience
    }

    /// Get current level
    pub fn level(&self) -> u32 {
        self.experience.level()
    }

    /// Get resources
    pub fn resources(&self) -> &ResourceCollection {
        &self.resources
    }

    /// Get mutable resources
    pub fn resources_mut(&mut self) -> &mut ResourceCollection {
        self.update_timestamp();
        &mut self.resources
    }

    /// Add resources to player inventory
    pub fn add_resources(&mut self, resources: &ResourceCollection) {
        if let Err(e) = self.resources.add_collection(resources) {
            warn!("Failed to add resources to player: {}", e);
        }
        self.update_timestamp();
    }

    /// Get current movement points
    pub fn movement_points(&self) -> u8 {
        self.movement_points
    }

    /// Get current action points
    pub fn action_points(&self) -> u8 {
        self.action_points
    }

    /// Get maximum movement points
    pub fn max_movement_points(&self) -> u8 {
        self.max_movement_points
    }

    /// Get maximum action points
    pub fn max_action_points(&self) -> u8 {
        self.max_action_points
    }

    /// Check if player can move
    pub fn can_move(&self) -> bool {
        self.movement_points > 0 && !self.is_incapacitated()
    }

    /// Check if player can take actions
    pub fn can_act(&self) -> bool {
        self.action_points > 0 && !self.is_incapacitated()
    }

    /// Move to a new position (costs movement points)
    pub fn move_to(&mut self, new_position: Position3D, movement_cost: u8) -> DomainResult<()> {
        if !self.can_move() {
            return Err(DomainError::PlayerError("Player cannot move".to_string()));
        }

        if self.movement_points < movement_cost {
            return Err(DomainError::PlayerError(format!(
                "Insufficient movement points. Need {}, have {}",
                movement_cost, self.movement_points
            )));
        }

        self.position = new_position;
        self.movement_points = self.movement_points.saturating_sub(movement_cost);

        // Update exploration data
        self.exploration_data.visit_location(new_position);

        self.update_timestamp();
        Ok(())
    }

    /// Consume action points
    pub fn consume_action_points(&mut self, cost: u8) -> DomainResult<()> {
        if !self.can_act() {
            return Err(DomainError::PlayerError(
                "Player cannot take actions".to_string(),
            ));
        }

        if self.action_points < cost {
            return Err(DomainError::PlayerError(format!(
                "Insufficient action points. Need {}, have {}",
                cost, self.action_points
            )));
        }

        self.action_points = self.action_points.saturating_sub(cost);
        self.update_timestamp();
        Ok(())
    }

    /// Restore movement and action points (typically at turn start)
    pub fn restore_points(&mut self) {
        self.movement_points = self.max_movement_points;
        self.action_points = self.max_action_points;
        self.update_timestamp();
    }

    /// Add movement points (capped at maximum)
    pub fn add_movement_points(&mut self, points: u8) {
        self.movement_points =
            (self.movement_points.saturating_add(points)).min(self.max_movement_points);
        self.update_timestamp();
    }

    /// Add action points (capped at maximum)
    pub fn add_action_points(&mut self, points: u8) {
        self.action_points =
            (self.action_points.saturating_add(points)).min(self.max_action_points);
        self.update_timestamp();
    }

    /// Subtract movement points safely (won't go below 0)
    pub fn subtract_movement_points(&mut self, points: u8) {
        self.movement_points = self.movement_points.saturating_sub(points);
        self.update_timestamp();
    }

    /// Subtract action points safely (won't go below 0)
    pub fn subtract_action_points(&mut self, points: u8) {
        self.action_points = self.action_points.saturating_sub(points);
        self.update_timestamp();
    }

    /// Add experience points
    pub fn add_experience(&mut self, points: u32) -> DomainResult<bool> {
        let old_level = self.experience.level();
        self.experience = self.experience.add_points(points)?;
        let new_level = self.experience.level();

        self.update_timestamp();

        if new_level > old_level {
            self.handle_level_up(old_level, new_level)?;
            Ok(true) // Level up occurred
        } else {
            Ok(false) // No level up
        }
    }

    /// Handle level up logic
    fn handle_level_up(&mut self, _old_level: u32, new_level: u32) -> DomainResult<()> {
        // Increase max points based on endurance
        let endurance_bonus = (self.stats.endurance as f32 / 10.0) as u8;
        self.max_movement_points = self.max_movement_points.saturating_add(1);
        self.max_action_points = self
            .max_action_points
            .saturating_add(endurance_bonus.max(1));

        // Restore points on level up
        self.restore_points();

        // Player gets stat points to allocate
        // This could be handled by the application layer

        println!("Player {} reached level {}!", self.name, new_level);
        Ok(())
    }

    /// Increase a stat (typically from level up)
    pub fn increase_stat(&mut self, stat_type: StatType) -> DomainResult<()> {
        self.stats = self.stats.increase_stat(stat_type)?;
        self.update_timestamp();
        Ok(())
    }

    /// Get stat modifier for dice rolls
    pub fn get_stat_modifier(&self, stat_type: StatType) -> i8 {
        let base_modifier = self.stats.get_modifier(stat_type);
        let equipment_modifier = self.equipment.get_stat_modifier(stat_type);
        base_modifier + equipment_modifier
    }

    /// Check if player is incapacitated
    pub fn is_incapacitated(&self) -> bool {
        self.status_effects
            .iter()
            .any(|effect| effect.is_incapacitating())
    }

    /// Add a status effect
    pub fn add_status_effect(&mut self, effect: StatusEffect) {
        // Remove existing effect of same type
        self.status_effects
            .retain(|e| e.effect_type != effect.effect_type);
        self.status_effects.push(effect);
        self.update_timestamp();
    }

    /// Remove status effects that have expired
    pub fn update_status_effects(&mut self, current_time: GameTime) {
        let initial_count = self.status_effects.len();
        self.status_effects
            .retain(|effect| !effect.is_expired(current_time));

        if self.status_effects.len() != initial_count {
            self.update_timestamp();
        }
    }

    /// Get exploration data
    pub fn exploration_data(&self) -> &ExplorationData {
        &self.exploration_data
    }

    /// Check if player has visited a location
    pub fn has_visited(&self, position: &Position3D) -> bool {
        self.exploration_data.has_visited(position)
    }

    /// Get total distance traveled
    pub fn total_distance_traveled(&self) -> u32 {
        self.exploration_data.total_distance()
    }

    /// Get locations visited count
    pub fn locations_visited_count(&self) -> usize {
        self.exploration_data.locations_visited()
    }

    /// Get equipment
    pub fn equipment(&self) -> &PlayerEquipment {
        &self.equipment
    }

    /// Equip an item
    pub fn equip_item(
        &mut self,
        slot: EquipmentSlot,
        item: Equipment,
    ) -> DomainResult<Option<Equipment>> {
        let old_item = self.equipment.equip(slot, item)?;
        self.update_timestamp();
        Ok(old_item)
    }

    /// Unequip an item
    pub fn unequip_item(&mut self, slot: EquipmentSlot) -> Option<Equipment> {
        let item = self.equipment.unequip(slot);
        if item.is_some() {
            self.update_timestamp();
        }
        item
    }

    /// Check if player meets level requirement
    pub fn meets_level_requirement(&self, required_level: u32) -> bool {
        self.level() >= required_level
    }

    /// Get creation timestamp
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Get last update timestamp
    pub fn last_updated(&self) -> DateTime<Utc> {
        self.last_updated
    }

    /// Get entity version
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Update the last modified timestamp and increment version
    fn update_timestamp(&mut self) {
        self.last_updated = Utc::now();
        self.version += 1;
    }

    /// Get player's current carrying capacity
    pub fn carrying_capacity(&self) -> u32 {
        let base_capacity = 100;
        let strength_bonus = self.stats.strength as u32 * 10;
        let equipment_bonus = self.equipment.get_carrying_capacity_bonus();
        base_capacity + strength_bonus + equipment_bonus
    }

    /// Check if player is overloaded
    pub fn is_overloaded(&self) -> bool {
        self.resources.storage_requirement() > self.carrying_capacity()
    }

    /// Get summary information for display
    pub fn summary(&self) -> PlayerSummary {
        PlayerSummary {
            name: self.name.clone(),
            level: self.level(),
            position: self.position,
            movement_points: self.movement_points,
            action_points: self.action_points,
            total_resources_value: self.resources.total_value(),
            locations_visited: self.locations_visited_count(),
            is_overloaded: self.is_overloaded(),
        }
    }

    /// Get player movement speed
    pub fn speed(&self) -> f32 {
        // Base speed modified by stats
        let base_speed = 100.0;
        let dexterity_modifier = (self.stats.dexterity - 10) as f32 * 5.0;
        (base_speed + dexterity_modifier).max(10.0)
    }
}

impl Player {
    /// Check if this player entity is valid
    pub fn is_valid(&self) -> bool {
        !self.name.is_empty()
            && self.name.len() <= 50
            && self.movement_points <= self.max_movement_points
            && self.action_points <= self.max_action_points
            && self.level() <= crate::domain::constants::MAX_PLAYER_LEVEL
    }
}

/// Player equipment system
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerEquipment {
    slots: HashMap<EquipmentSlot, Equipment>,
}

impl PlayerEquipment {
    /// Create new empty equipment
    pub fn new() -> Self {
        Self {
            slots: HashMap::new(),
        }
    }

    /// Equip an item in a slot
    pub fn equip(
        &mut self,
        slot: EquipmentSlot,
        item: Equipment,
    ) -> DomainResult<Option<Equipment>> {
        // Validate item can go in this slot
        if !item.can_equip_in_slot(slot) {
            return Err(DomainError::PlayerError(format!(
                "Cannot equip {} in {} slot",
                item.name, slot
            )));
        }

        let old_item = self.slots.insert(slot, item);
        Ok(old_item)
    }

    /// Unequip an item from a slot
    pub fn unequip(&mut self, slot: EquipmentSlot) -> Option<Equipment> {
        self.slots.remove(&slot)
    }

    /// Get equipped item in slot
    pub fn get_equipped(&self, slot: EquipmentSlot) -> Option<&Equipment> {
        self.slots.get(&slot)
    }

    /// Get stat modifier from all equipped items
    pub fn get_stat_modifier(&self, stat_type: StatType) -> i8 {
        self.slots
            .values()
            .map(|item| item.get_stat_modifier(stat_type))
            .sum()
    }

    /// Get carrying capacity bonus from equipment
    pub fn get_carrying_capacity_bonus(&self) -> u32 {
        self.slots
            .values()
            .map(|item| item.carrying_capacity_bonus)
            .sum()
    }

    /// Check if a slot is occupied
    pub fn is_slot_equipped(&self, slot: EquipmentSlot) -> bool {
        self.slots.contains_key(&slot)
    }
}

impl Default for PlayerEquipment {
    fn default() -> Self {
        Self::new()
    }
}

/// Equipment slots
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EquipmentSlot {
    MainHand,
    OffHand,
    Head,
    Body,
    Legs,
    Feet,
    Accessory1,
    Accessory2,
}

impl std::fmt::Display for EquipmentSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EquipmentSlot::MainHand => write!(f, "Main Hand"),
            EquipmentSlot::OffHand => write!(f, "Off Hand"),
            EquipmentSlot::Head => write!(f, "Head"),
            EquipmentSlot::Body => write!(f, "Body"),
            EquipmentSlot::Legs => write!(f, "Legs"),
            EquipmentSlot::Feet => write!(f, "Feet"),
            EquipmentSlot::Accessory1 => write!(f, "Accessory 1"),
            EquipmentSlot::Accessory2 => write!(f, "Accessory 2"),
        }
    }
}

/// Equipment item
#[derive(Debug, Clone, PartialEq)]
pub struct Equipment {
    pub name: String,
    pub equipment_type: EquipmentType,
    pub stat_modifiers: HashMap<StatType, i8>,
    pub carrying_capacity_bonus: u32,
    pub description: String,
}

impl Equipment {
    /// Create new equipment
    pub fn new(
        name: String,
        equipment_type: EquipmentType,
        stat_modifiers: HashMap<StatType, i8>,
        carrying_capacity_bonus: u32,
        description: String,
    ) -> Self {
        Self {
            name,
            equipment_type,
            stat_modifiers,
            carrying_capacity_bonus,
            description,
        }
    }

    /// Check if this equipment can be equipped in a slot
    pub fn can_equip_in_slot(&self, slot: EquipmentSlot) -> bool {
        self.equipment_type.valid_slots().contains(&slot)
    }

    /// Get stat modifier for a specific stat
    pub fn get_stat_modifier(&self, stat_type: StatType) -> i8 {
        self.stat_modifiers.get(&stat_type).copied().unwrap_or(0)
    }
}

/// Types of equipment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentType {
    Weapon,
    Shield,
    Helmet,
    Armor,
    Pants,
    Boots,
    Ring,
    Amulet,
    Tool,
}

impl EquipmentType {
    /// Get valid equipment slots for this type
    pub fn valid_slots(&self) -> Vec<EquipmentSlot> {
        match self {
            EquipmentType::Weapon => vec![EquipmentSlot::MainHand, EquipmentSlot::OffHand],
            EquipmentType::Shield => vec![EquipmentSlot::OffHand],
            EquipmentType::Helmet => vec![EquipmentSlot::Head],
            EquipmentType::Armor => vec![EquipmentSlot::Body],
            EquipmentType::Pants => vec![EquipmentSlot::Legs],
            EquipmentType::Boots => vec![EquipmentSlot::Feet],
            EquipmentType::Ring => vec![EquipmentSlot::Accessory1, EquipmentSlot::Accessory2],
            EquipmentType::Amulet => vec![EquipmentSlot::Accessory1, EquipmentSlot::Accessory2],
            EquipmentType::Tool => vec![EquipmentSlot::MainHand, EquipmentSlot::OffHand],
        }
    }
}

/// Status effects that can be applied to the player
#[derive(Debug, Clone, PartialEq)]
pub struct StatusEffect {
    pub effect_type: StatusEffectType,
    pub duration: GameTime,
    pub applied_at: GameTime,
    pub stat_modifiers: HashMap<StatType, i8>,
    pub description: String,
}

impl StatusEffect {
    /// Create a new status effect
    pub fn new(
        effect_type: StatusEffectType,
        duration: GameTime,
        applied_at: GameTime,
        stat_modifiers: HashMap<StatType, i8>,
        description: String,
    ) -> Self {
        Self {
            effect_type,
            duration,
            applied_at,
            stat_modifiers,
            description,
        }
    }

    /// Check if this effect has expired
    pub fn is_expired(&self, current_time: GameTime) -> bool {
        current_time.seconds() >= self.applied_at.seconds() + self.duration.seconds()
    }

    /// Check if this effect prevents actions
    pub fn is_incapacitating(&self) -> bool {
        matches!(
            self.effect_type,
            StatusEffectType::Paralyzed | StatusEffectType::Stunned
        )
    }
}

/// Types of status effects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusEffectType {
    Blessed,
    Cursed,
    Poisoned,
    Energized,
    Exhausted,
    Protected,
    Vulnerable,
    Paralyzed,
    Stunned,
    Lucky,
    Unlucky,
}

/// Tracks player's exploration history
#[derive(Debug, Clone, PartialEq)]
pub struct ExplorationData {
    visited_locations: std::collections::HashSet<Position3D>,
    total_distance: u32,
    first_visit_times: HashMap<Position3D, DateTime<Utc>>,
}

impl ExplorationData {
    /// Create new exploration data
    pub fn new() -> Self {
        Self {
            visited_locations: std::collections::HashSet::new(),
            total_distance: 0,
            first_visit_times: HashMap::new(),
        }
    }

    /// Record a visit to a location
    pub fn visit_location(&mut self, position: Position3D) {
        let is_new = self.visited_locations.insert(position);
        if is_new {
            self.first_visit_times.insert(position, Utc::now());
            // Add distance if not first location
            if self.visited_locations.len() > 1 {
                self.total_distance += 1; // Simplified distance calculation
            }
        }
    }

    /// Check if location has been visited
    pub fn has_visited(&self, position: &Position3D) -> bool {
        self.visited_locations.contains(position)
    }

    /// Get total distance traveled
    pub fn total_distance(&self) -> u32 {
        self.total_distance
    }

    /// Get number of unique locations visited
    pub fn locations_visited(&self) -> usize {
        self.visited_locations.len()
    }

    /// Get first visit time for a location
    pub fn first_visit_time(&self, position: &Position3D) -> Option<DateTime<Utc>> {
        self.first_visit_times.get(position).copied()
    }
}

impl Default for ExplorationData {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary information about a player for display
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerSummary {
    pub name: String,
    pub level: u32,
    pub position: Position3D,
    pub movement_points: u8,
    pub action_points: u8,
    pub total_resources_value: u32,
    pub locations_visited: usize,
    pub is_overloaded: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::StatType;

    #[test]
    fn player_creation() {
        let id = EntityId::generate();
        let name = "Test Player".to_string();
        let position = Position3D::origin();
        let stats = PlayerStats::starting_stats();

        let player = Player::new(id, name.clone(), position, stats).unwrap();
        assert_eq!(player.name(), &name);
        assert_eq!(player.position(), &position);
        assert_eq!(player.level(), 1);
        assert!(player.can_move());
        assert!(player.can_act());
    }

    #[test]
    fn player_movement() {
        let mut player =
            Player::create_new_character("Test".to_string(), Position3D::origin()).unwrap();

        let new_position = Position3D::new(1, 1, 0);
        let result = player.move_to(new_position, 1);
        assert!(result.is_ok());
        assert_eq!(player.position(), &new_position);
        assert_eq!(player.movement_points(), 2); // Started with 3, used 1
    }

    #[test]
    fn player_movement_insufficient_points() {
        let mut player =
            Player::create_new_character("Test".to_string(), Position3D::origin()).unwrap();

        // Try to move with more cost than available points
        let new_position = Position3D::new(1, 1, 0);
        let result = player.move_to(new_position, 10);
        assert!(result.is_err());
        assert_eq!(player.position(), &Position3D::origin()); // Shouldn't move
    }

    #[test]
    fn player_experience_and_level_up() {
        let mut player =
            Player::create_new_character("Test".to_string(), Position3D::origin()).unwrap();

        assert_eq!(player.level(), 1);

        let leveled_up = player.add_experience(100).unwrap();
        assert!(leveled_up);
        assert_eq!(player.level(), 2);
    }

    #[test]
    fn player_stat_modifiers() {
        let id = EntityId::generate();
        let name = "Test Player".to_string();
        let position = Position3D::origin();
        let stats = PlayerStats::new(15, 10, 10, 10, 10, 10).unwrap(); // High strength

        let player = Player::new(id, name, position, stats).unwrap();
        let strength_modifier = player.get_stat_modifier(StatType::Strength);
        assert_eq!(strength_modifier, 2); // 15 -> +2 modifier
    }

    #[test]
    fn player_equipment() {
        let mut player =
            Player::create_new_character("Test".to_string(), Position3D::origin()).unwrap();

        let mut stat_mods = HashMap::new();
        stat_mods.insert(StatType::Strength, 2);

        let sword = Equipment::new(
            "Iron Sword".to_string(),
            EquipmentType::Weapon,
            stat_mods,
            0,
            "A sturdy iron sword".to_string(),
        );

        let result = player.equip_item(EquipmentSlot::MainHand, sword);
        assert!(result.is_ok());
        assert!(player.equipment().is_slot_equipped(EquipmentSlot::MainHand));

        // Check that strength modifier is now higher
        let str_mod = player.get_stat_modifier(StatType::Strength);
        assert!(str_mod > 0);
    }

    #[test]
    fn player_status_effects() {
        let mut player =
            Player::create_new_character("Test".to_string(), Position3D::origin()).unwrap();

        let effect = StatusEffect::new(
            StatusEffectType::Blessed,
            GameTime::from_minutes(5),
            GameTime::new(0),
            HashMap::new(),
            "Blessed by the gods".to_string(),
        );

        player.add_status_effect(effect);
        assert_eq!(player.status_effects.len(), 1);
    }

    #[test]
    fn player_exploration_tracking() {
        let mut player =
            Player::create_new_character("Test".to_string(), Position3D::origin()).unwrap();

        let pos1 = Position3D::new(1, 0, 0);
        let pos2 = Position3D::new(2, 0, 0);

        player.move_to(pos1, 1).unwrap();
        player.move_to(pos2, 1).unwrap();

        assert_eq!(player.locations_visited_count(), 3); // Origin + 2 moves
        assert!(player.has_visited(&Position3D::origin()));
        assert!(player.has_visited(&pos1));
        assert!(player.has_visited(&pos2));
    }

    #[test]
    fn player_validation() {
        let player =
            Player::create_new_character("Test".to_string(), Position3D::origin()).unwrap();

        assert!(player.is_valid());

        // Test invalid name
        let invalid_player = Player::create_new_character("".to_string(), Position3D::origin());
        assert!(invalid_player.is_err());
    }

    #[test]
    fn player_carrying_capacity() {
        let id = EntityId::generate();
        let name = "Test Player".to_string();
        let position = Position3D::origin();
        let stats = PlayerStats::new(20, 10, 10, 10, 10, 10).unwrap(); // Max strength

        let player = Player::new(id, name, position, stats).unwrap();
        let capacity = player.carrying_capacity();
        assert!(capacity > 100); // Should be base + strength bonus
    }
}
