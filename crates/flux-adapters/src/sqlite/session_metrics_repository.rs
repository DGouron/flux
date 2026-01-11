use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

use rusqlite::{params, Connection, OptionalExtension};

use flux_core::{
    SessionId, SessionMetrics, SessionMetricsRepository, SessionMetricsRepositoryError,
};

pub struct SqliteSessionMetricsRepository {
    connection: Mutex<Connection>,
}

impl SqliteSessionMetricsRepository {
    pub fn new(path: &Path) -> Result<Self, SessionMetricsRepositoryError> {
        let connection = Connection::open(path)
            .map_err(|error| SessionMetricsRepositoryError::Persistence(error.to_string()))?;

        let repository = Self {
            connection: Mutex::new(connection),
        };
        repository.initialize_schema()?;

        Ok(repository)
    }

    pub fn in_memory() -> Result<Self, SessionMetricsRepositoryError> {
        let connection = Connection::open_in_memory()
            .map_err(|error| SessionMetricsRepositoryError::Persistence(error.to_string()))?;

        let repository = Self {
            connection: Mutex::new(connection),
        };
        repository.initialize_schema()?;

        Ok(repository)
    }

    fn initialize_schema(&self) -> Result<(), SessionMetricsRepositoryError> {
        let connection = self.connection.lock().unwrap();
        connection
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS session_metrics (
                    session_id INTEGER PRIMARY KEY,
                    context_switch_count INTEGER NOT NULL DEFAULT 0,
                    total_short_bursts INTEGER NOT NULL DEFAULT 0,
                    short_bursts_by_app TEXT NOT NULL DEFAULT '{}'
                );",
            )
            .map_err(|error| SessionMetricsRepositoryError::Persistence(error.to_string()))
    }
}

impl SessionMetricsRepository for SqliteSessionMetricsRepository {
    fn save(&self, metrics: &SessionMetrics) -> Result<(), SessionMetricsRepositoryError> {
        let connection = self.connection.lock().unwrap();

        let short_bursts_json = serde_json::to_string(&metrics.short_bursts_by_app)
            .map_err(|error| SessionMetricsRepositoryError::Persistence(error.to_string()))?;

        connection
            .execute(
                "INSERT OR REPLACE INTO session_metrics
                 (session_id, context_switch_count, total_short_bursts, short_bursts_by_app)
                 VALUES (?1, ?2, ?3, ?4)",
                params![
                    metrics.session_id,
                    metrics.context_switch_count,
                    metrics.total_short_bursts,
                    short_bursts_json
                ],
            )
            .map_err(|error| SessionMetricsRepositoryError::Persistence(error.to_string()))?;

        Ok(())
    }

    fn find_by_session(
        &self,
        session_id: SessionId,
    ) -> Result<Option<SessionMetrics>, SessionMetricsRepositoryError> {
        let connection = self.connection.lock().unwrap();

        let mut statement = connection
            .prepare(
                "SELECT session_id, context_switch_count, total_short_bursts, short_bursts_by_app
                 FROM session_metrics
                 WHERE session_id = ?1",
            )
            .map_err(|error| SessionMetricsRepositoryError::Persistence(error.to_string()))?;

        let result = statement
            .query_row(params![session_id], |row| Ok(row_to_session_metrics(row)))
            .optional()
            .map_err(|error| SessionMetricsRepositoryError::Persistence(error.to_string()))?;

        Ok(result)
    }

    fn find_by_sessions(
        &self,
        session_ids: &[SessionId],
    ) -> Result<Vec<SessionMetrics>, SessionMetricsRepositoryError> {
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
            "SELECT session_id, context_switch_count, total_short_bursts, short_bursts_by_app
             FROM session_metrics
             WHERE session_id IN ({})",
            placeholders
        );

        let mut statement = connection
            .prepare(&query)
            .map_err(|error| SessionMetricsRepositoryError::Persistence(error.to_string()))?;

        let metrics = statement
            .query_map(rusqlite::params_from_iter(session_ids.iter()), |row| {
                Ok(row_to_session_metrics(row))
            })
            .map_err(|error| SessionMetricsRepositoryError::Persistence(error.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| SessionMetricsRepositoryError::Persistence(error.to_string()))?;

        Ok(metrics)
    }

    fn delete_by_session(
        &self,
        session_id: SessionId,
    ) -> Result<(), SessionMetricsRepositoryError> {
        let connection = self.connection.lock().unwrap();

        connection
            .execute(
                "DELETE FROM session_metrics WHERE session_id = ?1",
                params![session_id],
            )
            .map_err(|error| SessionMetricsRepositoryError::Persistence(error.to_string()))?;

        Ok(())
    }
}

fn row_to_session_metrics(row: &rusqlite::Row) -> SessionMetrics {
    let session_id: i64 = row.get(0).unwrap();
    let context_switch_count: u32 = row.get(1).unwrap();
    let _total_short_bursts: u32 = row.get(2).unwrap();
    let short_bursts_json: String = row.get(3).unwrap();

    let short_bursts_by_app: HashMap<String, u32> =
        serde_json::from_str(&short_bursts_json).unwrap_or_default();

    SessionMetrics::new(session_id, context_switch_count, short_bursts_by_app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_and_retrieve_metrics() {
        let repository = SqliteSessionMetricsRepository::in_memory().unwrap();

        let mut short_bursts = HashMap::new();
        short_bursts.insert("discord".to_string(), 5);
        let metrics = SessionMetrics::new(1, 10, short_bursts);

        repository.save(&metrics).unwrap();

        let loaded = repository.find_by_session(1).unwrap().unwrap();
        assert_eq!(loaded.session_id, 1);
        assert_eq!(loaded.context_switch_count, 10);
        assert_eq!(loaded.total_short_bursts, 5);
        assert_eq!(loaded.short_bursts_by_app.get("discord"), Some(&5));
    }

    #[test]
    fn find_by_session_returns_none_when_not_found() {
        let repository = SqliteSessionMetricsRepository::in_memory().unwrap();

        let result = repository.find_by_session(999).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn find_by_sessions_returns_multiple() {
        let repository = SqliteSessionMetricsRepository::in_memory().unwrap();

        repository
            .save(&SessionMetrics::new(1, 5, HashMap::new()))
            .unwrap();
        repository
            .save(&SessionMetrics::new(2, 10, HashMap::new()))
            .unwrap();
        repository
            .save(&SessionMetrics::new(3, 15, HashMap::new()))
            .unwrap();

        let results = repository.find_by_sessions(&[1, 3]).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn find_by_sessions_returns_empty_for_empty_input() {
        let repository = SqliteSessionMetricsRepository::in_memory().unwrap();

        let results = repository.find_by_sessions(&[]).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn delete_by_session_removes_metrics() {
        let repository = SqliteSessionMetricsRepository::in_memory().unwrap();

        repository
            .save(&SessionMetrics::new(1, 10, HashMap::new()))
            .unwrap();
        repository.delete_by_session(1).unwrap();

        let result = repository.find_by_session(1).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn save_replaces_existing_metrics() {
        let repository = SqliteSessionMetricsRepository::in_memory().unwrap();

        repository
            .save(&SessionMetrics::new(1, 5, HashMap::new()))
            .unwrap();
        repository
            .save(&SessionMetrics::new(1, 20, HashMap::new()))
            .unwrap();

        let loaded = repository.find_by_session(1).unwrap().unwrap();
        assert_eq!(loaded.context_switch_count, 20);
    }
}
