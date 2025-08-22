//! Dice mechanics for RPG actions and events
//!
//! This module implements the core dice rolling system that drives all
//! game mechanics. Every action in the game uses dice rolls with modifiers
//! to determine outcomes.

use crate::domain::{DomainError, DomainResult};
use std::fmt;

/// Types of dice available in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiceType {
    /// 4-sided die (1-4)
    D4,
    /// 6-sided die (1-6) - most common for movement and simple actions
    D6,
    /// 8-sided die (1-8)
    D8,
    /// 10-sided die (1-10) - used for resource gathering
    D10,
    /// 12-sided die (1-12)
    D12,
    /// 20-sided die (1-20) - used for major actions and skill checks
    D20,
    /// 100-sided die (1-100) - used for rare events and critical outcomes
    D100,
}

impl DiceType {
    /// Get the maximum value for this die type
    pub fn max_value(&self) -> u8 {
        match self {
            DiceType::D4 => 4,
            DiceType::D6 => 6,
            DiceType::D8 => 8,
            DiceType::D10 => 10,
            DiceType::D12 => 12,
            DiceType::D20 => 20,
            DiceType::D100 => 100,
        }
    }

    /// Get the number of sides on this die type
    pub fn sides(&self) -> u8 {
        self.max_value()
    }

    /// Get the minimum value this die can roll
    pub fn min_value(&self) -> u8 {
        1
    }

    /// Check if a value is valid for this die type
    pub fn is_valid_value(&self, value: u8) -> bool {
        value >= self.min_value() && value <= self.max_value()
    }

    /// Get the average roll for this die type
    pub fn average_roll(&self) -> f32 {
        (self.min_value() as f32 + self.max_value() as f32) / 2.0
    }

    /// Calculate probability of rolling at or above a target number
    pub fn probability_at_or_above(&self, target: u8) -> f32 {
        if target > self.max_value() {
            return 0.0;
        }
        if target <= self.min_value() {
            return 1.0;
        }

        let favorable_outcomes = self.max_value() - target + 1;
        favorable_outcomes as f32 / self.max_value() as f32
    }
}

impl Default for DiceType {
    fn default() -> Self {
        DiceType::D6
    }
}

impl fmt::Display for DiceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiceType::D4 => write!(f, "d4"),
            DiceType::D6 => write!(f, "d6"),
            DiceType::D8 => write!(f, "d8"),
            DiceType::D10 => write!(f, "d10"),
            DiceType::D12 => write!(f, "d12"),
            DiceType::D20 => write!(f, "d20"),
            DiceType::D100 => write!(f, "d100"),
        }
    }
}

/// A dice roll configuration with count and modifiers
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiceRoll {
    pub count: u8,              // Number of dice to roll
    pub dice_type: DiceType,    // Type of dice
    pub modifier: DiceModifier, // Modifier to add to the roll
}

impl DiceRoll {
    /// Create a new dice roll
    pub fn new(count: u8, dice_type: DiceType, modifier: DiceModifier) -> DomainResult<Self> {
        if count == 0 {
            return Err(DomainError::InvalidDiceRoll(
                "Dice count must be at least 1".to_string(),
            ));
        }

        if count > 10 {
            return Err(DomainError::InvalidDiceRoll(
                "Maximum 10 dice allowed per roll".to_string(),
            ));
        }

        Ok(Self {
            count,
            dice_type,
            modifier,
        })
    }

    /// Create a simple dice roll with no modifier
    pub fn simple(count: u8, dice_type: DiceType) -> DomainResult<Self> {
        Self::new(count, dice_type, DiceModifier::none())
    }

    /// Create a single die roll
    pub fn single(dice_type: DiceType) -> DomainResult<Self> {
        Self::simple(1, dice_type)
    }

    /// Get minimum possible result
    pub fn min_result(&self) -> i32 {
        (self.count as i32) + self.modifier.total_modifier()
    }

    /// Get maximum possible result
    pub fn max_result(&self) -> i32 {
        (self.count as i32 * self.dice_type.max_value() as i32) + self.modifier.total_modifier()
    }

