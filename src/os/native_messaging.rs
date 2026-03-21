use anyhow::{Context, Result};
use serde_json::json;
use std::env;
use std::fs;
use std::path::PathBuf;
use tracing::info;

pub fn install_native_messaging_hosts() -> Result<()> {
    let exe_path = env::current_exe()?;
    let host_name = "com.antarcticite.router";

    let manifest = json!({
        "name": host_name,
        "description": "Antarcticite Browser Profile Router",
        "path": exe_path.to_string_lossy(),
        "type": "stdio",
        "allowed_origins": [
            "chrome-extension://*", // This is broad, but useful for development/unpacked
            "moz-extension://*"
        ]
    });

    let manifest_content = serde_json::to_string_pretty(&manifest)?;

    #[cfg(target_os = "macos")]
    {
        let home = env::var("HOME").context("Could not find HOME directory")?;
        let base_path = PathBuf::from(home).join("Library/Application Support");

        let paths = vec![
            base_path.join("Google/Chrome/NativeMessagingHosts"),
            base_path.join("Mozilla/NativeMessagingHosts"),
            base_path.join("Microsoft Edge/NativeMessagingHosts"),
        ];

        for path in paths {
            fs::create_dir_all(&path)?;
            let manifest_path = path.join(format!("{}.json", host_name));
            fs::write(&manifest_path, &manifest_content)?;
            info!(
                "Installed native messaging host manifest at {:?}",
                manifest_path
            );
        }
    }

    #[cfg(target_os = "linux")]
    {
        let home = env::var("HOME").context("Could not find HOME directory")?;
        let base_path = PathBuf::from(home).join(".config");

        let paths = vec![
            base_path.join("google-chrome/NativeMessagingHosts"),
            base_path.join("microsoft-edge/NativeMessagingHosts"),
            base_path.join("BraveSoftware/Brave-Browser/NativeMessagingHosts"),
            base_path.join("chromium/NativeMessagingHosts"),
        ];

        for path in paths {
            fs::create_dir_all(&path)?;
            let manifest_path = path.join(format!("{}.json", host_name));
            fs::write(&manifest_path, &manifest_content)?;
            info!(
                "Installed native messaging host manifest at {:?}",
                manifest_path
            );
        }

        // Firefox on Linux uses a different path
        let ff_path = PathBuf::from(home).join(".mozilla/native-messaging-hosts");
        fs::create_dir_all(&ff_path)?;
        let ff_manifest_path = ff_path.join(format!("{}.json", host_name));
        fs::write(&ff_manifest_path, &manifest_content)?;
        info!(
            "Installed native messaging host manifest at {:?}",
            ff_manifest_path
        );
    }

    #[cfg(target_os = "windows")]
    {
        // Windows uses Registry
        use std::process::Command;

        let registry_paths = vec![
            r"HKCU\Software\Google\Chrome\NativeMessagingHosts",
            r"HKCU\Software\Mozilla\NativeMessagingHosts",
            r"HKCU\Software\Microsoft\Edge\NativeMessagingHosts",
        ];

        let temp_dir = env::temp_dir();
        let manifest_path = temp_dir.join(format!("{}.json", host_name));
        fs::write(&manifest_path, &manifest_content)?;

        // Copy to a more permanent location
        let app_data = env::var("LOCALAPPDATA").context("Could not find LOCALAPPDATA")?;
        let target_dir = PathBuf::from(app_data).join("Antarcticite");
        fs::create_dir_all(&target_dir)?;
        let target_manifest = target_dir.join(format!("{}.json", host_name));
        fs::write(&target_manifest, &manifest_content)?;

        for reg_path in registry_paths {
            let full_reg_path = format!(r"{}\{}", reg_path, host_name);
            let status = Command::new("reg")
                .args([
                    "add",
                    &full_reg_path,
                    "/ve",
                    "/t",
                    "REG_SZ",
                    "/d",
                    &target_manifest.to_string_lossy(),
                    "/f",
                ])
                .status();

            if let Ok(s) = status {
                if s.success() {
                    info!(
                        "Registered native messaging host in registry at {}",
                        full_reg_path
                    );
                }
            }
        }
    }

    Ok(())
}
