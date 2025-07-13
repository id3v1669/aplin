use crate::common::{
    ab_battery::{ABBattery, ABBatteryState},
    ab_state::{Anc, EarCoverState},
};
use crate::data::shared_vars::{AB_MONITORS, ADAPTIVE_CAPABLE, BBWATCHING, CONFIG};
use tokio::sync::oneshot;

#[cfg(target_os = "linux")]
use ksni::TrayMethods;

#[derive(Debug, Clone)]
pub struct ABDevice {
    // apple/beats device
    pub model: String,
    pub model_id: u32,
    pub anc_state: Anc,
    pub last_anc_state: Option<Anc>,
    pub ear_cover_state: EarCoverState,
    pub last_ear_cover_state: Option<EarCoverState>,
    pub battery_state: ABBattery,
    pub data_stream: Option<std::sync::Arc<bluer::l2cap::SeqPacket>>,
}

#[allow(dead_code)]
struct Dummy;
#[allow(dead_code)]
impl Dummy {
    fn new() -> Self {
        Self
    }
    async fn update<T>(&self, _: T) {}
    fn shutdown(&self) {}
}

impl ABDevice {
    // create new instance of ABDevice with default values
    pub fn new() -> Self {
        Self {
            model: "Unknown".to_string(),
            model_id: 0,
            anc_state: Anc::Off,
            ear_cover_state: EarCoverState::None,
            last_anc_state: None,
            last_ear_cover_state: None,
            battery_state: ABBattery {
                single: None,
                left: None,
                right: None,
                case: None,
            },
            data_stream: None,
        }
    }
    pub async fn monitor(
        &mut self,
        pods: bluer::Device,
        adapter: bluer::Adapter,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut disconnect_tx: Option<oneshot::Sender<()>> = None;
        let (mtu, data_stream) = match Self::connect(pods.clone(), adapter.clone()).await {
            Some((mtu, data_stream)) => (mtu, data_stream),
            None => {
                log::error!("Failed to establish connection");
                return Ok(());
            }
        };

        self.data_stream = Some(data_stream.clone());

        // dummy to have better conditional code handling
        // won't be triggered in real use
        #[cfg(not(target_os = "linux"))]
        let gui = crate::common::ab_device::Dummy::new();

        #[cfg(target_os = "linux")]
        let gui = self.clone().spawn().await.unwrap();

        loop {
            let mut buf = vec![0u8; mtu.into()];
            match data_stream.recv(&mut buf).await {
                Ok(bytes) => {
                    let buf = &buf[0..bytes];
                    if buf.len() < 5 {
                        //FIXME trggered on data_stream_clone.shutdown(std::net::Shutdown::Both);
                        // used for now to break the loop, replace with tx wrapper
                        break;
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
                                    0x01 if charge >= 99 => ABBatteryState::Full, // >= 99 for old batteries that can't reach 100 when in use
                                    0x01 => ABBatteryState::Charging,
                                    0x02 if charge == 10 => ABBatteryState::Low10,
                                    0x02 if charge == 25 => ABBatteryState::Low25,
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
                            let battery_to_pass = self.battery_state;
                            tokio::spawn(async move {
                                battery_to_pass.battery_notify().await;
                            });
                        }
                        0x06 => {
                            if let Some(tx) = disconnect_tx.take() {
                                let _ = tx.send(());
                                log::debug!("Cancelled pending disconnect task");
                            }
                            log::debug!("Device info data");
                            self.cover_event(buf[6], buf[7]);
                            if self.ear_cover_state == EarCoverState::None {
                                let (tx, rx) = oneshot::channel();
                                let data_stream_clone = data_stream.clone();

                                #[cfg(target_os = "linux")]
                                let gui_clone = gui.clone();

                                tokio::spawn(async move {
                                    log::debug!("Starting 1-minute disconnect timer");

                                    tokio::select! {
                                        _ = tokio::time::sleep(std::time::Duration::from_secs(10)) => {
                                            log::debug!("Disconnect timer expired - disconnecting");
                                            gui_clone.shutdown();
                                            let _ = data_stream_clone.shutdown(std::net::Shutdown::Both);
                                        }
                                        _ = rx => {
                                            log::debug!("Disconnect timer cancelled");
                                        }
                                    }
                                });

                                disconnect_tx = Some(tx);
                            }
                        }
                        0x09 if buf[6] == 0x0d => {
                            self.anc_event(buf[7]);
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
                    BBWATCHING.lock().await.insert(pods.address(), false);
                    #[cfg(target_os = "linux")]
                    gui.shutdown();

                    break;
                }
            }
            #[cfg(target_os = "linux")]
            gui.update(|ab_device: &mut ABDevice| *ab_device = self.clone())
                .await;
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
                BBWATCHING.lock().await.remove(&pods.address());
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
            .send(&[
                0x00, 0x00, 0x04, 0x00, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00,
            ])
            .await
            .ok()?;
        // packet to to enshure init data recieved
        data_stream
            .send(&[0x04, 0x00, 0x04, 0x00, 0x0f, 0x00, 0xff, 0xff, 0xff, 0xff])
            .await
            .ok()?;
        Some((mtu, data_stream))
    }

    pub fn anc_event(&mut self, anc_byte: u8) {
        match anc_byte {
            0x01 => {
                log::debug!("Anc Off");
                self.anc_state = Anc::Off;
            }
            0x02 => {
                log::debug!("Anc NoiseCancelling");
                self.anc_state = Anc::NoiseCancelling;
            }
            0x03 => {
                log::debug!("Anc Transparency");
                self.anc_state = Anc::Transparency;
            }
            0x04 => {
                log::debug!("Anc Adaptive");
                self.anc_state = Anc::Adaptive;
            }
            _ => {
                log::debug!("Unknown Anc state: {}", anc_byte);
            }
        }
        let anc_to_pass = self.anc_state;
        if CONFIG.lock().unwrap().notify_on_anc_change {
            tokio::spawn(async move {
                crate::common::commands::status_notify(anc_to_pass).await;
            });
        }
    }

    pub async fn send_anc(&self, anc: Option<Anc>) {
        log::debug!("Sending Anc state: {:?}", anc);
        let anc_byte = if let Some(anc) = anc {
            match anc {
                Anc::Off => 0x01,
                Anc::NoiseCancelling => 0x02,
                Anc::Transparency => 0x03,
                Anc::Adaptive => 0x04,
            }
        } else {
            log::debug!("Anc state is None, falling back to default");
            0x03 // TODO: pull default from config
        };
        self.data_stream
            .as_ref()
            .unwrap()
            .send(&[
                0x04, 0x00, 0x04, 0x00, 0x09, 0x00, 0x0D, anc_byte, 0x00, 0x00, 0x00,
            ])
            .await
            .unwrap();
    }
    pub fn adaptive_capable(&self) -> bool {
        ADAPTIVE_CAPABLE.contains(&self.model_id)
    }
    pub fn is_monitors(&self) -> bool {
        AB_MONITORS.contains(&self.model_id)
    }
    pub fn cover_event(&mut self, left_cover: u8, right_cover: u8) {
        match (left_cover == 0, right_cover == 0) {
            (true, true) => {
                log::debug!("Both ears covered");
                if self.last_ear_cover_state != Some(EarCoverState::Both) {
                    let self_to_move = self.clone();
                    tokio::spawn(async move {
                        self_to_move.send_anc(self_to_move.last_anc_state).await;
                    });
                }
                self.last_ear_cover_state = Some(EarCoverState::Both);
                self.ear_cover_state = EarCoverState::Both;
            }
            (true, false) | (false, true) => {
                log::debug!("Single ear cover detected");
                self.ear_cover_state = EarCoverState::Single;
                // if self.last_ear_cover_state == Some(EarCoverState::Both) {
                //     self.last_anc_state = Some(self.anc_state);
                //     //TODO: trigger commands from config for single ear cover

                //     // FIXME: state doesn't change if only one ear is covered
                //     // Airpods Max doesn't support ANC change on Single state, check other devices for that functionality
                //     // let self_to_move = self.clone();
                //     // tokio::spawn(async move {
                //     //     self_to_move.send_anc(Anc::Transparency).await; // TODO: base on config in future
                //     // });
                // }
            }
            (false, false) => {
                log::debug!("No ears covered");
                self.ear_cover_state = EarCoverState::None;
                if self.last_ear_cover_state == Some(EarCoverState::Both) {
                    self.last_anc_state = Some(self.anc_state);
                }
                //TODO: trigger commands from config for None ear cover

                // FIXME: state doesn't change if only one ear is covered
                // Airpods Max doesn't support ANC change on None state and automatically sets ANC to Off
                // check other devices for that functionality
                // let self_to_move = self.clone();
                //     tokio::spawn(async move {
                //         self_to_move.send_anc(Anc::Off).await; // TODO: review and maybe base on config in future
                //     });
            }
        }
    }
}
