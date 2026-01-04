use super::Language;
use std::collections::HashMap;

const EN_TRANSLATIONS: &str = include_str!("locales/en.toml");
const FR_TRANSLATIONS: &str = include_str!("locales/fr.toml");

#[derive(Debug, Clone)]
pub struct Translator {
    language: Language,
    translations: HashMap<String, String>,
}

impl Translator {
    pub fn new(language: Language) -> Self {
        let content = match language {
            Language::En => EN_TRANSLATIONS,
            Language::Fr => FR_TRANSLATIONS,
        };

        let translations = parse_translations(content);

        Self {
            language,
            translations,
        }
    }

    pub fn language(&self) -> Language {
        self.language
    }

    pub fn get(&self, key: &str) -> String {
        self.translations
            .get(key)
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }

    pub fn format(&self, key: &str, args: &[(&str, &str)]) -> String {
        let mut result = self.get(key);
        for (name, value) in args {
            result = result.replace(&format!("{{{}}}", name), value);
        }
        result
    }
}

impl Default for Translator {
    fn default() -> Self {
        Self::new(Language::default())
    }
}

fn parse_translations(content: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();

    let parsed: toml::Value =
        toml::from_str(content).unwrap_or(toml::Value::Table(Default::default()));

    if let toml::Value::Table(sections) = parsed {
        for (section, values) in sections {
            if let toml::Value::Table(entries) = values {
                for (key, value) in entries {
                    if let toml::Value::String(text) = value {
                        result.insert(format!("{}.{}", section, key), text);
                    }
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translator_loads_english() {
        let translator = Translator::new(Language::En);

        assert_eq!(
            translator.get("init.welcome"),
            "Welcome to Flux! Let's configure your focus sessions."
        );
        assert_eq!(translator.get("session.started"), "Focus session started");
    }

    #[test]
    fn translator_loads_french() {
        let translator = Translator::new(Language::Fr);

        assert_eq!(
            translator.get("init.welcome"),
            "Bienvenue dans Flux ! Configurons vos sessions focus."
        );
        assert_eq!(translator.get("session.started"), "Session focus démarrée");
    }

    #[test]
    fn translator_returns_key_for_missing_translation() {
        let translator = Translator::new(Language::En);

        assert_eq!(translator.get("nonexistent.key"), "nonexistent.key");
    }

    #[test]
    fn translator_formats_placeholders() {
        let translator = Translator::new(Language::En);

        let result = translator.format("lang.current", &[("name", "English"), ("code", "en")]);
        assert_eq!(result, "Current language: English (en)");
    }

    #[test]
    fn translator_formats_french_placeholders() {
        let translator = Translator::new(Language::Fr);

        let result = translator.format("lang.current", &[("name", "Français"), ("code", "fr")]);
        assert_eq!(result, "Langue actuelle : Français (fr)");
    }
}
