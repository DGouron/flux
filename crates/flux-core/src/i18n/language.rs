use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    En,
    Fr,
}

#[derive(Error, Debug)]
#[error("langue non supportée: {0}. Langues disponibles: en, fr")]
pub struct UnsupportedLanguageError(String);

impl Language {
    pub fn code(&self) -> &'static str {
        match self {
            Language::En => "en",
            Language::Fr => "fr",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Language::En => "English",
            Language::Fr => "Français",
        }
    }

    pub fn available_languages() -> &'static [Language] {
        &[Language::En, Language::Fr]
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl FromStr for Language {
    type Err = UnsupportedLanguageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "en" | "english" => Ok(Language::En),
            "fr" | "french" | "français" => Ok(Language::Fr),
            other => Err(UnsupportedLanguageError(other.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[test]
    fn language_from_str_parses_codes() {
        assert_eq!("en".parse::<Language>().unwrap(), Language::En);
        assert_eq!("fr".parse::<Language>().unwrap(), Language::Fr);
        assert_eq!("EN".parse::<Language>().unwrap(), Language::En);
        assert_eq!("FR".parse::<Language>().unwrap(), Language::Fr);
    }

    #[test]
    fn language_from_str_rejects_unsupported() {
        assert!("de".parse::<Language>().is_err());
        assert!("es".parse::<Language>().is_err());
    }

    #[test]
    fn language_code_returns_correct_value() {
        assert_eq!(Language::En.code(), "en");
        assert_eq!(Language::Fr.code(), "fr");
    }

    #[test]
    fn default_language_is_english() {
        assert_eq!(Language::default(), Language::En);
    }

    #[test]
    fn language_deserializes_from_toml() {
        #[derive(Deserialize)]
        struct TestConfig {
            language: Language,
        }

        let toml = r#"language = "en""#;
        let config: TestConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.language, Language::En);

        let toml = r#"language = "fr""#;
        let config: TestConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.language, Language::Fr);
    }
}
