use std::collections::HashMap;

use anyhow::{Context, Result};
use chrono::{Duration, Local, Utc};
use flux_adapters::{SqliteAppTrackingRepository, SqliteSessionRepository};
use flux_core::{AppTrackingRepository, AppUsage, Config, Session, SessionRepository, Translator};

#[derive(Debug, Clone, Copy)]
pub enum Period {
    Today,
    Week,
    Month,
    All,
}

impl Period {
    pub fn from_str(value: &str) -> Option<Self> {
        match value.to_lowercase().as_str() {
            "today" => Some(Period::Today),
            "week" => Some(Period::Week),
            "month" => Some(Period::Month),
            "all" => Some(Period::All),
            _ => None,
        }
    }

    fn label(&self, translator: &Translator) -> String {
        match self {
            Period::Today => translator.get("command.stats_period_today"),
            Period::Week => translator.get("command.stats_period_week"),
            Period::Month => translator.get("command.stats_period_month"),
            Period::All => translator.get("command.stats_period_all"),
        }
    }
}

pub async fn execute(period: Period) -> Result<()> {
    let translator = get_translator();
    let repository = open_repository()?;
    let sessions = fetch_sessions(&repository, period)?;

    if sessions.is_empty() {
        println!("{}", translator.get("command.stats_no_sessions"));
        return Ok(());
    }

    let session_ids: Vec<i64> = sessions.iter().filter_map(|s| s.id).collect();
    let app_usages = fetch_app_tracking(&session_ids);

    let stats = compute_stats(&sessions, &app_usages);
    display_stats(&stats, period, &translator);

    Ok(())
}

fn get_translator() -> Translator {
    Config::load()
        .map(|config| Translator::new(config.general.language))
        .unwrap_or_default()
}

fn open_repository() -> Result<SqliteSessionRepository> {
    let data_dir = dirs::data_dir()
        .context("cannot find data directory")?
        .join("flux");

    let database_path = data_dir.join("sessions.db");

    if !database_path.exists() {
        anyhow::bail!("no session data. Start a session first with 'flux start'.");
    }

    SqliteSessionRepository::new(&database_path)
        .map_err(|error| anyhow::anyhow!("database access error: {}", error))
}

fn fetch_sessions(repository: &SqliteSessionRepository, period: Period) -> Result<Vec<Session>> {
    let since = match period {
        Period::Today => Local::now()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc),
        Period::Week => Utc::now() - Duration::days(7),
        Period::Month => Utc::now() - Duration::days(30),
        Period::All => Utc::now() - Duration::days(365 * 10),
    };

    repository
        .find_completed_since(since)
        .map_err(|error| anyhow::anyhow!("read error: {}", error))
}

fn fetch_app_tracking(session_ids: &[i64]) -> Vec<AppUsage> {
    let data_dir = match dirs::data_dir() {
        Some(dir) => dir.join("flux"),
        None => return Vec::new(),
    };

    let database_path = data_dir.join("sessions.db");

    if !database_path.exists() {
        return Vec::new();
    }

    let repository = match SqliteAppTrackingRepository::new(&database_path) {
        Ok(repo) => repo,
        Err(_) => return Vec::new(),
    };

    repository.find_by_sessions(session_ids).unwrap_or_default()
}

struct Stats {
    total_seconds: i64,
    session_count: usize,
    by_mode: HashMap<String, i64>,
    by_application: HashMap<String, i64>,
    total_check_ins: i32,
}

fn compute_stats(sessions: &[Session], app_usages: &[AppUsage]) -> Stats {
    let mut total_seconds = 0i64;
    let mut by_mode: HashMap<String, i64> = HashMap::new();
    let mut total_check_ins = 0i32;

    for session in sessions {
        let duration = session.duration_seconds.unwrap_or(0);
        total_seconds += duration;
        total_check_ins += session.check_in_count;

        let mode_key = session.mode.to_string();
        *by_mode.entry(mode_key).or_insert(0) += duration;
    }

    let mut by_application: HashMap<String, i64> = HashMap::new();
    for usage in app_usages {
        *by_application
            .entry(usage.application_name.clone())
            .or_insert(0) += usage.duration_seconds;
    }

    Stats {
        total_seconds,
        session_count: sessions.len(),
        by_mode,
        by_application,
        total_check_ins,
    }
}

