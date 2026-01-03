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
    Start,
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
        Commands::Start => {
            println!("Not implemented");
            Ok(())
        }
        Commands::Stop => {
            println!("Not implemented");
            Ok(())
        }
        Commands::Status { json } => commands::status(json).await,
    };

    if let Err(error) = result {
        eprintln!("Erreur: {}", error);
        std::process::exit(1);
    }
}
