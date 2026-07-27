use kdl::KdlDocument;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub command_both: Option<String>,
    pub command_single: Option<String>,
    pub command_none: Option<String>,
    pub command_disconnect: Option<String>,
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
            command_disconnect: None,
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
    fn default_path() -> PathBuf {
        let config_home = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
            format!(
                "{}/.config",
                std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
            )
        });
        PathBuf::from(format!("{}/aplin/config.kdl", config_home))
    }

    pub fn load(path: Option<PathBuf>) -> Self {
        let path_buf = path.unwrap_or_else(Self::default_path);

        if !path_buf.exists() {
            log::debug!("No config file at {}, using defaults", path_buf.display());
            return Self::default();
        }

        let content = match fs::read_to_string(&path_buf) {
            Ok(content) => content,
            Err(e) => {
                log::error!("Error reading config file: {}", e);
                log::error!("Falling back to default config");
                return Self::default();
            }
        };
        match Self::parse(&content) {
            Ok(config) => config,
            Err(e) => {
                log::error!("Error parsing config file: {}", e);
                log::error!("Falling back to default config");
                Self::default()
            }
        }
    }

    fn parse(content: &str) -> Result<Self, kdl::KdlError> {
        let doc: KdlDocument = content.parse()?;
        let mut config = Config::default();

        if let Some(covered) = children(&doc, "covered") {
            config.command_both = arg_str(covered, "both");
            config.command_single = arg_str(covered, "single");
            config.command_none = arg_str(covered, "none");
        }
        if let Some(notification) = children(&doc, "notification") {
            if let Some(secs) = arg_duration_secs(notification, "timeout") {
                config.notification_timeout = secs as u32;
            }
            if let Some(v) = arg_bool(notification, "full") {
                config.notify_on_full_charge = v;
            }
            if let Some(v) = arg_bool(notification, "on25") {
                config.notify_on_25_percent = v;
            }
            if let Some(v) = arg_bool(notification, "on10") {
                config.notify_on_10_percent = v;
            }
            if let Some(v) = arg_bool(notification, "anc") {
                config.notify_on_anc_change = v;
            }
        }
        if let Some(disconnect) = children(&doc, "disconnect") {
            config.command_disconnect = arg_str(disconnect, "command");
            if let Some(secs) = arg_duration_secs(disconnect, "timeout") {
                config.disconnect_timeout = secs;
            }
        }
        Ok(config)
    }
}

fn children<'a>(doc: &'a KdlDocument, name: &str) -> Option<&'a KdlDocument> {
    doc.get(name).and_then(|node| node.children())
}

fn arg_str(doc: &KdlDocument, name: &str) -> Option<String> {
    doc.get(name)?
        .entries()
        .first()?
        .value()
        .as_string()
        .map(str::to_string)
}

fn arg_bool(doc: &KdlDocument, name: &str) -> Option<bool> {
    doc.get(name)?.entries().first()?.value().as_bool()
}

fn arg_duration_secs(doc: &KdlDocument, name: &str) -> Option<u64> {
    let raw = arg_str(doc, name)?;
    let secs = parse_duration_secs(&raw);
    if secs.is_none() {
        log::error!(
            "Invalid duration {:?} for {:?}, using default (expected e.g. \"30s\", \"5m\", \"1h\")",
            raw,
            name
        );
    }
    secs
}

fn parse_duration_secs(s: &str) -> Option<u64> {
    let s = s.trim();
    if let Some(n) = s.strip_suffix('s') {
        return n.trim().parse().ok();
    }
    if let Some(n) = s.strip_suffix('m') {
        return n.trim().parse::<u64>().ok().map(|v| v * 60);
    }
    if let Some(n) = s.strip_suffix('h') {
        return n.trim().parse::<u64>().ok().map(|v| v * 3600);
    }
    s.parse().ok()
}
