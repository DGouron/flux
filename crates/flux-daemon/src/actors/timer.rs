use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, info};

use flux_core::{Config, FocusMode, Session, SessionRepository, Translator};

#[cfg(target_os = "linux")]
use super::TrayStateHandle;
use super::{AppTrackerHandle, CheckInResponse, NotifierHandle};

pub enum TimerMessage {
    Start { duration: Duration, mode: FocusMode },
    Stop,
    Pause,
    Resume,
    GetStatus { reply: oneshot::Sender<TimerStatus> },
}

#[derive(Debug, Clone)]
pub struct TimerStatus {
    pub active: bool,
    pub remaining: Duration,
    pub mode: Option<FocusMode>,
    pub paused: bool,
}

struct TimerState {
    mode: FocusMode,
    total_duration: Duration,
    remaining: Duration,
    last_tick: Instant,
    paused: bool,
    check_ins_done: [bool; 3],
}

const CHECK_IN_THRESHOLDS: [u8; 3] = [25, 50, 75];

pub struct TimerActor {
    receiver: mpsc::Receiver<TimerMessage>,
    state: Option<TimerState>,
    notifier: Option<NotifierHandle>,
    app_tracker: Option<AppTrackerHandle>,
    #[cfg(target_os = "linux")]
    tray_state: Option<TrayStateHandle>,
    session_repository: Option<Arc<dyn SessionRepository>>,
    current_session: Option<Session>,
    pending_check_in: Option<oneshot::Receiver<CheckInResponse>>,
}

#[derive(Clone)]
pub struct TimerHandle {
    sender: mpsc::Sender<TimerMessage>,
}

impl TimerHandle {
    pub async fn start(
        &self,
        duration: Duration,
        mode: FocusMode,
    ) -> Result<(), mpsc::error::SendError<TimerMessage>> {
        self.sender
            .send(TimerMessage::Start { duration, mode })
            .await
    }

    pub async fn stop(&self) -> Result<(), mpsc::error::SendError<TimerMessage>> {
        self.sender.send(TimerMessage::Stop).await
    }

    pub async fn pause(&self) -> Result<(), mpsc::error::SendError<TimerMessage>> {
        self.sender.send(TimerMessage::Pause).await
    }

    pub async fn resume(&self) -> Result<(), mpsc::error::SendError<TimerMessage>> {
        self.sender.send(TimerMessage::Resume).await
    }

    pub async fn get_status(&self) -> Option<TimerStatus> {
        let (reply_sender, reply_receiver) = oneshot::channel();
        self.sender
            .send(TimerMessage::GetStatus {
                reply: reply_sender,
            })
            .await
            .ok()?;
        reply_receiver.await.ok()
    }
}

impl TimerActor {
    #[cfg(target_os = "linux")]
    pub fn new(
        notifier: Option<NotifierHandle>,
        app_tracker: Option<AppTrackerHandle>,
        tray_state: Option<TrayStateHandle>,
        session_repository: Option<Arc<dyn SessionRepository>>,
    ) -> (Self, TimerHandle) {
        let (sender, receiver) = mpsc::channel(32);

        let actor = Self {
            receiver,
            state: None,
            notifier,
            app_tracker,
            tray_state,
            session_repository,
            current_session: None,
            pending_check_in: None,
        };

        let handle = TimerHandle { sender };

        (actor, handle)
    }

    #[cfg(not(target_os = "linux"))]
    pub fn new(
        notifier: Option<NotifierHandle>,
        app_tracker: Option<AppTrackerHandle>,
        session_repository: Option<Arc<dyn SessionRepository>>,
    ) -> (Self, TimerHandle) {
        let (sender, receiver) = mpsc::channel(32);

        let actor = Self {
            receiver,
            state: None,
            notifier,
            app_tracker,
            session_repository,
            current_session: None,
            pending_check_in: None,
        };

        let handle = TimerHandle { sender };

        (actor, handle)
    }