    /// Get average expected result
    pub fn average_result(&self) -> f32 {
        (self.count as f32 * self.dice_type.average_roll()) + self.modifier.total_modifier() as f32
    }

    /// Calculate probability of achieving at least the target value
    pub fn probability_at_least(&self, target: i32) -> f32 {
        // Simplified calculation for single die with modifier
        if self.count == 1 {
            let adjusted_target = target - self.modifier.total_modifier();
            if adjusted_target <= 0 {
                return 1.0;
            }
            self.dice_type
                .probability_at_or_above(adjusted_target as u8)
        } else {
            // For multiple dice, this becomes complex combinatorics
            // For now, return a rough approximation
            let avg = self.average_result();
            if target as f32 <= avg {
                0.5 // Rough approximation
            } else {
                0.25
            }
        }
    }

    /// Check if this is a critical success roll (natural max on all dice)
    pub fn is_critical_success_possible(&self) -> bool {
        self.dice_type == DiceType::D20 || self.dice_type == DiceType::D100
    }

    /// Get the value that would be a critical success
    pub fn critical_success_value(&self) -> Option<u8> {
        if self.is_critical_success_possible() {
            Some(self.dice_type.max_value())
        } else {
            None
        }
    }

    /// Get the dice rolls (returns empty vec for now, actual rolling would happen in a service)
    pub fn rolls(&self) -> Vec<u8> {
        // This would normally contain the actual rolled values
        // For now, return empty vec as this is just the roll specification
        Vec::new()
    }

    /// Get the total result (for compatibility, returns average for now)
    pub fn total(&self) -> i32 {
        // This would normally contain the actual rolled total
        // For now, return the average as an approximation
        self.average_result() as i32
    }

    /// Create a DiceRoll from pre-rolled values (for backward compatibility)
    pub fn from_rolls(dice_type: DiceType, rolls: Vec<u8>) -> DomainResult<Self> {
        if rolls.is_empty() {
            return Err(DomainError::InvalidDiceRoll(
                "Cannot create dice roll with no rolls".to_string(),
            ));
        }

        let count = rolls.len() as u8;
        if count > 10 {
            return Err(DomainError::InvalidDiceRoll(
                "Maximum 10 dice allowed per roll".to_string(),
            ));
        }

        Ok(Self {
            count,
            dice_type,
            modifier: DiceModifier::none(),
        })
    }
}

impl Default for DiceRoll {
    fn default() -> Self {
        DiceRoll {
            count: 1,
            dice_type: DiceType::default(),
            modifier: DiceModifier::default(),
        }
    }
}

impl fmt::Display for DiceRoll {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.count == 1 {
            if self.modifier.is_zero() {
                write!(f, "{}", self.dice_type)
            } else {
                write!(f, "{}{}", self.dice_type, self.modifier)
            }
        } else if self.modifier.is_zero() {
            write!(f, "{}{}", self.count, self.dice_type)
        } else {
            write!(f, "{}{}{}", self.count, self.dice_type, self.modifier)
        }
    }
}

/// Modifiers that can be applied to dice rolls
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiceModifier {
    pub stat_modifier: i8,        // Bonus/penalty from player stats
    pub equipment_bonus: i8,      // Bonus from equipment
    pub situational_modifier: i8, // Bonus/penalty from circumstances
    pub luck_bonus: i8,           // Bonus from luck events or abilities
}

impl DiceModifier {
    /// Create a new dice modifier
    pub fn new(
        stat_modifier: i8,
        equipment_bonus: i8,
        situational_modifier: i8,
        luck_bonus: i8,
    ) -> DomainResult<Self> {
        // Validate modifier ranges to prevent extreme values
        for &modifier in &[
            stat_modifier,
            equipment_bonus,
            situational_modifier,
            luck_bonus,
        ] {
            if modifier < -10 || modifier > 10 {
                return Err(DomainError::DiceModifierError(
                    "Individual modifiers must be between -10 and +10".to_string(),
                ));
            }
        }

        let total = stat_modifier + equipment_bonus + situational_modifier + luck_bonus;
        if total < -20 || total > 20 {
            return Err(DomainError::DiceModifierError(
                "Total modifier must be between -20 and +20".to_string(),
            ));
        }

        Ok(Self {
            stat_modifier,
            equipment_bonus,
            situational_modifier,
            luck_bonus,
        })
    }

