use std::time::Duration;

use flux_core::{Config, NotificationUrgency, Translator};
#[cfg(target_os = "linux")]
use notify_rust::Hint;
use notify_rust::{Notification, Urgency};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, info, warn};

const CHECK_IN_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckInResponse {
    Focused,
    NotFocused,
}

pub enum NotifierMessage {
    CheckIn {
        percent: u8,
        response_sender: oneshot::Sender<CheckInResponse>,
    },
    SessionStart {
        duration_minutes: u64,
    },
    SessionEnd {
        total_minutes: u64,
    },
    SessionPaused,
    SessionResumed,
    Alert {
        title: String,
        body: String,
    },
    DistractionAlert {
        title: String,
        body: String,
    },
}

#[derive(Clone)]
pub struct NotifierHandle {
    sender: mpsc::Sender<NotifierMessage>,
}

impl NotifierHandle {
    pub fn send_check_in(&self, percent: u8) -> oneshot::Receiver<CheckInResponse> {
        let (response_sender, response_receiver) = oneshot::channel();
        let sender = self.sender.clone();
        tokio::spawn(async move {
            if let Err(error) = sender
                .send(NotifierMessage::CheckIn {
                    percent,
                    response_sender,
                })
                .await
            {
                error!(%error, "failed to send check-in notification message");
            }
        });
        response_receiver
    }

    pub fn send_session_start(&self, duration_minutes: u64) {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            if let Err(error) = sender
                .send(NotifierMessage::SessionStart { duration_minutes })
                .await
            {
                error!(%error, "failed to send session start notification message");
            }
        });
    }

    pub fn send_session_end(&self, total_minutes: u64) {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            if let Err(error) = sender
                .send(NotifierMessage::SessionEnd { total_minutes })
                .await
            {
                error!(%error, "failed to send session end notification message");
            }
        });
    }

    pub fn send_session_paused(&self) {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            if let Err(error) = sender.send(NotifierMessage::SessionPaused).await {
                error!(%error, "failed to send session paused notification message");
            }
        });
    }

    pub fn send_session_resumed(&self) {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            if let Err(error) = sender.send(NotifierMessage::SessionResumed).await {
                error!(%error, "failed to send session resumed notification message");
            }
        });
    }

    pub fn send_alert(&self, title: String, body: String) {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            if let Err(error) = sender.send(NotifierMessage::Alert { title, body }).await {
                error!(%error, "failed to send alert notification message");
            }
        });
    }

    pub fn send_distraction_alert(&self, title: String, body: String) {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            if let Err(error) = sender
                .send(NotifierMessage::DistractionAlert { title, body })
                .await
            {
                error!(%error, "failed to send distraction alert notification message");
            }
        });
    }
}

pub struct NotifierActor {
    receiver: mpsc::Receiver<NotifierMessage>,
    urgency: Urgency,
    sound_enabled: bool,
}

impl NotifierActor {
    pub fn new(urgency: NotificationUrgency, sound_enabled: bool) -> (Self, NotifierHandle) {
        let (sender, receiver) = mpsc::channel(32);

        let urgency = match urgency {
            NotificationUrgency::Low => Urgency::Low,
            NotificationUrgency::Normal => Urgency::Normal,
            NotificationUrgency::Critical => Urgency::Critical,
        };

        let actor = Self {
            receiver,
            urgency,
            sound_enabled,
        };

        let handle = NotifierHandle { sender };

        (actor, handle)
    }

    pub async fn run(mut self) {
        info!("notifier actor started");

        while let Some(message) = self.receiver.recv().await {
            match message {
                NotifierMessage::CheckIn {
                    percent,
                    response_sender,
                } => {
                    self.send_check_in_notification(percent, response_sender);
                }
                NotifierMessage::SessionStart { duration_minutes } => {
                    self.send_session_start_notification(duration_minutes);
                }
                NotifierMessage::SessionEnd { total_minutes } => {
                    self.send_session_end_notification(total_minutes);
                }
                NotifierMessage::SessionPaused => {
                    self.send_session_paused_notification();
                }
                NotifierMessage::SessionResumed => {
                    self.send_session_resumed_notification();
                }
                NotifierMessage::Alert { title, body } => {
                    self.send_alert_notification(&title, &body);
                }
                NotifierMessage::DistractionAlert { title, body } => {
                    self.send_distraction_alert_notification(&title, &body);
                }
            }
        }

        debug!("notifier actor stopped");
    }

    fn get_translator(&self) -> Translator {
        Config::load()
            .map(|config| Translator::new(config.general.language))
            .unwrap_or_default()
    }

