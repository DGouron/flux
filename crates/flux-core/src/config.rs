use crate::i18n::Language;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("fichier de configuration introuvable: {path}")]
    NotFound { path: String },

    #[error("erreur de lecture: {source}")]
    Read {
        #[from]
        source: std::io::Error,
    },

    #[error("erreur de parsing TOML: {source}")]
    Parse {
        #[from]
        source: toml::de::Error,
    },
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub general: GeneralConfig,
    pub focus: FocusConfig,
    pub notifications: NotificationConfig,
    pub tray: TrayConfig,
    pub distractions: DistractionConfig,
    pub gitlab: Option<ProviderConfig>,
    pub github: Option<ProviderConfig>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct GeneralConfig {
    pub language: Language,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct FocusConfig {
    pub default_duration_minutes: u64,
    pub check_in_interval_minutes: u64,
    pub check_in_timeout_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct NotificationConfig {
    pub sound_enabled: bool,
    pub urgency: NotificationUrgency,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum NotificationUrgency {
    Low,
    #[default]
    Normal,
    Critical,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProviderConfig {
    pub base_url: String,
}

impl Default for FocusConfig {
    fn default() -> Self {
        Self {
            default_duration_minutes: 25,
            check_in_interval_minutes: 25,
            check_in_timeout_seconds: 120,
        }
    }
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            sound_enabled: true,
            urgency: NotificationUrgency::Normal,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct TrayConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct DistractionConfig {
    pub apps: HashSet<String>,
    pub alert_enabled: bool,
    pub alert_after_seconds: u64,
    pub friction_apps: HashSet<String>,
    pub friction_delay_seconds: u64,
}

impl Default for DistractionConfig {
    fn default() -> Self {
        Self {
            apps: HashSet::from([
                "discord".to_string(),
                "slack".to_string(),
                "telegram".to_string(),
                "whatsapp".to_string(),
                "twitter".to_string(),
                "youtube".to_string(),
                "reddit".to_string(),
            ]),
            alert_enabled: false,
            alert_after_seconds: 30,
            friction_apps: HashSet::new(),
            friction_delay_seconds: 10,
        }
    }
}

impl DistractionConfig {
    pub fn is_distraction(&self, application_name: &str) -> bool {
        let lowercase = application_name.to_lowercase();
        self.apps.iter().any(|app| lowercase.contains(app))
    }

    pub fn is_friction(&self, application_name: &str) -> bool {
        let lowercase = application_name.to_lowercase();
        self.friction_apps.iter().any(|app| lowercase.contains(app))
    }

    pub fn add_app(&mut self, app: &str) -> bool {
        self.apps.insert(app.to_lowercase())
    }

    pub fn remove_app(&mut self, app: &str) -> bool {
        self.apps.remove(&app.to_lowercase())
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let config_path = Config::config_path();
        let content = if config_path.exists() {
            std::fs::read_to_string(&config_path)?
        } else {
            String::new()
        };

        let updated = self.update_toml_content(&content);

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&config_path, updated)?;
        Ok(())
    }

    fn update_toml_content(&self, content: &str) -> String {
        let mut lines: Vec<String> = content.lines().map(String::from).collect();
        let mut in_distractions_section = false;
        let mut apps_updated = false;
        let mut distractions_section_exists = false;

        let mut sorted_apps: Vec<_> = self.apps.iter().collect();
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
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let path = Self::config_path();

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("flux")
            .join("config.toml")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_sensible_values() {
        let config = Config::default();

        assert_eq!(config.general.language, Language::En);
        assert_eq!(config.focus.default_duration_minutes, 25);
        assert_eq!(config.focus.check_in_interval_minutes, 25);
        assert_eq!(config.focus.check_in_timeout_seconds, 120);
        assert!(config.notifications.sound_enabled);
        assert!(!config.tray.enabled);
        assert!(!config.distractions.alert_enabled);
        assert_eq!(config.distractions.alert_after_seconds, 30);
        assert!(config.distractions.apps.contains("discord"));
        assert!(config.gitlab.is_none());
        assert!(config.github.is_none());
    }

    #[test]
    fn parse_minimal_config() {
        let toml = r#"
            [focus]
            default_duration_minutes = 50
        "#;

        let config: Config = toml::from_str(toml).unwrap();

        assert_eq!(config.focus.default_duration_minutes, 50);
        assert_eq!(config.focus.check_in_interval_minutes, 25);
    }

    #[test]
    fn parse_full_config() {
        let toml = r#"
            [focus]
            default_duration_minutes = 45
            check_in_interval_minutes = 15
            check_in_timeout_seconds = 60

            [notifications]
            sound_enabled = false
            urgency = "critical"

            [gitlab]
            base_url = "https://gitlab.example.com"

            [github]
            base_url = "https://github.com"
        "#;

        let config: Config = toml::from_str(toml).unwrap();

        assert_eq!(config.focus.default_duration_minutes, 45);
        assert_eq!(config.focus.check_in_interval_minutes, 15);
        assert!(!config.notifications.sound_enabled);
        assert!(matches!(
            config.notifications.urgency,
            NotificationUrgency::Critical
        ));
        assert_eq!(
            config.gitlab.as_ref().unwrap().base_url,
            "https://gitlab.example.com"
        );
        assert_eq!(
            config.github.as_ref().unwrap().base_url,
            "https://github.com"
        );
    }

    #[test]
    fn parse_tray_config() {
        let toml = r#"
            [tray]
            enabled = true
        "#;

        let config: Config = toml::from_str(toml).unwrap();

        assert!(config.tray.enabled);
    }

    #[test]
    fn parse_language_config() {
        let toml = r#"
            [general]
            language = "fr"
        "#;

        let config: Config = toml::from_str(toml).unwrap();

        assert_eq!(config.general.language, Language::Fr);
    }

    #[test]
    fn missing_language_defaults_to_english() {
        let toml = r#"
            [focus]
            default_duration_minutes = 25
        "#;

        let config: Config = toml::from_str(toml).unwrap();

        assert_eq!(config.general.language, Language::En);
    }

    #[test]
    fn parse_distractions_config() {
        let toml = r#"
            [distractions]
            apps = ["discord", "slack", "twitter"]
            alert_enabled = true
            alert_after_seconds = 60
        "#;

        let config: Config = toml::from_str(toml).unwrap();

        assert!(config.distractions.alert_enabled);
        assert_eq!(config.distractions.alert_after_seconds, 60);
        assert_eq!(config.distractions.apps.len(), 3);
        assert!(config.distractions.apps.contains("discord"));
        assert!(config.distractions.apps.contains("slack"));
    }

    #[test]
    fn is_distraction_matches_case_insensitive() {
        let config = DistractionConfig::default();

        assert!(config.is_distraction("Discord"));
        assert!(config.is_distraction("DISCORD"));
        assert!(config.is_distraction("discord"));
    }

    #[test]
    fn is_distraction_matches_partial_name() {
        let config = DistractionConfig::default();

        assert!(config.is_distraction("Discord-canary"));
        assert!(config.is_distraction("org.telegram.desktop"));
        assert!(config.is_distraction("youtube-music"));
    }

    #[test]
    fn is_distraction_returns_false_for_non_distraction() {
        let config = DistractionConfig::default();

        assert!(!config.is_distraction("cursor"));
        assert!(!config.is_distraction("firefox"));
        assert!(!config.is_distraction("code"));
    }
}
