//! Bevy Components - ECS Component Wrappers for 3D Isometric RPG Domain Entities
//!
//! This module contains Bevy ECS components that wrap our RPG domain entities,
//! providing the bridge between our pure domain model and Bevy's ECS system.
//! Components are designed for 3D isometric gameplay with dice mechanics.

use crate::domain::entities::base::BaseLevel;
use crate::domain::{
    Base, DiceRoll, EntityId, Event, GameSession, Map, Player, Position3D, Quest, Resource,
    ResourceType, TerrainType,
};
use bevy::prelude::*;
use std::collections::HashMap;

/// Bevy component wrapper for Player domain entity
#[derive(Component, Debug, Clone)]
pub struct PlayerComponent {
    player: Player,
}

impl PlayerComponent {
    /// Create a new player component
    pub fn new(player: Player) -> Self {
        Self { player }
    }

    /// Get reference to the player entity
    pub fn player(&self) -> &Player {
        &self.player
    }

    /// Get mutable reference to the player entity
    pub fn player_mut(&mut self) -> &mut Player {
        &mut self.player
    }

    /// Update the wrapped player entity
    pub fn update_player(&mut self, player: Player) {
        self.player = player;
    }

    /// Get player's current position
    pub fn position(&self) -> Position3D {
        *self.player.position()
    }

    /// Get player's current level
    pub fn level(&self) -> u32 {
        self.player.level()
    }

    /// Check if player can move to a position
    pub fn can_move_to(&self, position: Position3D) -> bool {
        // Add game-specific movement validation here
        true
    }
}

/// Bevy component wrapper for Base domain entity
#[derive(Component, Debug, Clone)]
pub struct BaseComponent {
    base: Base,
}

impl BaseComponent {
    /// Create a new base component
    pub fn new(base: Base) -> Self {
        Self { base }
    }

    /// Get reference to the base entity
    pub fn base(&self) -> &Base {
        &self.base
    }

    /// Get mutable reference to the base entity
    pub fn base_mut(&mut self) -> &mut Base {
        &mut self.base
    }

    /// Update the wrapped base entity
    pub fn update_base(&mut self, base: Base) {
        self.base = base;
    }

    /// Get base position
    pub fn position(&self) -> Position3D {
        *self.base.position()
    }

    /// Get base level
    pub fn level(&self) -> u32 {
        match self.base.level() {
            BaseLevel::Level1 => 1,
            BaseLevel::Level2 => 2,
            BaseLevel::Level3 => 3,
            BaseLevel::Level4 => 4,
            BaseLevel::Level5 => 5,
        }
    }
}

/// Component for 3D world position with isometric rendering support
#[derive(Component, Debug, Clone, Copy)]
pub struct Position3DComponent {
    position: Position3D,
    /// Screen position for isometric rendering
    screen_position: Vec2,
    /// Whether screen position needs recalculation
    dirty: bool,
}

impl Position3DComponent {
    /// Create a new 3D position component
    pub fn new(position: Position3D) -> Self {
        Self {
            position,
            screen_position: Vec2::ZERO,
            dirty: true,
        }
    }

    /// Get the 3D world position
    pub fn position(&self) -> Position3D {
        self.position
    }

    /// Set new 3D position
    pub fn set_position(&mut self, position: Position3D) {
        self.position = position;
        self.dirty = true;
    }

    /// Get screen position for rendering
    pub fn screen_position(&self) -> Vec2 {
        self.screen_position
    }

    /// Set screen position (called by rendering system)
    pub fn set_screen_position(&mut self, screen_pos: Vec2) {
        self.screen_position = screen_pos;
        self.dirty = false;
    }

    /// Check if screen position needs recalculation
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
}

impl From<Position3D> for Position3DComponent {
    fn from(position: Position3D) -> Self {
        Self::new(position)
    }
}

/// Component for terrain tiles in the 3D world
#[derive(Component, Debug, Clone)]
pub struct TerrainComponent {
    terrain_type: TerrainType,
    elevation: i32,
    is_passable: bool,
    movement_cost: u8,
}

