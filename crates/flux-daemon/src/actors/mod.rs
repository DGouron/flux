mod notifier;
mod timer;
#[cfg(target_os = "linux")]
mod tray;

pub use notifier::{NotifierActor, NotifierHandle};
pub use timer::{TimerActor, TimerHandle};
#[cfg(target_os = "linux")]
pub use tray::{check_for_updates, open_configuration, spawn_tray, TrayAction, TrayStateHandle};
