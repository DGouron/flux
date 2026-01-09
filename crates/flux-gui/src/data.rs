use std::collections::HashMap;

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Local, NaiveDate, Utc};
use flux_adapters::SqliteSessionRepository;
use flux_core::{Config, Session, SessionRepository, Translator};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Period {
    Today,
    Week,
    Month,
    All,
}

impl Period {
    pub fn label(&self, translator: &Translator) -> String {
        match self {
            Period::Today => translator.get("command.stats_period_today"),
            Period::Week => translator.get("command.stats_period_week"),
            Period::Month => translator.get("command.stats_period_month"),
            Period::All => translator.get("command.stats_period_all"),
        }
    }

    pub fn all() -> &'static [Period] {
        &[Period::Today, Period::Week, Period::Month, Period::All]
    }
}

#[derive(Debug, Clone, Default)]
pub struct Stats {
    pub total_seconds: i64,
    pub session_count: usize,
    pub by_mode: HashMap<String, i64>,
    pub total_check_ins: i32,
}

#[derive(Debug, Clone)]
pub struct DailyFocus {
    pub date: NaiveDate,
    pub minutes: i64,
    pub session_count: usize,
}

#[derive(Debug)]
pub struct StatsData {
    pub translator: Translator,
    pub sessions: Vec<Session>,
}

impl StatsData {
    pub fn stats_for_period(&self, period: Period) -> Stats {
        let filtered = self.sessions_for_period(period);
        compute_stats(&filtered)
    }

    pub fn sessions_for_period(&self, period: Period) -> Vec<&Session> {
        let since = period_start(period);
        self.sessions
            .iter()
            .filter(|session| session.started_at >= since)
            .collect()
    }

    pub fn daily_focus_for_period(&self, period: Period) -> Vec<DailyFocus> {
        let sessions = self.sessions_for_period(period);

        let mut by_day: HashMap<NaiveDate, (i64, usize)> = HashMap::new();

        for session in sessions {
            let local_date: DateTime<Local> = session.started_at.into();
            let date = local_date.date_naive();
            let seconds = session.duration_seconds.unwrap_or(0);

            let entry = by_day.entry(date).or_insert((0, 0));
            entry.0 += seconds;
            entry.1 += 1;
        }

        let mut daily: Vec<DailyFocus> = by_day
            .into_iter()
            .map(|(date, (seconds, count))| DailyFocus {
                date,
                minutes: seconds / 60,
                session_count: count,
            })
            .collect();

        daily.sort_by_key(|day| day.date);
        daily
    }

    pub fn has_sessions(&self) -> bool {
        !self.sessions.is_empty()
    }
}

pub fn load_initial_data() -> Result<StatsData> {
    let translator = get_translator();
    let sessions = load_all_sessions()?;

    Ok(StatsData {
        translator,
        sessions,
    })
}

fn get_translator() -> Translator {
    Config::load()
        .map(|config| Translator::new(config.general.language))
        .unwrap_or_default()
}

fn load_all_sessions() -> Result<Vec<Session>> {
    let data_dir = dirs::data_dir()
        .context("cannot find data directory")?
        .join("flux");

    let database_path = data_dir.join("sessions.db");

    if !database_path.exists() {
        return Ok(Vec::new());
    }

    let repository = SqliteSessionRepository::new(&database_path)
        .map_err(|error| anyhow::anyhow!("database access error: {}", error))?;

    let since = Utc::now() - Duration::days(365);

    repository
        .find_completed_since(since)
        .map_err(|error| anyhow::anyhow!("read error: {}", error))
}

fn period_start(period: Period) -> DateTime<Utc> {
    match period {
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
    }
}

fn compute_stats(sessions: &[&Session]) -> Stats {
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

    Stats {
        total_seconds,
        session_count: sessions.len(),
        by_mode,
        total_check_ins,
    }
}

pub fn format_duration(seconds: i64) -> String {
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
    fn stats_default_is_empty() {
        let stats = Stats::default();
        assert_eq!(stats.total_seconds, 0);
        assert_eq!(stats.session_count, 0);
        assert!(stats.by_mode.is_empty());
    }
}
