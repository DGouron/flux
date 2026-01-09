use crate::client::DaemonClient;
use anyhow::{bail, Context, Result};
use dialoguer::Confirm;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const GITHUB_API_URL: &str = "https://api.github.com/repos/DGouron/flux/releases/latest";
const INSTALL_SCRIPT_URL: &str = "https://raw.githubusercontent.com/DGouron/flux/main/install.sh";

pub async fn execute(skip_confirmation: bool) -> Result<()> {
    println!("Vérification des mises à jour...");

    let latest_version = fetch_latest_version()?;
    let current = format!("v{}", CURRENT_VERSION);

    println!("Version actuelle : {}", current);
    println!("Dernière version : {}", latest_version);
    println!();

    if current == latest_version {
        println!("✅ Flux est déjà à jour.");
        return Ok(());
    }

    let daemon_running = is_daemon_running().await;

    if daemon_running {
        println!("⚠️  Le daemon Flux est en cours d'exécution.");

        if !skip_confirmation {
            let confirmed = Confirm::new()
                .with_prompt("Arrêter le daemon et continuer ?")
                .default(true)
                .interact()?;

            if !confirmed {
                println!("Mise à jour annulée.");
                return Ok(());
            }
        }

        println!("Arrêt du daemon...");
        stop_daemon()?;
    }

    let backup_dir = create_backup()?;
    println!("Sauvegarde créée : {}", backup_dir.display());

    println!("Téléchargement et installation...");

    match run_install_script() {
        Ok(_) => {
            if verify_installation()? {
                cleanup_backup(&backup_dir);
                println!("\n✅ Flux mis à jour vers {}", latest_version);
            } else {
                println!("\n❌ Vérification échouée, restauration...");
                restore_backup(&backup_dir)?;
                println!("✅ Restauration réussie. Flux est toujours à {}.", current);
            }
        }
        Err(error) => {
            println!("\n❌ Mise à jour échouée : {}", error);
            println!("Restauration de la version précédente...");
            restore_backup(&backup_dir)?;
            println!("✅ Restauration réussie. Flux est toujours à {}.", current);
        }
    }

    Ok(())
}

fn fetch_latest_version() -> Result<String> {
    let response: serde_json::Value = ureq::get(GITHUB_API_URL)
        .set("User-Agent", "flux-cli")
        .call()
        .context("Impossible de contacter l'API GitHub")?
        .into_json()
        .context("Réponse API invalide")?;

    response["tag_name"]
        .as_str()
        .map(String::from)
        .context("Tag de version introuvable")
}

async fn is_daemon_running() -> bool {
    let client = DaemonClient::new();
    client.send(flux_protocol::Request::Ping).await.is_ok()
}

fn stop_daemon() -> Result<()> {
    Command::new("pkill")
        .args(["-f", "flux-daemon"])
        .status()
        .context("Impossible d'arrêter le daemon")?;

    std::thread::sleep(std::time::Duration::from_millis(500));
    Ok(())
}

fn get_binary_paths() -> Result<(PathBuf, PathBuf, Option<PathBuf>)> {
    let flux_path = which::which("flux").context("Binaire flux introuvable dans le PATH")?;
    let daemon_path =
        which::which("flux-daemon").context("Binaire flux-daemon introuvable dans le PATH")?;
    let gui_path = which::which("flux-gui").ok();
    Ok((flux_path, daemon_path, gui_path))
}

fn create_backup() -> Result<PathBuf> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let backup_dir = PathBuf::from(format!("/tmp/flux-backup-{}", timestamp));
    fs::create_dir_all(&backup_dir).context("Impossible de créer le répertoire de sauvegarde")?;

    let (flux_path, daemon_path, gui_path) = get_binary_paths()?;

    fs::copy(&flux_path, backup_dir.join("flux")).context("Impossible de sauvegarder flux")?;
    fs::copy(&daemon_path, backup_dir.join("flux-daemon"))
        .context("Impossible de sauvegarder flux-daemon")?;
    if let Some(gui) = gui_path {
        fs::copy(&gui, backup_dir.join("flux-gui"))
            .context("Impossible de sauvegarder flux-gui")?;
    }

    Ok(backup_dir)
}

fn run_install_script() -> Result<()> {
    let status = Command::new("bash")
        .args(["-c", &format!("curl -sSL {} | bash", INSTALL_SCRIPT_URL)])
        .status()
        .context("Impossible d'exécuter le script d'installation")?;

    if !status.success() {
        bail!("Le script d'installation a échoué");
    }

    Ok(())
}

fn verify_installation() -> Result<bool> {
    let output = Command::new("flux")
        .arg("--version")
        .output()
        .context("Impossible de vérifier l'installation")?;

    Ok(output.status.success())
}

fn restore_backup(backup_dir: &PathBuf) -> Result<()> {
    let (flux_path, daemon_path, gui_path) = get_binary_paths()?;

    fs::copy(backup_dir.join("flux"), &flux_path).context("Impossible de restaurer flux")?;
    fs::copy(backup_dir.join("flux-daemon"), &daemon_path)
        .context("Impossible de restaurer flux-daemon")?;
    if let Some(gui) = gui_path {
        let backup_gui = backup_dir.join("flux-gui");
        if backup_gui.exists() {
            fs::copy(&backup_gui, &gui).context("Impossible de restaurer flux-gui")?;
        }
    }

    cleanup_backup(backup_dir);
    Ok(())
}

fn cleanup_backup(backup_dir: &PathBuf) {
    let _ = fs::remove_dir_all(backup_dir);
}
