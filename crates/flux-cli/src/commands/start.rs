use crate::client::{ClientError, DaemonClient};
use anyhow::{bail, Result};
use flux_protocol::{FocusMode, Request, Response};

pub async fn execute(duration: Option<u64>, mode: Option<String>) -> Result<()> {
    let focus_mode = match mode.as_deref() {
        Some("prompting") => Some(FocusMode::Prompting),
        Some("review") => Some(FocusMode::Review),
        Some("architecture") => Some(FocusMode::Architecture),
        Some(custom) => Some(FocusMode::Custom(custom.to_string())),
        None => None,
    };

    let client = DaemonClient::new();

    match client
        .send(Request::StartSession {
            duration,
            mode: focus_mode.clone(),
        })
        .await
    {
        Ok(Response::Ok) => {
            let duration_display = duration.unwrap_or(25);
            let mode_display = focus_mode
                .map(format_mode)
                .unwrap_or_else(|| "prompting".to_string());

            println!("🚀 Session focus démarrée");
            println!("   Durée: {} min", duration_display);
            println!("   Mode: {}", mode_display);
        }
        Ok(Response::Error { message }) => {
            bail!("{}", message);
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

fn format_mode(mode: FocusMode) -> String {
    match mode {
        FocusMode::Prompting => "prompting".to_string(),
        FocusMode::Review => "review".to_string(),
        FocusMode::Architecture => "architecture".to_string(),
        FocusMode::Custom(name) => name,
    }
}