    /// Create a modifier with no bonuses or penalties
    pub fn none() -> Self {
        Self {
            stat_modifier: 0,
            equipment_bonus: 0,
            situational_modifier: 0,
            luck_bonus: 0,
        }
    }

    /// Create a modifier with only stat bonus
    pub fn from_stat(stat_modifier: i8) -> DomainResult<Self> {
        Self::new(stat_modifier, 0, 0, 0)
    }

    /// Get the total modifier value
    pub fn total_modifier(&self) -> i32 {
        (self.stat_modifier + self.equipment_bonus + self.situational_modifier + self.luck_bonus)
            as i32
    }

    /// Check if this modifier has any effect
    pub fn is_zero(&self) -> bool {
        self.total_modifier() == 0
    }

    /// Add another modifier to this one
    pub fn add(&self, other: &DiceModifier) -> DomainResult<Self> {
        Self::new(
            self.stat_modifier + other.stat_modifier,
            self.equipment_bonus + other.equipment_bonus,
            self.situational_modifier + other.situational_modifier,
            self.luck_bonus + other.luck_bonus,
        )
    }

    /// Create a situational modifier
    pub fn situational(modifier: i8) -> DomainResult<Self> {
        Self::new(0, 0, modifier, 0)
    }

    /// Create an equipment bonus
    pub fn equipment(bonus: i8) -> DomainResult<Self> {
        Self::new(0, bonus, 0, 0)
    }

    /// Create a luck bonus
    pub fn luck(bonus: i8) -> DomainResult<Self> {
        Self::new(0, 0, 0, bonus)
    }
}

impl Default for DiceModifier {
    fn default() -> Self {
        DiceModifier::none()
    }
}

impl fmt::Display for DiceModifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total = self.total_modifier();
        if total == 0 {
            Ok(())
        } else if total > 0 {
            write!(f, "+{}", total)
        } else {
            write!(f, "{}", total)
        }
    }
}

/// Result of a dice roll with detailed information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiceResult {
    pub roll: DiceRoll,
    pub raw_results: Vec<u8>, // Individual die results before modifier
    pub total_raw: u32,       // Sum of raw dice before modifier
    pub final_result: i32,    // Final result after applying modifier
    pub is_critical_success: bool,
    pub is_critical_failure: bool,
}

impl DiceResult {
    /// Create a new dice result
    pub fn new(
        roll: DiceRoll,
        raw_results: Vec<u8>,
        is_critical_success: bool,
        is_critical_failure: bool,
    ) -> DomainResult<Self> {
        if raw_results.len() != roll.count as usize {
            return Err(DomainError::InvalidDiceRoll(
                "Number of results must match dice count".to_string(),
            ));
        }

        // Validate each result is within valid range for the die type
        for &result in &raw_results {
            if !roll.dice_type.is_valid_value(result) {
                return Err(DomainError::InvalidDiceRoll(format!(
                    "Invalid die result {} for {}",
                    result, roll.dice_type
                )));
            }
        }

        let total_raw = raw_results.iter().map(|&x| x as u32).sum();
        let final_result = total_raw as i32 + roll.modifier.total_modifier();

        Ok(Self {
            roll,
            raw_results,
            total_raw,
            final_result,
            is_critical_success,
            is_critical_failure,
        })
    }

    /// Check if the result meets or exceeds a difficulty threshold
    pub fn meets_difficulty(&self, difficulty: u8) -> bool {
        self.final_result >= difficulty as i32
    }

    /// Get the margin of success/failure
    pub fn margin(&self, difficulty: u8) -> i32 {
        self.final_result - difficulty as i32
    }

    /// Check if this was a successful roll against difficulty
    pub fn is_success(&self, difficulty: u8) -> bool {
        self.meets_difficulty(difficulty)
    }

