use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc;
use tracing::{debug, error, info, trace, warn};

use flux_core::{
    AppTrackingRepository, AppUsage, Config, DistractionConfig, SessionId, Translator,
};

use super::NotifierHandle;

#[cfg(target_os = "linux")]
use crate::window::{WindowDetector, X11WindowDetector};

const POLLING_INTERVAL_SECONDS: u64 = 5;

pub enum AppTrackerMessage {
    Started { session_id: SessionId },
    Ended,
    Paused,
    Resumed,
}

#[derive(Clone)]
pub struct AppTrackerHandle {
    sender: mpsc::Sender<AppTrackerMessage>,
}

impl AppTrackerHandle {
    pub fn send_session_started(&self, session_id: SessionId) {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            if let Err(error) = sender.send(AppTrackerMessage::Started { session_id }).await {
                error!(%error, "failed to send session started message to app tracker");
            }
        });
    }

    pub fn send_session_ended(&self) {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            if let Err(error) = sender.send(AppTrackerMessage::Ended).await {
                error!(%error, "failed to send session ended message to app tracker");
            }
        });
    }

    pub fn send_session_paused(&self) {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            if let Err(error) = sender.send(AppTrackerMessage::Paused).await {
                error!(%error, "failed to send session paused message to app tracker");
            }
        });
    }

    pub fn send_session_resumed(&self) {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            if let Err(error) = sender.send(AppTrackerMessage::Resumed).await {
                error!(%error, "failed to send session resumed message to app tracker");
            }
        });
    }
}

struct TrackerState {
    session_id: SessionId,
    paused: bool,
    accumulated: HashMap<String, i64>,
    current_distraction: Option<String>,
    distraction_consecutive_seconds: u64,
    distraction_alert_sent: bool,
}

pub struct AppTrackerActor {
    receiver: mpsc::Receiver<AppTrackerMessage>,
    repository: Arc<dyn AppTrackingRepository>,
    distraction_config: DistractionConfig,
    notifier: NotifierHandle,
    #[cfg(target_os = "linux")]
    detector: Option<X11WindowDetector>,
    state: Option<TrackerState>,
}

impl AppTrackerActor {
    #[cfg(target_os = "linux")]
    pub fn new(
        repository: Arc<dyn AppTrackingRepository>,
        distraction_config: DistractionConfig,
        notifier: NotifierHandle,
    ) -> (Self, AppTrackerHandle) {
        let (sender, receiver) = mpsc::channel(32);

        let detector = X11WindowDetector::new();
        if detector.is_none() {
            warn!("X11 window detection not available, app tracking will be disabled");
        }

        let actor = Self {
            receiver,
            repository,
            distraction_config,
            notifier,
            detector,
            state: None,
        };

        let handle = AppTrackerHandle { sender };

        (actor, handle)
    }

    #[cfg(not(target_os = "linux"))]
    pub fn new(
        repository: Arc<dyn AppTrackingRepository>,
        distraction_config: DistractionConfig,
        notifier: NotifierHandle,
    ) -> (Self, AppTrackerHandle) {
        let (sender, receiver) = mpsc::channel(32);

        let actor = Self {
            receiver,
            repository,
            distraction_config,
            notifier,
            state: None,
        };

        let handle = AppTrackerHandle { sender };

        (actor, handle)
    }

    pub async fn run(mut self) {
        info!("app tracker actor started");

        let mut poll_interval =
            tokio::time::interval(Duration::from_secs(POLLING_INTERVAL_SECONDS));

        loop {
            tokio::select! {
                Some(message) = self.receiver.recv() => {
                    self.handle_message(message);
                }
                _ = poll_interval.tick() => {
                    self.poll_active_window();
                }
                else => break,
            }
        }

        debug!("app tracker actor stopped");
    }

