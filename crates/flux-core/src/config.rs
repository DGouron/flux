use crate::i18n::Language;
use crate::state::AppState;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::LazyLock;
use thiserror::Error;

static DEFAULT_PROFILE: LazyLock<Profile> = LazyLock::new(Profile::default);

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
pub struct Profile {
    pub focus: FocusConfig,
    pub notifications: NotificationConfig,
    pub distractions: DistractionConfig,
    pub digest: DigestConfig,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub general: GeneralConfig,
    pub tray: TrayConfig,
    pub gitlab: Option<ProviderConfig>,
    pub github: Option<ProviderConfig>,
    #[serde(default)]
    pub profile: HashMap<String, Profile>,

    #[serde(default)]
    focus: Option<FocusConfig>,
    #[serde(default)]
    notifications: Option<NotificationConfig>,
    #[serde(default)]
    distractions: Option<DistractionConfig>,
    #[serde(default)]
    digest: Option<DigestConfig>,
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
pub struct DigestConfig {
    pub enabled: bool,
    pub day: String,
    pub hour: u8,
}

impl Default for DigestConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            day: "monday".to_string(),
            hour: 9,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct DistractionConfig {
    pub apps: HashSet<String>,
    pub title_patterns: HashSet<String>,
    pub alert_enabled: bool,
    pub alert_after_seconds: u64,
    pub friction_apps: HashSet<String>,
    pub friction_delay_seconds: u64,
    pub whitelist_apps: HashSet<String>,
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
            title_patterns: HashSet::new(),
            alert_enabled: false,
            alert_after_seconds: 30,
            friction_apps: HashSet::new(),
            friction_delay_seconds: 10,
            whitelist_apps: HashSet::new(),
        }
    }
}

impl DistractionConfig {
    pub fn is_distraction(&self, application_name: &str) -> bool {
        let lowercase = application_name.to_lowercase();
        self.apps.iter().any(|app| lowercase.contains(app))
    }

    pub fn is_title_distraction(&self, window_title: &str) -> bool {
        if window_title.is_empty() {
            return false;
        }
        let lowercase = window_title.to_lowercase();
        self.title_patterns
            .iter()
            .any(|pattern| lowercase.contains(pattern))
    }

    pub fn is_friction(&self, application_name: &str) -> bool {
        let lowercase = application_name.to_lowercase();
        self.friction_apps.iter().any(|app| lowercase.contains(app))
    }

    pub fn add_app(&mut self, app: &str) -> bool {
        let app_lower = app.to_lowercase();
        self.whitelist_apps.remove(&app_lower);
        self.apps.insert(app_lower)
    }

    pub fn remove_app(&mut self, app: &str) -> bool {
        self.apps.remove(&app.to_lowercase())
    }

    pub fn is_whitelisted(&self, application_name: &str) -> bool {
        let lowercase = application_name.to_lowercase();
        self.whitelist_apps
            .iter()
            .any(|app| lowercase.contains(app))
    }

    pub fn add_to_whitelist(&mut self, app: &str) -> bool {
        let app_lower = app.to_lowercase();
        self.apps.remove(&app_lower);
        self.whitelist_apps.insert(app_lower)
    }

