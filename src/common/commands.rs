use crate::data::shared_vars::CONFIG;

pub async fn status_notify(anc: crate::common::ab_state::Anc) {
    default_notification(format!("Anc status changed to {}", anc.get_name())).await;
}

pub async fn cover_events(cover: crate::common::ab_state::EarCoverState) {
    if cover == CONFIG.lock().unwrap().on_cover {
        // mpris service controls autopause, duplicate here?
        // run command?
        log::debug!("Match cover event to config");
    }
}

pub async fn default_notification(body: String) {
    let config = CONFIG.lock().unwrap();
    match notify_rust::Notification::new()
        .summary("Aplin")
        .body(&body)
        .icon("headphones")
        .timeout(config.notification_timeout)
        .show()
    {
        Ok(_) => {}
        Err(e) => log::error!("Failed to send notification: {}", e),
    }
}
