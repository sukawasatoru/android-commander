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

use android_commander::data::key_map_repository::{
    MockPreferencesRepository, PreferencesRepository, PreferencesRepositoryImpl,
};
use android_commander::feature::main::{MainView, MainViewCommand};
use android_commander::feature::settings::{SettingsView, SettingsViewCommand};
use android_commander::model::preferences::Preferences;
use android_commander::prelude::*;
use iced::window::{resize, Settings as WindowSettings};
use iced::{
    button, executor, Application, Button, Column, Command, Element, Length, Row, Settings, Space,
    Subscription, Text,
};
use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{info, warn};

#[derive(Clone, Debug, PartialEq)]
enum ActiveView {
    Main,
    Settings,
}

#[derive(Clone, Debug)]
enum AppCommand {
    ActiveView(ActiveView),
    MainViewCommand(MainViewCommand),
    OnInit,
    OnNewPrefs(Option<Arc<Preferences>>),
    SettingsViewCommand(SettingsViewCommand),
}

#[derive(Debug, Default)]
struct WidgetStates {
    active_view_main_button: button::State,
    active_view_settings_button: button::State,
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
    app_module: AppModuleImpl,
    // app_module: MockAppModule,
    view_main: MainView,
    view_settings: SettingsView,
    widget_states: WidgetStates,
}

impl Application for App {
    type Executor = executor::Default;
    type Message = AppCommand;
    type Flags = AppFlags;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let config_file_path = flags.config_dir.join("preferences.toml");
        let prefs = Arc::new(Default::default());
        (
            Self {
                active_view: ActiveView::Main,
                app_module: AppModuleImpl::new(PreferencesRepositoryImpl::new(
                    config_file_path.to_owned(),
                )),
                // app_module: MockAppModule::new(),
                view_main: MainView::new(prefs),
                view_settings: SettingsView::new(config_file_path),
                widget_states: Default::default(),
            },
            Command::batch([
                Command::perform(
                    iced_futures::futures::future::ok::<(), Infallible>(()),
                    |_| AppCommand::OnInit,
                ),
                MainView::init_command().map(AppCommand::MainViewCommand),
            ]),
        )
    }

    fn title(&self) -> String {
        "Android Commander".into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        return match message {
            AppCommand::ActiveView(data) => {
                info!(?data, "onActiveView");
                self.active_view = data;

                let (w, h) = match self.active_view {
                    ActiveView::Main => MainView::view_size(),
                    ActiveView::Settings => self.view_settings.view_size(),
                };

                let mut commands = vec![resize(w, h)];

                match self.active_view {
                    ActiveView::Main => commands.push(self.load_prefs_command()),
                    ActiveView::Settings => (),
                };

                Command::batch(commands)
            }
            AppCommand::MainViewCommand(command) => self
                .view_main
                .update(command)
                .map(AppCommand::MainViewCommand),
            AppCommand::OnInit => self.load_prefs_command(),
            AppCommand::OnNewPrefs(prefs) => self
                .view_main
                .update(MainViewCommand::OnNewPrefs(prefs))
                .map(AppCommand::MainViewCommand),
            AppCommand::SettingsViewCommand(data) => self
                .view_settings
                .update(data)
                .map(AppCommand::SettingsViewCommand),
        };
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        self.view_main
            .subscription()
            .map(AppCommand::MainViewCommand)
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
                view.push(self.view_main.view().map(Self::Message::MainViewCommand))
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
    fn load_prefs_command(&self) -> Command<AppCommand> {
        let repo = self.app_module.prefs_repo();
        Command::perform(
            async move {
                match repo.load().await {
                    Ok(data) => Some(Arc::new(data)),
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
