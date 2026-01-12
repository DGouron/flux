use std::sync::Arc;

use chrono::{Datelike, Duration as ChronoDuration, Local, NaiveTime, Utc, Weekday};
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};
use tracing::{debug, info, warn};

use flux_core::{
    AppTrackingRepository, DigestConfig, DistractionConfig, Session, SessionRepository,
};

use super::NotifierHandle;

pub struct DigestSchedulerActor {
    notifier: NotifierHandle,
    config: DigestConfig,
    #[allow(dead_code)]
    distraction_config: DistractionConfig,
    session_repository: Arc<dyn SessionRepository>,
    #[allow(dead_code)]
    app_tracking_repository: Arc<dyn AppTrackingRepository>,
}

impl DigestSchedulerActor {
    pub fn new(
        notifier: NotifierHandle,
        config: DigestConfig,
        distraction_config: DistractionConfig,
        session_repository: Arc<dyn SessionRepository>,
        app_tracking_repository: Arc<dyn AppTrackingRepository>,
    ) -> Self {
        Self {
            notifier,
            config,
            distraction_config,
            session_repository,
            app_tracking_repository,
        }
    }

    pub async fn run(self, mut shutdown: broadcast::Receiver<()>) {
        if !self.config.enabled {
            info!("digest scheduler disabled");
            return;
        }

        info!("digest scheduler started");

        loop {
            let sleep_duration = self.calculate_next_digest_delay();
            let hours = sleep_duration.as_secs() / 3600;
            let minutes = (sleep_duration.as_secs() % 3600) / 60;
            info!(hours, minutes, "next digest scheduled");

            tokio::select! {
                _ = sleep(sleep_duration) => {
                    self.send_digest();
                }
                _ = shutdown.recv() => {
                    debug!("digest scheduler shutdown");
                    break;
                }
            }
        }
    }

    fn calculate_next_digest_delay(&self) -> Duration {
        let now = Local::now();
        let target_weekday = parse_weekday(&self.config.day);
        let target_time = NaiveTime::from_hms_opt(self.config.hour as u32, 0, 0)
            .unwrap_or_else(|| NaiveTime::from_hms_opt(9, 0, 0).unwrap());

        let current_weekday = now.weekday();
        let days_until = (target_weekday.num_days_from_monday() as i64
            - current_weekday.num_days_from_monday() as i64
            + 7)
            % 7;

        let days_until = if days_until == 0 && now.time() >= target_time {
            7
        } else {
            days_until
        };

        let target_date = now.date_naive() + ChronoDuration::days(days_until);
        let target_datetime = target_date.and_time(target_time);

        match target_datetime.and_local_timezone(Local) {
            chrono::LocalResult::Single(target) => {
                let diff = target.signed_duration_since(now);
                Duration::from_secs(diff.num_seconds().max(0) as u64)
            }
            _ => Duration::from_secs(7 * 24 * 3600),
        }
    }

    fn send_digest(&self) {
        info!("computing weekly digest");

        let now = Utc::now();
        let week_start = now - ChronoDuration::days(7);

        let sessions = match self.session_repository.find_completed_since(week_start) {
            Ok(s) => s,
            Err(error) => {
                warn!(%error, "failed to fetch sessions for digest");
                return;
            }
        };

        if sessions.is_empty() {
            debug!("no sessions for weekly digest");
            return;
        }

        let total_seconds = compute_total_time(&sessions);
        let total_time = format_duration(total_seconds);
        let session_count = sessions.len();

        self.notifier.send_weekly_digest(total_time, session_count);
    }
}

fn parse_weekday(day: &str) -> Weekday {
    match day.to_lowercase().as_str() {
        "monday" => Weekday::Mon,
        "tuesday" => Weekday::Tue,
        "wednesday" => Weekday::Wed,
        "thursday" => Weekday::Thu,
        "friday" => Weekday::Fri,
        "saturday" => Weekday::Sat,
        "sunday" => Weekday::Sun,
        _ => Weekday::Mon,
    }
}

fn compute_total_time(sessions: &[Session]) -> i64 {
    sessions
        .iter()
        .map(|s| s.duration_seconds.unwrap_or(0))
        .sum()
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
    fn parse_weekday_handles_all_days() {
        assert_eq!(parse_weekday("monday"), Weekday::Mon);
        assert_eq!(parse_weekday("TUESDAY"), Weekday::Tue);
        assert_eq!(parse_weekday("Wednesday"), Weekday::Wed);
        assert_eq!(parse_weekday("thursday"), Weekday::Thu);
        assert_eq!(parse_weekday("friday"), Weekday::Fri);
        assert_eq!(parse_weekday("saturday"), Weekday::Sat);
        assert_eq!(parse_weekday("sunday"), Weekday::Sun);
    }

    #[test]
    fn parse_weekday_defaults_to_monday() {
        assert_eq!(parse_weekday("invalid"), Weekday::Mon);
        assert_eq!(parse_weekday(""), Weekday::Mon);
    }

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
}