    /// Get success level based on margin
    pub fn success_level(&self, difficulty: u8) -> SuccessLevel {
        if self.is_critical_failure {
            return SuccessLevel::CriticalFailure;
        }
        if self.is_critical_success {
            return SuccessLevel::CriticalSuccess;
        }

        let margin = self.margin(difficulty);
        match margin {
            m if m >= 10 => SuccessLevel::ExceptionalSuccess,
            m if m >= 5 => SuccessLevel::GoodSuccess,
            m if m >= 0 => SuccessLevel::Success,
            m if m >= -5 => SuccessLevel::Failure,
            _ => SuccessLevel::CriticalFailure,
        }
    }

    /// Get a descriptive string for the roll result
    pub fn description(&self, difficulty: Option<u8>) -> String {
        match difficulty {
            Some(diff) => {
                let level = self.success_level(diff);
                format!("{} ({})", self.final_result, level)
            }
            None => self.final_result.to_string(),
        }
    }
}

impl Default for DiceResult {
    fn default() -> Self {
        DiceResult {
            roll: DiceRoll::default(),
            raw_results: Vec::new(),
            total_raw: 0,
            final_result: 0,
            is_critical_success: false,
            is_critical_failure: false,
        }
    }
}

impl fmt::Display for DiceResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.raw_results.len() == 1 {
            if self.roll.modifier.is_zero() {
                write!(f, "{}", self.final_result)
            } else {
                write!(
                    f,
                    "{}{}={}",
                    self.raw_results[0], self.roll.modifier, self.final_result
                )
            }
        } else {
            write!(
                f,
                "[{}]{}={}",
                self.raw_results
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
                self.roll.modifier,
                self.final_result
            )
        }
    }
}

/// Different levels of success/failure for dice rolls
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SuccessLevel {
    CriticalFailure,
    Failure,
    Success,
    GoodSuccess,
    ExceptionalSuccess,
    CriticalSuccess,
}

impl SuccessLevel {
    /// Check if this represents any kind of success
    pub fn is_success(&self) -> bool {
        matches!(
            self,
            SuccessLevel::Success
                | SuccessLevel::GoodSuccess
                | SuccessLevel::ExceptionalSuccess
                | SuccessLevel::CriticalSuccess
        )
    }

    /// Check if this is a critical result (either success or failure)
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            SuccessLevel::CriticalSuccess | SuccessLevel::CriticalFailure
        )
    }

    /// Get multiplier for rewards/consequences based on success level
    pub fn reward_multiplier(&self) -> f32 {
        match self {
            SuccessLevel::CriticalFailure => 0.0,
            SuccessLevel::Failure => 0.25,
            SuccessLevel::Success => 1.0,
            SuccessLevel::GoodSuccess => 1.25,
            SuccessLevel::ExceptionalSuccess => 1.5,
            SuccessLevel::CriticalSuccess => 2.0,
        }
    }
}

