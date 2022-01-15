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
use android_commander::data::key_map_repository::{
    MockPreferencesRepository, PreferencesRepository, PreferencesRepositoryImpl,
};
use android_commander::data::resource::Resource;
use android_commander::feature::main::MainView;
use android_commander::feature::settings::{SettingsView, SettingsViewCommand};
use android_commander::model::preferences::{KeyMap, Preferences};
use android_commander::model::send_event_key::SendEventKey;
use android_commander::model::AndroidDevice;
use android_commander::prelude::*;
use iced::keyboard::{Event as KeyboardEvent, KeyCode};
use iced::svg::Handle as SvgHandle;
use iced::window::{resize, Settings as WindowSettings};
use iced::{
    button, executor, pick_list, Application, Button, Checkbox, Column, Command, Element, Length,
    PickList, Row, Settings, Space, Subscription, Svg, Text,
};
use iced_native::subscription::events as native_events;
use iced_native::Event as NativeEvent;
use std::convert::Infallible;
use std::io::BufRead;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info, warn};

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

enum AdbConnectivity {
    Connected,
    Connecting,
    Disconnected,
}

#[derive(Clone, Debug, PartialEq)]
enum ActiveView {
    Main,
    Settings,
}

#[derive(Clone, Debug)]
enum AppCommand {
    ActiveView(ActiveView),
    AdbDevicesSelected(Arc<AndroidDevice>),
    AdbServerRecipeResult(AdbServerRecipeEvent),
    Event(NativeEvent),
    InvokeDevicesResult(Vec<Arc<AndroidDevice>>),
    OnAdbConnectClicked,
    OnAdbDevicesReloadClicked,
    OnInit,
    OnNewPrefs(Option<Preferences>),
    RequestSendEvent(SendEventKey),
    SettingsViewCommand(SettingsViewCommand),
}

#[derive(Debug, Default)]
struct WidgetStates {
    active_view_main_button: button::State,
    active_view_settings_button: button::State,
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

pub trait AppModule {
    type PrefsRepo: PreferencesRepository;

    fn prefs_repo(&self) -> Arc<Self::PrefsRepo>;
}

pub struct AppModuleImpl {
    prefs_repo: Arc<PreferencesRepositoryImpl>,
}

impl AppModuleImpl {
    #[allow(dead_code)]
    fn new(prefs_repo: PreferencesRepositoryImpl) -> Self {
        Self {
            prefs_repo: Arc::new(prefs_repo),
        }
    }
}

impl AppModule for AppModuleImpl {
    type PrefsRepo = PreferencesRepositoryImpl;

    fn prefs_repo(&self) -> Arc<Self::PrefsRepo> {
        self.prefs_repo.clone()
    }
}

pub struct MockAppModule {
    prefs_repo: Arc<MockPreferencesRepository>,
}

impl MockAppModule {
    #[allow(dead_code)]
    fn new() -> Self {
        Self {
            prefs_repo: Arc::new(MockPreferencesRepository),
        }
    }
}

impl AppModule for MockAppModule {
    type PrefsRepo = MockPreferencesRepository;

    fn prefs_repo(&self) -> Arc<Self::PrefsRepo> {
        self.prefs_repo.clone()
    }
}

#[derive(Default)]
struct AppFlags {
    config_dir: PathBuf,
}

struct App {
    active_view: ActiveView,
    adb_connectivity: AdbConnectivity,
    adb_devices: Vec<Arc<AndroidDevice>>,
    adb_devices_selected: Option<Arc<AndroidDevice>>,
    adb_server_rx: tokio::sync::watch::Receiver<String>,
    adb_server_tx: tokio::sync::watch::Sender<String>,
    app_module: AppModuleImpl,
    // app_module: MockAppModule,
    prefs: Preferences,

    #[allow(dead_code)]
    view_main: MainView,

    view_settings: SettingsView,
    widget_states: WidgetStates,
}

impl Application for App {
    type Executor = executor::Default;
    type Message = AppCommand;
    type Flags = AppFlags;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let (adb_server_tx, adb_server_rx) = tokio::sync::watch::channel("".into());
        (
            Self {
                active_view: ActiveView::Main,
                adb_connectivity: AdbConnectivity::Disconnected,
                adb_devices: vec![],
                adb_devices_selected: None,
                adb_server_rx,
                adb_server_tx,
                app_module: AppModuleImpl::new(PreferencesRepositoryImpl::new(
                    flags.config_dir.join("preferences.toml"),
                )),
                // app_module: MockAppModule::new(),
                prefs: Default::default(),
                view_main: MainView,
                view_settings: SettingsView,
                widget_states: Default::default(),
            },
            Command::perform(
                iced_futures::futures::future::ok::<(), Infallible>(()),
                |_| AppCommand::OnInit,
            ),
        )
    }

