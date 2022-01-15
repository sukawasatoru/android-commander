/*
 * Copyright 2022 sukawasatoru
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
use crate::feature::main::adb_server_recipe::{AdbServerRecipe, AdbServerRecipeEvent};
use crate::model::preferences::{KeyMap, Preferences};
use crate::model::send_event_key::SendEventKey;
use crate::model::AndroidDevice;
use crate::prelude::*;
use iced::keyboard::{Event as KeyboardEvent, KeyCode};
use iced::svg::Handle as SvgHandle;
use iced::{
    button, pick_list, Button, Checkbox, Column, Command, Element, Length, PickList, Row,
    Subscription, Svg, Text,
};
use iced_native::subscription::events as native_events;
use iced_native::widget::Space;
use iced_native::Event as NativeEvent;
use std::io::BufRead;
use std::sync::Arc;
use tracing::{debug, info, warn};

#[derive(Clone, Debug)]
pub enum MainViewCommand {
    AdbDevicesSelected(Arc<AndroidDevice>),
    AdbServerRecipeResult(AdbServerRecipeEvent),
    Event(NativeEvent),
    InvokeDevicesResult(Vec<Arc<AndroidDevice>>),
    OnAdbConnectClicked,
    OnAdbDevicesReloadClicked,
    OnNewPrefs(Option<Arc<Preferences>>),
    RequestSendEvent(SendEventKey),
}

pub struct MainView {
    adb_connectivity: AdbConnectivity,
    adb_devices: Vec<Arc<AndroidDevice>>,
    adb_devices_selected: Option<Arc<AndroidDevice>>,
    adb_server_rx: tokio::sync::watch::Receiver<String>,
    adb_server_tx: tokio::sync::watch::Sender<String>,
    prefs: Arc<Preferences>,
    widget_states: WidgetStates,
}

impl MainView {}

enum AdbConnectivity {
    Connected,
    Connecting,
    Disconnected,
}

#[derive(Debug, Default)]
struct WidgetStates {
    adb_devices_reload_button: button::State,
    adb_devices_state: pick_list::State<Arc<AndroidDevice>>,
    button_up: button::State,
    button_down: button::State,
    button_left: button::State,
    button_right: button::State,
    button_ok: button::State,
    button_back: button::State,
    button_home: button::State,
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
            widget_states: Default::default(),
        }
    }

    pub fn init_command() -> Command<MainViewCommand> {
        retrieve_devices_command()
    }

    pub fn update(&mut self, command: MainViewCommand) -> Command<MainViewCommand> {
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
                        return Command::none();
                    }
                }

                match data {
                    NativeEvent::Keyboard(data) => match data {
                        KeyboardEvent::KeyPressed { key_code, .. } => {
                            debug!(?key_code, "update KeyPressed");

                            let send_event_key = match create_send_event_key(key_code) {
                                Some(data) => data,
                                None => return Command::none(),
                            };

                            let ret = self.adb_server_tx.send(create_pressed_key_command(
                                &self.prefs.key_map,
                                &send_event_key,
                            ));

                            if let Err(e) = ret {
                                warn!(?e, "failed to send the sendevent");
                            }
                        }
                        KeyboardEvent::KeyReleased { key_code, .. } => {
                            debug!(?key_code, "update KeyReleased");

                            let send_event_key = match create_send_event_key(key_code) {
                                Some(data) => data,
                                None => return Command::none(),
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
                    // TODO: support long-press for button.
                    NativeEvent::Mouse(_) => (),
                    _ => (),
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
                    return Command::none();
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
                        return Command::none();
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
                if let Some(data) = prefs {
                    self.prefs = data;
                }
            }
        }
        Command::none()
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
                    Subscription::from_recipe(AdbServerRecipe {
                        device,
                        rx: self.adb_server_rx.clone(),
                    })
                    .map(MainViewCommand::AdbServerRecipeResult),
                    native_events().map(MainViewCommand::Event),
                ])
            }
            AdbConnectivity::Disconnected => Subscription::none(),
        }
    }

    pub fn view(&mut self) -> Element<'_, MainViewCommand> {
        let button_width = Length::Units(90);
        let button_height = Length::Units(30);

        Column::new()
            .push(Text::new("ADB:"))
            .push(
                Row::new()
                    .push(
                        Button::new(
                            &mut self.widget_states.adb_devices_reload_button,
                            Svg::new(SvgHandle::from_memory(
                                Resource::get("refresh_black_24dp.svg")
                                    .context("refresh_black_24dp.svg")
                                    .unwrap()
                                    .data,
                            )),
                        )
                        .on_press(MainViewCommand::OnAdbDevicesReloadClicked),
                    )
                    .push(PickList::new(
                        &mut self.widget_states.adb_devices_state,
                        &self.adb_devices,
                        self.adb_devices_selected.clone(),
                        MainViewCommand::AdbDevicesSelected,
                    ))
                    .height(button_height),
            )
            .push(Checkbox::new(
                match self.adb_connectivity {
                    AdbConnectivity::Connecting | AdbConnectivity::Disconnected => false,
                    AdbConnectivity::Connected => true,
                },
                "connect",
                |_| MainViewCommand::OnAdbConnectClicked,
            ))
            .push(Text::new(match self.adb_connectivity {
                AdbConnectivity::Connecting => "status: connecting",
                AdbConnectivity::Connected => "status: connected",
                AdbConnectivity::Disconnected => "status: disconnected",
            }))
            .push(Space::new(Length::Shrink, Length::Units(16)))
            // TODO: support disabled style.
            // TODO: support long press.
            .push(
                Row::new()
                    .push(Space::new(button_width, button_height))
                    .push(
                        Button::new(&mut self.widget_states.button_up, Text::new("Up (k)"))
                            .width(button_width)
                            .height(button_height)
                            .on_press(MainViewCommand::RequestSendEvent(SendEventKey::DpadUp)),
                    ),
            )
            // TODO: support disabled style.
            // TODO: support long press.
            .push(
                Row::new()
                    .push(
                        // TODO: support disabled style.
                        // TODO: support long press.
                        Button::new(&mut self.widget_states.button_left, Text::new("Left (h)"))
                            .width(button_width)
                            .height(button_height)
                            .on_press(MainViewCommand::RequestSendEvent(SendEventKey::DpadLeft)),
                    )
                    .push(
                        // TODO: support disabled style.
                        // TODO: support long press.
                        Button::new(&mut self.widget_states.button_ok, Text::new("OK"))
                            .width(button_width)
                            .height(button_height)
                            .on_press(MainViewCommand::RequestSendEvent(SendEventKey::Ok)),
                    )
                    .push(
                        // TODO: support disabled style.
                        // TODO: support long press.
                        Button::new(&mut self.widget_states.button_right, Text::new("Right (l)"))
                            .width(button_width)
                            .height(button_height)
                            .on_press(MainViewCommand::RequestSendEvent(SendEventKey::DpadRight)),
                    ),
            )
            .push(
                Row::new()
                    .push(Space::new(button_width, button_height))
                    .push(
                        Button::new(&mut self.widget_states.button_down, Text::new("Down (j)"))
                            .width(button_width)
                            .height(button_height)
                            .on_press(MainViewCommand::RequestSendEvent(SendEventKey::DpadDown)),
                    ),
            )
            .push(
                Row::new()
                    .push(
                        Button::new(&mut self.widget_states.button_back, Text::new("Back"))
                            .width(button_width)
                            .height(button_height)
                            .on_press(MainViewCommand::RequestSendEvent(SendEventKey::Back)),
                    )
                    .push(
                        Button::new(&mut self.widget_states.button_home, Text::new("Home"))
                            .width(button_width)
                            .height(button_height)
                            .on_press(MainViewCommand::RequestSendEvent(SendEventKey::Home)),
                    ),
            )
            .into()
    }

    pub fn view_size() -> (u32, u32) {
        (270, 320)
    }
}

fn create_send_event_key(key: KeyCode) -> Option<SendEventKey> {
    match key {
        KeyCode::J => Some(SendEventKey::DpadDown),
        KeyCode::K => Some(SendEventKey::DpadUp),
        KeyCode::H => Some(SendEventKey::DpadLeft),
        KeyCode::L => Some(SendEventKey::DpadRight),
        KeyCode::T => Some(SendEventKey::Home),
        KeyCode::Enter => Some(SendEventKey::Ok),
        KeyCode::Backspace => Some(SendEventKey::Back),
        _ => None,
    }
}

fn get_key<'a>(key_map: &'a KeyMap, key: &SendEventKey) -> &'a str {
    match key {
        SendEventKey::DpadUp => &key_map.dpad_up,
        SendEventKey::DpadDown => &key_map.dpad_down,
        SendEventKey::DpadLeft => &key_map.dpad_left,
        SendEventKey::DpadRight => &key_map.dpad_right,
        SendEventKey::Ok => &key_map.dpad_ok,
        SendEventKey::Back => &key_map.back,
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

fn retrieve_devices_command() -> Command<MainViewCommand> {
    Command::perform(
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
    let child = std::process::Command::new("adb")
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
