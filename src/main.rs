use clap::Parser;
use futures::StreamExt;

mod shared_vars;

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
    let mut device_event_streams = vec![];
    loop {
        log::debug!("Starting device scan");
        let adapter = match session.default_adapter().await {
            Ok(adapter) => adapter,
            Err(e) => {
                log::error!("Failed to get default adapter: {}", e);
                std::process::exit(1);
            }
        };
        let mut events = adapter.events().await.unwrap();
        for addr in adapter.device_addresses().await.unwrap() {
            if shared_vars::BBWATCHING.lock().unwrap().contains_key(&addr) {
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
                                && shared_vars::APPLE_DEVICES.contains(&modalias.product)
                            {
                                device_event_streams.push(device.events().await.unwrap());
                                log::debug!("Device {} is an Apple device", addr);
                                shared_vars::BBWATCHING
                                    .lock()
                                    .unwrap()
                                    .insert(addr, "Apple".to_string());
                                let adapter = adapter.clone();
                                tokio::task::spawn(async move {
                                    monitor_pods(device, adapter).await.unwrap();
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
        if device_event_streams.is_empty() {
            log::debug!("No devices to monitor, waiting for adapter events");
            while let Some(_) = events.next().await {
                log::debug!("Adapter event received");
                break;
            }
        }
        while let Some(ev) = futures::stream::select_all(&mut device_event_streams)
            .next()
            .await
        {
            println!("{:?}", ev);
            log::debug!("Device event received");
            break;
        }
    }
}

async fn monitor_pods(
    pods: bluer::Device,
    adapter: bluer::Adapter,
) -> Result<(), Box<dyn std::error::Error>> {
    //let (mtu, x) = connect(pods.clone(), adapter.clone()).await?;
    let (mut mtu, mut x) = match connect(pods.clone(), adapter.clone()).await {
        Some((mtu, x)) => (mtu, x),
        None => {
            log::error!("Failed to establish connection");
            return Ok(());
        }
    };

    loop {
        let mut buf = vec![0u8; mtu.into()];
        match x.recv(&mut buf).await {
            Ok(bytes) => {
                let buf = &buf[0..bytes];
                if buf.len() < 5 {
                    log::debug!("Useless?");
                    continue;
                }
                match buf[4] {
                    0x04 => {
                        log::debug!("battery data");
                        for sector in 0..buf[6] as usize {
                            let payload_sector_start = sector * 5 + 7;
                            let charge = if buf[payload_sector_start + 2] <= 100 {
                                buf[payload_sector_start + 2]
                            } else {
                                log::error!(
                                    "Invalid charge value: {}, setting default(0)",
                                    buf[payload_sector_start + 2]
                                );
                                0
                            };
                            let status = match buf[payload_sector_start + 3] {
                                0x01 => "Charging",
                                0x02 => "Discharging",
                                0x04 => "Disconnected",
                                0x00 | _ => {
                                    log::error!(
                                        "Unknown charging status: {}",
                                        buf[payload_sector_start + 3]
                                    );
                                    &("Unknown charging status: ".to_owned()
                                        + &buf[payload_sector_start + 3].to_string())
                                }
                            };

                            match buf[payload_sector_start] {
                                0x01 => {
                                    log::debug!(
                                        "Single state: {}. Single charge: {}",
                                        status,
                                        charge
                                    );
                                }
                                0x02 => {
                                    log::debug!(
                                        "Right state: {}. Right charge: {}",
                                        status,
                                        charge
                                    );
                                }
                                0x04 => {
                                    log::debug!("Left state: {}. Left charge: {}", status, charge)
                                }
                                0x08 => {
                                    log::debug!("Case state: {}. Case charge: {}", status, charge)
                                }
                                _ => {
                                    log::error!(
                                        "Unknown battery type {}",
                                        buf[payload_sector_start]
                                    );
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Err(e) => {
                log::warn!("Failed to receive data: {}\n Airpods disconnected?", e);
                let mut bbwatching = shared_vars::BBWATCHING.lock().unwrap();
                bbwatching.remove(&pods.address());
                break;
            }
        }
    }
    Ok(())
}

// fn that returns mtu and std::sync::Arc
async fn connect(
    pods: bluer::Device,
    adapter: bluer::Adapter,
) -> Option<(u16, std::sync::Arc<bluer::l2cap::SeqPacket>)> {
    let socket = bluer::l2cap::Socket::new_seq_packet().unwrap();
    socket
        .bind(bluer::l2cap::SocketAddr::new(
            adapter.address().await.unwrap(),
            bluer::AddressType::BrEdr,
            0,
        ))
        .unwrap();

    // TODO: figure out later PSM values
    // wether 129 to 255 for le is needed for any actions
    // ps. 4097 is for br/edr
    let socket_addr =
        bluer::l2cap::SocketAddr::new(pods.address(), bluer::AddressType::BrEdr, 4097);

    let stream = match socket.connect(socket_addr).await {
        Ok(stream) => stream,
        Err(e) => {
            log::error!("Failed to connect to device: {}", e);
            let mut bbwatching = shared_vars::BBWATCHING.lock().unwrap();
            bbwatching.remove(&pods.address());
            return None;
        }
    };

    let mtu = stream.as_ref().recv_mtu().unwrap();
    log::debug!("MTU: {}", mtu);

    let x = std::sync::Arc::new(stream);
    // TODO: figure out how to wait for the connection to be established instead of timer
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    x.send(&vec![
        0x00, 0x00, 0x04, 0x00, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ])
    .await
    .ok()?;
    x.send(&vec![
        0x04, 0x00, 0x04, 0x00, 0x0f, 0x00, 0xff, 0xff, 0xff, 0xff,
    ])
    .await
    .ok()?;
    return Some((mtu, x));
}
