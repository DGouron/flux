use std::path::Path;
use std::sync::Mutex;

use rusqlite::{params, Connection};

use flux_core::{AppTrackingRepository, AppTrackingRepositoryError, AppUsage, SessionId};

pub struct SqliteAppTrackingRepository {
    connection: Mutex<Connection>,
}

impl SqliteAppTrackingRepository {
    pub fn new(path: &Path) -> Result<Self, AppTrackingRepositoryError> {
        let connection =
            Connection::open(path).map_err(|error| AppTrackingRepositoryError::Storage {
                message: error.to_string(),
            })?;

        let repository = Self {
            connection: Mutex::new(connection),
        };
        repository.initialize_schema()?;

        Ok(repository)
    }

    pub fn in_memory() -> Result<Self, AppTrackingRepositoryError> {
        let connection =
            Connection::open_in_memory().map_err(|error| AppTrackingRepositoryError::Storage {
                message: error.to_string(),
            })?;

        let repository = Self {
            connection: Mutex::new(connection),
        };
        repository.initialize_schema()?;

        Ok(repository)
    }

    fn initialize_schema(&self) -> Result<(), AppTrackingRepositoryError> {
        let connection = self.connection.lock().unwrap();
        connection
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS app_tracking (
                    session_id INTEGER NOT NULL,
                    application_name TEXT NOT NULL,
                    duration_seconds INTEGER NOT NULL DEFAULT 0,
                    PRIMARY KEY (session_id, application_name)
                );",
            )
            .map_err(|error| AppTrackingRepositoryError::Storage {
                message: error.to_string(),
            })
    }
}

impl AppTrackingRepository for SqliteAppTrackingRepository {
    fn save_or_update(&self, usage: &AppUsage) -> Result<(), AppTrackingRepositoryError> {
        let connection = self.connection.lock().unwrap();

        connection
            .execute(
                "INSERT INTO app_tracking (session_id, application_name, duration_seconds)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT (session_id, application_name)
                 DO UPDATE SET duration_seconds = duration_seconds + excluded.duration_seconds",
                params![
                    usage.session_id,
                    &usage.application_name,
                    usage.duration_seconds
                ],
            )
            .map_err(|error| AppTrackingRepositoryError::Storage {
                message: error.to_string(),
            })?;

        Ok(())
    }

