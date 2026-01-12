mod actors;
mod server;
mod window;

use std::sync::Arc;

#[cfg(target_os = "linux")]
use actors::{check_for_updates, open_configuration, open_dashboard, spawn_tray, TrayAction};
use actors::{AppTrackerActor, DigestSchedulerActor, NotifierActor, TimerActor};
use anyhow::Result;
use flux_adapters::{
    SqliteAppTrackingRepository, SqliteSessionMetricsRepository, SqliteSessionRepository,
};
use flux_core::{AppTrackingRepository, Config, SessionMetricsRepository, SessionRepository};
use server::Server;
use tokio::sync::broadcast;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("flux_daemon=debug".parse()?))
        .init();

    info!("flux daemon starting");

    let config = Config::load().unwrap_or_else(|error| {
        warn!(%error, "failed to load config, using defaults");
        Config::default()
    });

    let (shutdown_sender, shutdown_receiver) = broadcast::channel::<()>(1);
    let sigint_shutdown_sender = shutdown_sender.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        info!("SIGINT received, initiating shutdown");
        sigint_shutdown_sender.send(()).ok();
    });

    let (notifier_actor, notifier_handle) = NotifierActor::new(
        config.notifications().urgency.clone(),
        config.notifications().sound_enabled,
    );
    tokio::spawn(notifier_actor.run());

    #[cfg(target_os = "linux")]
    let (tray_handle, tray_state, tray_action_receiver) = if config.tray.enabled {
        match spawn_tray() {
            Ok((handle, action_receiver)) => {
                let state = handle.state_handle.clone();
                (Some(handle), Some(state), Some(action_receiver))
            }
            Err(error) => {
                warn!(%error, "tray initialization failed, continuing without tray");
                (None, None, None)
            }
        }
    } else {
        (None, None, None)
    };

    #[cfg(target_os = "linux")]
    let _tray_handle = tray_handle;

    let session_repository = create_session_repository();
    let app_tracking_repository = create_app_tracking_repository();
    let session_metrics_repository = create_session_metrics_repository();

    let app_tracker_handle = if let (Some(repository), Some(metrics_repository)) =
        (app_tracking_repository.clone(), session_metrics_repository)
    {
        let (app_tracker_actor, handle) = AppTrackerActor::new(
            repository,
            metrics_repository,
            config.distractions().clone(),
            notifier_handle.clone(),
        );
        tokio::spawn(app_tracker_actor.run());
        Some(handle)
    } else {
        None
    };

    if let (Some(session_repo), Some(app_repo)) =
        (session_repository.clone(), app_tracking_repository)
    {
        let digest_scheduler = DigestSchedulerActor::new(
            notifier_handle.clone(),
            config.digest().clone(),
            config.distractions().clone(),
            session_repo,
            app_repo,
        );
        let digest_shutdown = shutdown_sender.subscribe();
        tokio::spawn(digest_scheduler.run(digest_shutdown));
    }

    #[cfg(target_os = "linux")]
    let (timer_actor, timer_handle) = TimerActor::new(
        Some(notifier_handle),
        app_tracker_handle,
        tray_state,
        session_repository,
    );

    #[cfg(not(target_os = "linux"))]
    let (timer_actor, timer_handle) = TimerActor::new(
        Some(notifier_handle),
        app_tracker_handle,
        session_repository,
    );
    tokio::spawn(timer_actor.run());

    #[cfg(target_os = "linux")]
    if let Some(action_receiver) = tray_action_receiver {
        let tray_timer_handle = timer_handle.clone();
        let tray_shutdown_sender = shutdown_sender.clone();
        let runtime_handle = tokio::runtime::Handle::current();
        std::thread::spawn(move || {
            while let Ok(action) = action_receiver.recv() {
                match action {
                    TrayAction::Pause => {
                        let handle = tray_timer_handle.clone();
                        runtime_handle.spawn(async move {
                            let _ = handle.pause().await;
                        });
                    }
                    TrayAction::Resume => {
                        let handle = tray_timer_handle.clone();
                        runtime_handle.spawn(async move {
                            let _ = handle.resume().await;
                        });
                    }
                    TrayAction::Stop => {
                        let handle = tray_timer_handle.clone();
                        runtime_handle.spawn(async move {
                            let _ = handle.stop().await;
                        });
                    }
                    TrayAction::OpenDashboard => {
                        open_dashboard();
                    }
                    TrayAction::CheckForUpdates => {
                        check_for_updates();
                    }
                    TrayAction::OpenConfiguration => {
                        open_configuration();
                    }
                    TrayAction::Quit => {
                        info!("quit requested from tray");
                        let _ = tray_shutdown_sender.send(());
                        break;
                    }
                }
            }
        });
    }

    let server = Server::new(timer_handle, shutdown_sender)?;
    server.run(shutdown_receiver).await?;

    info!("flux daemon stopped");
    std::process::exit(0);
}

fn create_session_repository() -> Option<Arc<dyn SessionRepository>> {
    let data_dir = dirs::data_dir()?.join("flux");

    if let Err(error) = std::fs::create_dir_all(&data_dir) {
        warn!(%error, "failed to create data directory, sessions will not be persisted");
        return None;
    }

    let database_path = data_dir.join("sessions.db");

    match SqliteSessionRepository::new(&database_path) {
        Ok(repository) => {
            info!(?database_path, "session persistence enabled");
            Some(Arc::new(repository))
        }
        Err(error) => {
            warn!(%error, "failed to initialize session repository, sessions will not be persisted");
            None
        }
    }
}

fn create_app_tracking_repository() -> Option<Arc<dyn AppTrackingRepository>> {
    let data_dir = dirs::data_dir()?.join("flux");

    if let Err(error) = std::fs::create_dir_all(&data_dir) {
        warn!(%error, "failed to create data directory, app tracking will not be persisted");
        return None;
    }

    let database_path = data_dir.join("sessions.db");

    match SqliteAppTrackingRepository::new(&database_path) {
        Ok(repository) => {
            info!("app tracking enabled");
            Some(Arc::new(repository))
        }
        Err(error) => {
            warn!(%error, "failed to initialize app tracking repository");
            None
        }
    }
}

fn create_session_metrics_repository() -> Option<Arc<dyn SessionMetricsRepository>> {
    let data_dir = dirs::data_dir()?.join("flux");

    if let Err(error) = std::fs::create_dir_all(&data_dir) {
        warn!(%error, "failed to create data directory, session metrics will not be persisted");
        return None;
    }

    let database_path = data_dir.join("sessions.db");

    match SqliteSessionMetricsRepository::new(&database_path) {
        Ok(repository) => {
            info!("session metrics enabled");
            Some(Arc::new(repository))
        }
        Err(error) => {
            warn!(%error, "failed to initialize session metrics repository");
            None
        }
    }
}
