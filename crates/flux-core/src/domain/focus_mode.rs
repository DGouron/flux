use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FocusMode {
    Prompting,
    Review,
    Architecture,
    Custom(String),
}

impl FocusMode {
    pub fn as_str(&self) -> &str {
        match self {
            FocusMode::Prompting => "prompting",
            FocusMode::Review => "review",
            FocusMode::Architecture => "architecture",
            FocusMode::Custom(name) => name,
        }
    }

    pub fn from_stored(value: &str) -> Self {
        match value {
            "prompting" => FocusMode::Prompting,
            "review" => FocusMode::Review,
            "architecture" => FocusMode::Architecture,
            other => FocusMode::Custom(other.to_string()),
        }
    }
}

impl std::fmt::Display for FocusMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_mode_roundtrip_for_known_variants() {
        let modes = [
            FocusMode::Prompting,
            FocusMode::Review,
            FocusMode::Architecture,
        ];

        for mode in modes {
            let stored = mode.as_str();
            let restored = FocusMode::from_stored(stored);
            assert_eq!(mode, restored);
        }
    }

    #[test]
    fn custom_mode_roundtrip() {
        let mode = FocusMode::Custom("deep-work".to_string());
        let stored = mode.as_str();
        let restored = FocusMode::from_stored(stored);
        assert_eq!(mode, restored);
    }

    #[test]
    fn unknown_value_becomes_custom() {
        let restored = FocusMode::from_stored("unknown-mode");
        assert_eq!(restored, FocusMode::Custom("unknown-mode".to_string()));
    }
}
