use crate::client::{ClientError, DaemonClient};
use anyhow::{bail, Result};
use flux_protocol::{Request, Response};

pub async fn execute() -> Result<()> {
    let client = DaemonClient::new();

    match client.send(Request::StopSession).await {
        Ok(Response::Ok) => {
            println!("🛑 Session focus terminée");
        }
        Ok(Response::Error { message }) => {
            if message.contains("aucune session") || message.contains("no session") {
                println!("⚪ Aucune session active");
            } else {
                bail!("{}", message);
            }
        }
        Ok(_) => {
            bail!("Réponse inattendue du daemon");
        }
        Err(ClientError::DaemonNotRunning) => {
            println!("⚪ Aucune session active");
            return Ok(());
        }
        Err(ClientError::Timeout) => {
            bail!("Timeout de connexion au daemon");
        }
        Err(error) => {
            bail!("{}", error);
        }
    }

    shutdown_daemon(&client).await;

    Ok(())
}

async fn shutdown_daemon(client: &DaemonClient) {
    if let Ok(Response::Ok) = client.send(Request::Shutdown).await {
        println!("   Daemon arrêté");
    }
}
