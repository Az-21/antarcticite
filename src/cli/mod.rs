use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// URL to route (optional, if provided directly it routes it)
    pub url: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Run the background routing daemon
    Daemon,
    /// Install and register as the default browser
    Install,
}
