use anyhow::{bail, Result};
use flux_core::{AppState, Config, Translator};

pub fn list() -> Result<()> {
    let config = Config::load().unwrap_or_default();
    let state = AppState::load();
    let translator = Translator::new(config.general.language);

    println!("\n{}:\n", translator.get("command.profile_list_header"));

    let mut names: Vec<_> = config.profile_names();
    names.sort();

    for (index, name) in names.iter().enumerate() {
        let marker = if *name == state.active_profile {
            "●"
        } else {
            " "
        };
        let prefix = if index == names.len() - 1 {
            "└──"
        } else {
            "├──"
        };
        println!("{} {} {}", prefix, marker, name);
    }

    println!();
    Ok(())
}

pub fn show(name: Option<String>) -> Result<()> {
    let config = Config::load().unwrap_or_default();
    let state = AppState::load();
    let translator = Translator::new(config.general.language);

    let profile_name = name.as_deref().unwrap_or(&state.active_profile);
    let profile = config.profile.get(profile_name).ok_or_else(|| {
        anyhow::anyhow!(translator.format("command.profile_not_found", &[("name", profile_name)]))
    })?;

    let active_marker = if profile_name == state.active_profile {
        format!(" ({})", translator.get("command.profile_active"))
    } else {
        String::new()
    };

    println!(
        "\n{}: {}{}\n",
        translator.get("command.profile_header"),
        profile_name,
        active_marker
    );

    println!("[focus]");
    println!(
        "  duration_minutes = {}",
        profile.focus.default_duration_minutes
    );
    println!(
        "  check_in_interval_minutes = {}",
        profile.focus.check_in_interval_minutes
    );
    println!(
        "  check_in_timeout_seconds = {}",
        profile.focus.check_in_timeout_seconds
    );

    println!("\n[distractions]");
    let mut apps: Vec<_> = profile.distractions.apps.iter().collect();
    apps.sort();
    println!(
        "  apps = [{}]",
        apps.iter()
            .map(|a| format!("\"{}\"", a))
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!("  alert_enabled = {}", profile.distractions.alert_enabled);
    println!(
        "  alert_after_seconds = {}",
        profile.distractions.alert_after_seconds
    );
    if !profile.distractions.friction_apps.is_empty() {
        let mut friction: Vec<_> = profile.distractions.friction_apps.iter().collect();
        friction.sort();
        println!(
            "  friction_apps = [{}]",
            friction
                .iter()
                .map(|a| format!("\"{}\"", a))
                .collect::<Vec<_>>()
                .join(", ")
        );
        println!(
            "  friction_delay_seconds = {}",
            profile.distractions.friction_delay_seconds
        );
    }

    println!("\n[notifications]");
    println!("  sound_enabled = {}", profile.notifications.sound_enabled);
    println!("  urgency = {:?}", profile.notifications.urgency);

    println!("\n[digest]");
    println!("  enabled = {}", profile.digest.enabled);
    println!("  day = \"{}\"", profile.digest.day);
    println!("  hour = {}", profile.digest.hour);

    println!();
    Ok(())
}

pub fn use_profile(name: &str) -> Result<()> {
    let config = Config::load().unwrap_or_default();
    let translator = Translator::new(config.general.language);

    if !config.profile.contains_key(name) {
        bail!(translator.format("command.profile_not_found", &[("name", name)]));
    }

    let mut state = AppState::load();
    state.set_active_profile(name);
    state.save()?;

    println!(
        "{}",
        translator.format("command.profile_switched", &[("name", name)])
    );
    Ok(())
}
