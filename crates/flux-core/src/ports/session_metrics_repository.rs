use thiserror::Error;

use crate::{SessionId, SessionMetrics};

#[derive(Error, Debug)]
pub enum SessionMetricsRepositoryError {
    #[error("erreur de persistance: {0}")]
    Persistence(String),
}

pub trait SessionMetricsRepository: Send + Sync {
    fn save(&self, metrics: &SessionMetrics) -> Result<(), SessionMetricsRepositoryError>;

    fn find_by_session(
        &self,
        session_id: SessionId,
    ) -> Result<Option<SessionMetrics>, SessionMetricsRepositoryError>;

    fn find_by_sessions(
        &self,
        session_ids: &[SessionId],
    ) -> Result<Vec<SessionMetrics>, SessionMetricsRepositoryError>;

    fn delete_by_session(&self, session_id: SessionId)
        -> Result<(), SessionMetricsRepositoryError>;
}
