use crate::common::{ab_battery::ABBatteryState, ab_device::ABDevice, ab_state::Anc};

impl ksni::Tray for ABDevice {
    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").into()
    }
    fn icon_name(&self) -> String {
        if self.is_monitors() {
            "headphones-monitors".into()
        } else {
            "headphones-buds".into()
        }
    }
    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        vec![to_icon(
            resvg::usvg::Options::default(),
            resvg::tiny_skia::Transform::identity(),
            if self.is_monitors() {
                include_str!("../../assets/icons/headphones-monitors.svg")
            } else {
                include_str!("../../assets/icons/headphones-buds.svg")
            },
        )]
    }
    fn title(&self) -> String {
        "APLin".into()
    }
    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        let mut mode = vec![
            RadioItem {
                label: "Off".into(),
                ..Default::default()
            },
            RadioItem {
                label: "Noise Cancelling".into(),
                ..Default::default()
            },
            RadioItem {
                label: "Transparency".into(),
                ..Default::default()
            },
        ];
        if self.adaptive_capable() {
            mode.push(RadioItem {
                label: "Adaptive".into(),
                ..Default::default()
            });
        }
        let mut tray_item = vec![
            StandardItem {
                label: self.model.clone(),
                enabled: false,
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            RadioGroup {
                selected: match &self.anc_state {
                    Anc::Off => 0,
                    Anc::NoiseCancelling => 1,
                    Anc::Transparency => 2,
                    Anc::Adaptive => 3,
                },
                select: Box::new(|this: &mut Self, option| {
                    let anc = match option {
                        0 => Anc::Off,
                        1 => Anc::NoiseCancelling,
                        2 => Anc::Transparency,
                        3 => Anc::Adaptive,
                        _ => {
                            log::error!("Unknown ANC option selected: {}", option);
                            Anc::Off
                        }
                    };
                    log::debug!("Setting Anc to {:?}", anc);
                    let self_to_move = this.clone();
                    tokio::spawn(async move {
                        self_to_move.send_anc(Some(anc)).await;
                    });
                }),
                options: mode,
            }
            .into(),
        ];
        if let Some((state, charge)) = self.battery_state.single {
            tray_item.push(
                StandardItem {
                    label: match state {
                        ABBatteryState::Charging => {
                            format!("   󰂈   {}%", charge)
                        }
                        ABBatteryState::Discharging
                        | ABBatteryState::Low25
                        | ABBatteryState::Low10 => {
                            format!("       {}%", charge)
                        }
                        ABBatteryState::Full => {
                            format!("   󰂄   {}%", charge)
                        }
                        ABBatteryState::Unknown | ABBatteryState::Disconnected => {
                            "     NA".to_string()
                        }
                    },
                    enabled: false,
                    ..Default::default()
                }
                .into(),
            );
        }
        if let (Some((lstate, lcharge)), Some((rstate, rcharge))) =
            (self.battery_state.left, self.battery_state.right)
        {
            tray_item.push(
                StandardItem {
                    label: {
                        //TODO: refactor that crap
                        let lsymb = match lstate {
                            ABBatteryState::Charging => {
                                " 󰂈 L󱡒  ".to_owned() + &lcharge.to_string() + "%"
                            }
                            ABBatteryState::Discharging
                            | ABBatteryState::Low25
                            | ABBatteryState::Low10 => {
                                "  L󱡒  ".to_owned() + &lcharge.to_string() + "%"
                            }
                            ABBatteryState::Full => {
                                "󰂄 L󱡒  ".to_owned() + &lcharge.to_string() + "%"
                            }
                            ABBatteryState::Disconnected => "       L󱡑 ".to_string(),
                            ABBatteryState::Unknown => "     L󱡏 NA".to_string(),
                        };
                        let rsymb = match rstate {
                            ABBatteryState::Charging => {
                                " 󰂈 R󱡒  ".to_owned() + &rcharge.to_string() + "%"
                            }
                            ABBatteryState::Discharging
                            | ABBatteryState::Low25
                            | ABBatteryState::Low10 => {
                                "  R󱡒  ".to_owned() + &rcharge.to_string() + "%"
                            }
                            ABBatteryState::Full => {
                                " 󰂄 R󱡏  ".to_owned() + &rcharge.to_string() + "%"
                            }
                            ABBatteryState::Disconnected => "  R󱡑".to_string(),
                            ABBatteryState::Unknown => "  R󱡏 NA".to_string(),
                        };
                        let battery = format!("{} |{}", lsymb, rsymb);
                        battery
                    },
                    enabled: false,
                    ..Default::default()
                }
                .into(),
            );
        }
        if let Some((state, charge)) = self.battery_state.case {
            tray_item.push(
                StandardItem {
                    label: match state {
                        ABBatteryState::Charging => {
                            format!("       󰂈   {}%", charge)
                        }
                        ABBatteryState::Discharging
                        | ABBatteryState::Low25
                        | ABBatteryState::Low10 => {
                            format!("           {}%", charge)
                        }
                        ABBatteryState::Full => {
                            format!("       󰂄   {}%", charge)
                        }
                        ABBatteryState::Unknown | ABBatteryState::Disconnected => {
                            "            NA".to_string()
                        }
                    },
                    enabled: false,
                    ..Default::default()
                }
                .into(),
            );
        }
        tray_item
    }
}

pub fn to_icon(
    options: resvg::usvg::Options,
    transform: resvg::tiny_skia::Transform,
    svg_str: &str,
) -> ksni::Icon {
    let rtree = resvg::usvg::Tree::from_str(svg_str, &options).unwrap_or_else(|e| {
        panic!("Failed to parse SVG: {e}");
    });

    let size = rtree.size();
    let width = size.width() as u32;
    let height = size.height() as u32;

    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height).unwrap();

    resvg::render(&rtree, transform, &mut pixmap.as_mut());

    let argb_data: Vec<u8> = pixmap
        .take()
        .chunks(4)
        .flat_map(|rgba| [rgba[3], rgba[0], rgba[1], rgba[2]])
        .collect();

    ksni::Icon {
        width: size.width() as i32,
        height: size.height() as i32,
        data: argb_data,
    }
}
