pub mod cli;
pub mod core;
pub mod extension;
pub mod os;

use clap::Parser;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Daemon) => {
            println!("Starting daemon...");
        }
        Some(Commands::Install) => {
            println!("Installing...");
        }
        None => {
            if let Some(url) = &cli.url {
                println!("Routing URL: {}", url);
            } else {
                println!("No command or URL provided. Use --help for usage.");
            }
        }
    }

    Ok(())
}
