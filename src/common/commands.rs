use crate::data::shared_vars::CONFIG;
use std::process::Stdio;
use tokio::process::Command;

pub async fn status_notify(anc: crate::common::ab_state::Anc) {
    default_notification(format!("Anc status changed to {}", anc.get_name())).await;
}

pub async fn default_notification(body: String) {
    tokio::task::spawn_blocking(move || {
        match notify_rust::Notification::new()
            .summary("Aplin")
            .body(&body)
            .icon("headphones")
            .timeout(notify_rust::Timeout::Milliseconds(
                CONFIG.lock().unwrap().notification_timeout * 1000,
            ))
            .show()
        {
            Ok(_) => {}
            Err(e) => log::error!("Failed to send notification: {}", e),
        }
    })
    .await
    .unwrap();
}

pub async fn run_system_command(command: &str) {
    if let Err(e) = Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        default_notification(format!("Error running command: {}", command)).await;
        default_notification(format!("Error: {}", e)).await;
        log::error!("Failed to execute {}", command);
        log::error!("Error: {}", e);
    }
}
