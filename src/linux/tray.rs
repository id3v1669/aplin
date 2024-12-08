impl ksni::Tray for crate::common::ab::ABDevice {
    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").into()
    }
    fn icon_name(&self) -> String {
        "help-about".into()
    }
    fn title(&self) -> String {
        "MyTray".into()
    }
    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        log::debug!("ab_device: {:?}", self);
        use ksni::menu::*;
        let mut tray_item = vec![
            StandardItem {
                label: self.model.clone(),
                enabled: false,
                ..Default::default()
            }.into(),
            MenuItem::Separator.into(),
            RadioGroup {
                selected: match self.anc_state {
                    crate::common::ab::ANC::Off => 0,
                    crate::common::ab::ANC::NoiseCancelling => 1,
                    crate::common::ab::ANC::Transparency => 2,
                    crate::common::ab::ANC::Adaptive => 3,
                },
                select: Box::new(|_, _| {}),
                options: vec! [
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
                    RadioItem {
                        label: "Adaptive".into(),
                        ..Default::default()
                    },
                ],
            }.into(),
        ];
        if let Some((state, charge)) = self.battery_state.single {
            tray_item.push(
                StandardItem {
                    label: 
                    {
                        let battery = match state {
                            crate::common::ab::ABBatteryState::Charging =>      format!("   󰂈   {}%", charge),
                            crate::common::ab::ABBatteryState::Discharging =>   format!("       {}%", charge),
                            crate::common::ab::ABBatteryState::Full =>          format!("   󰂄   {}%", charge),
                            crate::common::ab::ABBatteryState::Unknown | 
                            crate::common::ab::ABBatteryState::Disconnected =>  format!("     NA"),
                        };
                        battery
                    },
                    enabled: false,
                    ..Default::default()
                }
                .into(),
            );
        }
        if let (Some((lstate, lcharge)), Some((rstate, rcharge))) = (self.battery_state.left, self.battery_state.right) {
            tray_item.push(
                StandardItem {
                    label: 
                    {
                        //TODO: refactor that crap
                        let lsymb = match lstate {
                            crate::common::ab::ABBatteryState::Charging =>      " 󰂈 l󱡒  ".to_owned() + &lcharge.to_string() + "%",
                            crate::common::ab::ABBatteryState::Discharging =>   "   l󱡒  ".to_owned() + &lcharge.to_string() + "%",
                            crate::common::ab::ABBatteryState::Full =>          " 󰂄 l󱡏  ".to_owned() + &lcharge.to_string() + "%",
                            crate::common::ab::ABBatteryState::Disconnected =>  "   l󱡑     ".to_string(),
                            crate::common::ab::ABBatteryState::Unknown =>       "   l󱡏  NA".to_string(),
                        };
                        let rsymb = match rstate {
                            crate::common::ab::ABBatteryState::Charging =>      " 󰂈 r󱡒  ".to_owned() + &rcharge.to_string() + "%",
                            crate::common::ab::ABBatteryState::Discharging =>   "   r󱡒  ".to_owned() + &rcharge.to_string() + "%",
                            crate::common::ab::ABBatteryState::Full =>          " 󰂄 r󱡏  ".to_owned() + &rcharge.to_string() + "%",
                            crate::common::ab::ABBatteryState::Disconnected =>  "   r󱡑     ".to_string(),
                            crate::common::ab::ABBatteryState::Unknown =>       "   r󱡏  NA".to_string(),
                        };
                        let battery = format!("{}", (lsymb + " | " + &rsymb));
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
                    label: 
                    {
                        let battery = match state {
                            crate::common::ab::ABBatteryState::Charging =>      format!("    󰂈   {}%", charge),
                            crate::common::ab::ABBatteryState::Discharging =>   format!("        {}%", charge),
                            crate::common::ab::ABBatteryState::Full =>          format!("    󰂄   {}%", charge),
                            crate::common::ab::ABBatteryState::Unknown | 
                            crate::common::ab::ABBatteryState::Disconnected =>  format!("      NA"),
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
