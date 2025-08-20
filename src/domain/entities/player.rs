//! Player Entity - Core Player Domain Logic
//!
//! Represents a player in the Space Looter game with movement capabilities,
//! health tracking, and business rules for player behavior.

use crate::domain::{DomainError, DomainResult, Position, Velocity};

/// Unique identifier for a player
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlayerId(String);

impl PlayerId {
    pub fn new(id: String) -> DomainResult<Self> {
        if id.is_empty() {
            return Err(DomainError::PlayerError(
                "Player ID cannot be empty".to_string(),
            ));
        }
        Ok(PlayerId(id))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

/// Player entity with movement and game state
#[derive(Debug, Clone, PartialEq)]
pub struct Player {
    id: PlayerId,
    position: Position,
    velocity: Velocity,
    speed: f32,
    health: u32,
    max_health: u32,
}

impl Player {
    /// Maximum health for a player
    pub const MAX_HEALTH: u32 = 100;

    /// Minimum speed for a player
    pub const MIN_SPEED: f32 = 50.0;

    /// Maximum speed for a player
    pub const MAX_SPEED: f32 = 500.0;

    /// Create a new player with validation
    pub fn new(
        id: String,
        position: Position,
        velocity: Velocity,
        speed: f32,
    ) -> DomainResult<Self> {
        let player_id = PlayerId::new(id)?;

        if speed < Self::MIN_SPEED || speed > Self::MAX_SPEED {
            return Err(DomainError::PlayerError(format!(
                "Speed {} must be between {} and {}",
                speed,
                Self::MIN_SPEED,
                Self::MAX_SPEED
            )));
        }

        Ok(Player {
            id: player_id,
            position,
            velocity,
            speed,
            health: Self::MAX_HEALTH,
            max_health: Self::MAX_HEALTH,
        })
    }

    /// Get player ID
    pub fn id(&self) -> &PlayerId {
        &self.id
    }

    /// Get current position
    pub fn position(&self) -> &Position {
        &self.position
    }

    /// Get current velocity
    pub fn velocity(&self) -> &Velocity {
        &self.velocity
    }

    /// Get movement speed
    pub fn speed(&self) -> f32 {
        self.speed
    }

    /// Get current health
    pub fn health(&self) -> u32 {
        self.health
    }

    /// Get maximum health
    pub fn max_health(&self) -> u32 {
        self.max_health
    }

    /// Check if player is alive
    pub fn is_alive(&self) -> bool {
        self.health > 0
    }

    /// Check if player is at full health
    pub fn is_full_health(&self) -> bool {
        self.health == self.max_health
    }

    /// Move player to new position
    pub fn move_to(&mut self, new_position: Position) -> DomainResult<()> {
        if !self.is_alive() {
            return Err(DomainError::PlayerError(
                "Dead player cannot move".to_string(),
            ));
        }

        self.position = new_position;
        Ok(())
    }

    /// Set player velocity
    pub fn set_velocity(&mut self, new_velocity: Velocity) -> DomainResult<()> {
        if !self.is_alive() {
            return Err(DomainError::PlayerError(
                "Dead player cannot change velocity".to_string(),
            ));
        }

        self.velocity = new_velocity;
        Ok(())
    }

    /// Update position based on current velocity and delta time
    pub fn update_position(&mut self, delta_time: f32) -> DomainResult<()> {
        if !self.is_alive() {
            return Err(DomainError::PlayerError(
                "Dead player cannot move".to_string(),
            ));
        }

        let new_position = self.position.move_by_velocity(&self.velocity, delta_time)?;
        self.position = new_position;
        Ok(())
    }

    /// Take damage
    pub fn take_damage(&mut self, damage: u32) -> DomainResult<bool> {
        if damage == 0 {
            return Ok(false);
        }

        let was_alive = self.is_alive();
        self.health = self.health.saturating_sub(damage);
        let is_now_dead = !self.is_alive();

        Ok(was_alive && is_now_dead) // Returns true if player just died
    }

    /// Heal player
    pub fn heal(&mut self, amount: u32) -> DomainResult<()> {
        if amount == 0 {
            return Ok(());
        }

        self.health = (self.health + amount).min(self.max_health);
        Ok(())
    }

    /// Set movement speed
    pub fn set_speed(&mut self, new_speed: f32) -> DomainResult<()> {
        if new_speed < Self::MIN_SPEED || new_speed > Self::MAX_SPEED {
            return Err(DomainError::PlayerError(format!(
                "Speed {} must be between {} and {}",
                new_speed,
                Self::MIN_SPEED,
                Self::MAX_SPEED
            )));
        }

        self.speed = new_speed;
        Ok(())
    }

    /// Calculate movement velocity from input direction
    pub fn calculate_movement_velocity(
        &self,
        direction_x: f32,
        direction_y: f32,
    ) -> DomainResult<Velocity> {
        Velocity::from_direction(direction_x, direction_y, self.speed)
    }

    /// Respawn player at position with full health
    pub fn respawn(&mut self, spawn_position: Position) -> DomainResult<()> {
        self.position = spawn_position;
        self.velocity = Velocity::zero();
        self.health = self.max_health;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_player() -> Player {
        let position = Position::new(0.0, 0.0).unwrap();
        let velocity = Velocity::zero();
        Player::new("test_player".to_string(), position, velocity, 200.0).unwrap()
    }

    #[test]
    fn player_id_creation() {
        let id = PlayerId::new("player_1".to_string()).unwrap();
        assert_eq!(id.value(), "player_1");

        assert!(PlayerId::new("".to_string()).is_err());
    }

    #[test]
    fn player_creation_valid() {
        let position = Position::new(10.0, 20.0).unwrap();
        let velocity = Velocity::new(1.0, -1.0).unwrap();
        let player = Player::new("player_1".to_string(), position, velocity, 150.0).unwrap();

        assert_eq!(player.id().value(), "player_1");
        assert_eq!(player.position(), &position);
        assert_eq!(player.velocity(), &velocity);
        assert_eq!(player.speed(), 150.0);
        assert_eq!(player.health(), Player::MAX_HEALTH);
        assert!(player.is_alive());
    }

    #[test]
    fn player_creation_invalid_speed() {
        let position = Position::origin();
        let velocity = Velocity::zero();

        assert!(Player::new("player".to_string(), position, velocity, 0.0).is_err());
        assert!(Player::new("player".to_string(), position, velocity, 1000.0).is_err());
    }

    #[test]
    fn player_movement() {
        let mut player = create_test_player();
        let new_position = Position::new(10.0, 20.0).unwrap();

        assert!(player.move_to(new_position).is_ok());
        assert_eq!(player.position(), &new_position);
    }

    #[test]
    fn player_velocity_change() {
        let mut player = create_test_player();
        let new_velocity = Velocity::new(5.0, -3.0).unwrap();

        assert!(player.set_velocity(new_velocity).is_ok());
        assert_eq!(player.velocity(), &new_velocity);
    }

    #[test]
    fn player_position_update() {
        let mut player = create_test_player();
        let velocity = Velocity::new(10.0, 5.0).unwrap();
        player.set_velocity(velocity).unwrap();

        assert!(player.update_position(1.0).is_ok());
        assert_eq!(player.position().x(), 10.0);
        assert_eq!(player.position().y(), 5.0);
    }

    #[test]
    fn player_health_system() {
        let mut player = create_test_player();

        assert!(player.is_alive());
        assert!(player.is_full_health());

        let died = player.take_damage(50).unwrap();
        assert!(!died);
        assert_eq!(player.health(), 50);
        assert!(!player.is_full_health());

        player.heal(25).unwrap();
        assert_eq!(player.health(), 75);

        let died = player.take_damage(75).unwrap();
        assert!(died);
        assert!(!player.is_alive());
    }

    #[test]
    fn dead_player_restrictions() {
        let mut player = create_test_player();
        player.take_damage(Player::MAX_HEALTH).unwrap();

        assert!(player.move_to(Position::origin()).is_err());
        assert!(player.set_velocity(Velocity::zero()).is_err());
        assert!(player.update_position(1.0).is_err());
    }

    #[test]
    fn player_speed_limits() {
        let mut player = create_test_player();

        assert!(player.set_speed(Player::MIN_SPEED).is_ok());
        assert!(player.set_speed(Player::MAX_SPEED).is_ok());
        assert!(player.set_speed(Player::MIN_SPEED - 1.0).is_err());
        assert!(player.set_speed(Player::MAX_SPEED + 1.0).is_err());
    }

    #[test]
    fn player_movement_calculation() {
        let player = create_test_player();

        let velocity = player.calculate_movement_velocity(1.0, 0.0).unwrap();
        assert!((velocity.magnitude() - player.speed()).abs() < f32::EPSILON);

        let velocity = player.calculate_movement_velocity(1.0, 1.0).unwrap();
        assert!((velocity.magnitude() - player.speed()).abs() < f32::EPSILON);
    }

    #[test]
    fn player_respawn() {
        let mut player = create_test_player();
        let spawn_pos = Position::new(100.0, 200.0).unwrap();

        player.take_damage(50).unwrap();
        player
            .set_velocity(Velocity::new(10.0, 5.0).unwrap())
            .unwrap();

        player.respawn(spawn_pos).unwrap();

        assert_eq!(player.position(), &spawn_pos);
        assert_eq!(player.velocity(), &Velocity::zero());
        assert_eq!(player.health(), Player::MAX_HEALTH);
    }
}