    fn send_check_in_notification(
        &self,
        percent: u8,
        response_sender: oneshot::Sender<CheckInResponse>,
    ) {
        let translator = self.get_translator();
        let title = format!("Flux - {}", translator.get("notification.check_in_title"));
        let body = translator.format(
            "notification.check_in_body",
            &[("percent", &percent.to_string())],
        );
        let yes_label = translator.get("notification.check_in_yes");
        let no_label = translator.get("notification.check_in_no");

        let mut notification = self.build_notification(&title, &body);
        notification
            .action("yes", &yes_label)
            .action("no", &no_label)
            .timeout(CHECK_IN_TIMEOUT.as_millis() as i32);

        tokio::task::spawn_blocking(move || match notification.show() {
            Ok(handle) => {
                let mut response = CheckInResponse::Focused;

                handle.wait_for_action(|action| {
                    response = match action {
                        "no" => {
                            debug!(percent, "check-in response: not focused");
                            CheckInResponse::NotFocused
                        }
                        _ => {
                            debug!(percent, "check-in response: focused (action={})", action);
                            CheckInResponse::Focused
                        }
                    };
                });

                let _ = response_sender.send(response);
            }
            Err(error) => {
                warn!(%error, "failed to show check-in notification");
                let _ = response_sender.send(CheckInResponse::Focused);
            }
        });
    }

    fn send_session_start_notification(&self, duration_minutes: u64) {
        let translator = self.get_translator();
        let title = format!(
            "Flux - {}",
            translator.get("notification.session_start_title")
        );
        let body = translator.format(
            "notification.session_start_body",
            &[("duration", &duration_minutes.to_string())],
        );

        match self.build_notification(&title, &body).show() {
            Ok(_) => {
                debug!(duration_minutes, "session start notification sent");
            }
            Err(error) => {
                warn!(%error, "failed to show session start notification");
            }
        }
    }

    fn send_session_end_notification(&self, total_minutes: u64) {
        let translator = self.get_translator();
        let title = format!(
            "Flux - {}",
            translator.get("notification.session_end_title")
        );
        let body = translator.format(
            "notification.session_end_body",
            &[("duration", &total_minutes.to_string())],
        );

        match self.build_notification(&title, &body).show() {
            Ok(_) => {
                debug!(total_minutes, "session end notification sent");
            }
            Err(error) => {
                warn!(%error, "failed to show session end notification");
            }
        }
    }

    fn send_session_paused_notification(&self) {
        let translator = self.get_translator();
        let title = format!("Flux - {}", translator.get("notification.paused_title"));
        let body = translator.get("notification.paused_body");

        match self.build_notification(&title, &body).show() {
            Ok(_) => {
                debug!("session paused notification sent");
            }
            Err(error) => {
                warn!(%error, "failed to show session paused notification");
            }
        }
    }

    fn send_session_resumed_notification(&self) {
        let translator = self.get_translator();
        let title = format!("Flux - {}", translator.get("notification.resumed_title"));
        let body = translator.get("notification.resumed_body");

        match self.build_notification(&title, &body).show() {
            Ok(_) => {
                debug!("session resumed notification sent");
            }
            Err(error) => {
                warn!(%error, "failed to show session resumed notification");
            }
        }
    }

    fn send_alert_notification(&self, title: &str, body: &str) {
        match self.build_notification(title, body).show() {
            Ok(_) => {
                debug!(title, "alert notification sent");
            }
            Err(error) => {
                warn!(%error, title, "failed to show alert notification");
            }
        }
    }

    fn send_distraction_alert_notification(&self, title: &str, body: &str) {
        match self.build_distraction_notification(title, body).show() {
            Ok(_) => {
                debug!(title, "distraction alert notification sent");
            }
            Err(error) => {
                warn!(%error, title, "failed to show distraction alert notification");
            }
        }
    }

    fn build_notification(&self, summary: &str, body: &str) -> Notification {
        let mut notification = Notification::new();
        notification.summary(summary).body(body).appname("Flux");

        #[cfg(target_os = "linux")]
        notification.hint(Hint::Urgency(self.urgency));

        if self.sound_enabled {
            notification.sound_name("message-new-instant");
        }

        notification
    }

    fn build_distraction_notification(&self, summary: &str, body: &str) -> Notification {
        let mut notification = Notification::new();
        notification
            .summary(summary)
            .body(body)
            .appname("Flux")
            .icon("dialog-warning");

        #[cfg(target_os = "linux")]
        notification.hint(Hint::Urgency(Urgency::Critical));

        if self.sound_enabled {
            notification.sound_name("dialog-warning");
        }

        notification
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn handle_can_send_simple_messages() {
        let (actor, handle) = NotifierActor::new(NotificationUrgency::Normal, false);

        let actor_task = tokio::spawn(async move {
            tokio::time::timeout(std::time::Duration::from_millis(100), actor.run()).await
        });

        handle.send_session_start(45);
        handle.send_session_end(45);
        handle.send_session_paused();
        handle.send_session_resumed();
        handle.send_alert("Test".to_string(), "Body".to_string());

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        drop(handle);

        let _ = actor_task.await;
    }

    #[test]
    fn check_in_response_variants() {
        assert_ne!(CheckInResponse::Focused, CheckInResponse::NotFocused);
    }
}