impl fmt::Display for SuccessLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SuccessLevel::CriticalFailure => write!(f, "Critical Failure"),
            SuccessLevel::Failure => write!(f, "Failure"),
            SuccessLevel::Success => write!(f, "Success"),
            SuccessLevel::GoodSuccess => write!(f, "Good Success"),
            SuccessLevel::ExceptionalSuccess => write!(f, "Exceptional Success"),
            SuccessLevel::CriticalSuccess => write!(f, "Critical Success"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dice_type_properties() {
        assert_eq!(DiceType::D6.max_value(), 6);
        assert_eq!(DiceType::D20.max_value(), 20);
        assert_eq!(DiceType::D6.min_value(), 1);
        assert!(DiceType::D6.is_valid_value(3));
        assert!(!DiceType::D6.is_valid_value(7));
    }

    #[test]
    fn dice_roll_creation() {
        let modifier = DiceModifier::none();
        let roll = DiceRoll::new(2, DiceType::D6, modifier).unwrap();
        assert_eq!(roll.count, 2);
        assert_eq!(roll.dice_type, DiceType::D6);
    }

    #[test]
    fn dice_roll_invalid_count() {
        let modifier = DiceModifier::none();
        assert!(DiceRoll::new(0, DiceType::D6, modifier.clone()).is_err());
        assert!(DiceRoll::new(11, DiceType::D6, modifier).is_err());
    }

    #[test]
    fn dice_roll_simple_creation() {
        let roll = DiceRoll::simple(1, DiceType::D20).unwrap();
        assert_eq!(roll.count, 1);
        assert_eq!(roll.dice_type, DiceType::D20);
        assert!(roll.modifier.is_zero());
    }

    #[test]
    fn dice_roll_results() {
        let roll = DiceRoll::simple(1, DiceType::D6).unwrap();
        assert_eq!(roll.min_result(), 1);
        assert_eq!(roll.max_result(), 6);
        assert_eq!(roll.average_result(), 3.5);
    }

    #[test]
    fn dice_modifier_creation() {
        let modifier = DiceModifier::new(2, 1, -1, 0).unwrap();
        assert_eq!(modifier.total_modifier(), 2);
        assert!(!modifier.is_zero());
    }

    #[test]
    fn dice_modifier_limits() {
        // Individual modifier too high
        assert!(DiceModifier::new(11, 0, 0, 0).is_err());

        // Total modifier too high
        assert!(DiceModifier::new(10, 10, 5, 0).is_err());
    }

    #[test]
    fn dice_result_creation() {
        let roll = DiceRoll::simple(2, DiceType::D6).unwrap();
        let result = DiceResult::new(roll, vec![3, 5], false, false).unwrap();
        assert_eq!(result.total_raw, 8);
        assert_eq!(result.final_result, 8);
    }

    #[test]
    fn dice_result_with_modifier() {
        let modifier = DiceModifier::from_stat(2).unwrap();
        let roll = DiceRoll::new(1, DiceType::D20, modifier).unwrap();
        let result = DiceResult::new(roll, vec![15], false, false).unwrap();
        assert_eq!(result.total_raw, 15);
        assert_eq!(result.final_result, 17);
    }

    #[test]
    fn success_level_classification() {
        let roll = DiceRoll::simple(1, DiceType::D20).unwrap();
        let result = DiceResult::new(roll, vec![15], false, false).unwrap();

        let level = result.success_level(10);
        assert_eq!(level, SuccessLevel::GoodSuccess);
        assert!(level.is_success());
        assert!(!level.is_critical());
    }

    #[test]
    fn critical_success_detection() {
        let roll = DiceRoll::simple(1, DiceType::D20).unwrap();
        let result = DiceResult::new(roll, vec![20], true, false).unwrap();

        let level = result.success_level(15);
        assert_eq!(level, SuccessLevel::CriticalSuccess);
        assert!(level.is_success());
        assert!(level.is_critical());
        assert_eq!(level.reward_multiplier(), 2.0);
    }

    #[test]
    fn dice_roll_display() {
        let roll = DiceRoll::simple(1, DiceType::D6).unwrap();
        assert_eq!(roll.to_string(), "d6");

        let modifier = DiceModifier::from_stat(3).unwrap();
        let roll_with_modifier = DiceRoll::new(1, DiceType::D20, modifier).unwrap();
        assert_eq!(roll_with_modifier.to_string(), "d20+3");

        let multiple_dice = DiceRoll::simple(3, DiceType::D6).unwrap();
        assert_eq!(multiple_dice.to_string(), "3d6");
    }

    #[test]
    fn dice_result_display() {
        let roll = DiceRoll::simple(1, DiceType::D6).unwrap();
        let result = DiceResult::new(roll, vec![4], false, false).unwrap();
        assert_eq!(result.to_string(), "4");

        let modifier = DiceModifier::from_stat(2).unwrap();
        let roll_with_modifier = DiceRoll::new(1, DiceType::D20, modifier).unwrap();
        let result_with_modifier =
            DiceResult::new(roll_with_modifier, vec![15], false, false).unwrap();
        assert_eq!(result_with_modifier.to_string(), "15+2=17");
    }
}
