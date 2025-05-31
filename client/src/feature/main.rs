/*
 * Copyright 2022, 2025 sukawasatoru
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

mod adb_server_recipe;

use crate::data::resource::Resource;
use crate::feature::main::adb_server_recipe::{adb_server, find_adb_path, AdbServerRecipeEvent};
use crate::model::send_event_key::SendEventKey;
use crate::model::{AndroidDevice, KeyMap, Preferences, XMessage};
use crate::prelude::*;
use iced::keyboard::{self, key, Key};
use iced::widget::{
    button, checkbox, column, container, pick_list, row, svg, svg::Handle as SvgHandle, Space,
};
use iced::{Background, Color, Element, Event as NativeEvent, Length, Size, Subscription, Task};
use std::io::BufRead;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum MainViewCommand {
    AdbDevicesSelected(Arc<AndroidDevice>),
    AdbServerRecipeResult(AdbServerRecipeEvent),
    Event(NativeEvent),
    InvokeDevicesResult(Vec<Arc<AndroidDevice>>),
    OnAdbConnectClicked,
    OnAdbDevicesReloadClicked,
    OnNewPrefs(Option<Arc<Preferences>>),
    OnXMessage(XMessage),
    RequestSendEvent(SendEventKey),
}

pub struct MainView {
    adb_connectivity: AdbConnectivity,
    adb_devices: Vec<Arc<AndroidDevice>>,
    adb_devices_selected: Option<Arc<AndroidDevice>>,
    adb_server_rx: tokio::sync::watch::Receiver<String>,
    adb_server_tx: tokio::sync::watch::Sender<String>,
    prefs: Arc<Preferences>,
}

enum AdbConnectivity {
    Connected,
    Connecting,
    Disconnected,
}

impl MainView {
    pub fn new(prefs: Arc<Preferences>) -> Self {
        let (adb_server_tx, adb_server_rx) = tokio::sync::watch::channel("".into());
        Self {
            adb_connectivity: AdbConnectivity::Disconnected,
            adb_devices: vec![],
            adb_devices_selected: None,
            adb_server_rx,
            adb_server_tx,
            prefs,
        }
    }

    pub fn init_command() -> Task<MainViewCommand> {
        retrieve_devices_command()
    }

    pub fn update(&mut self, command: MainViewCommand) -> Task<MainViewCommand> {
        match command {
            MainViewCommand::AdbDevicesSelected(data) => {
                info!(%data, "device selected");
                self.adb_devices_selected = Some(data);
            }
            MainViewCommand::AdbServerRecipeResult(data) => match data {
                AdbServerRecipeEvent::Connected => {
                    info!("adb connected");
                    self.adb_connectivity = AdbConnectivity::Connected;
                }
                AdbServerRecipeEvent::Error => {
                    info!("some error occurred");
                }
                AdbServerRecipeEvent::Disconnected => {
                    info!("adb disconnected");
                    self.adb_connectivity = AdbConnectivity::Disconnected;
                    self.adb_server_tx.send("".into()).ok();
                }
            },
            MainViewCommand::Event(data) => {
                match self.adb_connectivity {
                    AdbConnectivity::Connected => (),
                    AdbConnectivity::Connecting | AdbConnectivity::Disconnected => {
                        debug!("skip broadcasting");
                        return Task::none();
                    }
                }

                match data {
                    iced::Event::Keyboard(data) => match data {
                        keyboard::Event::KeyPressed { key, .. } => {
                            debug!(?key, "update KeyPressed");

                            let send_event_key = match create_send_event_key(key) {
                                Some(data) => data,
                                None => return Task::none(),
                            };

                            let ret = self.adb_server_tx.send(create_pressed_key_command(
                                &self.prefs.key_map,
                                &send_event_key,
                            ));

                            if let Err(e) = ret {
                                warn!(?e, "failed to send the sendevent");
                            }
                        }
                        keyboard::Event::KeyReleased { key, .. } => {
                            debug!(?key, "update KeyReleased");

                            let send_event_key = match create_send_event_key(key) {
                                Some(data) => data,
                                None => return Task::none(),
                            };

                            let ret = self.adb_server_tx.send(create_release_key_command(
                                &self.prefs.key_map,
                                &send_event_key,
                            ));

                            if let Err(e) = ret {
                                warn!(?e, "failed to send the sendevent");
                            }
                        }
                        _ => (),
                    },
                    NativeEvent::Mouse(_) => {
                        // TODO: support long-press for button.
                    }
                    NativeEvent::Window(_) | NativeEvent::Touch(_) => {
                        // do nothing.
                    }
                }
            }
            MainViewCommand::InvokeDevicesResult(devices) => {
                info!("update InvokeDevicesResult");
                self.adb_devices = devices;
                match &self.adb_devices_selected {
                    Some(selected) => {
                        if !self.adb_devices.iter().any(|data| data == selected) {
                            self.adb_devices_selected = None;
                        }
                    }
                    None => {
                        if let Some(data) = self.adb_devices.first() {
                            self.adb_devices_selected = Some(data.clone())
                        }
                    }
                }
            }
            MainViewCommand::OnAdbConnectClicked => {
                if self.adb_devices_selected.is_none() {
                    info!("need to select device");
                    return Task::none();
                }

                match self.adb_connectivity {
                    AdbConnectivity::Disconnected => {
                        self.adb_connectivity = AdbConnectivity::Connecting
                    }
                    AdbConnectivity::Connecting => {
                        warn!("TODO");
                    }
                    AdbConnectivity::Connected => {
                        self.adb_connectivity = AdbConnectivity::Disconnected;
                        self.adb_server_tx.send("".into()).ok();
                    }
                }
            }
            MainViewCommand::OnAdbDevicesReloadClicked => {
                return retrieve_devices_command();
            }
            MainViewCommand::RequestSendEvent(data) => {
                info!(?data, "update RequestSendEvent");
                match self.adb_connectivity {
                    AdbConnectivity::Connected => (),
                    AdbConnectivity::Connecting | AdbConnectivity::Disconnected => {
                        debug!("skip broadcasting");
                        return Task::none();
                    }
                }

                let ret = self
                    .adb_server_tx
                    .send(create_click_key_command(&self.prefs.key_map, &data));

                if let Err(e) = ret {
                    warn!(?e, "failed to send the sendevent");
                }
            }
            MainViewCommand::OnNewPrefs(prefs) => {
                info!("OnNewPreferences");

                if let Some(data) = prefs {
                    self.prefs = data;
                }
            }
            MainViewCommand::OnXMessage(_) => {
                // do nothing.
            }
        }
        Task::none()
    }

    pub fn subscription(&self) -> Subscription<MainViewCommand> {
        match self.adb_connectivity {
            AdbConnectivity::Connecting | AdbConnectivity::Connected => {
                let device = match &self.adb_devices_selected {
                    Some(data) => data.clone(),
                    None => {
                        warn!("device not selected");
                        return Subscription::none();
                    }
                };

                Subscription::batch(vec![
                    adb_server(device, self.adb_server_rx.clone())
                        .map(MainViewCommand::AdbServerRecipeResult),
                    iced::event::listen().map(MainViewCommand::Event),
                ])
            }
            AdbConnectivity::Disconnected => Subscription::none(),
        }
    }

    pub fn view(&self) -> Element<MainViewCommand> {
        let button_width = Length::Fixed(90.0);
        let button_height = Length::Fixed(30.0);

        column![
            "ADB:",
            row![
                button(
                    svg(SvgHandle::from_memory(
                        Resource::get("arrow-path.svg")
                            .context("arrow-path.svg")
                            .unwrap()
                            .data,
                    ))
                    .width(24)
                )
                .style(button::secondary)
                .on_press(MainViewCommand::OnAdbDevicesReloadClicked),
                pick_list(
                    self.adb_devices.clone(),
                    self.adb_devices_selected.clone(),
                    MainViewCommand::AdbDevicesSelected,
                ),
            ]
            .height(button_height),
            Space::with_height(4),
            checkbox(
                "connect",
                match self.adb_connectivity {
                    AdbConnectivity::Connecting | AdbConnectivity::Disconnected => false,
                    AdbConnectivity::Connected => true,
                },
            )
            .on_toggle(|_| MainViewCommand::OnAdbConnectClicked),
            match self.adb_connectivity {
                AdbConnectivity::Connecting => "status: connecting",
                AdbConnectivity::Connected => "status: connected",
                AdbConnectivity::Disconnected => "status: disconnected",
            },
            Space::with_height(16),
            row![
                button(Space::new(Length::Fill, Length::Fill))
                    .width(70)
                    .height(button_height)
                    .style(|theme, status| {
                        button::Style {
                            background: Some(Background::Color(Color::new(1.0, 0.0, 0.0, 1.0))),
                            ..button::secondary(theme, status)
                        }
                    })
                    .on_press(MainViewCommand::RequestSendEvent(SendEventKey::ColorRed)),
                button(Space::new(Length::Fill, Length::Fill))
                    .width(70)
                    .height(button_height)
                    .style(|theme, status| {
                        button::Style {
                            background: Some(Background::Color(Color::new(0.0, 1.0, 0.0, 1.0))),
                            ..button::secondary(theme, status)
                        }
                    })
                    .on_press(MainViewCommand::RequestSendEvent(SendEventKey::ColorGreen)),
                button(Space::new(Length::Fill, Length::Fill))
                    .width(70)
                    .height(button_height)
                    .style(|theme, status| {
                        button::Style {
                            background: Some(Background::Color(Color::new(0.0, 0.0, 1.0, 1.0))),
                            ..button::secondary(theme, status)
                        }
                    })
                    .on_press(MainViewCommand::RequestSendEvent(SendEventKey::ColorBlue)),
                button(Space::new(Length::Fill, Length::Fill))
                    .width(70)
                    .height(button_height)
                    .style(|theme, status| {
                        button::Style {
                            background: Some(Background::Color(Color::new(1.0, 1.0, 0.0, 1.0))),
                            ..button::secondary(theme, status)
                        }
                    })
                    .on_press(MainViewCommand::RequestSendEvent(SendEventKey::ColorYellow)),
            ]
            .spacing(4),
            Space::with_height(8),
            row![
                Space::with_width(90 + 8),
                button("Up (k)")
                    .width(button_width)
                    .height(button_height)
                    .style(button::secondary)
                    .on_press(MainViewCommand::RequestSendEvent(SendEventKey::DpadUp)),
            ]
            .spacing(4),
            Space::with_height(4),
            row![
                Space::with_width(4),
                button("Left (h)")
                    .width(button_width)
                    .height(button_height)
                    .style(button::secondary)
                    .on_press(MainViewCommand::RequestSendEvent(SendEventKey::DpadLeft)),
                button("OK")
                    .width(button_width)
                    .height(button_height)
                    .style(button::secondary)
                    .on_press(MainViewCommand::RequestSendEvent(SendEventKey::DpadOk)),
                button("Right (l)")
                    .width(button_width)
                    .height(button_height)
                    .style(button::secondary)
                    .on_press(MainViewCommand::RequestSendEvent(SendEventKey::DpadRight)),
            ]
            .spacing(4),
            Space::with_height(4),
            row![
                Space::with_width(90 + 8),
                button("Down (j)")
                    .width(button_width)
                    .height(button_height)
                    .style(button::secondary)
                    .on_press(MainViewCommand::RequestSendEvent(SendEventKey::DpadDown)),
                Space::new(button_width, button_height),
            ]
            .spacing(4),
            Space::with_height(8),
            row![
                Space::with_width(4),
                button("Back")
                    .width(button_width)
                    .height(button_height)
                    .style(button::secondary)
                    .on_press(MainViewCommand::RequestSendEvent(SendEventKey::Back)),
                button("Home")
                    .width(button_width)
                    .height(button_height)
                    .style(button::secondary)
                    .on_press(MainViewCommand::RequestSendEvent(SendEventKey::Home)),
            ]
            .spacing(4),
            Space::with_height(8),
            row![
                Space::with_width(4),
                button(container("1").center_x(Length::Fill))
                    .width(button_width)
                    .height(button_height)
                    .style(button::secondary)
                    .on_press(MainViewCommand::RequestSendEvent(SendEventKey::Num1)),
                button(container("2").center_x(Length::Fill))
                    .width(button_width)
                    .height(button_height)
                    .style(button::secondary)
                    .on_press(MainViewCommand::RequestSendEvent(SendEventKey::Num2)),
                button(container("3").center_x(Length::Fill))
                    .width(button_width)
                    .height(button_height)
                    .style(button::secondary)
                    .on_press(MainViewCommand::RequestSendEvent(SendEventKey::Num3)),
            ]
            .spacing(4),
            Space::with_height(4),
            row![
                Space::with_width(4),
                button(container("4").center_x(Length::Fill))
                    .width(button_width)
                    .height(button_height)
                    .style(button::secondary)
                    .on_press(MainViewCommand::RequestSendEvent(SendEventKey::Num4)),
                button(container("5").center_x(Length::Fill))
                    .width(button_width)
                    .height(button_height)
                    .style(button::secondary)
                    .on_press(MainViewCommand::RequestSendEvent(SendEventKey::Num5)),
                button(container("6").center_x(Length::Fill))
                    .width(button_width)
                    .height(button_height)
                    .style(button::secondary)
                    .on_press(MainViewCommand::RequestSendEvent(SendEventKey::Num6)),
            ]
            .spacing(4),
            Space::with_height(4),
            row![
                Space::with_width(4),
                button(container("7").center_x(Length::Fill))
                    .width(button_width)
                    .height(button_height)
                    .style(button::secondary)
                    .on_press(MainViewCommand::RequestSendEvent(SendEventKey::Num7)),
                button(container("8").center_x(Length::Fill))
                    .width(button_width)
                    .height(button_height)
                    .style(button::secondary)
                    .on_press(MainViewCommand::RequestSendEvent(SendEventKey::Num8)),
                button(container("9").center_x(Length::Fill))
                    .width(button_width)
                    .height(button_height)
                    .style(button::secondary)
                    .on_press(MainViewCommand::RequestSendEvent(SendEventKey::Num9)),
            ]
            .spacing(4),
            Space::with_height(4),
            row![
                Space::with_width(90 + 8),
                button(container("0").center_x(Length::Fill))
                    .width(button_width)
                    .height(button_height)
                    .style(button::secondary)
                    .on_press(MainViewCommand::RequestSendEvent(SendEventKey::Num0)),
            ]
            .spacing(4),
        ]
        .into()
    }

    pub fn view_size() -> Size {
        Size::new(300.0, 480.0)
    }
}

fn create_send_event_key(key: Key) -> Option<SendEventKey> {
    match key.as_ref() {
        Key::Character("1") => Some(SendEventKey::Num1),
        Key::Character("2") => Some(SendEventKey::Num2),
        Key::Character("3") => Some(SendEventKey::Num3),
        Key::Character("4") => Some(SendEventKey::Num4),
        Key::Character("5") => Some(SendEventKey::Num5),
        Key::Character("6") => Some(SendEventKey::Num6),
        Key::Character("7") => Some(SendEventKey::Num7),
        Key::Character("8") => Some(SendEventKey::Num8),
        Key::Character("9") => Some(SendEventKey::Num9),
        Key::Character("0") => Some(SendEventKey::Num0),
        Key::Character("j") => Some(SendEventKey::DpadDown),
        Key::Character("k") => Some(SendEventKey::DpadUp),
        Key::Character("h") => Some(SendEventKey::DpadLeft),
        Key::Character("l") => Some(SendEventKey::DpadRight),
        Key::Character("t") => Some(SendEventKey::Home),
        Key::Named(key::Named::Enter) => Some(SendEventKey::DpadOk),
        Key::Named(key::Named::Backspace) => Some(SendEventKey::Back),
        _ => None,
    }
}

fn get_key<'a>(key_map: &'a KeyMap, key: &SendEventKey) -> &'a str {
    match key {
        SendEventKey::Back => &key_map.back,
        SendEventKey::ColorRed => &key_map.color_red,
        SendEventKey::ColorGreen => &key_map.color_green,
        SendEventKey::ColorBlue => &key_map.color_blue,
        SendEventKey::ColorYellow => &key_map.color_yellow,
        SendEventKey::DpadUp => &key_map.dpad_up,
        SendEventKey::DpadDown => &key_map.dpad_down,
        SendEventKey::DpadLeft => &key_map.dpad_left,
        SendEventKey::DpadRight => &key_map.dpad_right,
        SendEventKey::DpadOk => &key_map.dpad_ok,
        SendEventKey::Num0 => &key_map.num_0,
        SendEventKey::Num1 => &key_map.num_1,
        SendEventKey::Num2 => &key_map.num_2,
        SendEventKey::Num3 => &key_map.num_3,
        SendEventKey::Num4 => &key_map.num_4,
        SendEventKey::Num5 => &key_map.num_5,
        SendEventKey::Num6 => &key_map.num_6,
        SendEventKey::Num7 => &key_map.num_7,
        SendEventKey::Num8 => &key_map.num_8,
        SendEventKey::Num9 => &key_map.num_9,
        SendEventKey::Home => &key_map.home,
    }
}

fn create_pressed_key_command(key_map: &KeyMap, key: &SendEventKey) -> String {
    format!("down {}", get_key(key_map, key))
}

fn create_release_key_command(key_map: &KeyMap, key: &SendEventKey) -> String {
    format!("up {}", get_key(key_map, key))
}

fn create_click_key_command(key_map: &KeyMap, key: &SendEventKey) -> String {
    format!("down {code}\nup {code}", code = get_key(key_map, key))
}

fn retrieve_devices_command() -> Task<MainViewCommand> {
    Task::perform(
        async {
            match retrieve_devices().await {
                Ok(data) => data.into_iter().map(Arc::new).collect(),
                Err(e) => {
                    warn!(?e, "failed to retrieve devices");
                    vec![]
                }
            }
        },
        MainViewCommand::InvokeDevicesResult,
    )
}

async fn retrieve_devices() -> Fallible<Vec<AndroidDevice>> {
    let child = std::process::Command::new(find_adb_path())
        .arg("devices")
        .stdout(std::process::Stdio::piped())
        .spawn()
        .context("failed to invoke adb command")?;

    let mut reader = std::io::BufReader::new(child.stdout.context("adb stdout")?);
    let mut buf = String::new();
    let mut devices = vec![];
    loop {
        buf.clear();
        let bytes = reader.read_line(&mut buf).context("failed to read line")?;
        if bytes == 0 {
            break;
        }

        let segments = buf.split('\t').collect::<Vec<_>>();
        if segments.len() != 2 {
            debug!(%buf, "skip line");
            continue;
        }
        devices.push(AndroidDevice {
            serial: segments[0].to_string(),
        });
    }

    Ok(devices)
}
