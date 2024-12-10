#[derive(Debug, Clone)]
pub struct Config {
    pub on_cover: crate::common::ab::EarCoverState,
    pub notification_timeout: notify_rust::Timeout,
    pub notify_on_full_charge: bool,
    pub notify_on_25_percent: bool,
    pub notify_on_10_percent: bool,
    pub notify_on_anc_change: bool,
}

impl Config {
    pub fn default() -> Self {
        Self {
            on_cover: crate::common::ab::EarCoverState::None,
            notification_timeout: notify_rust::Timeout::Milliseconds(5000),
            notify_on_full_charge: true,
            notify_on_25_percent: true,
            notify_on_10_percent: true,
            notify_on_anc_change: false,
        }
    }
}
