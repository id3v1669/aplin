use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub const AB_DEVICES: &[u32] = &[
    0x2002, // AirPods 1
    0x200F, // AirPods 2
    0x2013, // AirPods 3
    //0x0000, // AirPods 4
    0x200E, // AirPods Pro
    0x2014, // AirPods Pro 2
    0x2024, // AirPods Pro 2 usb-c
    0x200A, // AirPods Max lightning
    //0x0000, // AirPods Max usb-c
    0x2012, // Beats Feat Pro
];

pub static BBWATCHING: Lazy<Arc<Mutex<HashMap<bluer::Address, bool>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

pub static CONFIG: Lazy<Mutex<crate::common::config::Config>> =
    Lazy::new(|| Mutex::new(crate::common::config::Config::default()));
