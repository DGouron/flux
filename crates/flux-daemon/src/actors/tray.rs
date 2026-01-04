use ksni::{self, Icon, TrayService};
use std::thread;
use tracing::{debug, info, warn};

#[allow(dead_code)]
const ICON_DATA: &[u8] = include_bytes!("../../assets/icons/flux-icon.svg");

struct FluxTray;

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
        "appointment-soon".to_string()
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        ksni::ToolTip {
            title: "Flux".to_string(),
            description: "Daemon actif".to_string(),
            icon_name: String::new(),
            icon_pixmap: vec![],
        }
    }
}

pub struct TrayHandle {
    service_handle: ksni::Handle<FluxTray>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl TrayHandle {
    pub fn shutdown(&mut self) {
        self.service_handle.shutdown();
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
    let service = TrayService::new(FluxTray);
    let service_handle = service.handle();

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
        service_handle,
        thread_handle: Some(thread_handle),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_data_is_embedded() {
        assert!(!ICON_DATA.is_empty());
        assert!(ICON_DATA.starts_with(b"<svg"));
    }
}
