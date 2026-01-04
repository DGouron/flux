use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;

const DAEMON_STARTUP_TIMEOUT: Duration = Duration::from_secs(5);
const DAEMON_POLL_INTERVAL: Duration = Duration::from_millis(100);

pub async fn ensure_daemon_running() -> Result<()> {
    println!("🔄 Démarrage du daemon...");

    spawn_daemon()?;
    wait_for_socket().await?;

    Ok(())
}

fn spawn_daemon() -> Result<()> {
    let daemon_path = find_daemon_binary()?;

    Command::new(&daemon_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("impossible de lancer {}", daemon_path.display()))?;

    Ok(())
}

fn find_daemon_binary() -> Result<PathBuf> {
    if let Ok(path) = std::env::var("FLUX_DAEMON_PATH") {
        return Ok(PathBuf::from(path));
    }

    let current_exe = std::env::current_exe().context("impossible de trouver l'exécutable")?;
    let exe_dir = current_exe
        .parent()
        .context("impossible de trouver le répertoire de l'exécutable")?;

    let daemon_name = if cfg!(windows) {
        "flux-daemon.exe"
    } else {
        "flux-daemon"
    };

    let sibling_path = exe_dir.join(daemon_name);
    if sibling_path.exists() {
        return Ok(sibling_path);
    }

    if let Ok(path) = which::which(daemon_name) {
        return Ok(path);
    }

    bail!(
        "flux-daemon introuvable. Assurez-vous qu'il est installé ou définissez FLUX_DAEMON_PATH"
    );
}

async fn wait_for_socket() -> Result<()> {
    let socket_path = default_socket_path();
    let start = std::time::Instant::now();

    while start.elapsed() < DAEMON_STARTUP_TIMEOUT {
        if socket_path.exists() {
            return Ok(());
        }
        sleep(DAEMON_POLL_INTERVAL).await;
    }

    bail!(
        "timeout: le daemon n'a pas démarré après {} secondes",
        DAEMON_STARTUP_TIMEOUT.as_secs()
    );
}

#[cfg(unix)]
fn default_socket_path() -> PathBuf {
    let uid = unsafe { libc::getuid() };
    PathBuf::from(format!("/run/user/{}/flux.sock", uid))
}

#[cfg(windows)]
fn default_socket_path() -> PathBuf {
    let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(format!(r"{}\flux\flux.sock", local_app_data))
}
