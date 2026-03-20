use notify_rust::Notification;
use tracing::{error, info};

/// Displays an OS-native notification
pub fn show_notification(summary: &str, body: &str) {
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