impl TerrainComponent {
    /// Create a new terrain component
    pub fn new(terrain_type: TerrainType, elevation: i32) -> Self {
        let (is_passable, movement_cost) = match terrain_type {
            TerrainType::Plains => (true, 1),
            TerrainType::Forest => (true, 2),
            TerrainType::Mountains => (true, 3),
            TerrainType::Desert => (true, 2),
            TerrainType::Tundra => (true, 3),
            TerrainType::Volcanic => (true, 4),
            TerrainType::Anomaly => (true, 5),
            TerrainType::Ocean => (false, 0),
            TerrainType::Swamp => (true, 4),
            TerrainType::Constructed => (true, 2),
            TerrainType::Cave => (true, 3),
            TerrainType::Crystal => (true, 2),
        };

        Self {
            terrain_type,
            elevation,
            is_passable,
            movement_cost,
        }
    }

    /// Get terrain type
    pub fn terrain_type(&self) -> TerrainType {
        self.terrain_type
    }

    /// Get elevation
    pub fn elevation(&self) -> i32 {
        self.elevation
    }

    /// Check if terrain is passable
    pub fn is_passable(&self) -> bool {
        self.is_passable
    }

    /// Get movement cost to enter this terrain
    pub fn movement_cost(&self) -> u8 {
        self.movement_cost
    }

    /// Set passability (for dynamic terrain changes)
    pub fn set_passable(&mut self, passable: bool) {
        self.is_passable = passable;
    }
}

/// Component for resources in the world
#[derive(Component, Debug, Clone)]
pub struct ResourceComponent {
    resource_type: ResourceType,
    amount: i32,
    max_amount: i32,
    regeneration_rate: i32,
    last_gathered: f32,
}

impl ResourceComponent {
    /// Create a new resource component
    pub fn new(resource_type: ResourceType, amount: i32, max_amount: i32) -> Self {
        Self {
            resource_type,
            amount,
            max_amount,
            regeneration_rate: 1,
            last_gathered: 0.0,
        }
    }

    /// Get resource type
    pub fn resource_type(&self) -> ResourceType {
        self.resource_type
    }

    /// Get current amount
    pub fn amount(&self) -> i32 {
        self.amount
    }

    /// Get maximum amount
    pub fn max_amount(&self) -> i32 {
        self.max_amount
    }

    /// Try to gather resources
    pub fn gather(&mut self, amount: i32, current_time: f32) -> i32 {
        let gathered = self.amount.min(amount);
        self.amount -= gathered;
        self.last_gathered = current_time;
        gathered
    }

    /// Update resource regeneration
    pub fn update_regeneration(&mut self, delta_time: f32) {
        if self.amount < self.max_amount {
            // Regenerate resources over time
            self.amount = (self.amount + self.regeneration_rate).min(self.max_amount);
        }
    }

    /// Check if resources are available
    pub fn is_available(&self) -> bool {
        self.amount > 0
    }
}

/// Component for interactive objects and NPCs
#[derive(Component, Debug, Clone)]
pub struct InteractableComponent {
    interaction_type: InteractionType,
    interaction_range: f32,
    requires_dice_roll: bool,
    dice_difficulty: u8,
    is_active: bool,
}

/// Types of interactions available in the RPG
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionType {
    /// Gather resources from this location
    ResourceGathering,
    /// Start a quest
    QuestGiver,
    /// Trigger a random event
    EventTrigger,
    /// Shop or trading post
    Shop,
    /// Examine or investigate
    Examine,
    /// Combat encounter
    Combat,
}

impl InteractableComponent {
    /// Create a new interactable component
    pub fn new(interaction_type: InteractionType, range: f32) -> Self {
        Self {
            interaction_type,
            interaction_range: range,
            requires_dice_roll: false,
            dice_difficulty: 10,
            is_active: true,
        }
    }

