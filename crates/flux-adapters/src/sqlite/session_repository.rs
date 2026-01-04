use std::path::Path;
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};

use flux_core::{FocusMode, Session, SessionId, SessionRepository, SessionRepositoryError};

pub struct SqliteSessionRepository {
    connection: Mutex<Connection>,
}

impl SqliteSessionRepository {
    pub fn new(path: &Path) -> Result<Self, SessionRepositoryError> {
        let connection =
            Connection::open(path).map_err(|error| SessionRepositoryError::Storage {
                message: error.to_string(),
            })?;

        let repository = Self {
            connection: Mutex::new(connection),
        };
        repository.initialize_schema()?;

        Ok(repository)
    }

    pub fn in_memory() -> Result<Self, SessionRepositoryError> {
        let connection =
            Connection::open_in_memory().map_err(|error| SessionRepositoryError::Storage {
                message: error.to_string(),
            })?;

        let repository = Self {
            connection: Mutex::new(connection),
        };
        repository.initialize_schema()?;

        Ok(repository)
    }

    fn initialize_schema(&self) -> Result<(), SessionRepositoryError> {
        let connection = self.connection.lock().unwrap();
        connection
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS sessions (
                    id INTEGER PRIMARY KEY,
                    mode TEXT NOT NULL,
                    started_at TEXT NOT NULL,
                    ended_at TEXT,
                    duration_seconds INTEGER,
                    check_in_count INTEGER DEFAULT 0
                );",
            )
            .map_err(|error| SessionRepositoryError::Storage {
                message: error.to_string(),
            })
    }
}

impl SessionRepository for SqliteSessionRepository {
    fn save(&self, session: &mut Session) -> Result<SessionId, SessionRepositoryError> {
        let connection = self.connection.lock().unwrap();

        connection
            .execute(
                "INSERT INTO sessions (mode, started_at, ended_at, duration_seconds, check_in_count)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    session.mode.as_str(),
                    session.started_at.to_rfc3339(),
                    session.ended_at.map(|dt| dt.to_rfc3339()),
                    session.duration_seconds,
                    session.check_in_count,
                ],
            )
            .map_err(|error| SessionRepositoryError::Storage {
                message: error.to_string(),
            })?;

        let id = connection.last_insert_rowid();
        session.id = Some(id);

