use clap::Parser;
use futures::StreamExt;

mod common;
#[cfg(target_os = "linux")]
mod linux;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Enable Debug Mode
    #[arg(short, long)]
    debug: bool,
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

    std::env::set_var("RUST_LOG", "warn");
    if args.debug {
        std::env::set_var("RUST_LOG", "debug");
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

    loop {
        log::debug!("Starting device scan");
        let adapter = match session.default_adapter().await {
            Ok(adapter) => adapter,
            Err(e) => {
                log::error!("Failed to get default adapter: {}", e);
                std::process::exit(1);
            }
        };
        let adapter_events: std::pin::Pin<Box<dyn futures::Stream<Item = MultiEvent>>> =
            Box::pin(adapter.events().await.unwrap().map(MultiEvent::Adapter));
        all_events.push(adapter_events);
        for addr in adapter.device_addresses().await.unwrap() {
            match crate::common::shared_vars::BBWATCHING
                .lock()
                .unwrap()
                .get(&addr)
            {
                Some(true) => {
                    log::debug!("Device {} is already being watched", addr);
                    continue;
                }
                _ => {}
            }
            let device = adapter.device(addr).unwrap();
            let modalias = match device.modalias().await {
                Ok(modalias) => {
                    if let Some(inner_modalias) = modalias {
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
                device.name().await.expect("Unknown").unwrap()
            );
            log::debug!(
                "modalias: \n source: {} \n vendor: {} \n product: {} \n device: {}",
                modalias.source,
                modalias.vendor,
                modalias.product,
                modalias.device
            );
            if modalias.vendor == 76
                && crate::common::shared_vars::AB_DEVICES.contains(&modalias.product)
            {
                log::debug!("Device {} is an Apple device", addr);
                if !crate::common::shared_vars::BBWATCHING
                    .lock()
                    .unwrap()
                    .contains_key(&addr)
                {
                    match device.events().await {
                        Ok(events) => {
                            crate::common::shared_vars::BBWATCHING
                                .lock()
                                .unwrap()
                                .insert(addr, false);
                            all_events.push(Box::pin(events.map(MultiEvent::Device)));
                        }
                        Err(e) => {
                            log::error!("Failed to get events for device {}: {} \n   Device won't be monitored", addr, e);
                            continue;
                        }
                    }
                }
                match device.is_connected().await {
                    Ok(connected) if connected => {
                        crate::common::shared_vars::BBWATCHING
                            .lock()
                            .unwrap()
                            .insert(addr, true);
                        let adapter = adapter.clone();
                        tokio::task::spawn(async move {
                            let mut ab_device = crate::common::ab::ABDevice::new();
                            ab_device.model = device.name().await.expect("Unknown").unwrap();
                            ab_device.model_id = modalias.product;
                            crate::common::ab::ABDevice::monitor(&mut ab_device, device, adapter)
                                .await
                                .unwrap();
                        });
                    }
                    Err(e) => {
                        log::error!("Failed to get connection status for device {}: {}", addr, e);
                    }
                    _ => {}
                }
            }
        }
        log::debug!("Waiting for events");
        while let Some(_) = futures::stream::select_all(&mut all_events).next().await {
            log::debug!("Some event received, updating device list");
            break;
        }
    }
}
