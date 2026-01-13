use chrono::{DateTime, Utc};

use super::FocusMode;

pub type SessionId = i64;

#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    pub id: Option<SessionId>,
    pub mode: FocusMode,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_seconds: Option<i64>,
    pub check_in_count: i32,
}

impl Session {
    pub fn start(mode: FocusMode) -> Self {
        Self {
            id: None,
            mode,
            started_at: Utc::now(),
            ended_at: None,
            duration_seconds: None,
            check_in_count: 0,
        }
    }

    pub fn end(&mut self) {
        let now = Utc::now();
        let duration = now.signed_duration_since(self.started_at);
        self.ended_at = Some(now);
        self.duration_seconds = Some(duration.num_seconds());
    }

    pub fn increment_check_in(&mut self) {
        self.check_in_count += 1;
    }

    pub fn is_active(&self) -> bool {
        self.ended_at.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_session_starts_active() {
        let session = Session::start(FocusMode::AiAssisted);

        assert!(session.id.is_none());
        assert!(session.is_active());
        assert!(session.ended_at.is_none());
        assert!(session.duration_seconds.is_none());
        assert_eq!(session.check_in_count, 0);
    }

    #[test]
    fn ending_session_sets_duration() {
        let mut session = Session::start(FocusMode::Review);
        session.end();

        assert!(!session.is_active());
        assert!(session.ended_at.is_some());
        assert!(session.duration_seconds.is_some());
    }

    #[test]
    fn check_in_increments_counter() {
        let mut session = Session::start(FocusMode::Architecture);

        session.increment_check_in();
        assert_eq!(session.check_in_count, 1);

        session.increment_check_in();
        assert_eq!(session.check_in_count, 2);
    }
}
