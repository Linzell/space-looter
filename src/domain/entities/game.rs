//! Game Session Entity - Game State Management
//!
//! Represents a game session with score tracking, timing, and state management.

use crate::domain::{DomainError, DomainResult, Score};
use std::time::{Duration, Instant};

/// Unique identifier for a game session
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameSessionId(String);

impl GameSessionId {
    pub fn new(id: String) -> DomainResult<Self> {
        if id.is_empty() {
            return Err(DomainError::GameSessionError(
                "Game session ID cannot be empty".to_string(),
            ));
        }
        Ok(GameSessionId(id))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

/// Game session states
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameSessionState {
    Created,
    Active,
    Paused,
    Ended,
}

/// Game session entity managing overall game state
#[derive(Debug, Clone, PartialEq)]
pub struct GameSession {
    id: GameSessionId,
    score: Score,
    state: GameSessionState,
    start_time: Option<Instant>,
    pause_time: Option<Instant>,
    total_paused_duration: Duration,
}

impl GameSession {
    /// Create a new game session
    pub fn new(id: String) -> DomainResult<Self> {
        let session_id = GameSessionId::new(id)?;

        Ok(GameSession {
            id: session_id,
            score: Score::zero(),
            state: GameSessionState::Created,
            start_time: None,
            pause_time: None,
            total_paused_duration: Duration::new(0, 0),
        })
    }

    /// Get session ID
    pub fn id(&self) -> &GameSessionId {
        &self.id
    }

    /// Get current score
    pub fn score(&self) -> &Score {
        &self.score
    }

    /// Get current state
    pub fn state(&self) -> &GameSessionState {
        &self.state
    }

    /// Start the game session
    pub fn start(&mut self) -> DomainResult<()> {
        match self.state {
            GameSessionState::Created => {
                self.state = GameSessionState::Active;
                self.start_time = Some(Instant::now());
                Ok(())
            }
            _ => Err(DomainError::GameSessionError(
                "Game can only be started from Created state".to_string(),
            )),
        }
    }

    /// Pause the game session
    pub fn pause(&mut self) -> DomainResult<()> {
        match self.state {
            GameSessionState::Active => {
                self.state = GameSessionState::Paused;
                self.pause_time = Some(Instant::now());
                Ok(())
            }
            _ => Err(DomainError::GameSessionError(
                "Game can only be paused when active".to_string(),
            )),
        }
    }

    /// Resume the game session
    pub fn resume(&mut self) -> DomainResult<()> {
        match self.state {
            GameSessionState::Paused => {
                self.state = GameSessionState::Active;
                if let Some(pause_start) = self.pause_time.take() {
                    self.total_paused_duration += pause_start.elapsed();
                }
                Ok(())
            }
            _ => Err(DomainError::GameSessionError(
                "Game can only be resumed when paused".to_string(),
            )),
        }
    }

    /// End the game session
    pub fn end(&mut self) -> DomainResult<()> {
        match self.state {
            GameSessionState::Active | GameSessionState::Paused => {
                self.state = GameSessionState::Ended;
                if let Some(pause_start) = self.pause_time.take() {
                    self.total_paused_duration += pause_start.elapsed();
                }
                Ok(())
            }
            _ => Err(DomainError::GameSessionError(
                "Game can only be ended when active or paused".to_string(),
            )),
        }
    }

    /// Update the score
    pub fn update_score(&mut self, new_score: Score) -> DomainResult<()> {
        if self.state != GameSessionState::Active {
            return Err(DomainError::GameSessionError(
                "Score can only be updated when game is active".to_string(),
            ));
        }

        self.score = new_score;
        Ok(())
    }

    /// Add points to the score
    pub fn add_points(&mut self, points: u32) -> DomainResult<()> {
        let new_score = self.score.add(points)?;
        self.update_score(new_score)
    }

    /// Get elapsed play time (excluding pauses)
    pub fn elapsed_play_time(&self) -> Duration {
        match (self.start_time, &self.state) {
            (Some(start), GameSessionState::Active) => start.elapsed() - self.total_paused_duration,
            (Some(start), GameSessionState::Paused) => {
                if let Some(pause_start) = self.pause_time {
                    start.elapsed() - self.total_paused_duration - pause_start.elapsed()
                } else {
                    start.elapsed() - self.total_paused_duration
                }
            }
            (Some(start), GameSessionState::Ended) => start.elapsed() - self.total_paused_duration,
            _ => Duration::new(0, 0),
        }
    }

    /// Check if game is active
    pub fn is_active(&self) -> bool {
        self.state == GameSessionState::Active
    }

    /// Check if game is paused
    pub fn is_paused(&self) -> bool {
        self.state == GameSessionState::Paused
    }

    /// Check if game has ended
    pub fn is_ended(&self) -> bool {
        self.state == GameSessionState::Ended
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_session_creation() {
        let session = GameSession::new("session_1".to_string()).unwrap();
        assert_eq!(session.id().value(), "session_1");
        assert_eq!(session.score(), &Score::zero());
        assert_eq!(session.state(), &GameSessionState::Created);
    }

    #[test]
    fn game_session_lifecycle() {
        let mut session = GameSession::new("test".to_string()).unwrap();

        assert!(session.start().is_ok());
        assert!(session.is_active());

        assert!(session.pause().is_ok());
        assert!(session.is_paused());

        assert!(session.resume().is_ok());
        assert!(session.is_active());

        assert!(session.end().is_ok());
        assert!(session.is_ended());
    }

    #[test]
    fn game_session_score_update() {
        let mut session = GameSession::new("test".to_string()).unwrap();
        session.start().unwrap();

        assert!(session.add_points(100).is_ok());
        assert_eq!(session.score().value(), 100);
    }

    #[test]
    fn game_session_invalid_transitions() {
        let mut session = GameSession::new("test".to_string()).unwrap();

        assert!(session.pause().is_err()); // Can't pause before starting
        assert!(session.resume().is_err()); // Can't resume before pausing
    }
}
