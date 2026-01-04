pub mod client;
mod commands;
pub mod daemon_launcher;

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
    /// Mettre la session en pause
    Pause,
    /// Reprendre une session en pause
    Resume,
    /// Afficher le statut de la session
    Status {
        /// Afficher en format JSON
        #[arg(long)]
        json: bool,
    },
    /// Afficher les statistiques d'utilisation
    Stats {
        /// Période: today, week, month, all (défaut: week)
        #[arg(short, long, default_value = "week")]
        period: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Start { duration, mode } => commands::start(duration, mode).await,
        Commands::Stop => commands::stop().await,
        Commands::Pause => commands::pause().await,
        Commands::Resume => commands::resume().await,
        Commands::Status { json } => commands::status(json).await,
        Commands::Stats { period } => {
            let period = commands::Period::from_str(&period).unwrap_or(commands::Period::Week);
            commands::stats(period).await
        }
    };

    if let Err(error) = result {
        eprintln!("Erreur: {}", error);
        std::process::exit(1);
    }
}
