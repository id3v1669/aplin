#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ABBatteryState {
    Charging,
    Discharging,
    Low10,
    Low25,
    Full,
    Disconnected,
    Unknown,
}
#[derive(Debug, Copy, Clone)]
pub struct ABBattery {
    pub single: Option<(ABBatteryState, u8)>,
    pub left: Option<(ABBatteryState, u8)>,
    pub right: Option<(ABBatteryState, u8)>,
    pub case: Option<(ABBatteryState, u8)>,
}
impl ABBattery {
    pub fn iter(&self) -> impl Iterator<Item = (String, &Option<(ABBatteryState, u8)>)> {
        vec![
            ("    ".to_string(), &self.single),
            ("  L󱡒  ".to_string(), &self.left),
            ("  R󱡒  ".to_string(), &self.right),
            ("    ".to_string(), &self.case),
        ]
        .into_iter()
    }
    pub async fn battery_notify(&self) {
        let config: crate::data::config::Config =
            crate::data::shared_vars::CONFIG.lock().unwrap().clone();
        for (name, battery) in self.iter() {
            if let Some((state, _charge)) = battery {
                match state {
                    ABBatteryState::Low25 if config.notify_on_25_percent => {
                        log::debug!("Battery is low25");
                        crate::common::commands::default_notification(format!(
                            "{} battery is low - 25%",
                            name
                        ))
                        .await;
                    }
                    ABBatteryState::Low10 if config.notify_on_10_percent => {
                        log::debug!("Battery is low10");
                        crate::common::commands::default_notification(format!(
                            "{} battery is low - 10%",
                            name
                        ))
                        .await;
                    }
                    ABBatteryState::Full if config.notify_on_full_charge => {
                        log::debug!("Battery is full");
                        crate::common::commands::default_notification(format!(
                            "{} battery is full",
                            name
                        ))
                        .await;
                    }
                    _ => {}
                }
            }
        }
    }
}
