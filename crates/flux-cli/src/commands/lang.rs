use anyhow::{Context, Result};
use flux_core::{Config, Language, Translator};
use std::fs;
use std::path::PathBuf;

pub fn execute(language: Option<String>) -> Result<()> {
    let config = Config::load().context("error.config_not_found")?;
    let translator = Translator::new(config.general.language);

    match language {
        Some(lang_code) => set_language(&lang_code, &translator),
        None => display_current_language(&config, &translator),
    }
}

fn display_current_language(config: &Config, translator: &Translator) -> Result<()> {
    let language = config.general.language;
    println!(
        "{}",
        translator.format(
            "lang.current",
            &[("name", language.display_name()), ("code", language.code())]
        )
    );
    Ok(())
}

fn set_language(lang_code: &str, translator: &Translator) -> Result<()> {
    let new_language: Language = lang_code.parse().map_err(|_| {
        anyhow::anyhow!(
            "{}",
            translator.format("lang.unsupported", &[("lang", lang_code)])
        )
    })?;

    update_config_language(new_language)?;

    let new_translator = Translator::new(new_language);
    println!(
        "{}",
        new_translator.format("lang.set", &[("name", new_language.display_name())])
    );

    Ok(())
}

fn update_config_language(language: Language) -> Result<()> {
    let config_path = get_config_path()?;
    let content = fs::read_to_string(&config_path).context("Cannot read config file")?;

    let updated_content = update_language_in_toml(&content, language);

    fs::write(&config_path, updated_content).context("Cannot write config file")?;

    Ok(())
}

fn update_language_in_toml(content: &str, language: Language) -> String {
    let mut lines: Vec<String> = content.lines().map(String::from).collect();
    let mut in_general_section = false;
    let mut language_updated = false;
    let mut general_section_exists = false;

    for line in &mut lines {
        let trimmed = line.trim();

        if trimmed.starts_with('[') {
            in_general_section = trimmed == "[general]";
            if in_general_section {
                general_section_exists = true;
            }
        }

        if in_general_section && trimmed.starts_with("language") {
            *line = format!("language = \"{}\"", language.code());
            language_updated = true;
        }
    }

    if !general_section_exists {
        let mut new_lines = vec![
            "[general]".to_string(),
            format!("language = \"{}\"", language.code()),
            String::new(),
        ];
        new_lines.extend(lines);
        return new_lines.join("\n");
    }

    if !language_updated {
        for (index, line) in lines.iter().enumerate() {
            if line.trim() == "[general]" {
                lines.insert(index + 1, format!("language = \"{}\"", language.code()));
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
    fn update_language_replaces_existing() {
        let content = r#"[general]
language = "en"

[focus]
duration = 25
"#;

        let result = update_language_in_toml(content, Language::Fr);

        assert!(result.contains("language = \"fr\""));
        assert!(!result.contains("language = \"en\""));
    }

    #[test]
    fn update_language_adds_to_existing_general_section() {
        let content = r#"[general]

[focus]
duration = 25
"#;

        let result = update_language_in_toml(content, Language::Fr);

        assert!(result.contains("language = \"fr\""));
    }

    #[test]
    fn update_language_creates_general_section() {
        let content = r#"[focus]
duration = 25
"#;

        let result = update_language_in_toml(content, Language::Fr);

        assert!(result.contains("[general]"));
        assert!(result.contains("language = \"fr\""));
    }
}
