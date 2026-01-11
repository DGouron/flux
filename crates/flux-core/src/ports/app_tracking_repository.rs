use thiserror::Error;

use crate::domain::{AppUsage, SessionId};

#[derive(Error, Debug)]
pub enum AppTrackingRepositoryError {
    #[error("erreur de persistence: {message}")]
    Storage { message: String },
}

pub trait AppTrackingRepository: Send + Sync {
    fn save_or_update(&self, usage: &AppUsage) -> Result<(), AppTrackingRepositoryError>;

    fn find_by_session(
        &self,
        session_id: SessionId,
    ) -> Result<Vec<AppUsage>, AppTrackingRepositoryError>;

    fn find_by_sessions(
        &self,
        session_ids: &[SessionId],
    ) -> Result<Vec<AppUsage>, AppTrackingRepositoryError>;

    fn delete_by_session(&self, session_id: SessionId) -> Result<(), AppTrackingRepositoryError>;
}
