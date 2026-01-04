use crate::client::{ClientError, DaemonClient};
use anyhow::{bail, Result};
use flux_core::{Config, Translator};
use flux_protocol::{Request, Response};

pub async fn execute() -> Result<()> {
    let translator = get_translator();
    let client = DaemonClient::new();

    match client.send(Request::PauseSession).await {
        Ok(Response::Ok) => {
            println!("{}", translator.get("command.pause_success"));
        }
        Ok(Response::Error { message }) => {
            if message.contains("aucune session") || message.contains("no session") {
                println!("{}", translator.get("status.no_session"));
            } else if message.contains("déjà en pause") || message.contains("already paused") {
                println!("{}", translator.get("command.pause_already"));
            } else {
                bail!("{}", message);
            }
        }
        Ok(_) => {
            bail!("{}", translator.get("error.unexpected_response"));
        }
        Err(ClientError::DaemonNotRunning) => {
            eprintln!("{}", translator.get("error.daemon_not_running"));
            eprintln!("{}", translator.get("error.daemon_not_running_hint"));
            std::process::exit(1);
        }
        Err(ClientError::Timeout) => {
            bail!("{}", translator.get("error.connection_timeout"));
        }
        Err(error) => {
            bail!("{}", error);
        }
    }

    Ok(())
}

fn get_translator() -> Translator {
    Config::load()
        .map(|config| Translator::new(config.general.language))
        .unwrap_or_default()
}