    fn handle_message(&mut self, message: AppTrackerMessage) {
        match message {
            AppTrackerMessage::Started { session_id } => {
                debug!(session_id, "app tracking started for session");
                self.state = Some(TrackerState {
                    session_id,
                    paused: false,
                    accumulated: HashMap::new(),
                    current_distraction: None,
                    distraction_consecutive_seconds: 0,
                    distraction_alert_sent: false,
                });
            }
            AppTrackerMessage::Ended => {
                if let Some(state) = self.state.take() {
                    Self::flush_to_repository(&self.repository, &state);
                    debug!(
                        session_id = state.session_id,
                        "app tracking ended for session"
                    );
                }
            }
            AppTrackerMessage::Paused => {
                if let Some(mut state) = self.state.take() {
                    state.paused = true;
                    Self::flush_to_repository(&self.repository, &state);
                    state.accumulated.clear();
                    state.current_distraction = None;
                    state.distraction_consecutive_seconds = 0;
                    state.distraction_alert_sent = false;
                    self.state = Some(state);
                    debug!("app tracking paused");
                }
            }
            AppTrackerMessage::Resumed => {
                if let Some(ref mut state) = self.state {
                    state.paused = false;
                    debug!("app tracking resumed");
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    fn poll_active_window(&mut self) {
        let Some(ref mut state) = self.state else {
            return;
        };

        if state.paused {
            return;
        }

        let Some(ref detector) = self.detector else {
            return;
        };

        let Some(application_name) = detector.get_active_application() else {
            return;
        };

        trace!(application_name = %application_name, "tracking active window");
        *state
            .accumulated
            .entry(application_name.clone())
            .or_insert(0) += POLLING_INTERVAL_SECONDS as i64;

        self.track_distraction(&application_name);
    }

    fn track_distraction(&mut self, application_name: &str) {
        let Some(ref mut state) = self.state else {
            return;
        };

        let is_distraction = self.distraction_config.is_distraction(application_name);

        if is_distraction {
            let same_distraction = state
                .current_distraction
                .as_ref()
                .map(|current| current == application_name)
                .unwrap_or(false);

            if same_distraction {
                state.distraction_consecutive_seconds += POLLING_INTERVAL_SECONDS;
            } else {
                state.current_distraction = Some(application_name.to_string());
                state.distraction_consecutive_seconds = POLLING_INTERVAL_SECONDS;
                state.distraction_alert_sent = false;
            }

            self.maybe_send_distraction_alert();
        } else {
            state.current_distraction = None;
            state.distraction_consecutive_seconds = 0;
            state.distraction_alert_sent = false;
        }
    }

    fn maybe_send_distraction_alert(&mut self) {
        if !self.distraction_config.alert_enabled {
            return;
        }

        let Some(ref mut state) = self.state else {
            return;
        };

        if state.distraction_alert_sent {
            return;
        }

        if state.distraction_consecutive_seconds < self.distraction_config.alert_after_seconds {
            return;
        }

        let Some(ref app) = state.current_distraction else {
            return;
        };

        let translator = Config::load()
            .map(|config| Translator::new(config.general.language))
            .unwrap_or_default();

        let title = format!(
            "Flux - {}",
            translator.get("notification.distraction_alert_title")
        );
        let body = translator.format(
            "notification.distraction_alert_body",
            &[
                ("app", app),
                (
                    "seconds",
                    &state.distraction_consecutive_seconds.to_string(),
                ),
            ],
        );

        self.notifier.send_alert(title, body);
        state.distraction_alert_sent = true;

        debug!(
            app,
            seconds = state.distraction_consecutive_seconds,
            "distraction alert sent"
        );
    }

    #[cfg(not(target_os = "linux"))]
    fn poll_active_window(&mut self) {
        // No-op on non-Linux platforms
    }

    fn flush_to_repository(repository: &Arc<dyn AppTrackingRepository>, state: &TrackerState) {
        for (application_name, seconds) in &state.accumulated {
            if *seconds > 0 {
                let usage =
                    AppUsage::with_duration(state.session_id, application_name.clone(), *seconds);

                if let Err(error) = repository.save_or_update(&usage) {
                    error!(%error, application_name, "failed to persist app usage");
                } else {
                    debug!(
                        session_id = state.session_id,
                        application_name, seconds, "flushed app usage to database"
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_core::{AppTrackingRepositoryError, NotificationUrgency};
    use std::collections::HashSet;
    use std::sync::Mutex;

    struct MockRepository {
        saved: Mutex<Vec<AppUsage>>,
    }

    impl MockRepository {
        fn new() -> Self {
            Self {
                saved: Mutex::new(Vec::new()),
            }
        }
    }

    impl AppTrackingRepository for MockRepository {
        fn save_or_update(&self, usage: &AppUsage) -> Result<(), AppTrackingRepositoryError> {
            self.saved.lock().unwrap().push(usage.clone());
            Ok(())
        }

        fn find_by_session(
            &self,
            _session_id: SessionId,
        ) -> Result<Vec<AppUsage>, AppTrackingRepositoryError> {
            Ok(Vec::new())
        }

        fn find_by_sessions(
            &self,
            _session_ids: &[SessionId],
        ) -> Result<Vec<AppUsage>, AppTrackingRepositoryError> {
            Ok(Vec::new())
        }

        fn delete_by_session(
            &self,
            _session_id: SessionId,
        ) -> Result<(), AppTrackingRepositoryError> {
            Ok(())
        }
    }

    fn create_test_notifier() -> NotifierHandle {
        let (_, handle) = super::super::NotifierActor::new(NotificationUrgency::Normal, false);
        handle
    }

    fn create_test_distraction_config() -> DistractionConfig {
        DistractionConfig {
            apps: HashSet::from(["discord".to_string(), "slack".to_string()]),
            alert_enabled: false,
            alert_after_seconds: 30,
        }
    }

    #[tokio::test]
    async fn handle_can_send_messages() {
        let repository = Arc::new(MockRepository::new());
        let (actor, handle) = AppTrackerActor::new(
            repository,
            create_test_distraction_config(),
            create_test_notifier(),
        );

        let actor_task = tokio::spawn(async move {
            tokio::time::timeout(Duration::from_millis(100), actor.run()).await
        });

        handle.send_session_started(1);
        handle.send_session_paused();
        handle.send_session_resumed();
        handle.send_session_ended();

        tokio::time::sleep(Duration::from_millis(50)).await;
        drop(handle);

        let _ = actor_task.await;
    }

    #[tokio::test]
    async fn session_end_flushes_accumulated_data() {
        let repository = Arc::new(MockRepository::new());
        let repository_clone = repository.clone();
        let (mut actor, _handle) = AppTrackerActor::new(
            repository,
            create_test_distraction_config(),
            create_test_notifier(),
        );

        actor.state = Some(TrackerState {
            session_id: 42,
            paused: false,
            accumulated: HashMap::from([("cursor".to_string(), 100), ("firefox".to_string(), 50)]),
            current_distraction: None,
            distraction_consecutive_seconds: 0,
            distraction_alert_sent: false,
        });

        actor.handle_message(AppTrackerMessage::Ended);

        let saved = repository_clone.saved.lock().unwrap();
        assert_eq!(saved.len(), 2);
    }

    #[test]
    fn track_distraction_increments_consecutive_seconds() {
        let repository = Arc::new(MockRepository::new());
        let (mut actor, _handle) = AppTrackerActor::new(
            repository,
            create_test_distraction_config(),
            create_test_notifier(),
        );

        actor.state = Some(TrackerState {
            session_id: 1,
            paused: false,
            accumulated: HashMap::new(),
            current_distraction: None,
            distraction_consecutive_seconds: 0,
            distraction_alert_sent: false,
        });

        actor.track_distraction("Discord");

        let state = actor.state.as_ref().unwrap();
        assert_eq!(state.current_distraction, Some("Discord".to_string()));
        assert_eq!(
            state.distraction_consecutive_seconds,
            POLLING_INTERVAL_SECONDS
        );

        actor.track_distraction("Discord");

        let state = actor.state.as_ref().unwrap();
        assert_eq!(
            state.distraction_consecutive_seconds,
            POLLING_INTERVAL_SECONDS * 2
        );
    }

    #[test]
    fn track_distraction_resets_when_switching_apps() {
        let repository = Arc::new(MockRepository::new());
        let (mut actor, _handle) = AppTrackerActor::new(
            repository,
            create_test_distraction_config(),
            create_test_notifier(),
        );

        actor.state = Some(TrackerState {
            session_id: 1,
            paused: false,
            accumulated: HashMap::new(),
            current_distraction: Some("Discord".to_string()),
            distraction_consecutive_seconds: 60,
            distraction_alert_sent: true,
        });

        actor.track_distraction("cursor");

        let state = actor.state.as_ref().unwrap();
        assert_eq!(state.current_distraction, None);
        assert_eq!(state.distraction_consecutive_seconds, 0);
        assert!(!state.distraction_alert_sent);
    }

    #[test]
    fn track_distraction_resets_when_switching_distractions() {
        let repository = Arc::new(MockRepository::new());
        let (mut actor, _handle) = AppTrackerActor::new(
            repository,
            create_test_distraction_config(),
            create_test_notifier(),
        );

        actor.state = Some(TrackerState {
            session_id: 1,
            paused: false,
            accumulated: HashMap::new(),
            current_distraction: Some("Discord".to_string()),
            distraction_consecutive_seconds: 60,
            distraction_alert_sent: true,
        });

        actor.track_distraction("Slack");

        let state = actor.state.as_ref().unwrap();
        assert_eq!(state.current_distraction, Some("Slack".to_string()));
        assert_eq!(
            state.distraction_consecutive_seconds,
            POLLING_INTERVAL_SECONDS
        );
        assert!(!state.distraction_alert_sent);
    }
}
