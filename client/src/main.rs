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

use anyhow::Context as AnyhowContext;
use iced::futures::stream::BoxStream;
use iced::keyboard::{Event as KeyboardEvent, KeyCode};
use iced::window::Settings as WindowSettings;
use iced::{
    button, executor, futures, Application, Button, Checkbox, Clipboard, Column, Command, Element,
    Length, Row, Settings, Space, Subscription, Text,
};
use iced_futures::subscription::Recipe;
use iced_native::subscription::events as native_events;
use iced_native::Event as NativeEvent;
use rust_embed::RustEmbed;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::File;
use std::hash::Hash;
use std::io::prelude::*;
use std::num::ParseIntError;
use tracing::{debug, info, warn};

#[derive(RustEmbed)]
#[folder = "../server/app/build/outputs"]
#[include = "android-commander-server"]
struct Asset;

#[derive(Clone, Debug)]
enum AdbServerRecipeEvent {
    Connected,
    Disconnected,
    Error,
}

enum AdbServerRecipeInternalState {
    Init(tokio::sync::watch::Receiver<String>),
    Ready(tokio::sync::watch::Receiver<String>, std::process::Child),
    Disconnecting,
    Finish,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum SendEventKey {
    KeyDpadUpClick,
    KeyDpadDownClick,
    KeyDpadLeftClick,
    KeyDpadRightClick,
    KeyEnterClick,
    KeyBackClick,
    KeyHomeClick,
}

impl TryFrom<KeyCode> for SendEventKey {
    type Error = ();

