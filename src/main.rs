/*
 * Copyright 2020 sukawasatoru
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

use iced::{
    button, executor, Application, Button, Checkbox, Column, Command, Element, Row, Settings,
    Subscription, Text,
};
use iced_futures::futures;
use iced_futures::subscription::Recipe;
use iced_futures::BoxStream;
use iced_native::{Length, Space};
use log::{debug, info, warn};
use std::io::prelude::*;

#[derive(Copy, Clone, Debug)]
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

#[derive(Copy, Clone, Debug)]
enum SendEventKey {
    KeyDpadUpClick,
    KeyDpadDownClick,
    KeyDpadLeftClick,
    KeyDpadRightClick,
    KeyEnterClick,
    KeyBackClick,
}

#[derive(Copy, Clone, Debug)]
enum SendEventDevice {
    Event0,
    Event1,
    Event2,
    Event3,
    Event4,
    Event5,
    Event6,
    Event7,
    Event8,
    Event9,
}

impl SendEventDevice {
    fn name(&self) -> &'static str {
        use SendEventDevice::*;

        match self {
            Event0 => "event0",
            Event1 => "event1",
            Event2 => "event2",
            Event3 => "event3",
            Event4 => "event4",
            Event5 => "event5",
            Event6 => "event6",
            Event7 => "event7",
            Event8 => "event8",
            Event9 => "event9",
        }
    }
}

impl SendEventKey {
    fn get_key_with_syn_type(&self) -> u8 {
        use SendEventKey::*;

        match self {
            KeyDpadUpClick | KeyDpadDownClick | KeyDpadLeftClick | KeyDpadRightClick
            | KeyEnterClick | KeyBackClick => 1,
        }
    }

    fn get_key_with_syn_code(&self) -> u16 {
        use SendEventKey::*;

        match self {
            KeyDpadUpClick => 103,
            KeyDpadDownClick => 108,
            KeyDpadLeftClick => 105,
            KeyDpadRightClick => 106,
            KeyEnterClick => 28,
            KeyBackClick => 158,
        }
    }
}

fn create_click_key_with_syn_sendevent(device: &SendEventDevice, key: &SendEventKey) -> String {
    let device = device.name();
    let type_val = key.get_key_with_syn_type();
    let code = key.get_key_with_syn_code();
    format!(
        "sendevent /dev/input/{} {} {} 1 && sendevent /dev/input/{} 0 0 0 && sendevent /dev/input/{} {} {} 0 && sendevent /dev/input/{} 0 0 0",
        device, type_val, code, device, device, type_val, code, device
    )
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
        use std::hash::Hash;

        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(self: Box<Self>, _input: BoxStream<I>) -> BoxStream<Self::Output> {
        use AdbServerRecipeEvent as RecipeEvent;
        use AdbServerRecipeInternalState as RecipeState;

        Box::pin(futures::stream::unfold(
            RecipeState::Init(self.rx),
            |state| async move {
                match state {
                    RecipeState::Init(rx) => match std::process::Command::new("adb")
                        .arg("shell")
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
                    },
                    RecipeState::Ready(mut rx, mut child) => {
                        while let Some(data) = rx.recv().await {
                            debug!("send data: {}", data);

                            // for ignore init value.
                            if data.is_empty() {
                                continue;
                            }

                            let ret = writeln!(child.stdin.as_mut().unwrap(), "{}", data);
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

#[derive(Copy, Clone, Debug)]
enum AppCommand {
    AdbServerRecipeResult(AdbServerRecipeEvent),
    InvokeAdbResult,
    OnAdbButton,
    OnAdbConnectClicked,
    RequestSendEvent(SendEventKey),
}

#[derive(Default)]
struct WidgetStates {
    adb_button: button::State,
    button_up: button::State,
    button_down: button::State,
    button_left: button::State,
    button_right: button::State,
    button_ok: button::State,
    button_back: button::State,
}

struct Hello {
    adb_connectivity: AdbConnectivity,
    adb_server_rx: tokio::sync::watch::Receiver<String>,
    adb_server_tx: tokio::sync::watch::Sender<String>,
    sendevent_device: SendEventDevice,
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
                sendevent_device: SendEventDevice::Event2,
                widget_states: Default::default(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Hello World".into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            AppCommand::AdbServerRecipeResult(data) => match data {
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
                    self.adb_server_tx.broadcast("".into()).ok();
                }
            },
            AppCommand::InvokeAdbResult => {
                info!("update InvokeAdbResult");
            }
            AppCommand::OnAdbButton => {
                info!("update OnAdbButton");
                return Command::perform(invoke_adb(), |_| AppCommand::InvokeAdbResult);
            }
            AppCommand::OnAdbConnectClicked => match self.adb_connectivity {
                AdbConnectivity::Disconnected => {
                    self.adb_connectivity = AdbConnectivity::Connecting
                }
                AdbConnectivity::Connecting => {
                    warn!("TODO");
                }
                AdbConnectivity::Connected => {
                    self.adb_connectivity = AdbConnectivity::Disconnected;
                    self.adb_server_tx.broadcast("".into()).ok();
                }
            },
            AppCommand::RequestSendEvent(data) => {
                info!("update RequestSendEvent: {:?}", data);
                match self.adb_connectivity {
                    AdbConnectivity::Connected => (),
                    AdbConnectivity::Connecting | AdbConnectivity::Disconnected => {
                        debug!("skip broadcasting");
                        return Command::none();
                    }
                }

                let ret = self
                    .adb_server_tx
                    .broadcast(create_click_key_with_syn_sendevent(
                        &self.sendevent_device,
                        &data,
                    ));
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
                iced::Subscription::from_recipe(AdbServerRecipe {
                    rx: self.adb_server_rx.clone(),
                })
                .map(AppCommand::AdbServerRecipeResult)
            }
            AdbConnectivity::Disconnected => iced::Subscription::none(),
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
                Button::new(&mut self.widget_states.button_back, Text::new("Back"))
                    .width(button_width.clone())
                    .height(button_height.clone())
                    .on_press(AppCommand::RequestSendEvent(SendEventKey::KeyBackClick)),
            )
            .into()
    }
}

async fn invoke_adb() {
    match std::process::Command::new("adb").arg("devices").spawn() {
        Ok(data) => {
            info!("invoke_adb succeeded: {:?}", data.stdout);
        }
        Err(e) => {
            info!("invoke_adb failed: {:?}", e);
        }
    }
}

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    info!("Hello");

    Hello::run(Settings {
        window: iced::window::Settings {
            size: (270, 320),
            ..Default::default()
        },
        ..Default::default()
    });

    info!("Bye");

    Ok(())
}
