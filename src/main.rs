use clap::Parser;

const APPLE_DEVICES: &[u32] = &[
    8202, // AirPods Max lightning
];

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Enable Debug Mode
    #[arg(short, long)]
    debug: bool,
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

    let adapter = match session.default_adapter().await {
        Ok(adapter) => adapter,
        Err(e) => {
            log::error!("Failed to get default adapter: {}", e);
            std::process::exit(1);
        }
    };

    for addr in adapter.device_addresses().await.unwrap() {
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
                        if modalias.vendor == 76 && APPLE_DEVICES.contains(&modalias.product) {
                            log::debug!("Device {} is an Apple device", addr);
                            monotor_pod(device).await;
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
}

async fn monotor_pod(pods: bluer::Device) {
    println!("Monitoring pods: {:?}", pods);
}