    /// Create interactable with dice roll requirement
    pub fn with_dice_roll(mut self, difficulty: u8) -> Self {
        self.requires_dice_roll = true;
        self.dice_difficulty = difficulty;
        self
    }

    /// Get interaction type
    pub fn interaction_type(&self) -> InteractionType {
        self.interaction_type
    }

    /// Get interaction range
    pub fn range(&self) -> f32 {
        self.interaction_range
    }

    /// Check if requires dice roll
    pub fn requires_dice_roll(&self) -> bool {
        self.requires_dice_roll
    }

    /// Get dice difficulty
    pub fn dice_difficulty(&self) -> u8 {
        self.dice_difficulty
    }

    /// Check if currently active
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Set active state
    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }

    /// Check if position is within interaction range
    pub fn in_range(&self, from: Position3D, to: Position3D) -> bool {
        let dx = (to.x - from.x) as f32;
        let dy = (to.y - from.y) as f32;
        let dz = (to.z - from.z) as f32;
        let distance = (dx * dx + dy * dy + dz * dz).sqrt();
        distance <= self.interaction_range
    }
}

/// Component for quest-related entities
#[derive(Component, Debug, Clone)]
pub struct QuestComponent {
    quest: Quest,
    is_available: bool,
    is_completed: bool,
}

impl QuestComponent {
    /// Create a new quest component
    pub fn new(quest: Quest) -> Self {
        Self {
            quest,
            is_available: true,
            is_completed: false,
        }
    }

    /// Get reference to the quest
    pub fn quest(&self) -> &Quest {
        &self.quest
    }

    /// Check if quest is available
    pub fn is_available(&self) -> bool {
        self.is_available && !self.is_completed
    }

    /// Check if quest is completed
    pub fn is_completed(&self) -> bool {
        self.is_completed
    }

    /// Mark quest as completed
    pub fn complete(&mut self) {
        self.is_completed = true;
        self.is_available = false;
    }
}

/// Component for event entities
#[derive(Component, Debug, Clone)]
pub struct EventComponent {
    event: Event,
    triggered: bool,
    trigger_time: f32,
}

impl EventComponent {
    /// Create a new event component
    pub fn new(event: Event) -> Self {
        Self {
            event,
            triggered: false,
            trigger_time: 0.0,
        }
    }

    /// Get reference to the event
    pub fn event(&self) -> &Event {
        &self.event
    }

    /// Check if event has been triggered
    pub fn is_triggered(&self) -> bool {
        self.triggered
    }

    /// Trigger the event
    pub fn trigger(&mut self, time: f32) {
        self.triggered = true;
        self.trigger_time = time;
    }

    /// Get trigger time
    pub fn trigger_time(&self) -> f32 {
        self.trigger_time
    }
}

/// Component for dice roll visualization
#[derive(Component, Debug, Clone)]
pub struct DiceRollComponent {
    dice_roll: DiceRoll,
    display_time: f32,
    fade_out: bool,
}

impl DiceRollComponent {
    /// Create a new dice roll component
    pub fn new(dice_roll: DiceRoll, display_time: f32) -> Self {
        Self {
            dice_roll,
            display_time,
            fade_out: false,
        }
    }

    /// Get the dice roll
    pub fn dice_roll(&self) -> &DiceRoll {
        &self.dice_roll
    }

    /// Update display time
    pub fn update(&mut self, delta_time: f32) {
        if self.display_time > 0.0 {
            self.display_time -= delta_time;
        } else {
            self.fade_out = true;
        }
    }

    /// Check if should fade out
    pub fn should_fade_out(&self) -> bool {
        self.fade_out
    }

    /// Get remaining display time
    pub fn display_time(&self) -> f32 {
        self.display_time
    }
}

/// Marker component for UI elements
#[derive(Component, Debug, Clone)]
pub struct UIComponent;

/// Marker component for game boundary entities (invisible collision boundaries)
#[derive(Component, Debug, Clone)]
pub struct BoundaryComponent;

/// Marker component for the main camera
#[derive(Component, Debug, Clone)]
pub struct MainCameraComponent;

