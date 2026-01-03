use flux_protocol::FocusMode;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, info};

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
    #[allow(dead_code)]
    total_duration: Duration,
    remaining: Duration,
    check_in_interval: Duration,
    last_tick: Instant,
    paused: bool,
}

pub struct TimerActor {
    receiver: mpsc::Receiver<TimerMessage>,
    state: Option<TimerState>,
    check_in_callback: Box<dyn Fn() + Send>,
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
    pub fn new<F>(check_in_callback: F) -> (Self, TimerHandle)
    where
        F: Fn() + Send + 'static,
    {
        let (sender, receiver) = mpsc::channel(32);

        let actor = Self {
            receiver,
            state: None,
            check_in_callback: Box::new(check_in_callback),
        };

        let handle = TimerHandle { sender };

        (actor, handle)
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
                            self.state = Some(TimerState {
                                mode,
                                total_duration: duration,
                                remaining: duration,
                                check_in_interval,
                                last_tick: Instant::now(),
                                paused: false,
                            });
                            time_since_check_in = Duration::ZERO;
                        }
                        TimerMessage::Stop => {
                            if self.state.is_some() {
                                info!("session stopped");
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
                                (self.check_in_callback)();
                                time_since_check_in = Duration::ZERO;
                            }
                        } else {
                            info!("session completed");
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
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn start_and_get_status() {
        let (actor, handle) = TimerActor::new(|| {});
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
        let (actor, handle) = TimerActor::new(|| {});
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
        let (actor, handle) = TimerActor::new(|| {});
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
    async fn check_in_callback_triggered() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let (actor, handle) = TimerActor::new(move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });
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

        let count = counter.load(Ordering::SeqCst);
        assert!(count >= 2, "expected at least 2 check-ins, got {}", count);
    }
}
