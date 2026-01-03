mod client;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "flux")]
#[command(about = "Flux CLI - Control the Flux daemon", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the Flux daemon
    Start,
    /// Stop the Flux daemon
    Stop,
    /// Show the daemon status
    Status,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start => {
            println!("Not implemented");
        }
        Commands::Stop => {
            println!("Not implemented");
        }
        Commands::Status => {
            println!("Not implemented");
        }
    }
}
