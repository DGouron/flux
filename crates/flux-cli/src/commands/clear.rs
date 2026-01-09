use anyhow::{Context, Result};
use dialoguer::Confirm;
use flux_adapters::SqliteSessionRepository;
use flux_core::{Config, SessionRepository, Translator};

pub async fn execute(skip_confirmation: bool) -> Result<()> {
    let translator = get_translator();
    let repository = open_repository()?;

    let count = repository
        .count_completed_sessions()
        .map_err(|error| anyhow::anyhow!("{}", error))?;

    if count == 0 {
        println!("{}", translator.get("command.clear_empty"));
        return Ok(());
    }

    let has_active = repository
        .has_active_session()
        .map_err(|error| anyhow::anyhow!("{}", error))?;

    if !skip_confirmation {
        let prompt = translator
            .get("command.clear_confirm")
            .replace("{count}", &count.to_string());

        let confirmed = Confirm::new()
            .with_prompt(prompt)
            .default(false)
            .interact()?;

        if !confirmed {
            println!("{}", translator.get("command.clear_cancelled"));
            return Ok(());
        }
    }

    let deleted = repository
        .clear_completed_sessions()
        .map_err(|error| anyhow::anyhow!("{}", error))?;

    let message = if has_active {
        translator
            .get("command.clear_success_with_active")
            .replace("{count}", &deleted.to_string())
    } else {
        translator
            .get("command.clear_success")
            .replace("{count}", &deleted.to_string())
    };

    println!("{}", message);

    Ok(())
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
