use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Local, NaiveDate, Utc};
use flux_adapters::{
    SqliteAppTrackingRepository, SqliteSessionMetricsRepository, SqliteSessionRepository,
};
use flux_core::{
    AppTrackingRepository, AppUsage, Config, DistractionConfig, Session, SessionId, SessionMetrics,
    SessionMetricsRepository, SessionRepository, Translator,
};

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
    pub focus_applications: HashMap<String, i64>,
    pub distraction_applications: HashMap<String, i64>,
    pub total_distraction_seconds: i64,
    pub total_check_ins: i32,
    pub average_focus_score: Option<u8>,
    pub total_context_switches: u32,
    pub total_short_bursts: u32,
    pub sessions_with_metrics: usize,
    pub short_bursts_by_app: HashMap<String, u32>,
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
    pub app_usages: Vec<AppUsage>,
    pub session_metrics: Vec<SessionMetrics>,
    pub distraction_config: DistractionConfig,
    database_path: Option<PathBuf>,
}

impl StatsData {
    pub fn stats_for_period(&self, period: Period) -> Stats {
        let filtered = self.sessions_for_period(period);
        let session_ids: Vec<i64> = filtered.iter().filter_map(|s| s.id).collect();
        let filtered_usages: Vec<&AppUsage> = self
            .app_usages
            .iter()
            .filter(|u| session_ids.contains(&u.session_id))
            .collect();
        let filtered_metrics: Vec<&SessionMetrics> = self
            .session_metrics
            .iter()
            .filter(|m| session_ids.contains(&m.session_id))
            .collect();
        compute_stats(
            &filtered,
            &filtered_usages,
            &filtered_metrics,
            &self.distraction_config,
        )
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

    pub fn delete_session(&mut self, id: SessionId) -> Result<()> {
        let database_path = self
            .database_path
            .as_ref()
            .context("database path not configured")?;

        let repository = SqliteSessionRepository::new(database_path)
            .map_err(|error| anyhow::anyhow!("database access error: {}", error))?;

        repository
            .delete_session(id)
            .map_err(|error| anyhow::anyhow!("delete error: {}", error))?;

        self.sessions.retain(|session| session.id != Some(id));
        Ok(())
    }

    pub fn clear_sessions(&mut self) -> Result<u32> {
        let database_path = self
            .database_path
            .as_ref()
            .context("database path not configured")?;

        let repository = SqliteSessionRepository::new(database_path)
            .map_err(|error| anyhow::anyhow!("database access error: {}", error))?;

        let count = repository
            .clear_completed_sessions()
            .map_err(|error| anyhow::anyhow!("clear error: {}", error))?;

        self.sessions.clear();
        Ok(count)
    }

    pub fn reload(&mut self) -> Result<()> {
        let (sessions, database_path) = load_all_sessions()?;
        let session_ids: Vec<i64> = sessions.iter().filter_map(|s| s.id).collect();
        let app_usages = load_app_usages(&session_ids, database_path.as_ref());
        let session_metrics = load_session_metrics(&session_ids, database_path.as_ref());

        self.sessions = sessions;
        self.app_usages = app_usages;
        self.session_metrics = session_metrics;
        self.database_path = database_path;

        let config = Config::load().unwrap_or_default();
        self.distraction_config = config.distractions().clone();

        Ok(())
    }

    pub fn toggle_distraction(&mut self, app_name: &str) -> Result<bool> {
        let is_distraction = self.distraction_config.is_distraction(app_name);

        if is_distraction {
            self.distraction_config.remove_app(app_name);
        } else {
            self.distraction_config.add_app(app_name);
        }

        self.distraction_config
            .save()
            .context("impossible de sauvegarder la configuration")?;

        Ok(!is_distraction)
    }

    pub fn toggle_whitelist(&mut self, app_name: &str) -> Result<bool> {
        let is_whitelisted = self.distraction_config.is_whitelisted(app_name);

        if is_whitelisted {
            self.distraction_config.remove_from_whitelist(app_name);
        } else {
            self.distraction_config.add_to_whitelist(app_name);
        }

        self.distraction_config
            .save()
            .context("impossible de sauvegarder la configuration")?;

        Ok(!is_whitelisted)
    }
}

pub fn load_initial_data() -> Result<StatsData> {
    let config = Config::load().unwrap_or_default();
    let translator = Translator::new(config.general.language);
    let distraction_config = config.distractions().clone();
    let (sessions, database_path) = load_all_sessions()?;

    let session_ids: Vec<i64> = sessions.iter().filter_map(|s| s.id).collect();
    let app_usages = load_app_usages(&session_ids, database_path.as_ref());
    let session_metrics = load_session_metrics(&session_ids, database_path.as_ref());

    Ok(StatsData {
        translator,
        sessions,
        app_usages,
        session_metrics,
        distraction_config,
        database_path,
    })
}

fn load_all_sessions() -> Result<(Vec<Session>, Option<PathBuf>)> {
    let data_dir = dirs::data_dir()
        .context("cannot find data directory")?
        .join("flux");

    let database_path = data_dir.join("sessions.db");

    if !database_path.exists() {
        return Ok((Vec::new(), None));
    }

    let repository = SqliteSessionRepository::new(&database_path)
        .map_err(|error| anyhow::anyhow!("database access error: {}", error))?;

    let since = Utc::now() - Duration::days(365);

    let sessions = repository
        .find_completed_since(since)
        .map_err(|error| anyhow::anyhow!("read error: {}", error))?;

    Ok((sessions, Some(database_path)))
}

fn load_app_usages(session_ids: &[i64], database_path: Option<&PathBuf>) -> Vec<AppUsage> {
    let Some(path) = database_path else {
        return Vec::new();
    };

    let repository = match SqliteAppTrackingRepository::new(path) {
        Ok(repo) => repo,
        Err(_) => return Vec::new(),
    };

    repository.find_by_sessions(session_ids).unwrap_or_default()
}

fn load_session_metrics(
    session_ids: &[i64],
    database_path: Option<&PathBuf>,
) -> Vec<SessionMetrics> {
    let Some(path) = database_path else {
        return Vec::new();
    };

    let repository = match SqliteSessionMetricsRepository::new(path) {
        Ok(repo) => repo,
        Err(_) => return Vec::new(),
    };

    repository.find_by_sessions(session_ids).unwrap_or_default()
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

fn compute_stats(
    sessions: &[&Session],
    app_usages: &[&AppUsage],
    session_metrics: &[&SessionMetrics],
    distraction_config: &DistractionConfig,
) -> Stats {
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

    let mut focus_applications: HashMap<String, i64> = HashMap::new();
    let mut distraction_applications: HashMap<String, i64> = HashMap::new();
    let mut total_distraction_seconds = 0i64;

    for usage in app_usages {
        if distraction_config.is_distraction(&usage.application_name) {
            *distraction_applications
                .entry(usage.application_name.clone())
                .or_insert(0) += usage.duration_seconds;
            total_distraction_seconds += usage.duration_seconds;
        } else {
            *focus_applications
                .entry(usage.application_name.clone())
                .or_insert(0) += usage.duration_seconds;
        }
    }

    let (average_focus_score, total_context_switches, total_short_bursts, short_bursts_by_app) =
        if session_metrics.is_empty() {
            (None, 0, 0, HashMap::new())
        } else {
            let sum_scores: u32 = session_metrics.iter().map(|m| m.focus_score() as u32).sum();
            let average = (sum_scores / session_metrics.len() as u32) as u8;
            let switches: u32 = session_metrics.iter().map(|m| m.context_switch_count).sum();
            let bursts: u32 = session_metrics.iter().map(|m| m.total_short_bursts).sum();

            let mut aggregated_bursts: HashMap<String, u32> = HashMap::new();
            for metrics in session_metrics {
                for (app, count) in &metrics.short_bursts_by_app {
                    *aggregated_bursts.entry(app.clone()).or_insert(0) += count;
                }
            }

            (Some(average), switches, bursts, aggregated_bursts)
        };

    Stats {
        total_seconds,
        session_count: sessions.len(),
        by_mode,
        focus_applications,
        distraction_applications,
        total_distraction_seconds,
        total_check_ins,
        average_focus_score,
        total_context_switches,
        total_short_bursts,
        sessions_with_metrics: session_metrics.len(),
        short_bursts_by_app,
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
        assert!(stats.focus_applications.is_empty());
        assert!(stats.distraction_applications.is_empty());
        assert_eq!(stats.total_distraction_seconds, 0);
        assert!(stats.average_focus_score.is_none());
        assert_eq!(stats.total_context_switches, 0);
        assert_eq!(stats.total_short_bursts, 0);
        assert_eq!(stats.sessions_with_metrics, 0);
        assert!(stats.short_bursts_by_app.is_empty());
    }
}
