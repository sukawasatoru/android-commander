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

#[allow(unused_imports)]
use android_commander::data::preferences_repository::MockPreferencesRepository;
use android_commander::data::preferences_repository::{
    PreferencesRepository, PreferencesRepositoryImpl,
};
use android_commander::feature::main::{MainView, MainViewCommand};
use android_commander::feature::migrate::migrate;
use android_commander::feature::settings::{
    SettingsView, SettingsViewCommand, ViewState as SettingsViewState,
};
use android_commander::model::XMessage;
use android_commander::model::{AppTheme, Preferences};
use android_commander::prelude::*;
use iced::widget::{button, column, container, row, Column, Space};
use iced::window::{resize, Settings as WindowSettings};
use iced::{executor, Application, Command, Element, Length, Settings, Subscription};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
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
    OnXMessage(XMessage),
    SettingsViewCommand(SettingsViewCommand),
    Sink,
}

#[derive(Default)]
struct AppFlags {
    config_dir: PathBuf,
}

struct App {
    active_view: ActiveView,

    prefs_repo: Arc<Mutex<PreferencesRepositoryImpl>>,
    // prefs_repo: Arc<Mutex<MockPreferencesRepository>>,
    state_view_settings: SettingsViewState,
    theme: AppTheme,
    view_main: MainView,
}

impl SettingsView for App {
    type PrefsRepo = PreferencesRepositoryImpl;
    // type PrefsRepo = MockPreferencesRepository;

    fn get_prefs_repo(&self) -> Arc<Mutex<Self::PrefsRepo>> {
        self.prefs_repo.clone()
    }

    fn get_state(&self) -> &SettingsViewState {
        &self.state_view_settings
    }

    fn get_state_mut(&mut self) -> &mut SettingsViewState {
        &mut self.state_view_settings
    }
}

impl Application for App {
    type Executor = executor::Default;
    type Message = AppCommand;
    type Theme = AppTheme;
    type Flags = AppFlags;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let config_file_path = flags.config_dir.join("preferences.toml");
        let prefs = Arc::new(Preferences::default());
        let theme = prefs.theme;
        (
            Self {
                active_view: ActiveView::Main,
                prefs_repo: Arc::new(Mutex::new(PreferencesRepositoryImpl::new(
                    config_file_path.to_owned(),
                ))),
                theme,
                state_view_settings: SettingsViewState::new(config_file_path, theme),
                view_main: MainView::new(prefs),
            },
            Command::batch([
                Command::perform(async {}, |_| AppCommand::OnInit),
                MainView::init_command().map(AppCommand::MainViewCommand),
            ]),
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
                    ActiveView::Settings => <Self as SettingsView>::view_size(self),
                };

                resize(w, h)
            }
            AppCommand::MainViewCommand(command) => self
                .view_main
                .update(command)
                .map(AppCommand::MainViewCommand),
            AppCommand::OnInit => self.load_prefs_command(),
            AppCommand::OnXMessage(x_message) => {
                let mut commands = vec![];
                match x_message {
                    XMessage::OnNewPreferences(ref prefs) => {
                        self.theme = prefs.theme;
                    }
                    XMessage::OnPrefsFileUpdated => {
                        commands.push(self.load_prefs_command());
                    }
                }
                commands.push(
                    self.view_main
                        .update(MainViewCommand::OnXMessage(x_message.clone()))
                        .map(AppCommand::MainViewCommand),
                );
                commands.push(
                    <Self as SettingsView>::update(
                        self,
                        SettingsViewCommand::OnXMessage(x_message),
                    )
                    .map(AppCommand::SettingsViewCommand),
                );
                Command::batch(commands)
            }
            AppCommand::SettingsViewCommand(data) => <Self as SettingsView>::update(self, data)
                .map(|command| {
                    if let SettingsViewCommand::SendXMessage(data) = command {
                        AppCommand::OnXMessage(data)
                    } else {
                        AppCommand::SettingsViewCommand(command)
                    }
                }),
            AppCommand::Sink => Command::none(),
        }
    }

    // noinspection for Rust plugin v.176.
    // noinspection RsTypeCheck
    fn view(&self) -> Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let button_width = Length::Units(90);
        let button_height = Length::Units(30);
        let mut view: Column<Self::Message, iced::Renderer<Self::Theme>> = column![
            row![
                button("Main")
                    .width(button_width)
                    .height(button_height)
                    .on_press(AppCommand::ActiveView(ActiveView::Main)),
                button("Settings")
                    .width(button_width)
                    .height(button_height)
                    .on_press(AppCommand::ActiveView(ActiveView::Settings)),
            ],
            Space::with_height(12.into()),
        ];

        view = match self.active_view {
            ActiveView::Main => view.push(
                container(self.view_main.view().map(Self::Message::MainViewCommand)).padding(4),
            ),
            ActiveView::Settings => view.push(
                container(
                    <Self as SettingsView>::view(self).map(Self::Message::SettingsViewCommand),
                )
                .padding(4),
            ),
        };

        view.into()
    }

    fn theme(&self) -> Self::Theme {
        self.theme
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch([self
            .view_main
            .subscription()
            .map(AppCommand::MainViewCommand)])
    }
}

impl App {
    fn load_prefs_command(&self) -> Command<AppCommand> {
        let repo = self.prefs_repo.clone();
        Command::perform(
            async move {
                match repo.lock().await.load().await {
                    Ok(data) => Some(Arc::new(data)),
                    Err(e) => {
                        warn!(?e, "failed to load key map");
                        None
                    }
                }
            },
            |data| match data {
                Some(data) => AppCommand::OnXMessage(XMessage::OnNewPreferences(data)),
                None => AppCommand::Sink,
            },
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

    migrate()?;

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
