use directories::ProjectDirs;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::EnvFilter;

/// Initialize logging with a rolling file appender in the OS data directory
pub fn init() -> anyhow::Result<tracing_appender::non_blocking::WorkerGuard> {
    // Determine the log directory
    let proj_dirs = ProjectDirs::from("com", "antarcticite", "router")
        .ok_or_else(|| anyhow::anyhow!("Could not determine project directories"))?;
    
    let log_dir = proj_dirs.data_local_dir().join("logs");
    
    // Create the log directory if it doesn't exist
    std::fs::create_dir_all(&log_dir)?;

    // Set up a rolling file appender
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY) // Rotate daily
        .filename_prefix("antarcticite")
        .filename_suffix("log")
        .build(&log_dir)?;

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(non_blocking)
        .with_ansi(false) // Disable ANSI escapes in log file
        .init();

    Ok(guard)
}
