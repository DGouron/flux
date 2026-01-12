#[cfg(target_os = "linux")]
mod x11_detector;

#[cfg(target_os = "linux")]
pub use x11_detector::X11WindowDetector;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WindowInfo {
    pub app_name: String,
    pub title: Option<String>,
}

impl WindowInfo {
    pub fn new(app_name: String, title: Option<String>) -> Self {
        Self { app_name, title }
    }

    pub fn title_or_empty(&self) -> &str {
        self.title.as_deref().unwrap_or("")
    }
}

pub trait WindowDetector: Send + Sync {
    fn get_active_window_info(&self) -> Option<WindowInfo>;
}
