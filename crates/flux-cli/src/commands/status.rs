use crate::client::{ClientError, DaemonClient};
use anyhow::Result;
use flux_protocol::{FocusMode, Request, Response};
use serde::Serialize;

#[derive(Serialize)]
struct StatusOutput {
    active: bool,
    mode: Option<String>,
    remaining_seconds: u64,
    remaining_formatted: String,
    paused: bool,
}

pub async fn execute(json: bool) -> Result<()> {
    let client = DaemonClient::new();

    match client.send(Request::GetStatus).await {
        Ok(Response::SessionStatus {
            active,
            remaining_seconds,
            mode,
            paused,
        }) => {
            if json {
                print_json(active, remaining_seconds, mode, paused)?;
            } else {
                print_formatted(active, remaining_seconds, mode, paused);
            }
        }
        Ok(Response::Error { message }) => {
            if json {
                println!(r#"{{"error": "{}"}}"#, message);
            } else {
                eprintln!("Erreur: {}", message);
            }
            std::process::exit(1);
        }
        Ok(_) => {
            if json {
                println!(r#"{{"error": "unexpected response"}}"#);
            } else {
                eprintln!("Erreur: réponse inattendue du daemon");
            }
            std::process::exit(1);
        }
        Err(ClientError::DaemonNotRunning) => {
            if json {
                println!(r#"{{"error": "daemon not running", "active": false}}"#);
            } else {
                println!("⚫ Daemon non démarré");
            }
        }
        Err(ClientError::Timeout) => {
            if json {
                println!(r#"{{"error": "timeout"}}"#);
            } else {
                eprintln!("Erreur: timeout de connexion au daemon");
            }
            std::process::exit(1);
        }
        Err(error) => {
            if json {
                println!(r#"{{"error": "{}"}}"#, error);
            } else {
                eprintln!("Erreur: {}", error);
            }
            std::process::exit(1);
        }
    }

    Ok(())
}

fn print_json(
    active: bool,
    remaining_seconds: u64,
    mode: Option<FocusMode>,
    paused: bool,
) -> Result<()> {
    let output = StatusOutput {
        active,
        mode: mode.map(format_mode),
        remaining_seconds,
        remaining_formatted: format_duration(remaining_seconds),
        paused,
    };
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn print_formatted(active: bool, remaining_seconds: u64, mode: Option<FocusMode>, paused: bool) {
    if !active {
        println!("⚪ Aucune session active");
        return;
    }

    if paused {
        println!("⏸️  Session en pause");
    } else {
        println!("🟢 Session focus active");
    }

    if let Some(focus_mode) = mode {
        println!("   Mode: {}", format_mode(focus_mode));
    }

    println!("   Temps restant: {}", format_duration(remaining_seconds));
}

fn format_mode(mode: FocusMode) -> String {
    match mode {
        FocusMode::Prompting => "prompting".to_string(),
        FocusMode::Review => "review".to_string(),
        FocusMode::Architecture => "architecture".to_string(),
        FocusMode::Custom(name) => name,
    }
}

fn format_duration(seconds: u64) -> String {
    let minutes = seconds / 60;
    let remaining_seconds = seconds % 60;

    if minutes > 0 {
        format!("{} min {} sec", minutes, remaining_seconds)
    } else {
        format!("{} sec", remaining_seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_duration_shows_minutes_and_seconds() {
        assert_eq!(format_duration(90), "1 min 30 sec");
        assert_eq!(format_duration(3600), "60 min 0 sec");
        assert_eq!(format_duration(1500), "25 min 0 sec");
    }

    #[test]
    fn format_duration_shows_only_seconds_when_under_minute() {
        assert_eq!(format_duration(45), "45 sec");
        assert_eq!(format_duration(0), "0 sec");
    }

    #[test]
    fn format_mode_returns_correct_strings() {
        assert_eq!(format_mode(FocusMode::Prompting), "prompting");
        assert_eq!(format_mode(FocusMode::Review), "review");
        assert_eq!(format_mode(FocusMode::Architecture), "architecture");
        assert_eq!(format_mode(FocusMode::Custom("deep-work".to_string())), "deep-work");
    }
}
