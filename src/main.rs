use clap::Parser;
use futures::StreamExt;

use crate::data::shared_vars::{AB_DEVICES, BBWATCHING, CONFIG};

mod common;
mod data;
#[cfg(target_os = "linux")]
mod linux;

#[derive(Parser)]
#[command(name = "aplin")]
#[command(version, about="Linux airpods helper", long_about = None)]
struct Args {
    /// Enable Debug Mode
    #[arg(short = 'd', long = "debug")]
    debug: bool,

    /// Path to the config file
    #[arg(short = 'c', long = "config")]
    config: Option<String>,
}

// dead code is here to suppress warning as we never read
// the value, but need just fact of it's existence
#[allow(dead_code)]
enum MultiEvent {
    Adapter(bluer::AdapterEvent),
    Device(bluer::DeviceEvent),
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    unsafe {
        std::env::set_var("RUST_LOG", "warn");
        if args.debug {
            std::env::set_var("RUST_LOG", "debug");
        }
    }

    env_logger::init();
    log::debug!("Logger initialized");

    let session = match bluer::Session::new().await {
        Ok(session) => session,
        Err(e) => {
            log::error!("Failed to create session: {}", e);
            std::process::exit(1);
        }
    };
    let mut all_events = vec![];

    let new_config = crate::data::config::Config::load(args.config.map(std::path::PathBuf::from));
    {
        let mut config = CONFIG.lock().unwrap();
        *config = new_config;
    }

    loop {
        if all_events.is_empty() {
            let events = match session.default_adapter().await {
                Ok(adapter) => adapter.events().await,
                Err(e) => Err(e),
            };
            match events {
                Ok(events) => {
                    let adapter_events: std::pin::Pin<Box<dyn futures::Stream<Item = MultiEvent>>> =
                        Box::pin(events.map(MultiEvent::Adapter));
                    all_events.push(adapter_events);
                    BBWATCHING.lock().await.clear();
                }
                Err(e) => {
                    log::warn!("Bluetooth adapter unavailable: {}, retrying in 3s", e);
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                    continue;
                }
            }
        }

        log::debug!("Starting device scan");
        let adapter = match session.default_adapter().await {
            Ok(adapter) => adapter,
            Err(e) => {
                log::warn!("Bluetooth adapter unavailable: {}, resetting", e);
                all_events.clear();
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                continue;
            }
        };
        let addresses = match adapter.device_addresses().await {
            Ok(addresses) => addresses,
            Err(e) => {
                log::warn!("Failed to list devices: {}, resetting", e);
                all_events.clear();
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                continue;
            }
        };
        for addr in addresses {
            if let Some(true) = BBWATCHING.lock().await.get(&addr) {
                log::debug!("Device {} is already being watched", addr);
                continue;
            }
            let device = match adapter.device(addr) {
                Ok(device) => device,
                Err(e) => {
                    log::error!("Failed to get device {}: {}", addr, e);
                    continue;
                }
            };
            let modalias = match device.modalias().await {
                Ok(modalias) => {
                    if let Some(inner_modalias) = modalias {
                        if inner_modalias.vendor != 76
                            || !AB_DEVICES.contains(&inner_modalias.product)
                        {
                            log::debug!("Device {} is not an Apple device", addr);
                            continue;
                        }
                        inner_modalias
                    } else {
                        log::warn!("Modalias is empty, skipping device: {}", addr);
                        continue;
                    }
                }
                Err(e) => {
                    log::error!("Failed to get modalias for device {}: {}", addr, e);
                    continue;
                }
            };
            log::debug!("Device {} has modalias:", addr);
            log::debug!(
                "Device name: {:?}",
                device
                    .name()
                    .await
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| "Unknown".to_string())
            );
            log::debug!(
                "modalias: \n source: {} \n vendor: {} \n product: {} \n device: {}",
                modalias.source,
                modalias.vendor,
                modalias.product,
                modalias.device
            );
            log::debug!("Device {} is an Apple device", addr);
            if let std::collections::hash_map::Entry::Vacant(e) =
                BBWATCHING.lock().await.entry(addr)
            {
                match device.events().await {
                    Ok(events) => {
                        e.insert(false);
                        all_events.push(Box::pin(events.map(MultiEvent::Device)));
                    }
                    Err(e) => {
                        log::error!(
                            "Failed to get events for device {}: {} \n   Device won't be monitored",
                            addr,
                            e
                        );
                        continue;
                    }
                }
            }
            match device.is_connected().await {
                Ok(connected) if connected => {
                    BBWATCHING.lock().await.insert(addr, true);
                    let adapter = adapter.clone();
                    let device_c = device.clone();
                    tokio::task::spawn(async move {
                        let mut ab_device = crate::common::ab_device::ABDevice::new();
                        ab_device.model = device
                            .name()
                            .await
                            .ok()
                            .flatten()
                            .unwrap_or_else(|| "Unknown".to_string());
                        ab_device.model_id = modalias.product;
                        crate::common::ab_device::ABDevice::monitor(
                            &mut ab_device,
                            device,
                            adapter,
                        )
                        .await
                        .unwrap();
                        log::debug!("Device closed {} ", addr);
                        BBWATCHING.lock().await.insert(addr, false);
                        device_c.disconnect().await.unwrap_or_else(|e| {
                            log::error!("Failed to disconnect device {}: {}", addr, e);
                        });
                    });
                }
                Err(e) => {
                    log::error!("Failed to get connection status for device {}: {}", addr, e);
                }
                _ => {}
            }
        }
        log::debug!("Waiting for events");
        if futures::stream::select_all(&mut all_events)
            .next()
            .await
            .is_none()
        {
            log::warn!("All bluetooth event streams ended, resetting");
            all_events.clear();
        } else {
            log::debug!("Some event received, updating device list");
        }
    }
}
