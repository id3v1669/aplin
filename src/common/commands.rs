use crate::data::shared_vars::CONFIG;

pub async fn status_notify(anc: crate::common::ab_state::Anc) {
    default_notification(format!("Anc status changed to {}", anc.get_name())).await;
}

pub async fn default_notification(body: String) {
    tokio::task::spawn_blocking(move || {
        match notify_rust::Notification::new()
            .summary("Aplin")
            .body(&body)
            .icon("headphones")
            .timeout(CONFIG.lock().unwrap().notification_timeout)
            .show()
        {
            Ok(_) => {}
            Err(e) => log::error!("Failed to send notification: {}", e),
        }
    })
    .await
    .unwrap();
}
