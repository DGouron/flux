mod actors;
mod server;

use actors::{NotifierActor, TimerActor};
use anyhow::Result;
use flux_core::Config;
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

    let notifier_for_timer = notifier_handle.clone();
    let (timer_actor, timer_handle) = TimerActor::new(move || {
        notifier_for_timer.send_check_in(25);
    });
    tokio::spawn(timer_actor.run());

    let server = Server::new(timer_handle)?;
    server.run(shutdown_receiver).await?;

    info!("flux daemon stopped");
    Ok(())
}
