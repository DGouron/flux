mod app_tracker;
mod digest_scheduler;
mod notifier;
mod timer;
#[cfg(target_os = "linux")]
mod tray;

pub use app_tracker::{AppTrackerActor, AppTrackerHandle};
pub use digest_scheduler::DigestSchedulerActor;
pub use notifier::{CheckInResponse, NotifierActor, NotifierHandle};
pub use timer::{TimerActor, TimerHandle};
#[cfg(target_os = "linux")]
pub use tray::{
    check_for_updates, open_configuration, open_dashboard, spawn_tray, TrayAction, TrayStateHandle,
};