        Ok(id)
    }

    fn update(&self, session: &Session) -> Result<(), SessionRepositoryError> {
        let id = session.id.ok_or_else(|| SessionRepositoryError::Storage {
            message: "cannot update session without id".to_string(),
        })?;

        let connection = self.connection.lock().unwrap();

        let rows_affected = connection
            .execute(
                "UPDATE sessions SET ended_at = ?1, duration_seconds = ?2, check_in_count = ?3
                 WHERE id = ?4",
                params![
                    session.ended_at.map(|dt| dt.to_rfc3339()),
                    session.duration_seconds,
                    session.check_in_count,
                    id,
                ],
            )
            .map_err(|error| SessionRepositoryError::Storage {
                message: error.to_string(),
            })?;

        if rows_affected == 0 {
            return Err(SessionRepositoryError::NotFound { id });
        }

        Ok(())
    }

    fn find_by_id(&self, id: SessionId) -> Result<Session, SessionRepositoryError> {
        let connection = self.connection.lock().unwrap();

        connection
            .query_row(
                "SELECT id, mode, started_at, ended_at, duration_seconds, check_in_count
                 FROM sessions WHERE id = ?1",
                params![id],
                |row| Ok(row_to_session(row)),
            )
            .map_err(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => SessionRepositoryError::NotFound { id },
                _ => SessionRepositoryError::Storage {
                    message: error.to_string(),
                },
            })
    }

    fn find_active(&self) -> Result<Option<Session>, SessionRepositoryError> {
        let connection = self.connection.lock().unwrap();

        let result = connection.query_row(
            "SELECT id, mode, started_at, ended_at, duration_seconds, check_in_count
             FROM sessions WHERE ended_at IS NULL ORDER BY started_at DESC LIMIT 1",
            [],
            |row| Ok(row_to_session(row)),
        );

        match result {
            Ok(session) => Ok(Some(session)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(SessionRepositoryError::Storage {
                message: error.to_string(),
            }),
        }
    }

    fn find_completed_since(
        &self,
        since: DateTime<Utc>,
    ) -> Result<Vec<Session>, SessionRepositoryError> {
        let connection = self.connection.lock().unwrap();

        let mut statement = connection
            .prepare(
                "SELECT id, mode, started_at, ended_at, duration_seconds, check_in_count
                 FROM sessions
                 WHERE ended_at IS NOT NULL AND started_at >= ?1
                 ORDER BY started_at DESC",
            )
            .map_err(|error| SessionRepositoryError::Storage {
                message: error.to_string(),
            })?;

        let sessions = statement
            .query_map(params![since.to_rfc3339()], |row| Ok(row_to_session(row)))
            .map_err(|error| SessionRepositoryError::Storage {
                message: error.to_string(),
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| SessionRepositoryError::Storage {
                message: error.to_string(),
            })?;

        Ok(sessions)
    }
}

fn row_to_session(row: &rusqlite::Row) -> Session {
    let id: i64 = row.get(0).unwrap();
    let mode_str: String = row.get(1).unwrap();
    let started_at_str: String = row.get(2).unwrap();
    let ended_at_str: Option<String> = row.get(3).unwrap();
    let duration_seconds: Option<i64> = row.get(4).unwrap();
    let check_in_count: i32 = row.get(5).unwrap();

    Session {
        id: Some(id),
        mode: FocusMode::from_stored(&mode_str),
        started_at: DateTime::parse_from_rfc3339(&started_at_str)
            .unwrap()
            .with_timezone(&Utc),
        ended_at: ended_at_str.map(|s| {
            DateTime::parse_from_rfc3339(&s)
                .unwrap()
                .with_timezone(&Utc)
        }),
        duration_seconds,
        check_in_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_and_retrieve_session() {
        let repository = SqliteSessionRepository::in_memory().unwrap();

        let mut session = Session::start(FocusMode::Prompting);
        let id = repository.save(&mut session).unwrap();

        assert!(id > 0);
        assert_eq!(session.id, Some(id));

        let retrieved = repository.find_by_id(id).unwrap();
        assert_eq!(retrieved.mode, FocusMode::Prompting);
        assert!(retrieved.is_active());
    }

    #[test]
    fn update_session_on_end() {
        let repository = SqliteSessionRepository::in_memory().unwrap();

        let mut session = Session::start(FocusMode::Review);
        repository.save(&mut session).unwrap();

        session.end();
        repository.update(&session).unwrap();

        let retrieved = repository.find_by_id(session.id.unwrap()).unwrap();
        assert!(!retrieved.is_active());
        assert!(retrieved.ended_at.is_some());
        assert!(retrieved.duration_seconds.is_some());
    }

    #[test]
    fn find_active_returns_active_session() {
        let repository = SqliteSessionRepository::in_memory().unwrap();

        let mut session = Session::start(FocusMode::Architecture);
        repository.save(&mut session).unwrap();

        let active = repository.find_active().unwrap();
        assert!(active.is_some());
        assert_eq!(active.unwrap().id, session.id);
    }

    #[test]
    fn find_active_returns_none_when_all_ended() {
        let repository = SqliteSessionRepository::in_memory().unwrap();

        let mut session = Session::start(FocusMode::Prompting);
        repository.save(&mut session).unwrap();
        session.end();
        repository.update(&session).unwrap();

        let active = repository.find_active().unwrap();
        assert!(active.is_none());
    }

    #[test]
    fn check_in_count_persists() {
        let repository = SqliteSessionRepository::in_memory().unwrap();

        let mut session = Session::start(FocusMode::Prompting);
        session.increment_check_in();
        session.increment_check_in();
        repository.save(&mut session).unwrap();

        let retrieved = repository.find_by_id(session.id.unwrap()).unwrap();
        assert_eq!(retrieved.check_in_count, 2);
    }

    #[test]
    fn custom_mode_roundtrip() {
        let repository = SqliteSessionRepository::in_memory().unwrap();

        let mut session = Session::start(FocusMode::Custom("deep-work".to_string()));
        repository.save(&mut session).unwrap();

        let retrieved = repository.find_by_id(session.id.unwrap()).unwrap();
        assert_eq!(retrieved.mode, FocusMode::Custom("deep-work".to_string()));
    }

    #[test]
    fn find_completed_since_filters_by_date() {
        use chrono::Duration;

        let repository = SqliteSessionRepository::in_memory().unwrap();

        let mut session1 = Session::start(FocusMode::Prompting);
        session1.end();
        repository.save(&mut session1).unwrap();

        let mut session2 = Session::start(FocusMode::Review);
        session2.end();
        repository.save(&mut session2).unwrap();

        let since = Utc::now() - Duration::hours(1);
        let sessions = repository.find_completed_since(since).unwrap();

        assert_eq!(sessions.len(), 2);
    }

    #[test]
    fn find_completed_since_excludes_active_sessions() {
        use chrono::Duration;

        let repository = SqliteSessionRepository::in_memory().unwrap();

        let mut completed = Session::start(FocusMode::Prompting);
        completed.end();
        repository.save(&mut completed).unwrap();

        let mut active = Session::start(FocusMode::Review);
        repository.save(&mut active).unwrap();

        let since = Utc::now() - Duration::hours(1);
        let sessions = repository.find_completed_since(since).unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].mode, FocusMode::Prompting);
    }
}
