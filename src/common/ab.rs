#[cfg(target_os = "linux")]
use ksni::TrayMethods;

#[derive(Debug, Clone)]
pub struct ABDevice {
    // apple/beats device
    pub model: String,
    pub anc_state: ANC,
    pub ear_cover_state: (bool, bool),
    pub battery_state: ABBattery,
}
#[derive(Debug, Copy, Clone)]
pub struct ABBattery {
    pub single: Option<(ABBatteryState, u8)>,
    pub left: Option<(ABBatteryState, u8)>,
    pub right: Option<(ABBatteryState, u8)>,
    pub case: Option<(ABBatteryState, u8)>,
}

// full to doesn't exist as a state, but added to simplify gui and daemon logic in future
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ABBatteryState {
    Charging,
    Discharging,
    Full,
    Disconnected,
    Unknown,
}
#[derive(Debug, Copy, Clone)]
pub enum ANC {
    Off,
    NoiseCancelling,
    Transparency,
    Adaptive,
}

#[allow(dead_code)]
struct Dummy;
#[allow(dead_code)]
impl Dummy {
    fn new() -> Self {
        Self
    }

    async fn update<T>(&self, _arg: T) {
        // Do nothing for non-Linux platforms
    }
}

impl ABDevice {
    // create new instance of ABDevice with default values
    pub fn new() -> Self {
        Self {
            model: "Unknown".to_string(),
            anc_state: ANC::Off,
            ear_cover_state: (false, false),
            battery_state: ABBattery {
                single: None,
                left: None,
                right: None,
                case: None,
            },
        }
    }
    pub async fn monitor(
        &mut self,
        pods: bluer::Device,
        adapter: bluer::Adapter,
        daemon: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (mtu, data_stream) = match Self::connect(pods.clone(), adapter.clone()).await {
            Some((mtu, data_stream)) => (mtu, data_stream),
            None => {
                log::error!("Failed to establish connection");
                return Ok(());
            }
        };
        self.model = pods.name().await.expect("Unknown").unwrap();

        // dummy to have better conditional code handling
        // won't be triggered in real use
        #[cfg(not(target_os = "linux"))]
        let gui = crate::common::ab::Dummy::new();

        #[cfg(target_os = "linux")]
        let gui = self.clone().spawn().await.unwrap();

        loop {
            let mut buf = vec![0u8; mtu.into()];
            match data_stream.recv(&mut buf).await {
                Ok(bytes) => {
                    let buf = &buf[0..bytes];
                    if buf.len() < 5 {
                        log::debug!("Useless?");
                        continue;
                    }
                    match buf[4] {
                        0x04 => {
                            log::debug!("battery data");
                            for payload_sector_start in (7..7 + buf[6] as usize * 5).step_by(5) {
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
                                    0x01 if charge == 100 => ABBatteryState::Full,
                                    0x01 => ABBatteryState::Charging,
                                    0x02 => ABBatteryState::Discharging,
                                    0x04 => ABBatteryState::Disconnected,
                                    _ => {
                                        log::error!(
                                            "Unknown charging status: {}",
                                            buf[payload_sector_start + 3]
                                        );
                                        ABBatteryState::Unknown
                                    }
                                };

                                match buf[payload_sector_start] {
                                    0x01 => {
                                        log::debug!(
                                            "Single state: {:?}. Single charge: {}",
                                            status,
                                            charge
                                        );
                                        self.battery_state.single = Some((status, charge));
                                    }
                                    0x02 => {
                                        log::debug!(
                                            "Right state: {:?}. Right charge: {}",
                                            status,
                                            charge
                                        );
                                        self.battery_state.right = Some((status, charge));
                                    }
                                    0x04 => {
                                        log::debug!(
                                            "Left state: {:?}. Left charge: {}",
                                            status,
                                            charge
                                        );
                                        self.battery_state.left = Some((status, charge));
                                    }
                                    0x08 => {
                                        log::debug!(
                                            "Case state: {:?}. Case charge: {}",
                                            status,
                                            charge
                                        );
                                        self.battery_state.case = Some((status, charge));
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
                        0x06 => {
                            log::debug!("headphones cover state updated");
                            log::debug!("left cover state: {:?}", buf[6] == 0);
                            log::debug!("right cover state: {:?}", buf[7] == 0);
                            self.ear_cover_state = (buf[6] == 0, buf[7] == 0);
                        }
                        0x09 if buf[6] == 0x0d => {
                            log::debug!("ANC settings");
                            match buf[7] {
                                0x01 => {
                                    log::debug!("ANC Off");
                                    self.anc_state = ANC::Off;
                                }
                                0x02 => {
                                    log::debug!("ANC NoiseCancelling");
                                    self.anc_state = ANC::NoiseCancelling;
                                }
                                0x03 => {
                                    log::debug!("ANC Transparency");
                                    self.anc_state = ANC::Transparency;
                                }
                                0x04 => {
                                    log::debug!("ANC Adaptive");
                                    self.anc_state = ANC::Adaptive;
                                }
                                _ => {
                                    log::debug!("Unknown ANC state: {}", buf[7]);
                                }
                            }
                        }
                        0x09 => {
                            log::debug!("Unknown settings type: {}", buf[6]);
                            // to check 0x17 0x1f 0x24 0x1b
                        }
                        _ => {
                            log::debug!("Unknown data type: {}", buf[4]);
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Failed to receive data: {}\n Airpods disconnected?", e);
                    let mut bbwatching = crate::common::shared_vars::BBWATCHING.lock().unwrap();
                    bbwatching.remove(&pods.address());
                    break;
                }
            }
            #[cfg(target_os = "linux")]
            {
                    gui.update(|ab_device: &mut ABDevice| *ab_device = self.clone())
                        .await;
            }
        }

        Ok(())
    }
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
                let mut bbwatching = crate::common::shared_vars::BBWATCHING.lock().unwrap();
                bbwatching.remove(&pods.address());
                return None;
            }
        };

        let mtu = stream.as_ref().recv_mtu().unwrap();
        log::debug!("MTU: {}", mtu);

        let data_stream = std::sync::Arc::new(stream);
        // TODO: figure out how to wait for the connection to be established instead of timer
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        // handshake
        data_stream
            .send(&vec![
                0x00, 0x00, 0x04, 0x00, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00,
            ])
            .await
            .ok()?;
        // packet to to enshure init data recieved
        data_stream
            .send(&vec![
                0x04, 0x00, 0x04, 0x00, 0x0f, 0x00, 0xff, 0xff, 0xff, 0xff,
            ])
            .await
            .ok()?;
        return Some((mtu, data_stream));
    }
}
