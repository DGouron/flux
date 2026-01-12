use anyhow::{Context, Result};
use flux_core::{Config, DistractionConfig, Translator};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

pub fn list() -> Result<()> {
    let config = Config::load().unwrap_or_default();
    let translator = Translator::new(config.general.language);
    let distractions = config.distractions();

    println!(
        "\n{}:\n",
        translator.get("command.distractions_apps_header")
    );

    let mut apps: Vec<_> = distractions.apps.iter().collect();
    apps.sort();

    for (index, app) in apps.iter().enumerate() {
        let prefix = if index == apps.len() - 1 {
            "└──"
        } else {
            "├──"
        };
        println!("{} {}", prefix, app);
    }

    println!(
        "\n{}:\n",
        translator.get("command.distractions_patterns_header")
    );

    let mut patterns: Vec<_> = distractions.title_patterns.iter().collect();
    patterns.sort();

    for (index, pattern) in patterns.iter().enumerate() {
        let prefix = if index == patterns.len() - 1 {
            "└──"
        } else {
            "├──"
        };
        println!("{} {}", prefix, pattern);
    }

    println!();
    Ok(())
}

pub fn add(app: &str) -> Result<()> {
    let config = Config::load().unwrap_or_default();
    let translator = Translator::new(config.general.language);

    let app_lower = app.to_lowercase();

    if config.distractions().apps.contains(&app_lower) {
        println!(
            "{}",
            translator.format(
                "command.distractions_already_exists",
                &[("app", &app_lower)]
            )
        );
        return Ok(());
    }

    let mut new_apps = config.distractions().apps.clone();
    new_apps.insert(app_lower.clone());

    update_distractions_config(&new_apps)?;

    println!(
        "{}",
        translator.format("command.distractions_added", &[("app", &app_lower)])
    );
    Ok(())
}

pub fn remove(app: &str) -> Result<()> {
    let config = Config::load().unwrap_or_default();
    let translator = Translator::new(config.general.language);

    let app_lower = app.to_lowercase();

    if !config.distractions().apps.contains(&app_lower) {
        println!(
            "{}",
            translator.format("command.distractions_not_found", &[("app", &app_lower)])
        );
        return Ok(());
    }

    let mut new_apps = config.distractions().apps.clone();
    new_apps.remove(&app_lower);

    update_distractions_config(&new_apps)?;

    println!(
        "{}",
        translator.format("command.distractions_removed", &[("app", &app_lower)])
    );
    Ok(())
}

pub fn add_pattern(pattern: &str) -> Result<()> {
    let config = Config::load().unwrap_or_default();
    let translator = Translator::new(config.general.language);

    let pattern_lower = pattern.to_lowercase();

    if config
        .distractions()
        .title_patterns
        .contains(&pattern_lower)
    {
        println!(
            "{}",
            translator.format(
                "command.distractions_pattern_already_exists",
                &[("pattern", &pattern_lower)]
            )
        );
        return Ok(());
    }

    let mut distractions = config.distractions().clone();
    distractions.add_title_pattern(&pattern_lower);
    distractions
        .save()
        .context("Cannot save distractions config")?;

    println!(
        "{}",
        translator.format(
            "command.distractions_pattern_added",
            &[("pattern", &pattern_lower)]
        )
    );
    Ok(())
}

pub fn remove_pattern(pattern: &str) -> Result<()> {
    let config = Config::load().unwrap_or_default();
    let translator = Translator::new(config.general.language);

    let pattern_lower = pattern.to_lowercase();

    if !config
        .distractions()
        .title_patterns
        .contains(&pattern_lower)
    {
        println!(
            "{}",
            translator.format(
                "command.distractions_pattern_not_found",
                &[("pattern", &pattern_lower)]
            )
        );
        return Ok(());
    }

    let mut distractions = config.distractions().clone();
    distractions.remove_title_pattern(&pattern_lower);
    distractions
        .save()
        .context("Cannot save distractions config")?;

    println!(
        "{}",
        translator.format(
            "command.distractions_pattern_removed",
            &[("pattern", &pattern_lower)]
        )
    );
    Ok(())
}

