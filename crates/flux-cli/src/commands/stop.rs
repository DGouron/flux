use crate::client::{ClientError, DaemonClient};
use anyhow::{bail, Result};
use flux_core::{Config, Translator};
use flux_protocol::{Request, Response};

pub async fn execute() -> Result<()> {
    let translator = get_translator();
    let client = DaemonClient::new();

    match client.send(Request::StopSession).await {
        Ok(Response::Ok) => {
            println!("{}", translator.get("command.stop_success"));
        }
        Ok(Response::Error { message }) => {
            if message.contains("aucune session") || message.contains("no session") {
                println!("{}", translator.get("status.no_session"));
            } else {
                bail!("{}", message);
            }
        }
        Ok(_) => {
            bail!("{}", translator.get("error.unexpected_response"));
        }
        Err(ClientError::DaemonNotRunning) => {
            println!("{}", translator.get("status.no_session"));
            return Ok(());
        }
        Err(ClientError::Timeout) => {
            bail!("{}", translator.get("error.connection_timeout"));
        }
        Err(error) => {
            bail!("{}", error);
        }
    }

    shutdown_daemon(&client, &translator).await;

    Ok(())
}

fn get_translator() -> Translator {
    Config::load()
        .map(|config| Translator::new(config.general.language))
        .unwrap_or_default()
}

async fn shutdown_daemon(client: &DaemonClient, translator: &Translator) {
    if let Ok(Response::Ok) = client.send(Request::Shutdown).await {
        println!("{}", translator.get("command.stop_daemon_stopped"));
    }
}