    fn total_minutes(&self) -> u64 {
        self.state
            .as_ref()
            .map(|state| state.total_duration.as_secs() / 60)
            .unwrap_or(0)
    }

    fn persist_new_session(&mut self, mode: FocusMode) {
        if let Some(ref repository) = self.session_repository {
            let mut session = Session::start(mode);
            match repository.save(&mut session) {
                Ok(_) => {
                    debug!("session persisted");
                    self.current_session = Some(session);
                }
                Err(err) => {
                    error!(%err, "failed to persist session");
                    self.notify_persistence_error();
                }
            }
        }
    }

    fn persist_session_end(&mut self) {
        if let (Some(ref repository), Some(ref mut session)) =
            (&self.session_repository, &mut self.current_session)
        {
            session.end();
            if let Err(err) = repository.update(session) {
                error!(%err, "failed to update session on end");
                self.notify_persistence_error();
            }
        }
        self.current_session = None;
    }

    fn persist_check_in(&mut self) {
        if let (Some(ref repository), Some(ref mut session)) =
            (&self.session_repository, &mut self.current_session)
        {
            session.increment_check_in();
            if let Err(err) = repository.update(session) {
                error!(%err, "failed to update session check-in count");
                self.notify_persistence_error();
            }
        }
    }

    fn notify_persistence_error(&self) {
        if let Some(ref notifier) = self.notifier {
            let translator = Self::get_translator();
            notifier.send_alert(
                translator.get("error.persistence_error_title"),
                translator.get("error.persistence_error_body"),
            );
        }
    }

    fn get_translator() -> Translator {
        Config::load()
            .map(|config| Translator::new(config.general.language))
            .unwrap_or_default()
    }

    #[cfg(target_os = "linux")]
    fn update_tray_active(&self, remaining: Duration, mode: FocusMode) {
        if let Some(ref tray) = self.tray_state {
            tray.set_active(remaining, mode);
        }
    }

    #[cfg(not(target_os = "linux"))]
    fn update_tray_active(&self, _remaining: Duration, _mode: FocusMode) {}

    #[cfg(target_os = "linux")]
    fn update_tray_paused(&self, remaining: Duration) {
        if let Some(ref tray) = self.tray_state {
            tray.set_paused(remaining);
        }
    }

    #[cfg(not(target_os = "linux"))]
    fn update_tray_paused(&self, _remaining: Duration) {}

    #[cfg(target_os = "linux")]
    fn update_tray_remaining(&self, remaining: Duration, mode: FocusMode) {
        if let Some(ref tray) = self.tray_state {
            tray.update_remaining(remaining, mode);
        }
    }

    #[cfg(not(target_os = "linux"))]
    fn update_tray_remaining(&self, _remaining: Duration, _mode: FocusMode) {}

    #[cfg(target_os = "linux")]
    fn update_tray_inactive(&self) {
        if let Some(ref tray) = self.tray_state {
            tray.set_inactive();
        }
    }

    #[cfg(not(target_os = "linux"))]
    fn update_tray_inactive(&self) {}

    #[cfg(target_os = "linux")]
    fn update_tray_check_in(&self) {
        if let Some(ref tray) = self.tray_state {
            tray.set_check_in_pending();
        }
    }

    #[cfg(not(target_os = "linux"))]
    fn update_tray_check_in(&self) {}

    fn elapsed_percent(&self) -> u8 {
        self.state
            .as_ref()
            .map(|state| {
                let elapsed = state.total_duration.saturating_sub(state.remaining);
                let percent = (elapsed.as_secs_f64() / state.total_duration.as_secs_f64()) * 100.0;
                percent.min(100.0) as u8
            })
            .unwrap_or(0)
    }

    fn next_check_in_threshold(&self) -> Option<(usize, u8)> {
        self.state.as_ref().and_then(|state| {
            let current_percent = self.elapsed_percent();
            CHECK_IN_THRESHOLDS
                .iter()
                .enumerate()
                .find(|(index, &threshold)| {
                    !state.check_ins_done[*index] && current_percent >= threshold
                })
                .map(|(index, &threshold)| (index, threshold))
        })
    }

