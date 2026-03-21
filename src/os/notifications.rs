use notify_rust::Notification;
use tracing::{error, info};

/// Displays an OS-native notification
pub fn show_notification(summary: &str, body: &str) {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        // Using 'display dialog' ensures it pops up even if notifications are muted
        let script = format!(
            "display dialog {:?} with title {:?} buttons {{\"OK\"}} default button \"OK\"",
            body, summary
        );
        let _ = Command::new("osascript").args(["-e", &script]).status();
    }

    let result = Notification::new()
        .appname("Antarcticite")
        .summary(summary)
        .body(body)
        .show();

    if let Err(e) = result {
        error!("Failed to show notification: {}", e);
    } else {
        info!("Notification shown: {} - {}", summary, body);
    }
}
