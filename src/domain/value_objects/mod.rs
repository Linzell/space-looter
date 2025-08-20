//! Value Objects - Immutable Domain Data Structures
//!
//! Value objects are immutable data structures that represent domain concepts.
//! They have no identity and are defined by their attributes.
//!
//! ## Characteristics
//! - Immutable after creation
//! - Equality based on value, not identity
//! - No side effects
//! - Self-validating

pub mod position;
pub mod score;
pub mod velocity;

pub use position::Position;
pub use score::Score;
pub use velocity::Velocity;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_objects_are_immutable() {
        let pos = Position::new(10.0, 20.0).unwrap();
        let vel = Velocity::new(1.0, -1.0).unwrap();
        let score = Score::new(100).unwrap();

        // These should compile and work correctly
        assert_eq!(pos.x(), 10.0);
        assert_eq!(vel.dx(), 1.0);
        assert_eq!(score.value(), 100);
    }

    #[test]
    fn value_objects_equality() {
        let pos1 = Position::new(5.0, 10.0).unwrap();
        let pos2 = Position::new(5.0, 10.0).unwrap();
        assert_eq!(pos1, pos2);

        let vel1 = Velocity::new(2.0, -1.0).unwrap();
        let vel2 = Velocity::new(2.0, -1.0).unwrap();
        assert_eq!(vel1, vel2);

        let score1 = Score::new(50).unwrap();
        let score2 = Score::new(50).unwrap();
        assert_eq!(score1, score2);
    }
}