    pub fn remove_from_whitelist(&mut self, app: &str) -> bool {
        self.whitelist_apps.remove(&app.to_lowercase())
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
        let mut whitelist_updated = false;
        let mut distractions_section_exists = false;

        let apps_line = Self::format_hashset_line("apps", &self.apps);
        let whitelist_line = Self::format_hashset_line("whitelist_apps", &self.whitelist_apps);

        for line in &mut lines {
            let trimmed = line.trim();

            if trimmed.starts_with('[') {
                in_distractions_section = trimmed == "[distractions]";
                if in_distractions_section {
                    distractions_section_exists = true;
                }
            }

            if in_distractions_section {
                if trimmed.starts_with("apps") && !trimmed.starts_with("apps =") {
                    continue;
                }
                if trimmed.starts_with("apps =") {
                    *line = apps_line.clone();
                    apps_updated = true;
                } else if trimmed.starts_with("whitelist_apps") {
                    *line = whitelist_line.clone();
                    whitelist_updated = true;
                }
            }
        }

        if !distractions_section_exists {
            if !lines.is_empty() && !lines.last().unwrap().is_empty() {
                lines.push(String::new());
            }
            lines.push("[distractions]".to_string());
            lines.push(apps_line);
            lines.push(whitelist_line);
            return lines.join("\n");
        }

        let mut insert_after_section = None;
        for (index, line) in lines.iter().enumerate() {
            if line.trim() == "[distractions]" {
                insert_after_section = Some(index);
                break;
            }
        }

        if let Some(section_index) = insert_after_section {
            if !whitelist_updated {
                lines.insert(section_index + 1, whitelist_line);
            }
            if !apps_updated {
                lines.insert(section_index + 1, apps_line);
            }
        }

        lines.join("\n")
    }

    fn format_hashset_line(key: &str, set: &HashSet<String>) -> String {
        let mut sorted: Vec<_> = set.iter().collect();
        sorted.sort();
        format!(
            "{} = [{}]",
            key,
            sorted
                .iter()
                .map(|a| format!("\"{}\"", a))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let path = Self::config_path();

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&path)?;
        let mut config: Config = toml::from_str(&content)?;
        config.migrate_legacy();
        Ok(config)
    }

    fn migrate_legacy(&mut self) {
        if self.profile.is_empty() {
            let profile = Profile {
                focus: self.focus.take().unwrap_or_default(),
                notifications: self.notifications.take().unwrap_or_default(),
                distractions: self.distractions.take().unwrap_or_default(),
                digest: self.digest.take().unwrap_or_default(),
            };
            self.profile.insert("default".to_string(), profile);
        }
    }

    pub fn active_profile(&self) -> &Profile {
        let state = AppState::load();
        self.profile
            .get(&state.active_profile)
            .or_else(|| self.profile.get("default"))
            .unwrap_or(&DEFAULT_PROFILE)
    }

