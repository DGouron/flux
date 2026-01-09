use anyhow::{Context, Result};
use flux_adapters::SqliteSessionRepository;
use flux_core::{Config, SessionRepository, SessionRepositoryError, Translator};

pub async fn execute(session_id: i64) -> Result<()> {
    let translator = get_translator();
    let repository = open_repository()?;

    match repository.delete_session(session_id) {
        Ok(()) => {
            let message = translator
                .get("command.delete_success")
                .replace("{id}", &session_id.to_string());
            println!("{}", message);
            Ok(())
        }
        Err(SessionRepositoryError::NotFound { .. }) => {
            let message = translator
                .get("command.delete_not_found")
                .replace("{id}", &session_id.to_string());
            anyhow::bail!("{}", message);
        }
        Err(SessionRepositoryError::ActiveSession { .. }) => {
            let message = translator.get("command.delete_active_session");
            anyhow::bail!("{}", message);
        }
        Err(error) => {
            anyhow::bail!("{}", error);
        }
    }
}

fn get_translator() -> Translator {
    Config::load()
        .map(|config| Translator::new(config.general.language))
        .unwrap_or_default()
}

fn open_repository() -> Result<SqliteSessionRepository> {
    let data_dir = dirs::data_dir()
        .context("cannot find data directory")?
        .join("flux");

    let database_path = data_dir.join("sessions.db");

    if !database_path.exists() {
        anyhow::bail!("no session data. Start a session first with 'flux start'.");
    }

    SqliteSessionRepository::new(&database_path)
        .map_err(|error| anyhow::anyhow!("database access error: {}", error))
}
