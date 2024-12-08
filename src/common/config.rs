#[derive(Debug, Clone)]
pub struct Config {
    pub on_cover: String,            // to be options both/single/none maybe use enum
    pub command_on_cover: String,    //to be tokio or std Command
    pub command_on_no_cover: String, //to be tokio or std Command
    pub notify_on_full_charge: bool,
    pub notify_on_25_percent: bool,
    pub notify_on_10_percent: bool,
    pub notify_on_status_change: bool,
}

impl Config {
    pub fn default() -> Self {
        Self {
            on_cover: "both".to_string(),
            command_on_cover: "echo cover".to_string(),
            command_on_no_cover: "echo no cover".to_string(),
            notify_on_full_charge: true,
            notify_on_25_percent: true,
            notify_on_10_percent: true,
            notify_on_status_change: true,
        }
    }
}
