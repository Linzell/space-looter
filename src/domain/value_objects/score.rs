//! Score Value Object
//!
//! Represents a player's score in the game with validation and business rules.

use crate::domain::{DomainError, DomainResult};

/// Immutable score value with business rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Score {
    value: u32,
}

impl Score {
    /// Maximum allowed score value
    pub const MAX_SCORE: u32 = 999_999;

    /// Create a new score with validation
    pub fn new(value: u32) -> DomainResult<Self> {
        if value > Self::MAX_SCORE {
            return Err(DomainError::InvalidScore(format!(
                "Score {} exceeds maximum allowed score of {}",
                value,
                Self::MAX_SCORE
            )));
        }

        Ok(Self { value })
    }

    /// Create a zero score (starting score)
    pub fn zero() -> Self {
        Self { value: 0 }
    }

    /// Get the score value
    pub fn value(&self) -> u32 {
        self.value
    }

    /// Add points to the score
    pub fn add(&self, points: u32) -> DomainResult<Score> {
        let new_value = self.value.saturating_add(points).min(Self::MAX_SCORE);
        Score::new(new_value)
    }

    /// Add points from enemy collision
    pub fn add_enemy_points(&self) -> DomainResult<Score> {
        self.add(crate::domain::constants::POINTS_PER_ENEMY)
    }

    /// Check if this is a new high score compared to another score
    pub fn is_higher_than(&self, other: &Score) -> bool {
        self.value > other.value
    }

    /// Check if score has reached maximum
    pub fn is_max(&self) -> bool {
        self.value == Self::MAX_SCORE
    }

    /// Get score as formatted string with thousands separators
    pub fn formatted(&self) -> String {
        format_score(self.value)
    }

    /// Calculate percentage of max score achieved
    pub fn percentage_of_max(&self) -> f32 {
        (self.value as f32 / Self::MAX_SCORE as f32) * 100.0
    }
}

impl Default for Score {
    fn default() -> Self {
        Self::zero()
    }
}

impl std::fmt::Display for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.formatted())
    }
}

impl From<u32> for Score {
    fn from(value: u32) -> Self {
        Score::new(value).unwrap_or_else(|_| Score::new(Self::MAX_SCORE).unwrap())
    }
}

/// Format score with thousands separators
fn format_score(score: u32) -> String {
    let score_str = score.to_string();
    let mut result = String::new();

    for (i, char) in score_str.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(char);
    }

    result.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_score_valid_value() {
        let score = Score::new(1000).unwrap();
        assert_eq!(score.value(), 1000);
    }

    #[test]
    fn new_score_max_value() {
        let score = Score::new(Score::MAX_SCORE).unwrap();
        assert_eq!(score.value(), Score::MAX_SCORE);
        assert!(score.is_max());
    }

    #[test]
    fn new_score_invalid_value() {
        let result = Score::new(Score::MAX_SCORE + 1);
        assert!(result.is_err());

        if let Err(DomainError::InvalidScore(msg)) = result {
            assert!(msg.contains("exceeds maximum"));
        }
    }

    #[test]
    fn score_zero() {
        let score = Score::zero();
        assert_eq!(score.value(), 0);
    }

    #[test]
    fn score_add_points() {
        let score = Score::new(100).unwrap();
        let new_score = score.add(50).unwrap();
        assert_eq!(new_score.value(), 150);
    }

    #[test]
    fn score_add_enemy_points() {
        let score = Score::zero();
        let new_score = score.add_enemy_points().unwrap();
        assert_eq!(
            new_score.value(),
            crate::domain::constants::POINTS_PER_ENEMY
        );
    }

    #[test]
    fn score_add_overflow_protection() {
        let score = Score::new(Score::MAX_SCORE - 5).unwrap();
        let new_score = score.add(10).unwrap();
        assert_eq!(new_score.value(), Score::MAX_SCORE);
        assert!(new_score.is_max());
    }

    #[test]
    fn score_comparison() {
        let score1 = Score::new(100).unwrap();
        let score2 = Score::new(200).unwrap();
        let score3 = Score::new(100).unwrap();

        assert!(score2.is_higher_than(&score1));
        assert!(!score1.is_higher_than(&score2));
        assert!(!score1.is_higher_than(&score3));
    }

    #[test]
    fn score_ordering() {
        let score1 = Score::new(100).unwrap();
        let score2 = Score::new(200).unwrap();
        let score3 = Score::new(300).unwrap();

        assert!(score1 < score2);
        assert!(score2 < score3);
        assert!(score3 > score1);
    }

    #[test]
    fn score_equality() {
        let score1 = Score::new(500).unwrap();
        let score2 = Score::new(500).unwrap();
        let score3 = Score::new(600).unwrap();

        assert_eq!(score1, score2);
        assert_ne!(score1, score3);
    }

    #[test]
    fn score_percentage_of_max() {
        let half_max = Score::new(Score::MAX_SCORE / 2).unwrap();
        assert!((half_max.percentage_of_max() - 50.0).abs() < 0.001);

        let quarter_max = Score::new(Score::MAX_SCORE / 4).unwrap();
        assert!((quarter_max.percentage_of_max() - 25.0).abs() < 0.001);
    }

    #[test]
    fn score_formatting() {
        assert_eq!(format_score(0), "0");
        assert_eq!(format_score(123), "123");
        assert_eq!(format_score(1234), "1,234");
        assert_eq!(format_score(12345), "12,345");
        assert_eq!(format_score(123456), "123,456");
        assert_eq!(format_score(1234567), "1,234,567");
    }

    #[test]
    fn score_display() {
        let score = Score::new(12345).unwrap();
        assert_eq!(score.to_string(), "12,345");
    }

    #[test]
    fn score_from_u32() {
        let score = Score::from(1000);
        assert_eq!(score.value(), 1000);

        // Test overflow protection
        let max_score = Score::from(Score::MAX_SCORE + 1000);
        assert_eq!(max_score.value(), Score::MAX_SCORE);
    }

    #[test]
    fn score_default() {
        let score = Score::default();
        assert_eq!(score.value(), 0);
    }
}