/// Marker component for entities that should be cleaned up when far from player
#[derive(Component, Debug, Clone)]
pub struct CleanupComponent {
    pub cleanup_distance: f32, // Distance from player at which entity should be cleaned up
}

impl CleanupComponent {
    /// Create new cleanup component with default cleanup distance
    pub fn new() -> Self {
        Self {
            cleanup_distance: 50.0, // 50 tiles from player
        }
    }

    /// Create cleanup component with custom distance
    pub fn with_distance(cleanup_distance: f32) -> Self {
        Self { cleanup_distance }
    }

    /// Check if entity should be cleaned up based on distance from player
    pub fn should_cleanup(&self, entity_pos: Position3D, player_pos: Position3D) -> bool {
        let dx = (entity_pos.x - player_pos.x) as f32;
        let dy = (entity_pos.y - player_pos.y) as f32;
        let dz = (entity_pos.z - player_pos.z) as f32;
        let distance = (dx * dx + dy * dy + dz * dz).sqrt();
        distance > self.cleanup_distance
    }
}

impl Default for CleanupComponent {
    fn default() -> Self {
        Self::new()
    }
}

// Backward compatibility components (to be removed after transition)

/// Bevy component wrapper for Enemy domain entity (legacy)
#[derive(Component, Debug, Clone)]
pub struct EnemyComponent {
    enemy: crate::domain::Enemy,
}

impl EnemyComponent {
    /// Create a new enemy component
    pub fn new(enemy: crate::domain::Enemy) -> Self {
        Self { enemy }
    }

    /// Get reference to the enemy entity
    pub fn enemy(&self) -> &crate::domain::Enemy {
        &self.enemy
    }

    /// Get mutable reference to the enemy entity
    pub fn enemy_mut(&mut self) -> &mut crate::domain::Enemy {
        &mut self.enemy
    }

    /// Update the wrapped enemy entity
    pub fn update_enemy(&mut self, enemy: crate::domain::Enemy) {
        self.enemy = enemy;
    }
}

/// Bevy component wrapper for Velocity domain value object (legacy)
#[derive(Component, Debug, Clone)]
pub struct VelocityComponent {
    velocity: crate::domain::Velocity,
}

impl VelocityComponent {
    /// Create a new velocity component
    pub fn new(velocity: crate::domain::Velocity) -> Self {
        Self { velocity }
    }

    /// Get reference to the velocity
    pub fn velocity(&self) -> &crate::domain::Velocity {
        &self.velocity
    }

    /// Get mutable reference to the velocity
    pub fn velocity_mut(&mut self) -> &mut crate::domain::Velocity {
        &mut self.velocity
    }

    /// Update the velocity
    pub fn update_velocity(&mut self, velocity: crate::domain::Velocity) {
        self.velocity = velocity;
    }
}

/// Marker component for score display UI elements (legacy)
#[derive(Component, Debug, Clone)]
pub struct ScoreDisplayComponent;

/// Component for animated sprites or models
#[derive(Component, Debug, Clone)]
pub struct AnimationComponent {
    current_animation: String,
    frame_timer: f32,
    frame_duration: f32,
    is_looping: bool,
    is_playing: bool,
}

impl AnimationComponent {
    /// Create a new animation component
    pub fn new(animation_name: String, frame_duration: f32) -> Self {
        Self {
            current_animation: animation_name,
            frame_timer: 0.0,
            frame_duration,
            is_looping: true,
            is_playing: true,
        }
    }

    /// Get current animation name
    pub fn current_animation(&self) -> &str {
        &self.current_animation
    }

    /// Set new animation
    pub fn set_animation(&mut self, animation_name: String) {
        if self.current_animation != animation_name {
            self.current_animation = animation_name;
            self.frame_timer = 0.0;
            self.is_playing = true;
        }
    }

