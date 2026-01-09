use flux_core::FocusMode;
use ksni::{self, menu::StandardItem, Icon, MenuItem, TrayService};
use std::process::Command;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tracing::{debug, info, warn};

#[derive(Debug, Clone)]
pub enum TrayAction {
    Pause,
    Resume,
    Stop,
    OpenDashboard,
    CheckForUpdates,
    OpenConfiguration,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TrayState {
    #[default]
    Inactive,
    Active,
    Paused,
    CheckInPending,
}

#[derive(Debug, Clone, Default)]
pub struct TrayDisplayInfo {
    pub state: TrayState,
    pub remaining: Option<Duration>,
    pub mode: Option<FocusMode>,
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
}

impl TrayDisplayInfo {
    fn format_remaining(&self) -> String {
        match self.remaining {
            Some(duration) => {
                let total_secs = duration.as_secs();
                let minutes = total_secs / 60;
                let seconds = total_secs % 60;
                format!("{:02}:{:02}", minutes, seconds)
            }
            None => String::new(),
        }
    }

    fn format_mode(&self) -> String {
        self.mode
            .as_ref()
            .map(|mode| format!("({})", mode))
            .unwrap_or_default()
    }

    fn tooltip_description(&self) -> String {
        match self.state {
            TrayState::Inactive => "No active session".to_string(),
            TrayState::Active => {
                let time = self.format_remaining();
                let mode = self.format_mode();
                if mode.is_empty() {
                    format!("{} remaining", time)
                } else {
                    format!("{} remaining {}", time, mode)
                }
            }
            TrayState::Paused => {
                let time = self.format_remaining();
                format!("Paused ({} remaining)", time)
            }
            TrayState::CheckInPending => "Check-in pending".to_string(),
        }
    }
}

struct FluxTray {
    state: Arc<Mutex<TrayState>>,
    display_info: Arc<Mutex<TrayDisplayInfo>>,
    action_sender: Sender<TrayAction>,
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
        let display_info = self.display_info.lock().unwrap();
        ksni::ToolTip {
            title: "Flux".to_string(),
            description: display_info.tooltip_description(),
            icon_name: String::new(),
            icon_pixmap: vec![],
        }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let state = *self.state.lock().unwrap();
        let mut items: Vec<MenuItem<Self>> = vec![];

        match state {
            TrayState::Inactive => {}
            TrayState::Active => {
                items.push(MenuItem::Standard(StandardItem {
                    label: "Pause".to_string(),
                    activate: Box::new(|tray: &mut Self| {
                        let _ = tray.action_sender.send(TrayAction::Pause);
                    }),
                    ..Default::default()
                }));
                items.push(MenuItem::Standard(StandardItem {
                    label: "Stop".to_string(),
                    activate: Box::new(|tray: &mut Self| {
                        let _ = tray.action_sender.send(TrayAction::Stop);
                    }),
                    ..Default::default()
                }));
            }
            TrayState::Paused => {
                items.push(MenuItem::Standard(StandardItem {
                    label: "Resume".to_string(),
                    activate: Box::new(|tray: &mut Self| {
                        let _ = tray.action_sender.send(TrayAction::Resume);
                    }),
                    ..Default::default()
                }));
                items.push(MenuItem::Standard(StandardItem {
                    label: "Stop".to_string(),
                    activate: Box::new(|tray: &mut Self| {
                        let _ = tray.action_sender.send(TrayAction::Stop);
                    }),
                    ..Default::default()
                }));
            }
            TrayState::CheckInPending => {
                items.push(MenuItem::Standard(StandardItem {
                    label: "Continue".to_string(),
                    activate: Box::new(|tray: &mut Self| {
                        let _ = tray.action_sender.send(TrayAction::Resume);
                    }),
                    ..Default::default()
                }));
                items.push(MenuItem::Standard(StandardItem {
                    label: "Pause".to_string(),
                    activate: Box::new(|tray: &mut Self| {
                        let _ = tray.action_sender.send(TrayAction::Pause);
                    }),
                    ..Default::default()
                }));
                items.push(MenuItem::Standard(StandardItem {
                    label: "Stop".to_string(),
                    activate: Box::new(|tray: &mut Self| {
                        let _ = tray.action_sender.send(TrayAction::Stop);
                    }),
                    ..Default::default()
                }));
            }
        }

        if !items.is_empty() {
            items.push(MenuItem::Separator);
        }

        items.push(MenuItem::Standard(StandardItem {
            label: "Dashboard".to_string(),
            activate: Box::new(|tray: &mut Self| {
                let _ = tray.action_sender.send(TrayAction::OpenDashboard);
            }),
            ..Default::default()
        }));

        items.push(MenuItem::Separator);

        items.push(MenuItem::Standard(StandardItem {
            label: "Check for updates".to_string(),
            activate: Box::new(|tray: &mut Self| {
                let _ = tray.action_sender.send(TrayAction::CheckForUpdates);
            }),
            ..Default::default()
        }));

        items.push(MenuItem::Standard(StandardItem {
            label: "Open configuration".to_string(),
            activate: Box::new(|tray: &mut Self| {
                let _ = tray.action_sender.send(TrayAction::OpenConfiguration);
            }),
            ..Default::default()
        }));

        items.push(MenuItem::Standard(StandardItem {
            label: "Quit".to_string(),
            activate: Box::new(|tray: &mut Self| {
                let _ = tray.action_sender.send(TrayAction::Quit);
            }),
            ..Default::default()
        }));

