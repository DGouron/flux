pub mod client;
mod commands;
pub mod daemon_launcher;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "flux")]
#[command(version)]
#[command(about = "Flux CLI - Gestionnaire de sessions focus", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialiser la configuration de Flux
    Init {
        /// Écraser la configuration existante
        #[arg(long)]
        force: bool,
    },
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
    /// Mettre à jour Flux vers la dernière version
    Update {
        /// Ignorer la confirmation si le daemon est en cours
        #[arg(short, long)]
        yes: bool,
    },
    /// Change or display the current language
    Lang {
        /// Language code to set (en, fr). Without argument: displays current language.
        language: Option<String>,
    },
    /// Ouvrir le dashboard graphique
    Dashboard,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init { force } => commands::init(force),
        Commands::Start { duration, mode } => {
            if !commands::config_exists() {
                eprintln!("Erreur: Aucune configuration trouvée. Lancez `flux init` pour configurer Flux.");
                std::process::exit(1);
            }
            commands::start(duration, mode).await
        }
        Commands::Stop => commands::stop().await,
        Commands::Pause => commands::pause().await,
        Commands::Resume => commands::resume().await,
        Commands::Status { json } => commands::status(json).await,
        Commands::Stats { period } => {
            let period = commands::Period::from_str(&period).unwrap_or(commands::Period::Week);
            commands::stats(period).await
        }
        Commands::Update { yes } => commands::update(yes).await,
        Commands::Lang { language } => commands::lang(language),
        Commands::Dashboard => commands::dashboard(),
    };

    if let Err(error) = result {
        eprintln!("Erreur: {}", error);
        std::process::exit(1);
    }
}
