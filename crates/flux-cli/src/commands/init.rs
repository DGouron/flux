use anyhow::{bail, Context, Result};
use dialoguer::{Confirm, Input};
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
struct InitConfig {
    tray_enabled: bool,
    default_duration_minutes: u64,
    check_in_interval_minutes: u64,
    sound_enabled: bool,
}

pub fn execute(force: bool) -> Result<()> {
    let config_path = get_config_path()?;

    if config_path.exists() && !force {
        bail!(
            "La configuration existe déjà : {}\nUtilisez --force pour écraser.",
            config_path.display()
        );
    }

    if force && config_path.exists() {
        println!("⚠️  Écrasement de la configuration existante.\n");
    }

    println!("Bienvenue dans Flux ! Configurons vos sessions focus.\n");

    let config = prompt_configuration()?;
    write_config(&config_path, &config)?;

    println!("\n✅ Configuration enregistrée : {}", config_path.display());
    println!("Lancez `flux start` pour démarrer votre première session focus.");

    Ok(())
}

fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .context("Impossible de déterminer le répertoire de configuration")?
        .join("flux");

    Ok(config_dir.join("config.toml"))
}

fn prompt_configuration() -> Result<InitConfig> {
    let tray_enabled = Confirm::new()
        .with_prompt("Activer l'icône dans la barre des tâches ?")
        .default(true)
        .interact()?;

    let default_duration_minutes: u64 = Input::new()
        .with_prompt("Durée par défaut des sessions focus (minutes)")
        .default(25)
        .validate_with(|input: &u64| {
            if *input >= 1 && *input <= 480 {
                Ok(())
            } else {
                Err("La durée doit être entre 1 et 480 minutes")
            }
        })
        .interact_text()?;

    let check_in_interval_minutes: u64 = Input::new()
        .with_prompt("Intervalle entre les check-ins (minutes)")
        .default(25)
        .validate_with(|input: &u64| {
            if *input >= 5 && *input <= 120 {
                Ok(())
            } else {
                Err("L'intervalle doit être entre 5 et 120 minutes")
            }
        })
        .interact_text()?;

    let sound_enabled = Confirm::new()
        .with_prompt("Activer les sons de notification ?")
        .default(true)
        .interact()?;

    Ok(InitConfig {
        tray_enabled,
        default_duration_minutes,
        check_in_interval_minutes,
        sound_enabled,
    })
}

fn write_config(path: &PathBuf, config: &InitConfig) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("Impossible de créer le répertoire de configuration")?;
    }

    let toml_content = format!(
        r#"[tray]
enabled = {}

[focus]
default_duration_minutes = {}
check_in_interval_minutes = {}

[notifications]
sound_enabled = {}
"#,
        config.tray_enabled,
        config.default_duration_minutes,
        config.check_in_interval_minutes,
        config.sound_enabled
    );

    fs::write(path, toml_content).context("Impossible d'écrire le fichier de configuration")?;

    Ok(())
}

pub fn config_exists() -> bool {
    get_config_path().map(|path| path.exists()).unwrap_or(false)
}
