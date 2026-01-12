use std::collections::HashMap;

use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use flux_adapters::{SqliteAppTrackingRepository, SqliteSessionRepository};
use flux_core::{
    AppTrackingRepository, AppUsage, Config, DigestStats, DistractionConfig, Session,
    SessionRepository, Translator, WeekStats,
};

pub async fn execute() -> Result<()> {
    let config = Config::load().unwrap_or_default();
    let translator = Translator::new(config.general.language);
    let repository = open_repository()?;

    let digest_stats = compute_digest_stats(&repository, &config.distractions)?;

    if digest_stats.current_week.session_count == 0 {
        println!("{}", translator.get("command.digest_no_data"));
        return Ok(());
    }

    display_digest(&digest_stats, &translator);

    Ok(())
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

fn compute_digest_stats(
    repository: &SqliteSessionRepository,
    distraction_config: &DistractionConfig,
) -> Result<DigestStats> {
    let now = Utc::now();

    let current_start = now - Duration::days(7);
    let current_sessions = repository
        .find_completed_since(current_start)
        .map_err(|error| anyhow::anyhow!("read error: {}", error))?;

    let previous_start = now - Duration::days(14);
    let previous_sessions = repository
        .find_completed_between(previous_start, current_start)
        .map_err(|error| anyhow::anyhow!("read error: {}", error))?;

    let current_session_ids: Vec<i64> = current_sessions.iter().filter_map(|s| s.id).collect();
    let previous_session_ids: Vec<i64> = previous_sessions.iter().filter_map(|s| s.id).collect();

    let current_app_usages = fetch_app_tracking(&current_session_ids);
    let previous_app_usages = fetch_app_tracking(&previous_session_ids);

    let current_week =
        compute_week_stats(&current_sessions, &current_app_usages, distraction_config);
    let previous_week = if previous_sessions.is_empty() {
        None
    } else {
        Some(compute_week_stats(
            &previous_sessions,
            &previous_app_usages,
            distraction_config,
        ))
    };

    Ok(DigestStats::new(current_week, previous_week))
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

fn compute_week_stats(
    sessions: &[Session],
    app_usages: &[AppUsage],
    distraction_config: &DistractionConfig,
) -> WeekStats {
    let mut total_seconds = 0i64;
    let mut by_mode: HashMap<String, i64> = HashMap::new();

    for session in sessions {
        let duration = session.duration_seconds.unwrap_or(0);
        total_seconds += duration;

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

    WeekStats {
        total_seconds,
        session_count: sessions.len(),
        by_mode,
        focus_applications,
        distraction_applications,
        total_distraction_seconds,
        average_focus_score: 0,
    }
}

fn display_digest(stats: &DigestStats, translator: &Translator) {
    println!();
    println!("{}", translator.get("command.digest_header"));
    println!();

    let time_delta = format_delta(stats.time_delta(), translator);
    println!(
        "{:16}: {} {}",
        translator.get("command.digest_total_time"),
        format_duration(stats.current_week.total_seconds),
        time_delta
    );

    let avg_duration = if stats.current_week.session_count > 0 {
        stats.current_week.total_seconds / stats.current_week.session_count as i64
    } else {
        0
    };

    println!(
        "{:16}: {} ({}: {})",
        translator.get("command.digest_sessions"),
        stats.current_week.session_count,
        translator.get("command.digest_average"),
        format_duration(avg_duration)
    );

    println!();

    if !stats.current_week.by_mode.is_empty() {
        println!("{}:", translator.get("command.digest_by_mode"));

        let mut modes: Vec<_> = stats.current_week.by_mode.iter().collect();
        modes.sort_by(|a, b| b.1.cmp(a.1));

        let total = stats.current_week.total_seconds.max(1) as f64;

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

    display_top_apps(
        &stats.current_week.focus_applications,
        &translator.get("command.digest_top_focus"),
        5,
    );

    display_distractions(stats, translator);

    println!();
}

fn display_top_apps(applications: &HashMap<String, i64>, header: &str, limit: usize) {
    if applications.is_empty() {
        return;
    }

    println!("{}:", header);

    let mut apps: Vec<_> = applications.iter().collect();
    apps.sort_by(|a, b| b.1.cmp(a.1));

    let top_apps: Vec<_> = apps.into_iter().take(limit).collect();

    for (index, (app, seconds)) in top_apps.iter().enumerate() {
        let prefix = if index == top_apps.len() - 1 {
            "└──"
        } else {
            "├──"
        };
        println!(
            "{} {:14} {:>8}",
            prefix,
            format!("{}:", app),
            format_duration(**seconds)
        );
    }
    println!();
}

fn display_distractions(stats: &DigestStats, translator: &Translator) {
    if stats.current_week.distraction_applications.is_empty() {
        return;
    }

    let distraction_delta = format_delta(stats.distraction_delta(), translator);
    println!(
        "{} ({}{}):",
        translator.get("command.digest_distractions"),
        format_duration(stats.current_week.total_distraction_seconds),
        distraction_delta
    );

    let mut apps: Vec<_> = stats.current_week.distraction_applications.iter().collect();
    apps.sort_by(|a, b| b.1.cmp(a.1));

    let top_apps: Vec<_> = apps.into_iter().take(5).collect();

    for (index, (app, seconds)) in top_apps.iter().enumerate() {
        let prefix = if index == top_apps.len() - 1 {
            "└──"
        } else {
            "├──"
        };
        println!(
            "{} {:14} {:>8}",
            prefix,
            format!("{}:", app),
            format_duration(**seconds)
        );
    }
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

fn format_delta(delta: Option<i64>, translator: &Translator) -> String {
    match delta {
        Some(d) if d > 0 => {
            let formatted = format_duration(d);
            translator.format("command.digest_delta_positive", &[("value", &formatted)])
        }
        Some(d) if d < 0 => {
            let formatted = format_duration(d.abs());
            format!("-{}", formatted)
        }
        _ => String::new(),
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
    fn compute_week_stats_aggregates_correctly() {
        use flux_core::FocusMode;

        let sessions = vec![
            create_test_session(FocusMode::Prompting, 1800),
            create_test_session(FocusMode::Prompting, 1200),
            create_test_session(FocusMode::Review, 900),
        ];
        let distraction_config = create_test_distraction_config();

        let stats = compute_week_stats(&sessions, &[], &distraction_config);

        assert_eq!(stats.total_seconds, 3900);
        assert_eq!(stats.session_count, 3);
        assert_eq!(stats.by_mode.get("prompting"), Some(&3000));
        assert_eq!(stats.by_mode.get("review"), Some(&900));
    }

    fn create_test_session(mode: flux_core::FocusMode, duration: i64) -> Session {
        let mut session = Session::start(mode);
        session.duration_seconds = Some(duration);
        session
    }

    fn create_test_distraction_config() -> DistractionConfig {
        use std::collections::HashSet;
        DistractionConfig {
            apps: HashSet::from(["discord".to_string(), "slack".to_string()]),
            alert_enabled: false,
            alert_after_seconds: 30,
            friction_apps: HashSet::new(),
            friction_delay_seconds: 10,
        }
    }
}
