use crate::client::{ClientError, DaemonClient};
use crate::daemon_launcher::ensure_daemon_running;
use anyhow::{bail, Result};
use flux_core::{Config, Translator};
use flux_protocol::{FocusMode, Request, Response};

pub async fn execute(duration: Option<u64>, mode: Option<String>) -> Result<()> {
    let translator = get_translator();

    let focus_mode = match mode.as_deref() {
        Some("prompting") => Some(FocusMode::Prompting),
        Some("review") => Some(FocusMode::Review),
        Some("architecture") => Some(FocusMode::Architecture),
        Some(custom) => Some(FocusMode::Custom(custom.to_string())),
        None => None,
    };

    let client = DaemonClient::new();

    let response = match client
        .send(Request::StartSession {
            duration,
            mode: focus_mode.clone(),
        })
        .await
    {
        Ok(response) => response,
        Err(ClientError::DaemonNotRunning) => {
            ensure_daemon_running().await?;
            client
                .send(Request::StartSession {
                    duration,
                    mode: focus_mode.clone(),
                })
                .await?
        }
        Err(error) => return Err(error.into()),
    };

    match response {
        Response::Ok => {
            let duration_display = duration.unwrap_or(25);
            let mode_display = focus_mode
                .map(format_mode)
                .unwrap_or_else(|| "prompting".to_string());

            println!("{}", translator.get("command.start_success"));
            println!(
                "{}",
                translator.format(
                    "command.start_duration",
                    &[("duration", &duration_display.to_string())]
                )
            );
            println!(
                "{}",
                translator.format("command.start_mode", &[("mode", &mode_display)])
            );
        }
        Response::Error { message } => {
            bail!("{}", message);
        }
        _ => {
            bail!("{}", translator.get("error.unexpected_response"));
        }
    }

    Ok(())
}

fn get_translator() -> Translator {
    Config::load()
        .map(|config| Translator::new(config.general.language))
        .unwrap_or_default()
}

fn format_mode(mode: FocusMode) -> String {
    match mode {
        FocusMode::Prompting => "prompting".to_string(),
        FocusMode::Review => "review".to_string(),
        FocusMode::Architecture => "architecture".to_string(),
        FocusMode::Custom(name) => name,
    }
}
