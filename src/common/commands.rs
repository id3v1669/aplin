pub async fn battery_notify(ab_battery: crate::common::ab::ABBattery) {
    let config: crate::common::config::Config =
        crate::common::shared_vars::CONFIG.lock().unwrap().clone();
    for (name, battery) in ab_battery.iter() {
        if let Some((state, _charge)) = battery {
            match state {
                crate::common::ab::ABBatteryState::Low25 if config.notify_on_25_percent => {
                    log::debug!("Battery is low25");
                    default_notification(format!("{} battery is low - 25%", name)).await;
                }
                crate::common::ab::ABBatteryState::Low10 if config.notify_on_10_percent => {
                    log::debug!("Battery is low10");
                    default_notification(format!("{} battery is low - 10%", name)).await;
                }
                crate::common::ab::ABBatteryState::Full if config.notify_on_full_charge => {
                    log::debug!("Battery is full");
                    default_notification(format!("{} battery is full", name)).await;
                }
                _ => {}
            }
        }
    }
}

pub async fn status_notify(anc: crate::common::ab::ANC) {
    default_notification(format!("ANC status changed to {}", anc.get_name())).await;
}

pub async fn cover_events(cover: crate::common::ab::EarCoverState) {
    if cover == crate::common::shared_vars::CONFIG.lock().unwrap().on_cover {
        // mpris service controls autopause, duplicate here?
        // run command?
        log::debug!("Match cover event to config");
    }
}

async fn default_notification(body: String) {
    let config = crate::common::shared_vars::CONFIG.lock().unwrap();
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
