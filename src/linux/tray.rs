impl ksni::Tray for crate::common::ab::ABDevice {
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
        "MyTray".into()
    }
    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        let local_self = self.clone();
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
            MenuItem::Separator.into(),
            RadioGroup {
                selected: match self.anc_state {
                    crate::common::ab::ANC::Off => 0,
                    crate::common::ab::ANC::NoiseCancelling => 1,
                    crate::common::ab::ANC::Transparency => 2,
                    crate::common::ab::ANC::Adaptive => 3,
                },
                select: Box::new(move |_x, option| {
                    let anc = match option {
                        1 => crate::common::ab::ANC::NoiseCancelling,
                        2 => crate::common::ab::ANC::Transparency,
                        3 => crate::common::ab::ANC::Adaptive,
                        0 | _ => crate::common::ab::ANC::Off,
                    };
                    log::debug!("Setting ANC to {:?}", anc);
                    let data_stream_to_send = local_self.data_stream.clone();
                    tokio::spawn(async move {
                        crate::common::ab::ABDevice::send_anc(&data_stream_to_send, anc).await;
                        log::debug!("ANC after set");
                    });
                }),
                options: mode,
            }
            .into(),
        ];
        if let Some((state, charge)) = self.battery_state.single {
            tray_item.push(
                StandardItem {
                    label: {
                        let battery = match state {
                            crate::common::ab::ABBatteryState::Charging => {
                                format!("   󰂈   {}%", charge)
                            }
                            crate::common::ab::ABBatteryState::Discharging
                            | crate::common::ab::ABBatteryState::Low25
                            | crate::common::ab::ABBatteryState::Low10 => {
                                format!("       {}%", charge)
                            }
                            crate::common::ab::ABBatteryState::Full => {
                                format!("   󰂄   {}%", charge)
                            }
                            crate::common::ab::ABBatteryState::Unknown
                            | crate::common::ab::ABBatteryState::Disconnected => {
                                format!("     NA")
                            }
                        };
                        battery
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
                            crate::common::ab::ABBatteryState::Charging => {
                                " 󰂈 L󱡒  ".to_owned() + &lcharge.to_string() + "%"
                            }
                            crate::common::ab::ABBatteryState::Discharging
                            | crate::common::ab::ABBatteryState::Low25
                            | crate::common::ab::ABBatteryState::Low10 => {
                                "  L󱡒  ".to_owned() + &lcharge.to_string() + "%"
                            }
                            crate::common::ab::ABBatteryState::Full => {
                                "󰂄 L󱡒  ".to_owned() + &lcharge.to_string() + "%"
                            }
                            crate::common::ab::ABBatteryState::Disconnected => {
                                "       L󱡑 ".to_string()
                            }
                            crate::common::ab::ABBatteryState::Unknown => "     L󱡏 NA".to_string(),
                        };
                        let rsymb = match rstate {
                            crate::common::ab::ABBatteryState::Charging => {
                                " 󰂈 R󱡒  ".to_owned() + &rcharge.to_string() + "%"
                            }
                            crate::common::ab::ABBatteryState::Discharging
                            | crate::common::ab::ABBatteryState::Low25
                            | crate::common::ab::ABBatteryState::Low10 => {
                                "  R󱡒  ".to_owned() + &rcharge.to_string() + "%"
                            }
                            crate::common::ab::ABBatteryState::Full => {
                                " 󰂄 R󱡏  ".to_owned() + &rcharge.to_string() + "%"
                            }
                            crate::common::ab::ABBatteryState::Disconnected => "  R󱡑".to_string(),
                            crate::common::ab::ABBatteryState::Unknown => "  R󱡏 NA".to_string(),
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
                    label: {
                        let battery = match state {
                            crate::common::ab::ABBatteryState::Charging => {
                                format!("       󰂈   {}%", charge)
                            }
                            crate::common::ab::ABBatteryState::Discharging
                            | crate::common::ab::ABBatteryState::Low25
                            | crate::common::ab::ABBatteryState::Low10 => {
                                format!("           {}%", charge)
                            }
                            crate::common::ab::ABBatteryState::Full => {
                                format!("       󰂄   {}%", charge)
                            }
                            crate::common::ab::ABBatteryState::Unknown
                            | crate::common::ab::ABBatteryState::Disconnected => {
                                format!("            NA")
                            }
                        };
                        battery
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
