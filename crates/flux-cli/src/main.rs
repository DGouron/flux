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
        /// Mode focus: ai-assisted, review, architecture, ou custom
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
    /// Afficher le résumé hebdomadaire
    Digest,
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
    /// Supprimer toutes les sessions terminées
    Clear {
        /// Confirmer automatiquement (pas de prompt)
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Supprimer une session spécifique
    Delete {
        /// Identifiant de la session à supprimer
        id: i64,
    },
    /// Gérer la liste des applications de distraction
    Distractions {
        #[command(subcommand)]
        action: DistractionsAction,
    },
    /// Afficher les suggestions de distractions détectées
    Suggestions {
        #[command(subcommand)]
        action: SuggestionsAction,
    },
    /// Gérer les profils de configuration
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },
}

#[derive(Subcommand)]
enum ProfileAction {
    /// Afficher la liste des profils disponibles
    List,
    /// Afficher les détails d'un profil
    Show {
        /// Nom du profil à afficher (défaut: profil actif)
        name: Option<String>,
    },
    /// Activer un profil
    Use {
        /// Nom du profil à activer
        name: String,
    },
}

#[derive(Subcommand)]
enum DistractionsAction {
    /// Afficher la liste des distractions configurées
    List,
    /// Ajouter une application à la liste des distractions
    Add {
        /// Nom de l'application à ajouter
        app: String,
    },
    /// Retirer une application de la liste des distractions
    Remove {
        /// Nom de l'application à retirer
        app: String,
    },
    /// Ajouter un pattern de titre de fenêtre (pour sites web)
    AddPattern {
        /// Pattern à détecter dans le titre (ex: linkedin, facebook)
        pattern: String,
    },
    /// Retirer un pattern de titre de fenêtre
    RemovePattern {
        /// Pattern à retirer
        pattern: String,
    },
    /// Réinitialiser la liste aux valeurs par défaut
    Reset,
}

#[derive(Subcommand)]
enum SuggestionsAction {
    /// Afficher les suggestions détectées
    List,
    /// Effacer les suggestions
    Clear,
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
        Commands::Digest => commands::digest().await,
        Commands::Update { yes } => commands::update(yes).await,
        Commands::Lang { language } => commands::lang(language),
        Commands::Dashboard => commands::dashboard(),
        Commands::Clear { yes } => commands::clear(yes).await,
        Commands::Delete { id } => commands::delete(id).await,
        Commands::Distractions { action } => match action {
            DistractionsAction::List => commands::distractions::list(),
            DistractionsAction::Add { app } => commands::distractions::add(&app),
            DistractionsAction::Remove { app } => commands::distractions::remove(&app),
            DistractionsAction::AddPattern { pattern } => {
                commands::distractions::add_pattern(&pattern)
            }
            DistractionsAction::RemovePattern { pattern } => {
                commands::distractions::remove_pattern(&pattern)
            }
            DistractionsAction::Reset => commands::distractions::reset(),
        },
        Commands::Suggestions { action } => match action {
            SuggestionsAction::List => commands::suggestions::list(),
            SuggestionsAction::Clear => commands::suggestions::clear(),
        },
        Commands::Profile { action } => match action {
            ProfileAction::List => commands::profile::list(),
            ProfileAction::Show { name } => commands::profile::show(name),
            ProfileAction::Use { name } => commands::profile::use_profile(&name),
        },
    };

    if let Err(error) = result {
        eprintln!("Erreur: {}", error);
        std::process::exit(1);
    }
}
