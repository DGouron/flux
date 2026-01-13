use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FocusMode {
    AiAssisted,
    Review,
    Architecture,
    Veille,
    Custom(String),
}

impl FocusMode {
    pub fn as_str(&self) -> &str {
        match self {
            FocusMode::AiAssisted => "ai-assisted",
            FocusMode::Review => "review",
            FocusMode::Architecture => "architecture",
            FocusMode::Veille => "veille",
            FocusMode::Custom(name) => name,
        }
    }

    pub fn from_stored(value: &str) -> Self {
        match value {
            "prompting" | "ai-assisted" => FocusMode::AiAssisted,
            "review" => FocusMode::Review,
            "architecture" => FocusMode::Architecture,
            "veille" => FocusMode::Veille,
            other => FocusMode::Custom(other.to_string()),
        }
    }

    pub fn disables_interruptions(&self) -> bool {
        matches!(self, FocusMode::Veille)
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
            FocusMode::AiAssisted,
            FocusMode::Review,
            FocusMode::Architecture,
            FocusMode::Veille,
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

    #[test]
    fn legacy_prompting_value_maps_to_ai_assisted() {
        let restored = FocusMode::from_stored("prompting");
        assert_eq!(restored, FocusMode::AiAssisted);
    }

    #[test]
    fn veille_mode_disables_interruptions() {
        assert!(FocusMode::Veille.disables_interruptions());
        assert!(!FocusMode::AiAssisted.disables_interruptions());
        assert!(!FocusMode::Review.disables_interruptions());
        assert!(!FocusMode::Architecture.disables_interruptions());
        assert!(!FocusMode::Custom("custom".to_string()).disables_interruptions());
    }
}
