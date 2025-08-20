//! Update Score Use Case - Score Management Logic
//!
//! Handles score update requests with business rule validation
//! and coordinates with domain entities and services.

use crate::application::{dto::UpdateScoreInput, dto::UpdateScoreOutput, ApplicationResult};
use crate::domain::Score;

/// Use case for handling score update operations
pub struct UpdateScoreUseCase;

impl UpdateScoreUseCase {
    /// Create a new update score use case
    pub fn new() -> Self {
        Self
    }

    /// Execute score update with business rules
    pub fn execute(&self, input: UpdateScoreInput) -> ApplicationResult<UpdateScoreOutput> {
        // Get current score (in real implementation, this would come from game session)
        let old_score = Score::zero(); // Placeholder

        // Add points to current score
        let new_score = old_score
            .add(input.points_to_add)
            .map_err(|e| crate::application::ApplicationError::DomainError(e))?;

        Ok(UpdateScoreOutput {
            session_id: input.session_id,
            old_score,
            new_score,
        })
    }
}

impl Default for UpdateScoreUseCase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_score_use_case_creation() {
        let use_case = UpdateScoreUseCase::new();
        let _ = use_case;
    }

    #[test]
    fn update_score_execution() {
        let use_case = UpdateScoreUseCase::new();
        let input = UpdateScoreInput {
            session_id: "test_session".to_string(),
            points_to_add: 100,
        };

        let result = use_case.execute(input);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.old_score.value(), 0);
        assert_eq!(output.new_score.value(), 100);
    }
}
