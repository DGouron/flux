use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::domain::{Session, SessionId};

#[derive(Error, Debug)]
pub enum SessionRepositoryError {
    #[error("session introuvable: {id}")]
    NotFound { id: SessionId },

    #[error("impossible de supprimer une session active: {id}")]
    ActiveSession { id: SessionId },

    #[error("erreur de persistence: {message}")]
    Storage { message: String },
}

pub trait SessionRepository: Send + Sync {
    fn save(&self, session: &mut Session) -> Result<SessionId, SessionRepositoryError>;

    fn update(&self, session: &Session) -> Result<(), SessionRepositoryError>;

    fn find_by_id(&self, id: SessionId) -> Result<Session, SessionRepositoryError>;

    fn find_active(&self) -> Result<Option<Session>, SessionRepositoryError>;

    fn find_completed_since(
        &self,
        since: DateTime<Utc>,
    ) -> Result<Vec<Session>, SessionRepositoryError>;

    fn count_completed_sessions(&self) -> Result<u32, SessionRepositoryError>;

    fn clear_completed_sessions(&self) -> Result<u32, SessionRepositoryError>;

    fn has_active_session(&self) -> Result<bool, SessionRepositoryError>;

    fn delete_session(&self, id: SessionId) -> Result<(), SessionRepositoryError>;
}
