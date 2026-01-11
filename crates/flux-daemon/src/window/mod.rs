#[cfg(target_os = "linux")]
mod x11_detector;

#[cfg(target_os = "linux")]
pub use x11_detector::X11WindowDetector;

pub trait WindowDetector: Send + Sync {
    fn get_active_application(&self) -> Option<String>;
}
