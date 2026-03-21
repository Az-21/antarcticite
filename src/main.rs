pub mod cli;
pub mod core;
pub mod extension;
pub mod os;

use clap::Parser;
use cli::{Cli, Commands};
use extension::native_messaging::{
    AckData, ExtensionMessage, NativeMessage, read_message, write_message,
};
use tracing::{error, info};

async fn daemon_loop(should_exit_on_disconnect: bool) {
    loop {
        match read_message() {
            Ok(Some(ExtensionMessage::ResolvedUrl(data))) => {
                info!(
                    "Received resolved URL from extension: {}",
                    data.resolved_url
                );

                // Load config and route the resolved URL
                let config = match core::config::load_config() {
                    Ok(c) => c,
                    Err(e) => {
                        error!("Failed to load configuration: {}", e);
                        continue;
                    }
                };

                if let Err(e) = core::router::open_url(&data.resolved_url, &config) {
                    error!("Failed to route resolved URL: {}", e);
                } else {
                    // Acknowledge back to the extension
                    let ack = NativeMessage::Ack(AckData {
                        status: "success".to_string(),
                        message: "URL routed successfully".to_string(),
                    });
                    let _ = write_message(&ack);
                }
            }
            Ok(None) => {
                info!("Extension disconnected.");
                if should_exit_on_disconnect {
                    info!("Exiting process as requested on disconnect.");
                    std::process::exit(0);
                }
                // If we shouldn't exit, just return and end this thread's loop
                return;
            }
            Err(e) => {
                error!("Error reading message from extension: {}", e);
                if should_exit_on_disconnect {
                    std::process::exit(1);
                }
                return;
            }
        }
    }
}

async fn start_daemon(should_exit_on_disconnect: bool) -> anyhow::Result<()> {
    use tao::event_loop::{ControlFlow, EventLoopBuilder};

    let event_loop = EventLoopBuilder::new().build();

    // Initialize system tray
    let tray_app = match os::tray::TrayApp::new() {
        Ok(app) => Some(app),
        Err(e) => {
            error!("Failed to initialize tray icon: {}", e);
            None
        }
    };

    // Spawn the extension message loop in a separate thread
    // because it blocks on stdin, which would block the macOS event loop.
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            daemon_loop(should_exit_on_disconnect).await;
        });
    });

    // Run the event loop on the main thread
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::WaitUntil(
            std::time::Instant::now() + std::time::Duration::from_millis(100),
        );

        match event {
            tao::event::Event::MainEventsCleared => {
                if let Some(app) = &tray_app {
                    app.handle_events();
                }
            }
            tao::event::Event::Opened { urls } => {
                info!("Event loop: Received URLs: {:?}", urls);
                for url in urls {
                    let url_str = url.to_string();
                    info!("URL opened via event loop: {}", url_str);

                    // Load config and route the URL
                    match core::config::load_config() {
                        Ok(config) => {
                            if let Err(e) = core::router::open_url(&url_str, &config) {
                                error!(
                                    "Failed to route URL from event loop: {}. URL: {}",
                                    e, url_str
                                );
                            } else {
                                info!("Successfully routed URL: {}", url_str);
                            }
                        }
                        Err(e) => {
                            error!("Failed to load config in event loop: {}", e);
                        }
                    }
                }
            }
            _ => (),
        }
    });
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    let _log_guard = core::logger::init().unwrap_or_else(|e| {
        eprintln!("Failed to initialize logging: {}", e);
        let _ = tracing_subscriber::fmt().try_init();
        let (_non_blocking, guard) = tracing_appender::non_blocking(std::io::sink());
        guard
    });

    let cli = Cli::parse();
    info!(
        "Arguments parsed: command={:?}, url={:?}",
        cli.command, cli.url
    );

    match &cli.command {
        Some(Commands::Daemon) => {
            info!("Starting daemon/native messaging host...");
            // User manually started the daemon, we want it to stay alive for the tray
            start_daemon(false).await?;
        }

        Some(Commands::Install) => {
            // ...
            info!("Running installation...");
            if let Err(e) = os::default_browser::install() {
                error!("Installation failed: {}", e);
                eprintln!("Installation failed: {}", e);
            }
        }
        None => {
            if let Some(url) = &cli.url {
                info!("Routing URL from CLI argument: {}", url);
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
                    return Err(e);
                }
            } else {
                #[cfg(target_os = "macos")]
                {
                    // On macOS, if no URL is provided via CLI, it might be coming via EventLoop
                    // Or the user just launched the app. In either case, starting the daemon
                    // is a reasonable default when launched as an app.
                    info!("No URL or command provided. Starting daemon mode...");
                    start_daemon(false).await?;
                }

                #[cfg(not(target_os = "macos"))]
                {
                    println!("No command or URL provided. Use --help for usage.");
                }
            }
        }
    }

    Ok(())
}
