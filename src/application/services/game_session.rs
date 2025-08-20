//! Game Session Service - Application-Level Game State Management
//!
//! Coordinates game session operations, manages application state,
//! and orchestrates between use cases for game lifecycle management.

use crate::application::{ApplicationError, ApplicationResult};
use crate::domain::{GameSession, Score};
use std::collections::HashMap;

/// Service for managing game sessions at the application level
pub struct GameSessionService {
    active_sessions: HashMap<String, GameSession>,
}

impl GameSessionService {
    /// Create a new game session service
    pub fn new() -> Self {
        Self {
            active_sessions: HashMap::new(),
        }
    }

    /// Create a new game session
    pub fn create_session(&mut self, session_id: String) -> ApplicationResult<String> {
        if self.active_sessions.contains_key(&session_id) {
            return Err(ApplicationError::InvalidSession(
                "Session ID already exists".to_string(),
            ));
        }

        let session =
            GameSession::new(session_id.clone()).map_err(|e| ApplicationError::DomainError(e))?;

        self.active_sessions.insert(session_id.clone(), session);
        Ok(session_id)
    }

    /// Start a game session
    pub fn start_session(&mut self, session_id: &str) -> ApplicationResult<()> {
        let session = self
            .active_sessions
            .get_mut(session_id)
            .ok_or_else(|| ApplicationError::InvalidSession("Session not found".to_string()))?;

        session
            .start()
            .map_err(|e| ApplicationError::DomainError(e))?;

        Ok(())
    }

    /// Pause a game session
    pub fn pause_session(&mut self, session_id: &str) -> ApplicationResult<()> {
        let session = self
            .active_sessions
            .get_mut(session_id)
            .ok_or_else(|| ApplicationError::InvalidSession("Session not found".to_string()))?;

        session
            .pause()
            .map_err(|e| ApplicationError::DomainError(e))?;

        Ok(())
    }

    /// Resume a game session
    pub fn resume_session(&mut self, session_id: &str) -> ApplicationResult<()> {
        let session = self
            .active_sessions
            .get_mut(session_id)
            .ok_or_else(|| ApplicationError::InvalidSession("Session not found".to_string()))?;

        session
            .resume()
            .map_err(|e| ApplicationError::DomainError(e))?;

        Ok(())
    }

    /// End a game session
    pub fn end_session(&mut self, session_id: &str) -> ApplicationResult<Score> {
        let session = self
            .active_sessions
            .get_mut(session_id)
            .ok_or_else(|| ApplicationError::InvalidSession("Session not found".to_string()))?;

        session
            .end()
            .map_err(|e| ApplicationError::DomainError(e))?;

        let final_score = *session.score();
        Ok(final_score)
    }

    /// Update score for a session
    pub fn update_session_score(
        &mut self,
        session_id: &str,
        points_to_add: u32,
    ) -> ApplicationResult<Score> {
        let session = self
            .active_sessions
            .get_mut(session_id)
            .ok_or_else(|| ApplicationError::InvalidSession("Session not found".to_string()))?;

        session
            .add_points(points_to_add)
            .map_err(|e| ApplicationError::DomainError(e))?;

        Ok(*session.score())
    }

    /// Get current score for a session
    pub fn get_session_score(&self, session_id: &str) -> ApplicationResult<Score> {
        let session = self
            .active_sessions
            .get(session_id)
            .ok_or_else(|| ApplicationError::InvalidSession("Session not found".to_string()))?;

        Ok(*session.score())
    }

    /// Check if session is active
    pub fn is_session_active(&self, session_id: &str) -> bool {
        self.active_sessions
            .get(session_id)
            .map(|s| s.is_active())
            .unwrap_or(false)
    }

    /// Remove a completed session
    pub fn remove_session(&mut self, session_id: &str) -> ApplicationResult<()> {
        if !self.active_sessions.contains_key(session_id) {
            return Err(ApplicationError::InvalidSession(
                "Session not found".to_string(),
            ));
        }

        self.active_sessions.remove(session_id);
        Ok(())
    }

    /// Get all active session IDs
    pub fn get_active_sessions(&self) -> Vec<String> {
        self.active_sessions.keys().cloned().collect()
    }
}

impl Default for GameSessionService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_session_service_creation() {
        let service = GameSessionService::new();
        assert!(service.get_active_sessions().is_empty());
    }

    #[test]
    fn create_and_manage_session() {
        let mut service = GameSessionService::new();

        // Create session
        let session_id = service.create_session("test_session".to_string()).unwrap();
        assert_eq!(session_id, "test_session");
        assert_eq!(service.get_active_sessions().len(), 1);

        // Start session
        assert!(service.start_session(&session_id).is_ok());
        assert!(service.is_session_active(&session_id));

        // Update score
        let new_score = service.update_session_score(&session_id, 100).unwrap();
        assert_eq!(new_score.value(), 100);

        // End session
        let final_score = service.end_session(&session_id).unwrap();
        assert_eq!(final_score.value(), 100);
    }

    #[test]
    fn session_lifecycle() {
        let mut service = GameSessionService::new();
        let session_id = service
            .create_session("lifecycle_test".to_string())
            .unwrap();

        // Start -> Pause -> Resume -> End
        assert!(service.start_session(&session_id).is_ok());
        assert!(service.pause_session(&session_id).is_ok());
        assert!(service.resume_session(&session_id).is_ok());
        assert!(service.end_session(&session_id).is_ok());
    }

    #[test]
    fn invalid_session_operations() {
        let mut service = GameSessionService::new();

        // Operations on non-existent session
        assert!(service.start_session("non_existent").is_err());
        assert!(service.get_session_score("non_existent").is_err());
        assert!(!service.is_session_active("non_existent"));
    }
}