    /// Update animation timer
    pub fn update(&mut self, delta_time: f32) {
        if self.is_playing {
            self.frame_timer += delta_time;
            if self.frame_timer >= self.frame_duration {
                self.frame_timer = 0.0;
                if !self.is_looping {
                    self.is_playing = false;
                }
            }
        }
    }

    /// Check if animation is playing
    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    /// Set looping behavior
    pub fn set_looping(&mut self, looping: bool) {
        self.is_looping = looping;
    }

    /// Play animation
    pub fn play(&mut self) {
        self.is_playing = true;
        self.frame_timer = 0.0;
    }

    /// Stop animation
    pub fn stop(&mut self) {
        self.is_playing = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{PlayerStats, Position3D, ResourceType, StatType, TerrainType};

    #[test]
    fn position_3d_component_creation() {
        let pos = Position3D::new(5, 3, 1);
        let component = Position3DComponent::new(pos);
        assert_eq!(component.position(), pos);
        assert!(component.is_dirty());
    }

    #[test]
    fn terrain_component_creation() {
        let terrain = TerrainComponent::new(TerrainType::Forest, 2);
        assert_eq!(terrain.terrain_type(), TerrainType::Forest);
        assert_eq!(terrain.elevation(), 2);
        assert!(terrain.is_passable());
        assert_eq!(terrain.movement_cost(), 2);
    }

    #[test]
    fn terrain_water_not_passable() {
        let terrain = TerrainComponent::new(TerrainType::Ocean, 0);
        assert!(!terrain.is_passable());
        assert_eq!(terrain.movement_cost(), 0);
    }

    #[test]
    fn resource_component_gathering() {
        let mut resource = ResourceComponent::new(ResourceType::Metal, 10, 20);
        let gathered = resource.gather(5, 0.0);
        assert_eq!(gathered, 5);
        assert_eq!(resource.amount(), 5);
        assert!(resource.is_available());
    }

    #[test]
    fn resource_component_over_gathering() {
        let mut resource = ResourceComponent::new(ResourceType::Energy, 3, 10);
        let gathered = resource.gather(5, 0.0);
        assert_eq!(gathered, 3);
        assert_eq!(resource.amount(), 0);
        assert!(!resource.is_available());
    }

    #[test]
    fn interactable_component_range_check() {
        let interactable = InteractableComponent::new(InteractionType::ResourceGathering, 2.0);
        let pos1 = Position3D::new(0, 0, 0);
        let pos2 = Position3D::new(1, 1, 0); // Distance = sqrt(2) ≈ 1.41
        let pos3 = Position3D::new(3, 3, 0); // Distance = sqrt(18) ≈ 4.24

        assert!(interactable.in_range(pos1, pos2));
        assert!(!interactable.in_range(pos1, pos3));
    }

    #[test]
    fn cleanup_component_distance_check() {
        let cleanup = CleanupComponent::with_distance(10.0);
        let entity_pos = Position3D::new(0, 0, 0);
        let near_player = Position3D::new(5, 5, 0); // Distance = sqrt(50) ≈ 7.07
        let far_player = Position3D::new(10, 10, 0); // Distance = sqrt(200) ≈ 14.14

        assert!(!cleanup.should_cleanup(entity_pos, near_player));
        assert!(cleanup.should_cleanup(entity_pos, far_player));
    }

    #[test]
    fn animation_component_creation() {
        let mut anim = AnimationComponent::new("idle".to_string(), 0.1);
        assert_eq!(anim.current_animation(), "idle");
        assert!(anim.is_playing());

        anim.set_animation("walk".to_string());
        assert_eq!(anim.current_animation(), "walk");
    }

    #[test]
    fn dice_roll_component_fade_out() {
        let dice_roll = DiceRoll::default();
        let mut component = DiceRollComponent::new(dice_roll, 1.0);

        assert!(!component.should_fade_out());
        assert_eq!(component.display_time(), 1.0);

        component.update(0.5);
        assert_eq!(component.display_time(), 0.5);
        assert!(!component.should_fade_out());

        component.update(0.6);
        assert!(component.should_fade_out());
    }
}
