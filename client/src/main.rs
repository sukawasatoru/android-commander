/*
 * Copyright 2020, 2021, 2022, 2025, 2026 sukawasatoru
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

#![windows_subsystem = "windows"]

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
use android_commander::model::Preferences;
use android_commander::model::XMessage;
use android_commander::prelude::*;
use android_commander::widget_style::button_secondary;
use iced::widget::{button, column, container, row, space};
use iced::window::{self, resize};
use iced::{Element, Length, Subscription, Task, Theme, application};
use std::sync::Arc;
use tokio::sync::Mutex;

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
    WindowID(window::Id),
}

struct App {
    active_view: ActiveView,

    prefs_repo: Arc<Mutex<PreferencesRepositoryImpl>>,
    // prefs_repo: Arc<Mutex<MockPreferencesRepository>>,
    state_view_settings: SettingsViewState,
    theme: Theme,
    view_main: MainView,
    window_id: Option<window::Id>,
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

impl App {
    fn title(&self) -> String {
        "Android Commander".into()
    }

    fn update(&mut self, message: AppCommand) -> Task<AppCommand> {
        match message {
            AppCommand::ActiveView(data) => {
                info!(?data, "onActiveView");
                self.active_view = data;

                self.window_id
                    .map(|id| {
                        resize(
                            id,
                            match self.active_view {
                                ActiveView::Main => MainView::view_size(),
                                ActiveView::Settings => <Self as SettingsView>::view_size(self),
                            },
                        )
                    })
                    .unwrap_or_else(Task::none)
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
                        self.theme = prefs.theme.clone();
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
                Task::batch(commands)
            }
            AppCommand::SettingsViewCommand(data) => <Self as SettingsView>::update(self, data)
                .map(|command| {
                    if let SettingsViewCommand::SendXMessage(data) = command {
                        AppCommand::OnXMessage(data)
                    } else {
                        AppCommand::SettingsViewCommand(command)
                    }
                }),
            AppCommand::Sink => Task::none(),
            AppCommand::WindowID(id) => {
                self.window_id = Some(id);
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, AppCommand> {
        let button_width = Length::Fixed(90.0);
        let button_height = Length::Fixed(30.0);
        let mut view = column![
            row![
                button("Main")
                    .width(button_width)
                    .height(button_height)
                    .style(button_secondary)
                    .on_press(AppCommand::ActiveView(ActiveView::Main)),
                button("Settings")
                    .width(button_width)
                    .height(button_height)
                    .style(button_secondary)
                    .on_press(AppCommand::ActiveView(ActiveView::Settings)),
            ],
            space().height(12),
        ];

        view = match self.active_view {
            ActiveView::Main => view
                .push(container(self.view_main.view().map(AppCommand::MainViewCommand)).padding(4)),
            ActiveView::Settings => view.push(
                container(<Self as SettingsView>::view(self).map(AppCommand::SettingsViewCommand))
                    .padding(4),
            ),
        };

        view.into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn subscription(&self) -> Subscription<AppCommand> {
        let mut subscriptions = vec![];

        if self.window_id.is_none() {
            subscriptions.push(iced::event::listen_with(|event, _status, id| {
                if let iced::event::Event::Window(window::Event::Opened { .. }) = event {
                    Some(AppCommand::WindowID(id))
                } else {
                    None
                }
            }));
        }

        subscriptions.push(
            self.view_main
                .subscription()
                .map(AppCommand::MainViewCommand),
        );

        Subscription::batch(subscriptions)
    }

    fn load_prefs_command(&self) -> Task<AppCommand> {
        let repo = self.prefs_repo.clone();
        Task::perform(
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

fn boot() -> (App, Task<AppCommand>) {
    let config_dir = directories::ProjectDirs::from("com", "sukawasatoru", "AndroidCommander")
        .unwrap()
        .config_dir()
        .to_path_buf();

    migrate().unwrap();

    let config_file_path = config_dir.join("preferences.toml");
    let prefs = Arc::new(Preferences::default());
    (
        App {
            active_view: ActiveView::Main,
            prefs_repo: Arc::new(Mutex::new(PreferencesRepositoryImpl::new(
                config_file_path.to_owned(),
            ))),
            theme: prefs.theme.clone(),
            state_view_settings: SettingsViewState::new(config_file_path, prefs.theme.clone()),
            view_main: MainView::new(prefs),
            window_id: None,
        },
        Task::batch([
            Task::done(AppCommand::OnInit),
            MainView::init_command().map(AppCommand::MainViewCommand),
        ]),
    )
}

fn main() -> iced::Result {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    #[cfg(target_os = "windows")]
    let icon = {
        use android_commander::data::resource::Resource;

        let png_data = Resource::get("icon.png").expect("icon.png not found");
        let img = image::load_from_memory(&png_data.data).expect("failed to load icon.png");
        let width = img.width();
        let height = img.height();
        let icon = window::icon::from_rgba(img.to_rgba8().into_vec(), width, height)
            .expect("failed to create icon from icon.png");
        Some(icon)
    };

    #[cfg(not(target_os = "windows"))]
    let icon = None;

    application(boot, App::update, App::view)
        .title(App::title)
        .theme(App::theme)
        .subscription(App::subscription)
        .window(window::Settings {
            size: MainView::view_size(),
            icon,
            ..Default::default()
        })
        .run()
}