    fn mark_check_in_done(&mut self, index: usize) {
        if let Some(ref mut state) = self.state {
            state.check_ins_done[index] = true;
        }
    }

    fn check_pending_check_in_response(&mut self) {
        if let Some(ref mut receiver) = self.pending_check_in {
            match receiver.try_recv() {
                Ok(CheckInResponse::NotFocused) => {
                    info!("check-in response: not focused, pausing session");
                    self.pending_check_in = None;
                    self.pause_session_internal();
                }
                Ok(CheckInResponse::Focused) => {
                    debug!("check-in response: focused, continuing");
                    self.pending_check_in = None;
                }
                Err(oneshot::error::TryRecvError::Empty) => {}
                Err(oneshot::error::TryRecvError::Closed) => {
                    debug!("check-in response channel closed, assuming focused");
                    self.pending_check_in = None;
                }
            }
        }
    }

    fn pause_session_internal(&mut self) {
        if let Some(ref mut state) = self.state {
            if !state.paused {
                state.paused = true;
                let remaining = state.remaining;
                info!("session paused from check-in");

                if let Some(ref app_tracker) = self.app_tracker {
                    app_tracker.send_session_paused();
                }

                self.update_tray_paused(remaining);

                if let Some(ref notifier) = self.notifier {
                    notifier.send_session_paused();
                }
            }
        }
    }

    pub async fn run(mut self) {
        let mut tick_interval = tokio::time::interval(Duration::from_secs(1));

        loop {
            tokio::select! {
                Some(message) = self.receiver.recv() => {
                    match message {
                        TimerMessage::Start { duration, mode } => {
                            info!(?mode, ?duration, "session started");
                            let duration_minutes = duration.as_secs() / 60;
                            self.state = Some(TimerState {
                                mode: mode.clone(),
                                total_duration: duration,
                                remaining: duration,
                                last_tick: Instant::now(),
                                paused: false,
                                check_ins_done: [false; 3],
                            });

                            self.persist_new_session(mode.clone());
                            self.update_tray_active(duration, mode);

                            if let Some(ref notifier) = self.notifier {
                                notifier.send_session_start(duration_minutes);
                            }

                            if let (Some(ref app_tracker), Some(ref session)) = (&self.app_tracker, &self.current_session) {
                                if let Some(session_id) = session.id {
                                    app_tracker.send_session_started(session_id);
                                }
                            }
                        }
                        TimerMessage::Stop => {
                            if self.state.is_some() {
                                let total = self.total_minutes();
                                info!("session stopped");

                                if let Some(ref app_tracker) = self.app_tracker {
                                    app_tracker.send_session_ended();
                                }

                                self.persist_session_end();
                                self.update_tray_inactive();

                                if let Some(ref notifier) = self.notifier {
                                    notifier.send_session_end(total);
                                }

                                self.state = None;
                            }
                        }
                        TimerMessage::Pause => {
                            if let Some(ref mut state) = self.state {
                                if !state.paused {
                                    state.paused = true;
                                    let remaining = state.remaining;
                                    info!("session paused");

                                    if let Some(ref app_tracker) = self.app_tracker {
                                        app_tracker.send_session_paused();
                                    }

                                    self.update_tray_paused(remaining);

                                    if let Some(ref notifier) = self.notifier {
                                        notifier.send_session_paused();
                                    }
                                }
                            }
                        }
                        TimerMessage::Resume => {
                            if let Some(ref mut state) = self.state {
                                if state.paused {
                                    state.paused = false;
                                    state.last_tick = Instant::now();
                                    let remaining = state.remaining;
                                    let mode = state.mode.clone();
                                    info!("session resumed");

                                    if let Some(ref app_tracker) = self.app_tracker {
                                        app_tracker.send_session_resumed();
                                    }

                                    self.update_tray_active(remaining, mode);

                                    if let Some(ref notifier) = self.notifier {
                                        notifier.send_session_resumed();
                                    }
                                }
                            }
                        }
                        TimerMessage::GetStatus { reply } => {
                            let status = self.current_status();
                            let _ = reply.send(status);
                        }
                    }
                }
                _ = tick_interval.tick() => {
                    let tick_result = {
                        if let Some(ref mut state) = self.state {
                            if state.paused {
                                None
                            } else {
                                let elapsed = state.last_tick.elapsed();
                                state.last_tick = Instant::now();

                                if state.remaining > elapsed {
                                    state.remaining -= elapsed;
                                    let remaining = state.remaining;
                                    let mode = state.mode.clone();
                                    Some((remaining, mode, false))
                                } else {
                                    Some((Duration::ZERO, state.mode.clone(), true))
                                }
                            }
                        } else {
                            None
                        }
                    };

                    if let Some((remaining, mode, session_complete)) = tick_result {
                        if session_complete {
                            let total = self.total_minutes();
                            info!("session completed");

                            if let Some(ref app_tracker) = self.app_tracker {
                                app_tracker.send_session_ended();
                            }

                            self.persist_session_end();
                            self.update_tray_inactive();

                            if let Some(ref notifier) = self.notifier {
                                notifier.send_session_end(total);
                            }

                            self.state = None;
                        } else {
                            self.update_tray_remaining(remaining, mode.clone());

                            self.check_pending_check_in_response();

                            if self.pending_check_in.is_none() {
                                if let Some((index, threshold)) = self.next_check_in_threshold() {
                                    debug!(threshold, "check-in triggered at {}%", threshold);
                                    self.mark_check_in_done(index);
                                    self.persist_check_in();
                                    self.update_tray_check_in();
                                    if let Some(ref notifier) = self.notifier {
                                        let receiver = notifier.send_check_in(threshold);
                                        self.pending_check_in = Some(receiver);
                                    }
                                }
                            }
                        }
                    }
                }
                else => break,
            }
        }

        debug!("timer actor stopped");
    }

