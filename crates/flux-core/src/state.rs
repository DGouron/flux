use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub active_profile: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active_profile: "default".to_string(),
        }
    }
}

impl AppState {
    pub fn load() -> Self {
        let path = Self::path();
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|content| toml::from_str(&content).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content =
            toml::to_string_pretty(self).map_err(|e| std::io::Error::other(e.to_string()))?;
        std::fs::write(path, content)
    }

    pub fn path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("flux")
            .join("state.toml")
    }

    pub fn set_active_profile(&mut self, name: &str) {
        self.active_profile = name.to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_has_default_profile() {
        let state = AppState::default();
        assert_eq!(state.active_profile, "default");
    }

    #[test]
    fn set_active_profile_updates_value() {
        let mut state = AppState::default();
        state.set_active_profile("deep_focus");
        assert_eq!(state.active_profile, "deep_focus");
    }
}