    fn try_from(value: KeyCode) -> Result<Self, Self::Error> {
        use KeyCode::*;

        match value {
            J => Ok(Self::KeyDpadDownClick),
            K => Ok(Self::KeyDpadUpClick),
            H => Ok(Self::KeyDpadLeftClick),
            L => Ok(Self::KeyDpadRightClick),
            T => Ok(Self::KeyHomeClick),
            Enter => Ok(Self::KeyEnterClick),
            Backspace => Ok(Self::KeyBackClick),
            _ => Err(()),
        }
    }
}

impl SendEventKey {
    fn get_android_key_name(&self) -> &'static str {
        match self {
            SendEventKey::KeyDpadUpClick => "KEYCODE_DPAD_UP",
            SendEventKey::KeyDpadDownClick => "KEYCODE_DPAD_DOWN",
            SendEventKey::KeyDpadLeftClick => "KEYCODE_DPAD_LEFT",
            SendEventKey::KeyDpadRightClick => "KEYCODE_DPAD_RIGHT",
            SendEventKey::KeyEnterClick => "KEYCODE_ENTER",
            SendEventKey::KeyBackClick => "KEYCODE_BACK",
            SendEventKey::KeyHomeClick => "KEYCODE_HOME",
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

struct AdbServerRecipe {
    rx: tokio::sync::watch::Receiver<String>,
}

impl<H, I> Recipe<H, I> for AdbServerRecipe
where
    H: std::hash::Hasher,
{
    type Output = AdbServerRecipeEvent;

    fn hash(&self, state: &mut H) {
        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(self: Box<Self>, _: BoxStream<'_, I>) -> BoxStream<'_, Self::Output> {
        use AdbServerRecipeEvent as RecipeEvent;
        use AdbServerRecipeInternalState as RecipeState;

        Box::pin(futures::stream::unfold(
            RecipeState::Init(self.rx),
            |state| async move {
                match state {
                    RecipeState::Init(rx) => {
                        let server_path = match tempfile::tempdir() {
                            Ok(data) => data.path().join("android-commander-server"),
                            Err(e) => {
                                warn!(?e, "failed to prepare temporary directory");
                                return Some((RecipeEvent::Error, RecipeState::Finish));
                            }
                        };

                        info!(?server_path);

                        match std::fs::create_dir_all(&server_path.parent().unwrap()) {
                            Ok(_) => (),
                            Err(e) => {
                                warn!(?e, "failed to create temporary directory");
                                return Some((RecipeEvent::Error, RecipeState::Finish));
                            }
                        }

                        let server_file = match File::create(&server_path) {
                            Ok(data) => data,
                            Err(e) => {
                                warn!(?e, "failed to create temporary file");
                                return Some((RecipeEvent::Error, RecipeState::Finish));
                            }
                        };

                        let server_bin = match Asset::get("android-commander-server") {
                            Some(data) => data as rust_embed::EmbeddedFile,
                            None => {
                                warn!("failed to get asset");
                                return Some((RecipeEvent::Error, RecipeState::Finish));
                            }
                        };

                        let mut buf = std::io::BufWriter::new(server_file);
                        match buf.write_all(&server_bin.data) {
                            Ok(_) => (),
                            Err(e) => {
                                warn!(?e, "failed to write server data");
                                return Some((RecipeEvent::Error, RecipeState::Finish));
                            }
                        }

                        match std::process::Command::new("adb")
                            .args(&[
                                "push",
                                server_path.to_str().unwrap(),
                                "/data/local/tmp/android-commander-server",
                            ])
                            .spawn()
                        {
                            Ok(_) => (),
                            Err(e) => {
                                warn!(?e, "failed to push server file");
                                return Some((RecipeEvent::Error, RecipeState::Finish));
                            }
                        }

                        // TODO:
                        match std::process::Command::new("adb")
                            .args(&["shell", "CLASSPATH=/data/local/tmp/android-commander-server app_process / jp.tinyport.androidcommander.server.MainKt"])
                            .stdin(std::process::Stdio::piped())
                            .spawn()
                        {
                            Ok(mut data) => match &data.stdin {
                                Some(_) => Some((RecipeEvent::Connected, RecipeState::Ready(rx, data))),
                                None => {
                                    warn!("stdin not found");
                                    data.kill().ok();
                                    data.wait().ok();
                                    Some((RecipeEvent::Error, RecipeState::Finish))
                                }
                            },
                            Err(e) => {
                                warn!("{:?}", e);
                                Some((RecipeEvent::Error, RecipeState::Finish))
                            }
                        }
                    }
                    RecipeState::Ready(mut rx, mut child) => {
                        loop {
                            if rx.changed().await.is_err() {
                                break;
                            }

                            let data = rx.borrow();
                            debug!("send data: {}", data.as_str());

                            // for ignore init value.
                            if data.is_empty() {
                                continue;
                            }

                            let ret = writeln!(child.stdin.as_mut().unwrap(), "{}", data.as_str());
                            if let Err(e) = ret {
                                warn!("{:?}", e);
                                child.kill().ok();
                                child.wait().ok();
                                return Some((RecipeEvent::Error, RecipeState::Disconnecting));
                            }
                        }

                        debug!("channel closed");
                        child.kill().ok();
                        child.wait().ok();
                        Some((RecipeEvent::Disconnected, RecipeState::Finish))
                    }
                    RecipeState::Disconnecting => {
                        Some((RecipeEvent::Disconnected, RecipeState::Finish))
                    }
                    RecipeState::Finish => {
                        debug!("finish");
                        None
                    }
                }
            },
        ))
    }
}

enum AdbConnectivity {
    Connected,
    Connecting,
    Disconnected,
}

#[derive(Clone, Debug)]
enum AppCommand {
    AdbServerRecipeResult(AdbServerRecipeEvent),
    Event(NativeEvent),
    InvokeAdbResult,
    OnAdbButton,
    OnAdbConnectClicked,
    RequestSendEvent(SendEventKey),
}

#[derive(Debug, Default)]
struct WidgetStates {
    adb_button: button::State,
    button_up: button::State,
    button_down: button::State,
    button_left: button::State,
    button_right: button::State,
    button_ok: button::State,
    button_back: button::State,
    button_home: button::State,
}

struct Hello {
    adb_connectivity: AdbConnectivity,
    adb_server_rx: tokio::sync::watch::Receiver<String>,
    adb_server_tx: tokio::sync::watch::Sender<String>,
    widget_states: WidgetStates,
}

impl Application for Hello {
    type Executor = executor::Default;
    type Message = AppCommand;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let (adb_server_tx, adb_server_rx) = tokio::sync::watch::channel("".into());
        (
            Self {
                adb_connectivity: AdbConnectivity::Disconnected,
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
            InvokeAdbResult => {
                info!("update InvokeAdbResult");
            }
            OnAdbButton => {
                info!("update OnAdbButton");
                return Command::perform(invoke_adb(), |_| AppCommand::InvokeAdbResult);
            }
            OnAdbConnectClicked => match self.adb_connectivity {
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
            },
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
            AdbConnectivity::Connecting | AdbConnectivity::Connected => Subscription::batch(vec![
                Subscription::from_recipe(AdbServerRecipe {
                    rx: self.adb_server_rx.clone(),
                })
                .map(AppCommand::AdbServerRecipeResult),
                native_events().map(AppCommand::Event),
            ]),
            AdbConnectivity::Disconnected => Subscription::none(),
        }
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let button_width = Length::Units(90);
        let button_height = Length::Units(30);

        Column::new()
            .push(
                Button::new(&mut self.widget_states.adb_button, Text::new("devices"))
                    .on_press(AppCommand::OnAdbButton),
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
                    .push(Space::new(button_width.clone(), button_height.clone()))
                    .push(
                        Button::new(&mut self.widget_states.button_up, Text::new("Up (k)"))
                            .width(button_width.clone())
                            .height(button_height.clone())
                            .on_press(AppCommand::RequestSendEvent(SendEventKey::KeyDpadUpClick)),
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
                            .width(button_width.clone())
                            .height(button_height.clone())
                            .on_press(AppCommand::RequestSendEvent(SendEventKey::KeyDpadLeftClick)),
                    )
                    .push(
                        // TODO: support disabled style.
                        // TODO: support long press.
                        Button::new(&mut self.widget_states.button_ok, Text::new("Enter"))
                            .width(button_width.clone())
                            .height(button_height.clone())
                            .on_press(AppCommand::RequestSendEvent(SendEventKey::KeyEnterClick)),
                    )
                    .push(
                        // TODO: support disabled style.
                        // TODO: support long press.
                        Button::new(&mut self.widget_states.button_right, Text::new("Right (l)"))
                            .width(button_width.clone())
                            .height(button_height.clone())
                            .on_press(AppCommand::RequestSendEvent(
                                SendEventKey::KeyDpadRightClick,
                            )),
                    ),
            )
            .push(
                Row::new()
                    .push(Space::new(button_width.clone(), button_height.clone()))
                    .push(
                        Button::new(&mut self.widget_states.button_down, Text::new("Down (j)"))
                            .width(button_width.clone())
                            .height(button_height.clone())
                            .on_press(AppCommand::RequestSendEvent(SendEventKey::KeyDpadDownClick)),
                    ),
            )
            .push(
                Row::new()
                    .push(
                        Button::new(&mut self.widget_states.button_back, Text::new("Back"))
                            .width(button_width.clone())
                            .height(button_height.clone())
                            .on_press(AppCommand::RequestSendEvent(SendEventKey::KeyBackClick)),
                    )
                    .push(
                        Button::new(&mut self.widget_states.button_home, Text::new("Home"))
                            .width(button_width.clone())
                            .height(button_height.clone())
                            .on_press(AppCommand::RequestSendEvent(SendEventKey::KeyHomeClick)),
                    ),
            )
            .into()
    }
}

async fn invoke_adb() {
    match std::process::Command::new("adb").arg("devices").spawn() {
        Ok(data) => {
            info!(?data.stdout, "invoke_adb succeeded");
        }
        Err(e) => {
            info!(?e, "invoke_adb failed");
        }
    }
}

#[derive(Debug)]
struct DeviceInput {
    input_path: String,
    name: String,
    keys: Vec<u16>,
    key_names: Vec<String>,
}

fn hex_str_to_u16(data: &[&str]) -> Result<Vec<u16>, ParseIntError> {
    let mut ret = Vec::with_capacity(data.len());
    for entry in data {
        match u16::from_str_radix(entry, 16) {
            Ok(d) => ret.push(d),
            Err(e) => return Err(e),
        }
    }
    Ok(ret)
}

fn main() -> anyhow::Result<()> {
    // TODO: disable log.
    #[cfg(target_os = "windows")]
    if false {
        let code = unsafe { winapi::um::wincon::FreeConsole() };
        if code == 0 {
            anyhow::bail!("unable to detach the console")
        }
    }

    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    info!("Hello");

    Hello::run(Settings {
        window: WindowSettings {
            size: (270, 320),
            ..Default::default()
        },
        ..Default::default()
    });

    info!("Bye");

    Ok(())
}