    fn current_status(&self) -> TimerStatus {
        match &self.state {
            Some(state) => TimerStatus {
                active: true,
                remaining: state.remaining,
                mode: Some(state.mode.clone()),
                paused: state.paused,
            },
            None => TimerStatus {
                active: false,
                remaining: Duration::ZERO,
                mode: None,
                paused: false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "linux")]
    fn create_test_actor() -> (TimerActor, TimerHandle) {
        TimerActor::new(None, None, None, None)
    }

    #[cfg(not(target_os = "linux"))]
    fn create_test_actor() -> (TimerActor, TimerHandle) {
        TimerActor::new(None, None, None)
    }

    #[tokio::test]
    async fn start_and_get_status() {
        let (actor, handle) = create_test_actor();
        tokio::spawn(actor.run());

        handle
            .start(Duration::from_secs(60), FocusMode::Prompting)
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(50)).await;

        let status = handle.get_status().await.unwrap();
        assert!(status.active);
        assert!(status.remaining.as_secs() >= 59);
        assert_eq!(status.mode, Some(FocusMode::Prompting));
        assert!(!status.paused);
    }

    #[tokio::test]
    async fn pause_and_resume() {
        let (actor, handle) = create_test_actor();
        tokio::spawn(actor.run());

        handle
            .start(Duration::from_secs(60), FocusMode::Review)
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(50)).await;

        handle.pause().await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        let status = handle.get_status().await.unwrap();
        assert!(status.paused);

        handle.resume().await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        let status = handle.get_status().await.unwrap();
        assert!(!status.paused);
    }

    #[tokio::test]
    async fn stop_clears_session() {
        let (actor, handle) = create_test_actor();
        tokio::spawn(actor.run());

        handle
            .start(Duration::from_secs(60), FocusMode::Architecture)
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(50)).await;
        handle.stop().await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        let status = handle.get_status().await.unwrap();
        assert!(!status.active);
    }

    #[tokio::test]
    async fn check_in_thresholds_are_correct() {
        assert_eq!(CHECK_IN_THRESHOLDS, [25, 50, 75]);
    }
}