    pub fn profile_names(&self) -> Vec<&str> {
        self.profile.keys().map(|s| s.as_str()).collect()
    }

    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("flux")
            .join("config.toml")
    }

    pub fn focus(&self) -> &FocusConfig {
        &self.active_profile().focus
    }

    pub fn notifications(&self) -> &NotificationConfig {
        &self.active_profile().notifications
    }

    pub fn distractions(&self) -> &DistractionConfig {
        &self.active_profile().distractions
    }

    pub fn digest(&self) -> &DigestConfig {
        &self.active_profile().digest
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_with_migration(toml_content: &str) -> Config {
        let mut config: Config = toml::from_str(toml_content).unwrap();
        config.migrate_legacy();
        config
    }

    #[test]
    fn default_profile_has_sensible_values() {
        let profile = Profile::default();

        assert_eq!(profile.focus.default_duration_minutes, 25);
        assert_eq!(profile.focus.check_in_interval_minutes, 25);
        assert_eq!(profile.focus.check_in_timeout_seconds, 120);
        assert!(profile.notifications.sound_enabled);
        assert!(!profile.distractions.alert_enabled);
        assert_eq!(profile.distractions.alert_after_seconds, 30);
        assert!(profile.distractions.apps.contains("discord"));
    }

    #[test]
    fn default_config_has_sensible_values() {
        let mut config = Config::default();
        config.migrate_legacy();

        assert_eq!(config.general.language, Language::En);
        assert!(!config.tray.enabled);
        assert!(config.gitlab.is_none());
        assert!(config.github.is_none());
        assert!(config.profile.contains_key("default"));
    }

    #[test]
    fn parse_legacy_minimal_config() {
        let config = parse_with_migration(
            r#"
            [focus]
            default_duration_minutes = 50
        "#,
        );

        assert_eq!(config.focus().default_duration_minutes, 50);
        assert_eq!(config.focus().check_in_interval_minutes, 25);
    }

    #[test]
    fn parse_legacy_full_config() {
        let config = parse_with_migration(
            r#"
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
        "#,
        );

        assert_eq!(config.focus().default_duration_minutes, 45);
        assert_eq!(config.focus().check_in_interval_minutes, 15);
        assert!(!config.notifications().sound_enabled);
        assert!(matches!(
            config.notifications().urgency,
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
        let config: Config = toml::from_str(
            r#"
            [tray]
            enabled = true
        "#,
        )
        .unwrap();

        assert!(config.tray.enabled);
    }

    #[test]
    fn parse_language_config() {
        let config: Config = toml::from_str(
            r#"
            [general]
            language = "fr"
        "#,
        )
        .unwrap();

        assert_eq!(config.general.language, Language::Fr);
    }

    #[test]
    fn missing_language_defaults_to_english() {
        let config: Config = toml::from_str(
            r#"
            [focus]
            default_duration_minutes = 25
        "#,
        )
        .unwrap();

        assert_eq!(config.general.language, Language::En);
    }

    #[test]
    fn parse_legacy_distractions_config() {
        let config = parse_with_migration(
            r#"
            [distractions]
            apps = ["discord", "slack", "twitter"]
            alert_enabled = true
            alert_after_seconds = 60
        "#,
        );

        assert!(config.distractions().alert_enabled);
        assert_eq!(config.distractions().alert_after_seconds, 60);
        assert_eq!(config.distractions().apps.len(), 3);
        assert!(config.distractions().apps.contains("discord"));
        assert!(config.distractions().apps.contains("slack"));
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

    #[test]
    fn is_title_distraction_matches_configured_patterns() {
        let mut config = DistractionConfig::default();
        config.title_patterns = HashSet::from(["youtube".to_string(), "linkedin".to_string()]);

        assert!(config.is_title_distraction("YouTube - Video Name"));
        assert!(config.is_title_distraction("linkedin.com/feed"));
        assert!(!config.is_title_distraction("localhost:3000"));
        assert!(!config.is_title_distraction("GitHub - Pull Request"));
        assert!(!config.is_title_distraction(""));
    }

    #[test]
    fn default_digest_config_is_monday_9am() {
        let config = DigestConfig::default();

        assert!(config.enabled);
        assert_eq!(config.day, "monday");
        assert_eq!(config.hour, 9);
    }

    #[test]
    fn parse_legacy_digest_config() {
        let config = parse_with_migration(
            r#"
            [digest]
            enabled = false
            day = "sunday"
            hour = 18
        "#,
        );

        assert!(!config.digest().enabled);
        assert_eq!(config.digest().day, "sunday");
        assert_eq!(config.digest().hour, 18);
    }

    #[test]
    fn parse_profile_config() {
        let config: Config = toml::from_str(
            r#"
            [general]
            language = "fr"

            [profile.default.focus]
            default_duration_minutes = 25

            [profile.deep_focus.focus]
            default_duration_minutes = 50

            [profile.deep_focus.distractions]
            apps = ["discord", "slack", "twitter"]
            alert_enabled = true
        "#,
        )
        .unwrap();

        assert_eq!(config.profile.len(), 2);
        assert!(config.profile.contains_key("default"));
        assert!(config.profile.contains_key("deep_focus"));

        let deep_focus = config.profile.get("deep_focus").unwrap();
        assert_eq!(deep_focus.focus.default_duration_minutes, 50);
        assert!(deep_focus.distractions.alert_enabled);
    }

    #[test]
    fn profile_names_returns_all_profiles() {
        let config: Config = toml::from_str(
            r#"
            [profile.default.focus]
            default_duration_minutes = 25

            [profile.work.focus]
            default_duration_minutes = 45

            [profile.creative.focus]
            default_duration_minutes = 60
        "#,
        )
        .unwrap();

        let names = config.profile_names();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"default"));
        assert!(names.contains(&"work"));
        assert!(names.contains(&"creative"));
    }
}
