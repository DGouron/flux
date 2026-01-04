use crate::i18n::Language;
use serde::Deserialize;
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
}