    fn find_by_session(
        &self,
        session_id: SessionId,
    ) -> Result<Vec<AppUsage>, AppTrackingRepositoryError> {
        let connection = self.connection.lock().unwrap();

        let mut statement = connection
            .prepare(
                "SELECT session_id, application_name, duration_seconds
                 FROM app_tracking
                 WHERE session_id = ?1
                 ORDER BY duration_seconds DESC",
            )
            .map_err(|error| AppTrackingRepositoryError::Storage {
                message: error.to_string(),
            })?;

        let usages = statement
            .query_map(params![session_id], |row| Ok(row_to_app_usage(row)))
            .map_err(|error| AppTrackingRepositoryError::Storage {
                message: error.to_string(),
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| AppTrackingRepositoryError::Storage {
                message: error.to_string(),
            })?;

        Ok(usages)
    }

    fn find_by_sessions(
        &self,
        session_ids: &[SessionId],
    ) -> Result<Vec<AppUsage>, AppTrackingRepositoryError> {
        if session_ids.is_empty() {
            return Ok(Vec::new());
        }

        let connection = self.connection.lock().unwrap();

        let placeholders: String = session_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(",");
        let query = format!(
            "SELECT 0 as session_id, application_name, SUM(duration_seconds) as total_seconds
             FROM app_tracking
             WHERE session_id IN ({})
             GROUP BY application_name
             ORDER BY total_seconds DESC",
            placeholders
        );

        let mut statement =
            connection
                .prepare(&query)
                .map_err(|error| AppTrackingRepositoryError::Storage {
                    message: error.to_string(),
                })?;

        let usages = statement
            .query_map(rusqlite::params_from_iter(session_ids.iter()), |row| {
                Ok(row_to_app_usage(row))
            })
            .map_err(|error| AppTrackingRepositoryError::Storage {
                message: error.to_string(),
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| AppTrackingRepositoryError::Storage {
                message: error.to_string(),
            })?;

        Ok(usages)
    }

    fn delete_by_session(&self, session_id: SessionId) -> Result<(), AppTrackingRepositoryError> {
        let connection = self.connection.lock().unwrap();

        connection
            .execute(
                "DELETE FROM app_tracking WHERE session_id = ?1",
                params![session_id],
            )
            .map_err(|error| AppTrackingRepositoryError::Storage {
                message: error.to_string(),
            })?;

        Ok(())
    }
}

fn row_to_app_usage(row: &rusqlite::Row) -> AppUsage {
    let session_id: i64 = row.get(0).unwrap();
    let application_name: String = row.get(1).unwrap();
    let duration_seconds: i64 = row.get(2).unwrap();

    AppUsage {
        session_id,
        application_name,
        duration_seconds,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_and_retrieve_app_usage() {
        let repository = SqliteAppTrackingRepository::in_memory().unwrap();

        let usage = AppUsage::with_duration(1, "cursor".to_string(), 60);
        repository.save_or_update(&usage).unwrap();

        let usages = repository.find_by_session(1).unwrap();

        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].application_name, "cursor");
        assert_eq!(usages[0].duration_seconds, 60);
    }

    #[test]
    fn save_or_update_accumulates_duration() {
        let repository = SqliteAppTrackingRepository::in_memory().unwrap();

        let usage1 = AppUsage::with_duration(1, "cursor".to_string(), 30);
        repository.save_or_update(&usage1).unwrap();

        let usage2 = AppUsage::with_duration(1, "cursor".to_string(), 25);
        repository.save_or_update(&usage2).unwrap();

        let usages = repository.find_by_session(1).unwrap();

        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].duration_seconds, 55);
    }

    #[test]
    fn multiple_apps_in_same_session() {
        let repository = SqliteAppTrackingRepository::in_memory().unwrap();

        repository
            .save_or_update(&AppUsage::with_duration(1, "cursor".to_string(), 100))
            .unwrap();
        repository
            .save_or_update(&AppUsage::with_duration(1, "firefox".to_string(), 50))
            .unwrap();
        repository
            .save_or_update(&AppUsage::with_duration(1, "alacritty".to_string(), 30))
            .unwrap();

        let usages = repository.find_by_session(1).unwrap();

        assert_eq!(usages.len(), 3);
        assert_eq!(usages[0].application_name, "cursor");
        assert_eq!(usages[1].application_name, "firefox");
        assert_eq!(usages[2].application_name, "alacritty");
    }

    #[test]
    fn find_by_sessions_aggregates_across_sessions() {
        let repository = SqliteAppTrackingRepository::in_memory().unwrap();

        repository
            .save_or_update(&AppUsage::with_duration(1, "cursor".to_string(), 100))
            .unwrap();
        repository
            .save_or_update(&AppUsage::with_duration(2, "cursor".to_string(), 50))
            .unwrap();
        repository
            .save_or_update(&AppUsage::with_duration(1, "firefox".to_string(), 30))
            .unwrap();

        let usages = repository.find_by_sessions(&[1, 2]).unwrap();

        assert_eq!(usages.len(), 2);
        assert_eq!(usages[0].application_name, "cursor");
        assert_eq!(usages[0].duration_seconds, 150);
        assert_eq!(usages[1].application_name, "firefox");
        assert_eq!(usages[1].duration_seconds, 30);
    }

    #[test]
    fn find_by_sessions_returns_empty_for_empty_input() {
        let repository = SqliteAppTrackingRepository::in_memory().unwrap();

        let usages = repository.find_by_sessions(&[]).unwrap();

        assert!(usages.is_empty());
    }

    #[test]
    fn delete_by_session_removes_all_app_data() {
        let repository = SqliteAppTrackingRepository::in_memory().unwrap();

        repository
            .save_or_update(&AppUsage::with_duration(1, "cursor".to_string(), 100))
            .unwrap();
        repository
            .save_or_update(&AppUsage::with_duration(1, "firefox".to_string(), 50))
            .unwrap();
        repository
            .save_or_update(&AppUsage::with_duration(2, "cursor".to_string(), 30))
            .unwrap();

        repository.delete_by_session(1).unwrap();

        let session1_usages = repository.find_by_session(1).unwrap();
        let session2_usages = repository.find_by_session(2).unwrap();

        assert!(session1_usages.is_empty());
        assert_eq!(session2_usages.len(), 1);
    }

    #[test]
    fn results_ordered_by_duration_descending() {
        let repository = SqliteAppTrackingRepository::in_memory().unwrap();

        repository
            .save_or_update(&AppUsage::with_duration(1, "low".to_string(), 10))
            .unwrap();
        repository
            .save_or_update(&AppUsage::with_duration(1, "high".to_string(), 100))
            .unwrap();
        repository
            .save_or_update(&AppUsage::with_duration(1, "medium".to_string(), 50))
            .unwrap();

        let usages = repository.find_by_session(1).unwrap();

        assert_eq!(usages[0].application_name, "high");
        assert_eq!(usages[1].application_name, "medium");
        assert_eq!(usages[2].application_name, "low");
    }
}