        items
    }
}

#[derive(Clone)]
pub struct TrayStateHandle {
    state: Arc<Mutex<TrayState>>,
    display_info: Arc<Mutex<TrayDisplayInfo>>,
    ksni_handle: ksni::Handle<FluxTray>,
}

impl TrayStateHandle {
    fn update_display(
        &self,
        new_state: TrayState,
        remaining: Option<Duration>,
        mode: Option<FocusMode>,
    ) {
        {
            let mut state = self.state.lock().unwrap();
            *state = new_state;
        }
        {
            let mut info = self.display_info.lock().unwrap();
            info.state = new_state;
            info.remaining = remaining;
            info.mode = mode;
        }
        self.ksni_handle.update(|_| {});
    }

    pub fn set_active(&self, remaining: Duration, mode: FocusMode) {
        debug!(?remaining, ?mode, "tray set active");
        self.update_display(TrayState::Active, Some(remaining), Some(mode));
    }

    pub fn set_paused(&self, remaining: Duration) {
        debug!(?remaining, "tray set paused");
        self.update_display(TrayState::Paused, Some(remaining), None);
    }

    pub fn set_inactive(&self) {
        debug!("tray set inactive");
        self.update_display(TrayState::Inactive, None, None);
    }

    pub fn set_check_in_pending(&self) {
        debug!("tray set check-in pending");
        self.update_display(TrayState::CheckInPending, None, None);
    }

    pub fn update_remaining(&self, remaining: Duration, mode: FocusMode) {
        {
            let mut info = self.display_info.lock().unwrap();
            info.remaining = Some(remaining);
            info.mode = Some(mode);
        }
        self.ksni_handle.update(|_| {});
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

pub fn spawn_tray() -> Result<(TrayHandle, std::sync::mpsc::Receiver<TrayAction>), String> {
    let state = Arc::new(Mutex::new(TrayState::Inactive));
    let display_info = Arc::new(Mutex::new(TrayDisplayInfo::default()));
    let (action_sender, action_receiver) = std::sync::mpsc::channel();

    let tray = FluxTray {
        state: Arc::clone(&state),
        display_info: Arc::clone(&display_info),
        action_sender,
    };

    let service = TrayService::new(tray);
    let ksni_handle = service.handle();

    let state_handle = TrayStateHandle {
        state,
        display_info,
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

    let handle = TrayHandle {
        ksni_handle,
        thread_handle: Some(thread_handle),
        state_handle,
    };

    Ok((handle, action_receiver))
}

pub fn open_configuration() {
    if let Some(config_dir) = dirs::config_dir() {
        let config_path = config_dir.join("flux").join("config.toml");
        if let Err(error) = Command::new("xdg-open").arg(&config_path).spawn() {
            warn!(%error, "failed to open configuration file");
        }
    }
}

pub fn check_for_updates() {
    let update_script = "flux update; echo ''; echo 'Appuyez sur Entrée pour fermer...'; read";
    let terminals = ["gnome-terminal", "konsole", "xfce4-terminal", "xterm"];

    for terminal in terminals {
        let result = match terminal {
            "gnome-terminal" => Command::new(terminal)
                .args(["--", "bash", "-c", update_script])
                .spawn(),
            "konsole" => Command::new(terminal)
                .args(["-e", "bash", "-c", update_script])
                .spawn(),
            "xfce4-terminal" => Command::new(terminal)
                .args(["-e", &format!("bash -c '{}'", update_script)])
                .spawn(),
            "xterm" => Command::new(terminal)
                .args(["-e", "bash", "-c", update_script])
                .spawn(),
            _ => continue,
        };

        if result.is_ok() {
            return;
        }
    }

    warn!("no suitable terminal emulator found for update");
}

pub fn open_dashboard() {
    if let Ok(gui_path) = which::which("flux-gui") {
        if let Err(error) = Command::new(gui_path).spawn() {
            warn!(%error, "failed to spawn flux-gui");
        }
        return;
    }

    if let Ok(current_exe) = std::env::current_exe() {
        let sibling_path = current_exe.with_file_name("flux-gui");
        if sibling_path.exists() {
            if let Err(error) = Command::new(sibling_path).spawn() {
                warn!(%error, "failed to spawn flux-gui");
            }
            return;
        }
    }

    warn!("flux-gui not found in PATH or alongside daemon binary");
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
    fn inactive_tooltip_shows_no_session() {
        let info = TrayDisplayInfo {
            state: TrayState::Inactive,
            remaining: None,
            mode: None,
        };
        assert_eq!(info.tooltip_description(), "No active session");
    }

    #[test]
    fn active_tooltip_shows_remaining_time_and_mode() {
        let info = TrayDisplayInfo {
            state: TrayState::Active,
            remaining: Some(Duration::from_secs(754)),
            mode: Some(FocusMode::Prompting),
        };
        assert_eq!(info.tooltip_description(), "12:34 remaining (prompting)");
    }

    #[test]
    fn paused_tooltip_shows_remaining_time() {
        let info = TrayDisplayInfo {
            state: TrayState::Paused,
            remaining: Some(Duration::from_secs(300)),
            mode: None,
        };
        assert_eq!(info.tooltip_description(), "Paused (05:00 remaining)");
    }

    #[test]
    fn check_in_pending_tooltip() {
        let info = TrayDisplayInfo {
            state: TrayState::CheckInPending,
            remaining: None,
            mode: None,
        };
        assert_eq!(info.tooltip_description(), "Check-in pending");
    }
}
