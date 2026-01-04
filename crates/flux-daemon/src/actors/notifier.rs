use flux_core::NotificationUrgency;
use notify_rust::{Notification, Urgency};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

pub enum NotifierMessage {
    CheckIn { session_minutes_elapsed: u64 },
    SessionStart { duration_minutes: u64 },
    SessionEnd { total_minutes: u64 },
    SessionPaused,
    SessionResumed,
    Alert { title: String, body: String },
}

#[derive(Clone)]
pub struct NotifierHandle {
    sender: mpsc::Sender<NotifierMessage>,
}

impl NotifierHandle {
    pub fn send_check_in(&self, session_minutes_elapsed: u64) {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            if let Err(error) = sender
                .send(NotifierMessage::CheckIn {
                    session_minutes_elapsed,
                })
                .await
            {
                error!(%error, "failed to send check-in notification message");
            }
        });
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
            if let Err(error) = sender
                .send(NotifierMessage::Alert { title, body })
                .await
            {
                error!(%error, "failed to send alert notification message");
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
                    session_minutes_elapsed,
                } => {
                    self.send_check_in_notification(session_minutes_elapsed);
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
            }
        }

        debug!("notifier actor stopped");
    }

    fn send_check_in_notification(&self, session_minutes_elapsed: u64) {
        let body = format!(
            "{}min écoulées. Toujours concentré ?",
            session_minutes_elapsed
        );

        match self.build_notification("Flux - Check-in", &body).show() {
            Ok(_) => {
                debug!(session_minutes_elapsed, "check-in notification sent");
            }
            Err(error) => {
                warn!(%error, "failed to show check-in notification");
            }
        }
    }

    fn send_session_start_notification(&self, duration_minutes: u64) {
        let body = format!(
            "Session focus de {}min démarrée. Bonne concentration !",
            duration_minutes
        );

        match self
            .build_notification("Flux - Session démarrée", &body)
            .show()
        {
            Ok(_) => {
                debug!(duration_minutes, "session start notification sent");
            }
            Err(error) => {
                warn!(%error, "failed to show session start notification");
            }
        }
    }

    fn send_session_end_notification(&self, total_minutes: u64) {
        let body = format!("Session de {}min terminée. Bien joué !", total_minutes);

        match self
            .build_notification("Flux - Session terminée", &body)
            .show()
        {
            Ok(_) => {
                debug!(total_minutes, "session end notification sent");
            }
            Err(error) => {
                warn!(%error, "failed to show session end notification");
            }
        }
    }

    fn send_session_paused_notification(&self) {
        match self
            .build_notification("Flux - Pause", "Session mise en pause")
            .show()
        {
            Ok(_) => {
                debug!("session paused notification sent");
            }
            Err(error) => {
                warn!(%error, "failed to show session paused notification");
            }
        }
    }

    fn send_session_resumed_notification(&self) {
        match self
            .build_notification("Flux - Reprise", "Session reprise. Bonne concentration !")
            .show()
        {
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

    fn build_notification(&self, summary: &str, body: &str) -> Notification {
        let mut notification = Notification::new();
        notification
            .summary(summary)
            .body(body)
            .urgency(self.urgency)
            .appname("Flux");

        if self.sound_enabled {
            notification.sound_name("message-new-instant");
        }

        notification
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn handle_can_send_messages() {
        let (actor, handle) = NotifierActor::new(NotificationUrgency::Normal, false);

        let actor_task = tokio::spawn(async move {
            tokio::time::timeout(std::time::Duration::from_millis(100), actor.run()).await
        });

        handle.send_check_in(25);
        handle.send_session_start(45);
        handle.send_session_end(45);
        handle.send_alert("Test".to_string(), "Body".to_string());

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        drop(handle);

        let _ = actor_task.await;
    }
}
