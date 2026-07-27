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

pub async fn text_input_dialog() -> Option<String> {
    let fallbacks: &[(&str, &[&str])] = &[
        (
            "zenity",
            &["--entry", "--title", "aplin", "--text", "New name:"],
        ),
        (
            "kdialog",
            &["--title", "aplin", "--inputbox", "New name:", ""],
        ),
        (
            "yad",
            &["--entry", "--title", "aplin", "--text", "New name:"],
        ),
    ];
    let mut found = None;
    for (bin, args) in fallbacks {
        match Command::new(bin).args(*args).output().await {
            Ok(out) => {
                found = Some(out);
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => {
                log::error!("Failed to run {}: {}", bin, e);
                return None;
            }
        }
    }
    let output = match found {
        Some(out) => out,
        None => {
            default_notification(
                "No dialog tool found - install zenity, kdialog or yad".to_string(),
            )
            .await;
            return None;
        }
    };

    if !output.status.success() {
        let stderr_raw = String::from_utf8_lossy(&output.stderr);
        let stderr = stderr_raw.trim();
        if stderr.is_empty() {
            log::debug!("Rename dialog cancelled");
        } else {
            log::warn!("Rename dialog exited {}: {}", output.status, stderr);
        }
        return None;
    }
    let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if name.is_empty() { None } else { Some(name) }
}
