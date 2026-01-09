use anyhow::{Context, Result};
use eframe::egui;
use tracing::info;

mod app;
mod data;
mod theme;
mod views;

fn main() -> Result<()> {
    setup_tracing();

    info!("starting flux dashboard");

    let stats_data = data::load_initial_data().context("failed to load stats data")?;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Flux Dashboard")
            .with_maximized(true)
            .with_min_inner_size([500.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Flux Dashboard",
        options,
        Box::new(|creation_context| {
            egui_extras::install_image_loaders(&creation_context.egui_ctx);
            Ok(Box::new(app::FluxApp::new(stats_data)))
        }),
    )
    .map_err(|error| anyhow::anyhow!("eframe error: {}", error))
}

fn setup_tracing() {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();
}
