use anyhow::Result;
#[cfg(any(target_os = "linux", target_os = "windows"))]
use std::env;
#[cfg(target_os = "linux")]
use std::fs;
#[cfg(any(target_os = "linux", target_os = "windows"))]
use std::process::Command;
#[allow(unused_imports)]
use tracing::{info, warn};

#[cfg(target_os = "linux")]
use directories::BaseDirs;

pub fn install() -> Result<()> {
    #[cfg(target_os = "linux")]
    return install_linux();

    #[cfg(target_os = "macos")]
    return install_macos();

    #[cfg(target_os = "windows")]
    return install_windows();

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        warn!("Unsupported OS for automatic default browser registration.");
        Ok(())
    }
}

#[cfg(target_os = "linux")]
fn install_linux() -> Result<()> {
    let exe_path = env::current_exe()?;
    let desktop_entry = format!(
        r#"[Desktop Entry]
Version=1.0
Name=Antarcticite Router
GenericName=Web Browser
Comment=Routes URLs to specific browser profiles
Exec={} %U
Terminal=false
Type=Application
Icon=web-browser
Categories=Network;WebBrowser;
MimeType=text/html;text/xml;application/xhtml+xml;x-scheme-handler/http;x-scheme-handler/https;
"#,
        exe_path.display()
    );

    let base_dirs =
        BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find base directories"))?;
    let applications_dir = base_dirs.data_local_dir().join("applications");
    fs::create_dir_all(&applications_dir)?;

    let desktop_file = applications_dir.join("antarcticite.desktop");
    fs::write(&desktop_file, desktop_entry)?;
    info!("Created desktop entry at {:?}", desktop_file);

    let status = Command::new("xdg-settings")
        .args(["set", "default-web-browser", "antarcticite.desktop"])
        .status();

    match status {
        Ok(s) if s.success() => {
            info!("Successfully registered as the default web browser via xdg-settings.");
        }
        _ => {
            warn!(
                "Failed to set default browser automatically using xdg-settings. Please do it manually."
            );
        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn install_macos() -> Result<()> {
    info!(
        "On macOS, automatic default browser registration requires specific APIs or user interaction."
    );
    info!("To register Antarcticite as your default browser:");
    info!("1. Ensure the app is bundled as an .app (e.g. using cargo-bundle).");
    info!("2. Open System Settings -> Desktop & Dock -> Default web browser.");
    info!("3. Select Antarcticite from the dropdown list.");
    Ok(())
}

#[cfg(target_os = "windows")]
fn install_windows() -> Result<()> {
    info!("Registering Antarcticite for Windows Default Apps...");
    let exe_path = env::current_exe()?;
    let exe_str = exe_path.to_string_lossy().to_string();

    let prog_id = "AntarcticiteURL";
    let app_name = "Antarcticite Router";

    let commands = vec![
        // Register ProgID
        format!(
            "reg add \"HKCU\\Software\\Classes\\{}\" /ve /t REG_SZ /d \"{} HTML Document\" /f",
            prog_id, app_name
        ),
        format!(
            "reg add \"HKCU\\Software\\Classes\\{}\\shell\\open\\command\" /ve /t REG_SZ /d \"\\\"{}\\\" \\\"%1\\\"\" /f",
            prog_id, exe_str
        ),
        // Register Application
        format!(
            "reg add \"HKCU\\Software\\Clients\\StartMenuInternet\\{}\" /ve /t REG_SZ /d \"{}\" /f",
            app_name, app_name
        ),
        format!(
            "reg add \"HKCU\\Software\\Clients\\StartMenuInternet\\{}\\Capabilities\\URLAssociations\" /v \"http\" /t REG_SZ /d \"{}\" /f",
            app_name, prog_id
        ),
        format!(
            "reg add \"HKCU\\Software\\Clients\\StartMenuInternet\\{}\\Capabilities\\URLAssociations\" /v \"https\" /t REG_SZ /d \"{}\" /f",
            app_name, prog_id
        ),
        // Register in RegisteredApplications
        format!(
            "reg add \"HKCU\\Software\\RegisteredApplications\" /v \"{}\" /t REG_SZ /d \"Software\\Clients\\StartMenuInternet\\{}\\Capabilities\" /f",
            app_name, app_name
        ),
    ];

    for cmd_str in commands {
        let status = Command::new("cmd").args(["/C", &cmd_str]).status();
        if let Err(e) = status {
            warn!("Failed to execute registry command: {} - {}", cmd_str, e);
        }
    }

    info!("Opening Windows Default Apps settings...");
    let _ = Command::new("cmd")
        .args(["/C", "start", "ms-settings:defaultapps"])
        .status();

    Ok(())
}
