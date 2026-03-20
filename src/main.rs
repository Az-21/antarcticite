pub mod cli;
pub mod core;
pub mod extension;
pub mod os;

use clap::Parser;
use cli::{Cli, Commands};
use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging, but don't fail hard if it doesn't work (might be permissions issue)
    let _log_guard = core::logger::init().unwrap_or_else(|e| {
        eprintln!("Failed to initialize logging: {}", e);
        // Fallback to stdout for CLI usage if we can't create log files
        let _ = tracing_subscriber::fmt().try_init();
        // Return a dummy guard by initializing a simple non-blocking writer
        let (_non_blocking, guard) = tracing_appender::non_blocking(std::io::sink());
        guard
    });

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Daemon) => {
            info!("Starting daemon...");
            println!("Daemon mode not fully implemented yet.");
        }
        Some(Commands::Install) => {
            info!("Running installation...");
            if let Err(e) = os::default_browser::install() {
                error!("Installation failed: {}", e);
                eprintln!("Installation failed: {}", e);
            }
        }
        None => {
            if let Some(url) = &cli.url {
                info!("Routing URL: {}", url);
                let config = match core::config::load_config() {
                    Ok(c) => c,
                    Err(e) => {
                        error!("Failed to load configuration: {}", e);
                        return Err(e);
                    }
                };
                
                if let Err(e) = core::router::open_url(url, &config) {
                    error!("Failed to route URL: {}", e);
                    eprintln!("Failed to route URL: {}", e);
                }
            } else {
                println!("No command or URL provided. Use --help for usage.");
            }
        }
    }

    Ok(())
}
