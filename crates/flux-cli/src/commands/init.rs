use anyhow::{bail, Context, Result};
use dialoguer::{Confirm, Input, Select};
use flux_core::{Language, Translator};
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
struct InitConfig {
    language: Language,
    tray_enabled: bool,
    default_duration_minutes: u64,
    check_in_interval_minutes: u64,
    sound_enabled: bool,
}

pub fn execute(force: bool) -> Result<()> {
    let config_path = get_config_path()?;

    if config_path.exists() && !force {
        let existing_translator = load_existing_translator();
        bail!(
            "{}",
            existing_translator.format(
                "init.config_exists",
                &[("path", &config_path.display().to_string())]
            )
        );
    }

    let language = prompt_language()?;
    let translator = Translator::new(language);

    if force && config_path.exists() {
        println!("{}\n", translator.get("init.overwriting"));
    }

    println!("{}\n", translator.get("init.welcome"));

    let config = prompt_configuration(language, &translator)?;
    write_config(&config_path, &config)?;

    println!(
        "\n{}",
        translator.format(
            "init.config_saved",
            &[("path", &config_path.display().to_string())]
        )
    );
    println!("{}", translator.get("init.next_step"));

    Ok(())
}

fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .context("Cannot determine config directory")?
        .join("flux");

    Ok(config_dir.join("config.toml"))
}

fn load_existing_translator() -> Translator {
    flux_core::Config::load()
        .map(|config| Translator::new(config.general.language))
        .unwrap_or_default()
}

fn prompt_language() -> Result<Language> {
    let languages = Language::available_languages();
    let items: Vec<&str> = languages.iter().map(|l| l.display_name()).collect();

    let selection = Select::new()
        .with_prompt("Choose your language / Choisissez votre langue")
        .items(&items)
        .default(0)
        .interact()?;

    Ok(languages[selection])
}

fn prompt_configuration(language: Language, translator: &Translator) -> Result<InitConfig> {
    let tray_enabled = Confirm::new()
        .with_prompt(translator.get("init.prompt_tray"))
        .default(true)
        .interact()?;

    let default_duration_minutes: u64 = Input::new()
        .with_prompt(translator.get("init.prompt_duration"))
        .default(25)
        .validate_with(|input: &u64| {
            if *input >= 1 && *input <= 480 {
                Ok(())
            } else {
                Err("Duration must be between 1 and 480 minutes")
            }
        })
        .interact_text()?;

    let check_in_interval_minutes: u64 = Input::new()
        .with_prompt(translator.get("init.prompt_check_in"))
        .default(25)
        .validate_with(|input: &u64| {
            if *input >= 5 && *input <= 120 {
                Ok(())
            } else {
                Err("Interval must be between 5 and 120 minutes")
            }
        })
        .interact_text()?;

    let sound_enabled = Confirm::new()
        .with_prompt(translator.get("init.prompt_sound"))
        .default(true)
        .interact()?;

    Ok(InitConfig {
        language,
        tray_enabled,
        default_duration_minutes,
        check_in_interval_minutes,
        sound_enabled,
    })
}

fn write_config(path: &PathBuf, config: &InitConfig) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("Cannot create config directory")?;
    }

    let toml_content = format!(
        r#"[general]
language = "{}"

[tray]
enabled = {}

[focus]
default_duration_minutes = {}
check_in_interval_minutes = {}

[notifications]
sound_enabled = {}
"#,
        config.language.code(),
        config.tray_enabled,
        config.default_duration_minutes,
        config.check_in_interval_minutes,
        config.sound_enabled
    );

    fs::write(path, toml_content).context("Cannot write config file")?;

    Ok(())
}

pub fn config_exists() -> bool {
    get_config_path().map(|path| path.exists()).unwrap_or(false)
}
