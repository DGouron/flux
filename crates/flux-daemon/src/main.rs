mod actors;
mod server;

use actors::TimerActor;
use anyhow::Result;
use server::Server;
use tokio::sync::broadcast;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("flux_daemon=debug".parse()?))
        .init();

    info!("flux daemon starting");

    let (shutdown_sender, shutdown_receiver) = broadcast::channel::<()>(1);

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        info!("SIGINT received, initiating shutdown");
        shutdown_sender.send(()).ok();
    });

    let (timer_actor, timer_handle) = TimerActor::new(|| {
        info!("check-in triggered - notification will go here");
    });
    tokio::spawn(timer_actor.run());

    let server = Server::new(timer_handle)?;
    server.run(shutdown_receiver).await?;

    info!("flux daemon stopped");
    Ok(())
}
