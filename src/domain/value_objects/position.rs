//! Position Value Object
//!
//! Represents a 2D coordinate in the game world.

use crate::domain::{DomainError, DomainResult};

/// Immutable 2D position coordinates
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    x: f32,
    y: f32,
}

impl Position {
    /// Create a new position with validation
    pub fn new(x: f32, y: f32) -> DomainResult<Self> {
        if !x.is_finite() || !y.is_finite() {
            return Err(DomainError::InvalidPosition(x, y));
        }

        Ok(Self { x, y })
    }

    /// Create position at origin (0, 0)
    pub fn origin() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// Get x coordinate
    pub fn x(&self) -> f32 {
        self.x
    }

    /// Get y coordinate
    pub fn y(&self) -> f32 {
        self.y
    }

    /// Calculate distance to another position
    pub fn distance_to(&self, other: &Position) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Move position by offset
    pub fn add(&self, dx: f32, dy: f32) -> DomainResult<Position> {
        Position::new(self.x + dx, self.y + dy)
    }

    /// Create a new position moved by velocity
    pub fn move_by_velocity(
        &self,
        velocity: &crate::domain::value_objects::Velocity,
        delta_time: f32,
    ) -> DomainResult<Position> {
        let dx = velocity.dx() * delta_time;
        let dy = velocity.dy() * delta_time;
        self.add(dx, dy)
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::origin()
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Position({:.2}, {:.2})", self.x, self.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_position_valid_coordinates() {
        let pos = Position::new(10.0, -5.0).unwrap();
        assert_eq!(pos.x(), 10.0);
        assert_eq!(pos.y(), -5.0);
    }

    #[test]
    fn new_position_invalid_coordinates() {
        assert!(Position::new(f32::NAN, 0.0).is_err());
        assert!(Position::new(0.0, f32::INFINITY).is_err());
        assert!(Position::new(f32::NEG_INFINITY, 0.0).is_err());
    }

    #[test]
    fn position_distance_calculation() {
        let pos1 = Position::new(0.0, 0.0).unwrap();
        let pos2 = Position::new(3.0, 4.0).unwrap();
        assert_eq!(pos1.distance_to(&pos2), 5.0);
    }

    #[test]
    fn position_add_offset() {
        let pos = Position::new(10.0, 20.0).unwrap();
        let new_pos = pos.add(5.0, -10.0).unwrap();
        assert_eq!(new_pos.x(), 15.0);
        assert_eq!(new_pos.y(), 10.0);
    }

    #[test]
    fn position_origin() {
        let origin = Position::origin();
        assert_eq!(origin.x(), 0.0);
        assert_eq!(origin.y(), 0.0);
    }

    #[test]
    fn position_equality() {
        let pos1 = Position::new(1.5, 2.5).unwrap();
        let pos2 = Position::new(1.5, 2.5).unwrap();
        let pos3 = Position::new(1.5, 3.0).unwrap();

        assert_eq!(pos1, pos2);
        assert_ne!(pos1, pos3);
    }

    #[test]
    fn position_display() {
        let pos = Position::new(12.34, -56.78).unwrap();
        assert_eq!(pos.to_string(), "Position(12.34, -56.78)");
    }
}
