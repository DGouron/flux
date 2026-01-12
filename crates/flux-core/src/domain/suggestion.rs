use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistractionSuggestion {
    pub application_name: String,
    pub short_burst_count: u32,
    pub reason: SuggestionReason,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionReason {
    FrequentShortBursts,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SuggestionReport {
    pub generated_at: Option<DateTime<Utc>>,
    pub session_id: Option<i64>,
    pub context_switch_count: u32,
    pub suggestions: Vec<DistractionSuggestion>,
}

const MIN_SHORT_BURSTS_FOR_SUGGESTION: u32 = 3;

impl SuggestionReport {
    pub fn from_session_data(
        session_id: i64,
        short_burst_count: &HashMap<String, u32>,
        context_switch_count: u32,
        existing_distractions: &std::collections::HashSet<String>,
        whitelist_apps: &std::collections::HashSet<String>,
    ) -> Self {
        let mut suggestions: Vec<DistractionSuggestion> = short_burst_count
            .iter()
            .filter(|(app, count)| {
                let app_lower = app.to_lowercase();
                let is_distraction = existing_distractions.iter().any(|d| app_lower.contains(d));
                let is_whitelisted = whitelist_apps.iter().any(|w| app_lower.contains(w));
                **count >= MIN_SHORT_BURSTS_FOR_SUGGESTION && !is_distraction && !is_whitelisted
            })
            .map(|(app, count)| DistractionSuggestion {
                application_name: app.clone(),
                short_burst_count: *count,
                reason: SuggestionReason::FrequentShortBursts,
            })
            .collect();

        suggestions.sort_by(|a, b| b.short_burst_count.cmp(&a.short_burst_count));

        Self {
            generated_at: Some(Utc::now()),
            session_id: Some(session_id),
            context_switch_count,
            suggestions,
        }
    }

    pub fn save(&self) -> Result<PathBuf, std::io::Error> {
        let path = Self::file_path()?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;

        fs::write(&path, content)?;

        Ok(path)
    }

    pub fn load() -> Result<Self, std::io::Error> {
        let path = Self::file_path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)?;

        serde_json::from_str(&content)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
    }

    pub fn clear() -> Result<(), std::io::Error> {
        let path = Self::file_path()?;

        if path.exists() {
            fs::remove_file(&path)?;
        }

        Ok(())
    }

    fn file_path() -> Result<PathBuf, std::io::Error> {
        dirs::config_dir()
            .map(|dir| dir.join("flux").join("suggestions.json"))
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "config directory not found")
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn from_session_data_filters_below_threshold() {
        let mut short_burst_count = HashMap::new();
        short_burst_count.insert("discord".to_string(), 2);
        short_burst_count.insert("slack".to_string(), 5);

        let existing = HashSet::new();
        let whitelist = HashSet::new();
        let report =
            SuggestionReport::from_session_data(1, &short_burst_count, 10, &existing, &whitelist);

        assert_eq!(report.suggestions.len(), 1);
        assert_eq!(report.suggestions[0].application_name, "slack");
    }

    #[test]
    fn from_session_data_excludes_existing_distractions() {
        let mut short_burst_count = HashMap::new();
        short_burst_count.insert("discord".to_string(), 10);
        short_burst_count.insert("slack".to_string(), 5);

        let mut existing = HashSet::new();
        existing.insert("discord".to_string());
        let whitelist = HashSet::new();

        let report =
            SuggestionReport::from_session_data(1, &short_burst_count, 10, &existing, &whitelist);

        assert_eq!(report.suggestions.len(), 1);
        assert_eq!(report.suggestions[0].application_name, "slack");
    }

    #[test]
    fn from_session_data_excludes_whitelisted_apps() {
        let mut short_burst_count = HashMap::new();
        short_burst_count.insert("cursor".to_string(), 10);
        short_burst_count.insert("slack".to_string(), 5);

        let existing = HashSet::new();
        let mut whitelist = HashSet::new();
        whitelist.insert("cursor".to_string());

        let report =
            SuggestionReport::from_session_data(1, &short_burst_count, 10, &existing, &whitelist);

        assert_eq!(report.suggestions.len(), 1);
        assert_eq!(report.suggestions[0].application_name, "slack");
    }

    #[test]
    fn from_session_data_sorts_by_burst_count() {
        let mut short_burst_count = HashMap::new();
        short_burst_count.insert("twitter".to_string(), 5);
        short_burst_count.insert("youtube".to_string(), 15);
        short_burst_count.insert("reddit".to_string(), 8);

        let existing = HashSet::new();
        let whitelist = HashSet::new();
        let report =
            SuggestionReport::from_session_data(1, &short_burst_count, 10, &existing, &whitelist);

        assert_eq!(report.suggestions.len(), 3);
        assert_eq!(report.suggestions[0].application_name, "youtube");
        assert_eq!(report.suggestions[1].application_name, "reddit");
        assert_eq!(report.suggestions[2].application_name, "twitter");
    }
}
