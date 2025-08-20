//! Velocity Value Object
//!
//! Represents a 2D movement vector in the game world.

use crate::domain::{DomainError, DomainResult};

/// Immutable 2D velocity vector
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Velocity {
    dx: f32,
    dy: f32,
}

impl Velocity {
    /// Create a new velocity with validation
    pub fn new(dx: f32, dy: f32) -> DomainResult<Self> {
        if !dx.is_finite() || !dy.is_finite() {
            return Err(DomainError::InvalidVelocity(dx, dy));
        }

        Ok(Self { dx, dy })
    }

    /// Create zero velocity (no movement)
    pub fn zero() -> Self {
        Self { dx: 0.0, dy: 0.0 }
    }

    /// Get horizontal component
    pub fn dx(&self) -> f32 {
        self.dx
    }

    /// Get vertical component
    pub fn dy(&self) -> f32 {
        self.dy
    }

    /// Calculate the magnitude (speed) of this velocity
    pub fn magnitude(&self) -> f32 {
        (self.dx * self.dx + self.dy * self.dy).sqrt()
    }

    /// Normalize velocity to unit vector (magnitude = 1)
    pub fn normalize(&self) -> DomainResult<Velocity> {
        let mag = self.magnitude();
        if mag == 0.0 {
            return Ok(Velocity::zero());
        }
        Velocity::new(self.dx / mag, self.dy / mag)
    }

    /// Scale velocity by a factor
    pub fn scale(&self, factor: f32) -> DomainResult<Velocity> {
        if !factor.is_finite() {
            return Err(DomainError::InvalidVelocity(
                self.dx * factor,
                self.dy * factor,
            ));
        }
        Velocity::new(self.dx * factor, self.dy * factor)
    }

    /// Create velocity from direction and speed
    pub fn from_direction(dx: f32, dy: f32, speed: f32) -> DomainResult<Velocity> {
        let temp = Velocity::new(dx, dy)?;
        let normalized = temp.normalize()?;
        normalized.scale(speed)
    }

    /// Check if velocity is zero (no movement)
    pub fn is_zero(&self) -> bool {
        self.dx == 0.0 && self.dy == 0.0
    }

    /// Add another velocity to this one
    pub fn add(&self, other: &Velocity) -> DomainResult<Velocity> {
        Velocity::new(self.dx + other.dx, self.dy + other.dy)
    }
}

impl Default for Velocity {
    fn default() -> Self {
        Self::zero()
    }
}

impl std::fmt::Display for Velocity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Velocity({:.2}, {:.2})", self.dx, self.dy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_velocity_valid_components() {
        let vel = Velocity::new(5.0, -3.0).unwrap();
        assert_eq!(vel.dx(), 5.0);
        assert_eq!(vel.dy(), -3.0);
    }

    #[test]
    fn new_velocity_invalid_components() {
        assert!(Velocity::new(f32::NAN, 0.0).is_err());
        assert!(Velocity::new(0.0, f32::INFINITY).is_err());
        assert!(Velocity::new(f32::NEG_INFINITY, 0.0).is_err());
    }

    #[test]
    fn velocity_magnitude() {
        let vel = Velocity::new(3.0, 4.0).unwrap();
        assert_eq!(vel.magnitude(), 5.0);

        let zero_vel = Velocity::zero();
        assert_eq!(zero_vel.magnitude(), 0.0);
    }

    #[test]
    fn velocity_normalize() {
        let vel = Velocity::new(3.0, 4.0).unwrap();
        let normalized = vel.normalize().unwrap();
        assert!((normalized.magnitude() - 1.0).abs() < f32::EPSILON);
        assert_eq!(normalized.dx(), 0.6);
        assert_eq!(normalized.dy(), 0.8);
    }

    #[test]
    fn velocity_normalize_zero() {
        let zero_vel = Velocity::zero();
        let normalized = zero_vel.normalize().unwrap();
        assert!(normalized.is_zero());
    }

    #[test]
    fn velocity_scale() {
        let vel = Velocity::new(2.0, 3.0).unwrap();
        let scaled = vel.scale(2.5).unwrap();
        assert_eq!(scaled.dx(), 5.0);
        assert_eq!(scaled.dy(), 7.5);
    }

    #[test]
    fn velocity_from_direction() {
        let vel = Velocity::from_direction(1.0, 1.0, 10.0).unwrap();
        let expected_component = 10.0 / (2.0_f32).sqrt();
        assert!((vel.dx() - expected_component).abs() < f32::EPSILON);
        assert!((vel.dy() - expected_component).abs() < f32::EPSILON);
        assert!((vel.magnitude() - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn velocity_is_zero() {
        assert!(Velocity::zero().is_zero());
        assert!(!Velocity::new(0.1, 0.0).unwrap().is_zero());
        assert!(!Velocity::new(0.0, 0.1).unwrap().is_zero());
    }

    #[test]
    fn velocity_add() {
        let vel1 = Velocity::new(2.0, 3.0).unwrap();
        let vel2 = Velocity::new(1.0, -1.0).unwrap();
        let result = vel1.add(&vel2).unwrap();
        assert_eq!(result.dx(), 3.0);
        assert_eq!(result.dy(), 2.0);
    }

    #[test]
    fn velocity_equality() {
        let vel1 = Velocity::new(1.5, 2.5).unwrap();
        let vel2 = Velocity::new(1.5, 2.5).unwrap();
        let vel3 = Velocity::new(1.5, 3.0).unwrap();

        assert_eq!(vel1, vel2);
        assert_ne!(vel1, vel3);
    }

    #[test]
    fn velocity_display() {
        let vel = Velocity::new(12.34, -56.78).unwrap();
        assert_eq!(vel.to_string(), "Velocity(12.34, -56.78)");
    }
}
