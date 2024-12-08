use clap::Parser;
use futures::StreamExt;

mod common;
mod linux;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Enable Debug Mode
    #[arg(short, long)]
    debug: bool,

    /// Run in daemon mode
    #[arg(short = 'D', long)]
    daemon: bool,
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
            if crate::common::shared_vars::BBWATCHING
                .lock()
                .unwrap()
                .contains_key(&addr)
            {
                log::debug!("Device {} is already being watched", addr);
                continue;
            }
            let device = adapter.device(addr).unwrap();
            match device.is_connected().await {
                Ok(connected) => {
                    if connected {
                        log::debug!("Device {} is connected", addr);
                        let modalias = match device.modalias().await {
                            Ok(modalias) => modalias,
                            Err(e) => {
                                log::error!("Failed to get modalias for device {}: {}", addr, e);
                                continue;
                            }
                        };
                        if let Some(modalias) = modalias {
                            log::debug!("Device {} has modalias:", addr);
                            log::debug!(
                            "modalias: \n source: {} \n vendor: {} \n product: {} \n device: {}",
                            modalias.source,
                            modalias.vendor,
                            modalias.product,
                            modalias.device
                        );
                            if modalias.vendor == 76
                                && crate::common::shared_vars::APPLE_DEVICES
                                    .contains(&modalias.product)
                            {
                                log::debug!("Device {} is an Apple device", addr);
                                log::debug!(
                                    "Device name: {:?}",
                                    device.name().await.expect("Unknown").unwrap()
                                );
                                if let Ok(events) = device.events().await {
                                    all_events.push(Box::pin(events.map(MultiEvent::Device)));
                                }
                                crate::common::shared_vars::BBWATCHING
                                    .lock()
                                    .unwrap()
                                    .insert(addr, "Apple".to_string());
                                let adapter = adapter.clone();
                                tokio::task::spawn(async move {
                                    let mut ab_device = crate::common::ab::ABDevice::new();
                                    crate::common::ab::ABDevice::monitor(
                                        &mut ab_device,
                                        device,
                                        adapter,
                                        args.daemon,
                                    )
                                    .await
                                    .unwrap();
                                });
                            }
                        }
                    } else {
                        log::debug!("Device {} is not connected", addr);
                    }
                }
                Err(e) => {
                    log::error!("Failed to get connection status for device {}: {}", addr, e);
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
