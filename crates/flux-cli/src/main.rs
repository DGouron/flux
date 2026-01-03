pub mod client;
mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "flux")]
#[command(about = "Flux CLI - Gestionnaire de sessions focus", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Démarrer une session focus
    Start {
        /// Durée en minutes (défaut: 25)
        #[arg(short, long)]
        duration: Option<u64>,
        /// Mode focus: prompting, review, architecture, ou custom
        #[arg(short, long)]
        mode: Option<String>,
    },
    /// Arrêter la session en cours
    Stop,
    /// Afficher le statut de la session
    Status {
        /// Afficher en format JSON
        #[arg(long)]
        json: bool,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Start { duration, mode } => commands::start(duration, mode).await,
        Commands::Stop => commands::stop().await,
        Commands::Status { json } => commands::status(json).await,
    };

    if let Err(error) = result {
        eprintln!("Erreur: {}", error);
        std::process::exit(1);
    }
}
