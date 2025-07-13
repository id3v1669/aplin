use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigRead {
    pub command_both: Option<String>,
    pub command_single: Option<String>,
    pub command_none: Option<String>,
    pub notification_timeout: Option<u32>,
    pub disconnect_timeout: Option<u64>,
    pub notify_on_full_charge: Option<bool>,
    pub notify_on_25_percent: Option<bool>,
    pub notify_on_10_percent: Option<bool>,
    pub notify_on_anc_change: Option<bool>,
}

impl ConfigRead {
    pub fn into_config(self) -> Config {
        let default_config = Config::default();
        Config {
            command_both: self.command_both,
            command_single: self.command_single,
            command_none: self.command_none,
            notification_timeout: self
                .notification_timeout
                .unwrap_or(default_config.notification_timeout),
            disconnect_timeout: self
                .disconnect_timeout
                .unwrap_or(default_config.disconnect_timeout),
            notify_on_full_charge: self
                .notify_on_full_charge
                .unwrap_or(default_config.notify_on_full_charge),
            notify_on_25_percent: self
                .notify_on_25_percent
                .unwrap_or(default_config.notify_on_25_percent),
            notify_on_10_percent: self
                .notify_on_10_percent
                .unwrap_or(default_config.notify_on_10_percent),
            notify_on_anc_change: self
                .notify_on_anc_change
                .unwrap_or(default_config.notify_on_anc_change),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Config {
    pub command_both: Option<String>,
    pub command_single: Option<String>,
    pub command_none: Option<String>,
    pub notification_timeout: u32,
    pub disconnect_timeout: u64,
    pub notify_on_full_charge: bool,
    pub notify_on_25_percent: bool,
    pub notify_on_10_percent: bool,
    pub notify_on_anc_change: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            command_both: None,
            command_single: None,
            command_none: None,
            notification_timeout: 5,
            disconnect_timeout: 60,
            notify_on_full_charge: true,
            notify_on_25_percent: true,
            notify_on_10_percent: true,
            notify_on_anc_change: false,
        }
    }
}

impl Config {
    pub fn load(path: Option<PathBuf>) -> Self {
        let config = Config::default();
        let path_buf = if let Some(p) = path {
            p
        } else {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(format!("{}/.config/aplin/config", home))
        };

        if !path_buf.exists() {
            // Create the directory if it doesn't exist
            if let Some(parent) = path_buf.parent() {
                if !parent.exists() {
                    if let Err(e) = fs::create_dir_all(parent) {
                        log::error!("Error creating config directory: {}", e);
                        log::error!("Falling back to default config");
                        return Self::default();
                    }
                }
            }
            match fs::File::create(&path_buf) {
                Ok(mut file) => {
                    log::debug!("Created default aplin config at: {:#?}", path_buf);
                    let _ = file.write_all(serde_yml::to_string(&config).unwrap().as_bytes());
                    config
                }
                Err(err) => {
                    log::error!("Error creating config file: {}", err);
                    log::error!("Falling back to default config");
                    return Self::default();
                }
            };
        };

        let config_content_raw = match fs::read_to_string(path_buf) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading config file: {}", e);
                log::error!("Falling back to default config");
                return Self::default();
            }
        };
        let config_read: ConfigRead = serde_yml::from_str(&config_content_raw).unwrap();
        config_read.into_config()
    }
}
