use crate::data::config::Config;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub const AB_DEVICES: &[u32] = &[
    0x2002, // AirPods 1
    0x200F, // AirPods 2
    0x2013, // AirPods 3
    0x2019, // AirPods 4
    0x200E, // AirPods Pro
    0x2014, // AirPods Pro 2
    0x2024, // AirPods Pro 2 usb-c
    0x2027, // AirPods Pro 3
    0x200A, // AirPods Max lightning
    0x201f, // AirPods Max usb-c 2024
    0x2012, // Beats Fit Pro
    0x2003, // Powerbeats 3
    0x2005, // Beats X
    0x2006, // Beats Solo 3
    0x2009, // Beats Studio 3
    0x200B, // Powerbeats Pro
    0x2010, // Beats Flex
];

pub const ADAPTIVE_CAPABLE: &[u32] = &[
    0x2019, // AirPods 4
    0x2014, // AirPods Pro 2
    0x2024, // AirPods Pro 2 usb-c
    0x2027, // AirPods Pro 3
];

// const var with fullsize headphones
pub const AB_MONITORS: &[u32] = &[
    0x200A, // AirPods Max lightning
    0x201f, // AirPods Max usb-c 2024
    0x2006, // Beats Solo 3 (on-ear)
    0x2009, // Beats Studio 3 (over-ear)
];

pub static BBWATCHING: Lazy<Arc<tokio::sync::Mutex<HashMap<bluer::Address, bool>>>> =
    Lazy::new(|| Arc::new(tokio::sync::Mutex::new(HashMap::new())));

pub static CONFIG: Lazy<Mutex<Config>> = Lazy::new(|| Mutex::new(Config::default()));