fn display_stats(stats: &Stats, period: Period, translator: &Translator) {
    println!();
    println!(
        "{} ({})",
        translator.get("command.stats_header"),
        period.label(translator)
    );
    println!();
    println!(
        "{}: {}",
        translator.get("command.stats_total_time"),
        format_duration(stats.total_seconds)
    );
    println!(
        "{}: {}",
        translator.get("command.stats_total_sessions"),
        stats.session_count
    );
    println!();

    if !stats.by_mode.is_empty() {
        println!("{}:", translator.get("command.status_mode"));

        let mut modes: Vec<_> = stats.by_mode.iter().collect();
        modes.sort_by(|a, b| b.1.cmp(a.1));

        let total = stats.total_seconds.max(1) as f64;

        for (index, (mode, seconds)) in modes.iter().enumerate() {
            let percentage = (**seconds as f64 / total * 100.0) as u32;
            let prefix = if index == modes.len() - 1 {
                "└──"
            } else {
                "├──"
            };
            println!(
                "{} {:14} {:>8} ({}%)",
                prefix,
                format!("{}:", mode),
                format_duration(**seconds),
                percentage
            );
        }
        println!();
    }

    if !stats.by_application.is_empty() {
        println!("Applications:");

        let mut apps: Vec<_> = stats.by_application.iter().collect();
        apps.sort_by(|a, b| b.1.cmp(a.1));

        let total_app_time: i64 = apps.iter().map(|(_, s)| **s).sum();
        let total = total_app_time.max(1) as f64;

        for (index, (app, seconds)) in apps.iter().enumerate() {
            let percentage = (**seconds as f64 / total * 100.0) as u32;
            let prefix = if index == apps.len() - 1 {
                "└──"
            } else {
                "├──"
            };
            println!(
                "{} {:14} {:>8} ({}%)",
                prefix,
                format!("{}:", app),
                format_duration(**seconds),
                percentage
            );
        }
        println!();
    }

    if stats.session_count > 0 {
        let avg_seconds = stats.total_seconds / stats.session_count as i64;
        println!(
            "{}: {}",
            translator.get("command.stats_average_duration"),
            format_duration(avg_seconds)
        );
    }

    if stats.total_check_ins > 0 {
        println!(
            "{}: {}",
            translator.get("command.stats_check_ins"),
            stats.total_check_ins
        );
    }

    println!();
}

fn format_duration(seconds: i64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;

    if hours > 0 {
        format!("{}h {:02}min", hours, minutes)
    } else {
        format!("{}min", minutes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_duration_shows_hours_and_minutes() {
        assert_eq!(format_duration(3661), "1h 01min");
        assert_eq!(format_duration(7200), "2h 00min");
    }

    #[test]
    fn format_duration_shows_only_minutes_when_under_hour() {
        assert_eq!(format_duration(1500), "25min");
        assert_eq!(format_duration(60), "1min");
        assert_eq!(format_duration(0), "0min");
    }

    #[test]
    fn period_from_str_parses_valid_values() {
        assert!(matches!(Period::from_str("today"), Some(Period::Today)));
        assert!(matches!(Period::from_str("WEEK"), Some(Period::Week)));
        assert!(matches!(Period::from_str("Month"), Some(Period::Month)));
        assert!(matches!(Period::from_str("all"), Some(Period::All)));
    }

    #[test]
    fn period_from_str_returns_none_for_invalid() {
        assert!(Period::from_str("invalid").is_none());
        assert!(Period::from_str("").is_none());
    }

    #[test]
    fn compute_stats_aggregates_correctly() {
        use flux_core::FocusMode;

        let sessions = vec![
            create_test_session(FocusMode::Prompting, 1800, 2),
            create_test_session(FocusMode::Prompting, 1200, 1),
            create_test_session(FocusMode::Review, 900, 0),
        ];

        let stats = compute_stats(&sessions, &[]);

        assert_eq!(stats.total_seconds, 3900);
        assert_eq!(stats.session_count, 3);
        assert_eq!(stats.by_mode.get("prompting"), Some(&3000));
        assert_eq!(stats.by_mode.get("review"), Some(&900));
        assert_eq!(stats.total_check_ins, 3);
        assert!(stats.by_application.is_empty());
    }

    #[test]
    fn compute_stats_aggregates_app_usage() {
        use flux_core::FocusMode;

        let sessions = vec![create_test_session(FocusMode::Prompting, 1800, 0)];
        let app_usages = vec![
            AppUsage::with_duration(1, "cursor".to_string(), 1000),
            AppUsage::with_duration(1, "firefox".to_string(), 500),
        ];

        let stats = compute_stats(&sessions, &app_usages);

        assert_eq!(stats.by_application.get("cursor"), Some(&1000));
        assert_eq!(stats.by_application.get("firefox"), Some(&500));
    }

    fn create_test_session(mode: flux_core::FocusMode, duration: i64, check_ins: i32) -> Session {
        let mut session = Session::start(mode);
        session.duration_seconds = Some(duration);
        session.check_in_count = check_ins;
        session
    }
}
