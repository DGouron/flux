use ksni::{self, Icon, TrayService};
use std::sync::{Arc, Mutex};
use std::thread;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TrayState {
    #[default]
    Inactive,
    Active,
    Paused,
    CheckInPending,
}

impl TrayState {
    fn icon_name(&self) -> &'static str {
        match self {
            TrayState::Inactive => "appointment-soon",
            TrayState::Active => "user-available",
            TrayState::Paused => "user-away",
            TrayState::CheckInPending => "dialog-warning",
        }
    }

    fn tooltip_description(&self) -> &'static str {
        match self {
            TrayState::Inactive => "Aucune session active",
            TrayState::Active => "Session focus en cours",
            TrayState::Paused => "Session en pause",
            TrayState::CheckInPending => "Check-in en attente",
        }
    }
}

struct FluxTray {
    state: Arc<Mutex<TrayState>>,
}

impl ksni::Tray for FluxTray {
    fn id(&self) -> String {
        "flux".to_string()
    }

    fn title(&self) -> String {
        "Flux".to_string()
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        vec![]
    }

    fn icon_name(&self) -> String {
        let state = self.state.lock().unwrap();
        state.icon_name().to_string()
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        let state = self.state.lock().unwrap();
        ksni::ToolTip {
            title: "Flux".to_string(),
            description: state.tooltip_description().to_string(),
            icon_name: String::new(),
            icon_pixmap: vec![],
        }
    }
}

#[derive(Clone)]
pub struct TrayStateHandle {
    state: Arc<Mutex<TrayState>>,
    ksni_handle: ksni::Handle<FluxTray>,
}

impl TrayStateHandle {
    pub fn set_state(&self, new_state: TrayState) {
        {
            let mut state = self.state.lock().unwrap();
            if *state == new_state {
                return;
            }
            *state = new_state;
        }
        debug!(?new_state, "tray state updated");
        self.ksni_handle.update(|_| {});
    }

    pub fn set_active(&self) {
        self.set_state(TrayState::Active);
    }

    pub fn set_paused(&self) {
        self.set_state(TrayState::Paused);
    }

    pub fn set_inactive(&self) {
        self.set_state(TrayState::Inactive);
    }

    pub fn set_check_in_pending(&self) {
        self.set_state(TrayState::CheckInPending);
    }
}

pub struct TrayHandle {
    ksni_handle: ksni::Handle<FluxTray>,
    thread_handle: Option<thread::JoinHandle<()>>,
    pub state_handle: TrayStateHandle,
}

impl TrayHandle {
    pub fn shutdown(&mut self) {
        self.ksni_handle.shutdown();
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for TrayHandle {
    fn drop(&mut self) {
        self.shutdown();
    }
}

pub fn spawn_tray() -> Result<TrayHandle, String> {
    let state = Arc::new(Mutex::new(TrayState::Inactive));
    let tray = FluxTray {
        state: Arc::clone(&state),
    };

    let service = TrayService::new(tray);
    let ksni_handle = service.handle();

    let state_handle = TrayStateHandle {
        state,
        ksni_handle: ksni_handle.clone(),
    };

    let thread_handle = thread::Builder::new()
        .name("flux-tray".to_string())
        .spawn(move || {
            info!("tray icon initialized");
            if let Err(error) = service.run() {
                warn!(%error, "tray service error");
            }
            debug!("tray service stopped");
        })
        .map_err(|error| format!("failed to spawn tray thread: {}", error))?;

    Ok(TrayHandle {
        ksni_handle,
        thread_handle: Some(thread_handle),
        state_handle,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tray_state_has_correct_icons() {
        assert_eq!(TrayState::Inactive.icon_name(), "appointment-soon");
        assert_eq!(TrayState::Active.icon_name(), "user-available");
        assert_eq!(TrayState::Paused.icon_name(), "user-away");
        assert_eq!(TrayState::CheckInPending.icon_name(), "dialog-warning");
    }

    #[test]
    fn tray_state_has_tooltips() {
        assert!(!TrayState::Inactive.tooltip_description().is_empty());
        assert!(!TrayState::Active.tooltip_description().is_empty());
        assert!(!TrayState::Paused.tooltip_description().is_empty());
        assert!(!TrayState::CheckInPending.tooltip_description().is_empty());
    }
}
