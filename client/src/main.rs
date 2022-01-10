/*
 * Copyright 2020, 2021, 2022 sukawasatoru
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

use android_commander::adb_server_recipe::{AdbServerRecipe, AdbServerRecipeEvent};
use android_commander::data::resource::Resource;
use android_commander::model::AndroidDevice;
use android_commander::prelude::*;
use anyhow::Context;
use iced::keyboard::{Event as KeyboardEvent, KeyCode};
use iced::window::Settings as WindowSettings;
use iced::{
    button, executor, pick_list, Application, Button, Checkbox, Clipboard, Column, Command,
    Element, Length, PickList, Row, Settings, Space, Subscription, Svg, Text,
};
use iced_native::subscription::events as native_events;
use iced_native::widget::svg::Handle;
use iced_native::Event as NativeEvent;
use std::hash::Hash;
use std::io::BufRead;
use std::sync::Arc;
use tracing::{debug, info, warn};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum SendEventKey {
    DpadUp,
    DpadDown,
    DpadLeft,
    DpadRight,
    Enter,
    Back,
    Home,
}

impl TryFrom<KeyCode> for SendEventKey {
    type Error = ();

    fn try_from(value: KeyCode) -> Result<Self, Self::Error> {
        use KeyCode::*;

        match value {
            J => Ok(Self::DpadDown),
            K => Ok(Self::DpadUp),
            H => Ok(Self::DpadLeft),
            L => Ok(Self::DpadRight),
            T => Ok(Self::Home),
            Enter => Ok(Self::Enter),
            Backspace => Ok(Self::Back),
            _ => Err(()),
        }
    }
}

impl SendEventKey {
    fn get_android_key_name(&self) -> &'static str {
        match self {
            SendEventKey::DpadUp => "KEYCODE_DPAD_UP",
            SendEventKey::DpadDown => "KEYCODE_DPAD_DOWN",
            SendEventKey::DpadLeft => "KEYCODE_DPAD_LEFT",
            SendEventKey::DpadRight => "KEYCODE_DPAD_RIGHT",
            SendEventKey::Enter => "KEYCODE_ENTER",
            SendEventKey::Back => "KEYCODE_BACK",
            SendEventKey::Home => "KEYCODE_HOME",
        }
    }
}

fn create_pressed_key_command(key: &SendEventKey) -> String {
    format!("down {}", key.get_android_key_name())
}

fn create_release_key_command(key: &SendEventKey) -> String {
    format!("up {}", key.get_android_key_name())
}

fn create_click_key_command(key: &SendEventKey) -> String {
    let code = key.get_android_key_name();
    format!("down {code}\nup {code}", code = code)
}

enum AdbConnectivity {
    Connected,
    Connecting,
    Disconnected,
}

#[derive(Clone, Debug)]
enum AppCommand {
    AdbDevicesSelected(Arc<AndroidDevice>),
    AdbServerRecipeResult(AdbServerRecipeEvent),
    Event(NativeEvent),
    InvokeDevicesResult(Vec<Arc<AndroidDevice>>),
    OnAdbConnectClicked,
    OnAdbDevicesReloadClicked,
    RequestSendEvent(SendEventKey),
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

struct App {
    adb_connectivity: AdbConnectivity,
    adb_devices: Vec<Arc<AndroidDevice>>,
    adb_devices_selected: Option<Arc<AndroidDevice>>,
    adb_server_rx: tokio::sync::watch::Receiver<String>,
    adb_server_tx: tokio::sync::watch::Sender<String>,
    widget_states: WidgetStates,
}

impl Application for App {
    type Executor = executor::Default;
    type Message = AppCommand;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let (adb_server_tx, adb_server_rx) = tokio::sync::watch::channel("".into());
        (
            Self {
                adb_connectivity: AdbConnectivity::Disconnected,
                adb_devices: vec![],
                adb_devices_selected: None,
                adb_server_rx,
                adb_server_tx,
                widget_states: Default::default(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Android Commander".into()
    }

    fn update(
        &mut self,
        message: Self::Message,
        _clipboard: &mut Clipboard,
    ) -> Command<Self::Message> {
        use AppCommand::*;

        match message {
            AdbDevicesSelected(data) => {
                info!(%data, "device selected");
                self.adb_devices_selected = Some(data);
            }
            AdbServerRecipeResult(data) => match data {
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
            Event(data) => {
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
                            debug!("update KeyPressed: {:?}", key_code);

                            let send_event_key = match SendEventKey::try_from(key_code) {
                                Ok(data) => data,
                                Err(_) => return Command::none(),
                            };

                            let ret = self
                                .adb_server_tx
                                .send(create_pressed_key_command(&send_event_key));
                            if let Err(e) = ret {
                                warn!("failed to send the sendevent: {:?}", e);
                            }
                        }
                        KeyboardEvent::KeyReleased { key_code, .. } => {
                            debug!("update KeyReleased: {:?}", key_code);

                            let send_event_key = match SendEventKey::try_from(key_code) {
                                Ok(data) => data,
                                Err(_) => return Command::none(),
                            };

                            let ret = self
                                .adb_server_tx
                                .send(create_release_key_command(&send_event_key));
                            if let Err(e) = ret {
                                warn!("failed to send the sendevent: {:?}", e);
                            }
                        }
                        _ => (),
                    },
                    // TODO: support long-press for button.
                    NativeEvent::Mouse(_) => (),
                    _ => (),
                }
            }
            InvokeDevicesResult(devices) => {
                info!("update InvokeDevicesResult");
                self.adb_devices = devices;
                if let Some(selected) = &self.adb_devices_selected {
                    if !self.adb_devices.iter().any(|data| data == selected) {
                        self.adb_devices_selected = None;
                    }
                }

                return Command::none();
            }
            OnAdbConnectClicked => {
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
            OnAdbDevicesReloadClicked => {
                return Command::perform(invoke_retrieve_devices(), |data| {
                    AppCommand::InvokeDevicesResult(data)
                });
            }
            RequestSendEvent(data) => {
                info!("update RequestSendEvent: {:?}", data);
                match self.adb_connectivity {
                    AdbConnectivity::Connected => (),
                    AdbConnectivity::Connecting | AdbConnectivity::Disconnected => {
                        debug!("skip broadcasting");
                        return Command::none();
                    }
                }

                let ret = self.adb_server_tx.send(create_click_key_command(&data));
                if let Err(e) = ret {
                    warn!("failed to send the sendevent: {:?}", e);
                }
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
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
                    .map(AppCommand::AdbServerRecipeResult),
                    native_events().map(AppCommand::Event),
                ])
            }
            AdbConnectivity::Disconnected => Subscription::none(),
        }
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let button_width = Length::Units(90);
        let button_height = Length::Units(30);

        Column::new()
            .push(
                Row::new()
                    .push(
                        Button::new(
                            &mut self.widget_states.adb_devices_reload_button,
                            Svg::new(Handle::from_memory(
                                Resource::get("refresh_black_24dp.svg")
                                    .context("refresh_black_24dp.svg")
                                    .unwrap()
                                    .data,
                            )),
                        )
                        .on_press(AppCommand::OnAdbDevicesReloadClicked),
                    )
                    .push(PickList::new(
                        &mut self.widget_states.adb_devices_state,
                        &self.adb_devices,
                        self.adb_devices_selected.clone(),
                        AppCommand::AdbDevicesSelected,
                    ))
                    .height(button_height),
            )
            .push(Checkbox::new(
                match self.adb_connectivity {
                    AdbConnectivity::Connecting | AdbConnectivity::Disconnected => false,
                    AdbConnectivity::Connected => true,
                },
                "login",
                |_| AppCommand::OnAdbConnectClicked,
            ))
            .push(Text::new(match self.adb_connectivity {
                AdbConnectivity::Connecting => "adb: connecting",
                AdbConnectivity::Connected => "adb: connected",
                AdbConnectivity::Disconnected => "adb: disconnected",
            }))
            // TODO: support disabled style.
            // TODO: support long press.
            .push(
                Row::new()
                    .push(Space::new(button_width, button_height))
                    .push(
                        Button::new(&mut self.widget_states.button_up, Text::new("Up (k)"))
                            .width(button_width)
                            .height(button_height)
                            .on_press(AppCommand::RequestSendEvent(SendEventKey::DpadUp)),
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
                            .on_press(AppCommand::RequestSendEvent(SendEventKey::DpadLeft)),
                    )
                    .push(
                        // TODO: support disabled style.
                        // TODO: support long press.
                        Button::new(&mut self.widget_states.button_ok, Text::new("Enter"))
                            .width(button_width)
                            .height(button_height)
                            .on_press(AppCommand::RequestSendEvent(SendEventKey::Enter)),
                    )
                    .push(
                        // TODO: support disabled style.
                        // TODO: support long press.
                        Button::new(&mut self.widget_states.button_right, Text::new("Right (l)"))
                            .width(button_width)
                            .height(button_height)
                            .on_press(AppCommand::RequestSendEvent(SendEventKey::DpadRight)),
                    ),
            )
            .push(
                Row::new()
                    .push(Space::new(button_width, button_height))
                    .push(
                        Button::new(&mut self.widget_states.button_down, Text::new("Down (j)"))
                            .width(button_width)
                            .height(button_height)
                            .on_press(AppCommand::RequestSendEvent(SendEventKey::DpadDown)),
                    ),
            )
            .push(
                Row::new()
                    .push(
                        Button::new(&mut self.widget_states.button_back, Text::new("Back"))
                            .width(button_width)
                            .height(button_height)
                            .on_press(AppCommand::RequestSendEvent(SendEventKey::Back)),
                    )
                    .push(
                        Button::new(&mut self.widget_states.button_home, Text::new("Home"))
                            .width(button_width)
                            .height(button_height)
                            .on_press(AppCommand::RequestSendEvent(SendEventKey::Home)),
                    ),
            )
            .into()
    }
}

async fn invoke_retrieve_devices() -> Vec<Arc<AndroidDevice>> {
    match retrieve_devices().await {
        Ok(data) => data.into_iter().map(Arc::new).collect(),
        Err(e) => {
            warn!(?e, "failed to retrieve devices");
            vec![]
        }
    }
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

fn main() -> ! {
    // TODO: disable log.
    #[cfg(target_os = "windows")]
    if false {
        let code = unsafe { winapi::um::wincon::FreeConsole() };
        if code == 0 {
            panic!("unable to detach the console")
        }
    }

    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    info!("Hello");

    App::run(Settings {
        window: WindowSettings {
            size: (270, 320),
            ..Default::default()
        },
        ..Default::default()
    })
    .unwrap();

    unreachable!()
}