pub fn reset() -> Result<()> {
    let config = Config::load().unwrap_or_default();
    let translator = Translator::new(config.general.language);

    let default_distractions = DistractionConfig::default();
    default_distractions
        .save()
        .context("Cannot save distractions config")?;

    println!("{}", translator.get("command.distractions_reset"));
    Ok(())
}

fn update_distractions_config(apps: &HashSet<String>) -> Result<()> {
    let config_path = get_config_path()?;

    let content = if config_path.exists() {
        fs::read_to_string(&config_path).context("Cannot read config file")?
    } else {
        String::new()
    };

    let updated_content = update_distractions_in_toml(&content, apps);

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).context("Cannot create config directory")?;
    }

    fs::write(&config_path, updated_content).context("Cannot write config file")?;

    Ok(())
}

fn update_distractions_in_toml(content: &str, apps: &HashSet<String>) -> String {
    let mut lines: Vec<String> = content.lines().map(String::from).collect();
    let mut in_distractions_section = false;
    let mut apps_updated = false;
    let mut distractions_section_exists = false;

    let mut sorted_apps: Vec<_> = apps.iter().collect();
    sorted_apps.sort();
    let apps_line = format!(
        "apps = [{}]",
        sorted_apps
            .iter()
            .map(|a| format!("\"{}\"", a))
            .collect::<Vec<_>>()
            .join(", ")
    );

    for line in &mut lines {
        let trimmed = line.trim();

        if trimmed.starts_with('[') {
            in_distractions_section = trimmed == "[distractions]";
            if in_distractions_section {
                distractions_section_exists = true;
            }
        }

        if in_distractions_section && trimmed.starts_with("apps") {
            *line = apps_line.clone();
            apps_updated = true;
        }
    }

    if !distractions_section_exists {
        if !lines.is_empty() && !lines.last().unwrap().is_empty() {
            lines.push(String::new());
        }
        lines.push("[distractions]".to_string());
        lines.push(apps_line);
        return lines.join("\n");
    }

    if !apps_updated {
        for (index, line) in lines.iter().enumerate() {
            if line.trim() == "[distractions]" {
                lines.insert(index + 1, apps_line);
                break;
            }
        }
    }

    lines.join("\n")
}

fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .context("Cannot determine config directory")?
        .join("flux");

    Ok(config_dir.join("config.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_distractions_replaces_existing_apps() {
        let content = r#"[distractions]
apps = ["discord", "slack"]
alert_enabled = false
"#;

        let new_apps: HashSet<String> =
            HashSet::from(["discord".to_string(), "twitter".to_string()]);
        let result = update_distractions_in_toml(content, &new_apps);

        assert!(result.contains("\"discord\""));
        assert!(result.contains("\"twitter\""));
        assert!(!result.contains("\"slack\""));
        assert!(result.contains("alert_enabled = false"));
    }

    #[test]
    fn update_distractions_creates_section_if_missing() {
        let content = r#"[focus]
duration = 25
"#;

        let new_apps: HashSet<String> = HashSet::from(["discord".to_string()]);
        let result = update_distractions_in_toml(content, &new_apps);

        assert!(result.contains("[distractions]"));
        assert!(result.contains("apps = [\"discord\"]"));
    }

    #[test]
    fn update_distractions_adds_apps_to_empty_section() {
        let content = r#"[distractions]
alert_enabled = true
"#;

        let new_apps: HashSet<String> = HashSet::from(["slack".to_string()]);
        let result = update_distractions_in_toml(content, &new_apps);

        assert!(result.contains("apps = [\"slack\"]"));
        assert!(result.contains("alert_enabled = true"));
    }

    #[test]
    fn update_distractions_sorts_apps_alphabetically() {
        let content = "";

        let new_apps: HashSet<String> = HashSet::from([
            "youtube".to_string(),
            "discord".to_string(),
            "slack".to_string(),
        ]);
        let result = update_distractions_in_toml(content, &new_apps);

        let apps_pos_discord = result.find("\"discord\"").unwrap();
        let apps_pos_slack = result.find("\"slack\"").unwrap();
        let apps_pos_youtube = result.find("\"youtube\"").unwrap();

        assert!(apps_pos_discord < apps_pos_slack);
        assert!(apps_pos_slack < apps_pos_youtube);
    }
}
