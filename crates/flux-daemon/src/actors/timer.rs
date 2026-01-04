use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, info};

use flux_core::{FocusMode, Session, SessionRepository};

use super::NotifierHandle;

pub enum TimerMessage {
    Start {
        duration: Duration,
        mode: FocusMode,
        check_in_interval: Duration,
    },
    Stop,
    Pause,
    Resume,
    GetStatus {
        reply: oneshot::Sender<TimerStatus>,
    },
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
    check_in_interval: Duration,
    last_tick: Instant,
    paused: bool,
}

pub struct TimerActor {
    receiver: mpsc::Receiver<TimerMessage>,
    state: Option<TimerState>,
    notifier: Option<NotifierHandle>,
    session_repository: Option<Arc<dyn SessionRepository>>,
    current_session: Option<Session>,
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
        check_in_interval: Duration,
    ) -> Result<(), mpsc::error::SendError<TimerMessage>> {
        self.sender
            .send(TimerMessage::Start {
                duration,
                mode,
                check_in_interval,
            })
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
    pub fn new(
        notifier: Option<NotifierHandle>,
        session_repository: Option<Arc<dyn SessionRepository>>,
    ) -> (Self, TimerHandle) {
        let (sender, receiver) = mpsc::channel(32);

        let actor = Self {
            receiver,
            state: None,
            notifier,
            session_repository,
            current_session: None,
        };

        let handle = TimerHandle { sender };

        (actor, handle)
    }

    fn elapsed_minutes(&self) -> u64 {
        self.state
            .as_ref()
            .map(|state| {
                let elapsed = state.total_duration.saturating_sub(state.remaining);
                elapsed.as_secs() / 60
            })
            .unwrap_or(0)
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
            notifier.send_alert(
                "Flux - Erreur".to_string(),
                "Impossible de sauvegarder la session. Les données pourraient être perdues."
                    .to_string(),
            );
        }
    }

    pub async fn run(mut self) {
        let mut tick_interval = tokio::time::interval(Duration::from_secs(1));
        let mut time_since_check_in = Duration::ZERO;

        loop {
            tokio::select! {
                Some(message) = self.receiver.recv() => {
                    match message {
                        TimerMessage::Start { duration, mode, check_in_interval } => {
                            info!(?mode, ?duration, "session started");
                            let duration_minutes = duration.as_secs() / 60;
                            self.state = Some(TimerState {
                                mode: mode.clone(),
                                total_duration: duration,
                                remaining: duration,
                                check_in_interval,
                                last_tick: Instant::now(),
                                paused: false,
                            });
                            time_since_check_in = Duration::ZERO;

                            self.persist_new_session(mode);

                            if let Some(ref notifier) = self.notifier {
                                notifier.send_session_start(duration_minutes);
                            }
                        }
                        TimerMessage::Stop => {
                            if self.state.is_some() {
                                let total = self.total_minutes();
                                info!("session stopped");

                                self.persist_session_end();

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
                                    info!("session paused");
                                }
                            }
                        }
                        TimerMessage::Resume => {
                            if let Some(ref mut state) = self.state {
                                if state.paused {
                                    state.paused = false;
                                    state.last_tick = Instant::now();
                                    info!("session resumed");
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
                    if let Some(ref mut state) = self.state {
                        if state.paused {
                            continue;
                        }

                        let elapsed = state.last_tick.elapsed();
                        state.last_tick = Instant::now();

                        if state.remaining > elapsed {
                            state.remaining -= elapsed;
                            time_since_check_in += elapsed;

                            if time_since_check_in >= state.check_in_interval {
                                debug!("check-in triggered");
                                self.persist_check_in();
                                if let Some(ref notifier) = self.notifier {
                                    notifier.send_check_in(self.elapsed_minutes());
                                }
                                time_since_check_in = Duration::ZERO;
                            }
                        } else {
                            let total = self.total_minutes();
                            info!("session completed");

                            self.persist_session_end();

                            if let Some(ref notifier) = self.notifier {
                                notifier.send_session_end(total);
                            }

                            self.state = None;
                            time_since_check_in = Duration::ZERO;
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

    #[tokio::test]
    async fn start_and_get_status() {
        let (actor, handle) = TimerActor::new(None, None);
        tokio::spawn(actor.run());

        handle
            .start(
                Duration::from_secs(60),
                FocusMode::Prompting,
                Duration::from_secs(30),
            )
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
        let (actor, handle) = TimerActor::new(None, None);
        tokio::spawn(actor.run());

        handle
            .start(
                Duration::from_secs(60),
                FocusMode::Review,
                Duration::from_secs(30),
            )
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
        let (actor, handle) = TimerActor::new(None, None);
        tokio::spawn(actor.run());

        handle
            .start(
                Duration::from_secs(60),
                FocusMode::Architecture,
                Duration::from_secs(30),
            )
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(50)).await;
        handle.stop().await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        let status = handle.get_status().await.unwrap();
        assert!(!status.active);
    }

    #[tokio::test]
    async fn check_in_triggers_at_interval() {
        let (actor, handle) = TimerActor::new(None, None);
        tokio::spawn(actor.run());

        handle
            .start(
                Duration::from_secs(10),
                FocusMode::Prompting,
                Duration::from_secs(1),
            )
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(3500)).await;

        let status = handle.get_status().await.unwrap();
        assert!(status.active);
        assert!(status.remaining.as_secs() <= 7);
    }
}
