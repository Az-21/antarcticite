use anyhow::{Context, Result};
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
use std::env;
use std::fs;
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
use std::process::Command;
#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

#[cfg(target_os = "linux")]
use directories::BaseDirs;

use crate::os::native_messaging;
use crate::os::notifications;

pub fn install() -> Result<()> {
    info!("Installing Native Messaging Hosts...");
    native_messaging::install_native_messaging_hosts()?;

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
    use std::path::PathBuf;

    info!("Registering Antarcticite as a web browser on macOS...");

    let home = env::var("HOME").context("Could not find HOME directory")?;
    let app_dir = PathBuf::from(home).join("Applications/Antarcticite.app");
    let contents_dir = app_dir.join("Contents");
    let macos_dir = contents_dir.join("MacOS");
    let exe_path = env::current_exe()?;

    info!("Source binary: {:?}", exe_path);
    info!("Target app bundle: {:?}", app_dir);

    info!("Creating application bundle at {:?}", app_dir);
    fs::create_dir_all(&macos_dir)?;

    // Copy the binary to the bundle
    let target_exe = macos_dir.join("antarcticite");
    if target_exe.exists() {
        let _ = fs::remove_file(&target_exe);
    }
    fs::copy(&exe_path, &target_exe).context("Failed to copy binary to app bundle")?;

    // Set executable permissions explicitly
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&target_exe)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&target_exe, perms)?;
        info!("Set executable permissions for {:?}", target_exe);
    }

    let info_plist = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>antarcticite</string>
    <key>CFBundleIdentifier</key>
    <string>com.antarcticite.router</string>
    <key>CFBundleName</key>
    <string>Antarcticite</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundleURLTypes</key>
    <array>
        <dict>
            <key>CFBundleURLName</key>
            <string>Web site URL</string>
            <key>CFBundleURLSchemes</key>
            <array>
                <string>http</string>
                <string>https</string>
            </array>
        </dict>
    </array>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeExtensions</key>
            <array>
                <string>html</string>
                <string>htm</string>
                <string>shtml</string>
                <string>xhtml</string>
            </array>
            <key>CFBundleTypeRole</key>
            <string>Viewer</string>
        </dict>
    </array>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.utilities</string>
    <key>LSHandlerRank</key>
    <string>Owner</string>
    <key>LSBackgroundOnly</key>
    <false/>
    <key>LSFileQuarantineEnabled</key>
    <false/>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
"#;
    fs::write(contents_dir.join("Info.plist"), info_plist).context("Failed to write Info.plist")?;

    info!("Applying local codesign...");
    let _ = Command::new("codesign")
        .args(["-s", "-", "--force", &app_dir.to_string_lossy()])
        .status();

    // "Touch" the bundle to tell the OS it changed
    let _ = Command::new("touch").arg(&app_dir).status();

    info!("Registering with LaunchServices...");
    // Try multiple possible paths for lsregister
    let ls_paths = [
        "/System/Library/Frameworks/CoreServices.framework/Versions/A/Frameworks/LaunchServices.framework/Versions/A/Support/lsregister",
        "/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister",
    ];

    let mut registered = false;
    for ls_path in ls_paths {
        if std::path::Path::new(ls_path).exists() {
            info!("Found lsregister at {}", ls_path);

            // Unregister first to clear stale entries
            let _ = Command::new(ls_path)
                .args(["-u", &app_dir.to_string_lossy()])
                .status();

            // Register with force and seed
            let output = Command::new(ls_path)
                .args(["-f", "-v", "-seed", "-r", &app_dir.to_string_lossy()])
                .output();

            match output {
                Ok(out) => {
                    if out.status.success() {
                        info!("LaunchServices registration successful via {}.", ls_path);
                        registered = true;
                        break;
                    } else {
                        warn!(
                            "LaunchServices registration failed via {}: {}",
                            ls_path, out.status
                        );
                    }
                }
                Err(e) => {
                    warn!("Failed to execute lsregister via {}: {}", ls_path, e);
                }
            }
        }
    }

    if !registered {
        warn!(
            "Could not successfully run lsregister. The app may not appear in the default browser list immediately."
        );
    }

    // Attempt to "launch" the app once in the background to force macOS to acknowledge it
    info!("Triggering initial background launch to verify indexing...");
    let _ = Command::new("open")
        .args(["-g", "-j", &app_dir.to_string_lossy()])
        .status();

    if app_dir.exists() {
        info!("Confirmed: App bundle exists at {:?}", app_dir);
    } else {
        error!("Error: App bundle was NOT created at {:?}", app_dir);
    }

    info!("To finish registering Antarcticite as your default browser:");
    info!("1. Open System Settings -> Desktop & Dock -> Default web browser.");
    info!("2. Select 'Antarcticite' from the dropdown list.");

    notifications::show_notification(
        "Antarcticite Installed",
        "Please select Antarcticite as your default browser in System Settings.",
    );

    info!("Opening System Settings for you...");
    let _ = Command::new("open")
        .arg("x-apple.systempreferences:com.apple.Desktop-Settings.extension")
        .status();

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