    fn title(&self) -> String {
        "Android Commander".into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            AppCommand::ActiveView(data) => {
                info!(?data, "onActiveView");
                self.active_view = data;

                let (w, h) = match self.active_view {
                    ActiveView::Main => MainView::view_size(),
                    ActiveView::Settings => self.view_settings.view_size(),
                };

                let mut commands = vec![resize(w, h)];

                match self.active_view {
                    ActiveView::Main => commands.push(self.load_key_map_command()),
                    ActiveView::Settings => (),
                };

                return Command::batch(commands);
            }
            AppCommand::AdbDevicesSelected(data) => {
                info!(%data, "device selected");
                self.adb_devices_selected = Some(data);
            }
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
                    self.adb_server_tx.send("".into()).ok();
                }
            },
            AppCommand::Event(data) => {
                if self.active_view != ActiveView::Main {
                    return Command::none();
                }

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

                            let send_event_key = match create_send_event_key(key_code) {
                                Some(data) => data,
                                None => return Command::none(),
                            };

                            let ret = self.adb_server_tx.send(create_pressed_key_command(
                                &self.prefs.key_map,
                                &send_event_key,
                            ));

                            if let Err(e) = ret {
                                warn!("failed to send the sendevent: {:?}", e);
                            }
                        }
                        KeyboardEvent::KeyReleased { key_code, .. } => {
                            debug!("update KeyReleased: {:?}", key_code);

                            let send_event_key = match create_send_event_key(key_code) {
                                Some(data) => data,
                                None => return Command::none(),
                            };

                            let ret = self.adb_server_tx.send(create_release_key_command(
                                &self.prefs.key_map,
                                &send_event_key,
                            ));

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
            AppCommand::InvokeDevicesResult(devices) => {
                info!("update InvokeDevicesResult");
                self.adb_devices = devices;
                if let Some(selected) = &self.adb_devices_selected {
                    if !self.adb_devices.iter().any(|data| data == selected) {
                        self.adb_devices_selected = None;
                    }
                }

                return Command::none();
            }
            AppCommand::OnAdbConnectClicked => {
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
            AppCommand::OnAdbDevicesReloadClicked => {
                return Command::perform(invoke_retrieve_devices(), |data| {
                    AppCommand::InvokeDevicesResult(data)
                });
            }
            AppCommand::OnInit => return self.load_key_map_command(),
            AppCommand::OnNewPrefs(prefs) => {
                if let Some(data) = prefs {
                    self.prefs = data;
                }
            }
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
                    .send(create_click_key_command(&self.prefs.key_map, &data));

                if let Err(e) = ret {
                    warn!("failed to send the sendevent: {:?}", e);
                }
            }
            AppCommand::SettingsViewCommand(_) => {
                return self
                    .view_settings
                    .update()
                    .map(AppCommand::SettingsViewCommand);
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

        let mut view = Column::new()
            .push(
                Row::new()
                    .push(
                        Button::new(
                            &mut self.widget_states.active_view_main_button,
                            Text::new("Main"),
                        )
                        .width(button_width)
                        .height(button_height)
                        .on_press(AppCommand::ActiveView(ActiveView::Main)),
                    )
                    .push(
                        Button::new(
                            &mut self.widget_states.active_view_settings_button,
                            Text::new("Settings"),
                        )
                        .width(button_width)
                        .height(button_height)
                        .on_press(AppCommand::ActiveView(ActiveView::Settings)),
                    ),
            )
            .push(Space::new(Length::Shrink, Length::Units(16)));

        view = match self.active_view {
            ActiveView::Main => {
                view.push(Text::new("ADB:"))
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
                        "connect",
                        |_| AppCommand::OnAdbConnectClicked,
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
                                Button::new(
                                    &mut self.widget_states.button_left,
                                    Text::new("Left (h)"),
                                )
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
                                    .on_press(AppCommand::RequestSendEvent(SendEventKey::Ok)),
                            )
                            .push(
                                // TODO: support disabled style.
                                // TODO: support long press.
                                Button::new(
                                    &mut self.widget_states.button_right,
                                    Text::new("Right (l)"),
                                )
                                .width(button_width)
                                .height(button_height)
                                .on_press(AppCommand::RequestSendEvent(SendEventKey::DpadRight)),
                            ),
                    )
                    .push(
                        Row::new()
                            .push(Space::new(button_width, button_height))
                            .push(
                                Button::new(
                                    &mut self.widget_states.button_down,
                                    Text::new("Down (j)"),
                                )
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
            }
            ActiveView::Settings => view.push(
                self.view_settings
                    .view()
                    .map(Self::Message::SettingsViewCommand),
            ),
        };

        view.into()
    }
}

impl App {
    fn load_key_map_command(&self) -> Command<AppCommand> {
        let repo = self.app_module.prefs_repo();
        Command::perform(
            async move {
                match repo.load().await {
                    Ok(data) => Some(data),
                    Err(e) => {
                        warn!(?e, "failed to load key map");
                        None
                    }
                }
            },
            AppCommand::OnNewPrefs,
        )
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

fn main() -> Fallible<()> {
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

    let config_dir = directories::ProjectDirs::from("com", "sukawasatoru", "AndroidCommander")
        .context("directories")?
        .config_dir()
        .to_path_buf();

    App::run(Settings {
        window: WindowSettings {
            size: MainView::view_size(),
            ..Default::default()
        },
        flags: AppFlags { config_dir },
        ..Default::default()
    })?;

    info!("Bye");
    Ok(())
}
