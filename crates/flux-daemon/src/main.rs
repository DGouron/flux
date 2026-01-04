mod actors;
mod server;

use std::sync::Arc;

use actors::{NotifierActor, TimerActor};
use anyhow::Result;
use flux_adapters::SqliteSessionRepository;
use flux_core::{Config, SessionRepository};
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

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        info!("SIGINT received, initiating shutdown");
        shutdown_sender.send(()).ok();
    });

    let (notifier_actor, notifier_handle) = NotifierActor::new(
        config.notifications.urgency.clone(),
        config.notifications.sound_enabled,
    );
    tokio::spawn(notifier_actor.run());

    let session_repository = create_session_repository();

    let (timer_actor, timer_handle) = TimerActor::new(Some(notifier_handle), session_repository);
    tokio::spawn(timer_actor.run());

    let server = Server::new(timer_handle)?;
    server.run(shutdown_receiver).await?;

    info!("flux daemon stopped");
    Ok(())
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
