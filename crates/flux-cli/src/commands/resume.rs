use crate::client::{ClientError, DaemonClient};
use anyhow::{bail, Result};
use flux_protocol::{Request, Response};

pub async fn execute() -> Result<()> {
    let client = DaemonClient::new();

    match client.send(Request::ResumeSession).await {
        Ok(Response::Ok) => {
            println!("▶️  Session reprise");
        }
        Ok(Response::Error { message }) => {
            if message.contains("aucune session") || message.contains("no session") {
                println!("⚪ Aucune session active");
            } else if message.contains("pas en pause") || message.contains("not paused") {
                println!("🟢 Session déjà active");
            } else {
                bail!("{}", message);
            }
        }
        Ok(_) => {
            bail!("Réponse inattendue du daemon");
        }
        Err(ClientError::DaemonNotRunning) => {
            eprintln!("⚫ Le daemon n'est pas démarré");
            eprintln!("   Lancez d'abord: flux-daemon");
            std::process::exit(1);
        }
        Err(ClientError::Timeout) => {
            bail!("Timeout de connexion au daemon");
        }
        Err(error) => {
            bail!("{}", error);
        }
    }

    Ok(())
}
