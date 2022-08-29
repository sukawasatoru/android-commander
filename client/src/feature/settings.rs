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

use crate::data::preferences_repository::PreferencesRepository;
use crate::model::ButtonStyle;
use crate::model::{AppTheme, XMessage};
use crate::prelude::Fallible;
use iced::widget::{button, column, pick_list, row, text, Column};
use iced::{Command, Element};
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, warn};

pub struct ViewState {
    config_file_path: PathBuf,
    theme: AppTheme,
}

impl ViewState {
    pub fn new(config_file_path: PathBuf, theme: AppTheme) -> Self {
        Self {
            config_file_path,
            theme,
        }
    }
}

#[derive(Clone, Debug)]
pub enum SettingsViewCommand {
    OnThemeSelected(AppTheme),
    OnOpenKeycodeReferencesButtonClicked,
    OnOpenPrefsButtonClicked,
    OnOpenPrefsDirButtonClicked,
    OnXMessage(XMessage),
    SendXMessage(XMessage),
    Sink,
}

pub trait SettingsView {
    type PrefsRepo: PreferencesRepository + 'static;

    fn get_prefs_repo(&self) -> Arc<Mutex<Self::PrefsRepo>>;

    fn get_state(&self) -> &ViewState;

    fn get_state_mut(&mut self) -> &mut ViewState;

    fn update(&mut self, command: SettingsViewCommand) -> Command<SettingsViewCommand> {
        match command {
            SettingsViewCommand::OnThemeSelected(theme) => {
                return Command::perform(
                    save_theme(self.get_prefs_repo(), theme),
                    |data| match data {
                        Ok(_) => SettingsViewCommand::SendXMessage(XMessage::OnPrefsFileUpdated),
                        Err(e) => {
                            warn!(?e, "failed to save theme");
                            SettingsViewCommand::Sink
                        }
                    },
                );
            }
            SettingsViewCommand::OnOpenPrefsButtonClicked => open_prefs(self.get_state()),
            SettingsViewCommand::OnOpenPrefsDirButtonClicked => {
                open_prefs_directory(self.get_state())
            }
            SettingsViewCommand::OnOpenKeycodeReferencesButtonClicked => open_keycode_references(),
            SettingsViewCommand::OnXMessage(data) => match data {
                XMessage::OnPrefsFileUpdated => {
                    // do nothing.
                }
                XMessage::OnNewPreferences(prefs) => {
                    self.get_state_mut().theme = prefs.theme;
                }
            },
            SettingsViewCommand::SendXMessage(_) | SettingsViewCommand::Sink => {
                // do nothing.
            }
        }

        Command::none()
    }

    fn view<'a, Theme>(&'a self) -> Element<'a, SettingsViewCommand, iced::Renderer<Theme>>
    where
        Theme: button::StyleSheet<Style = ButtonStyle> + 'a,
        Theme: pick_list::StyleSheet,
        Theme: text::StyleSheet,
    {
        let view: Column<SettingsViewCommand, iced::Renderer<Theme>> = column![
            button("Reload preferences").width(292.into()).on_press(
                SettingsViewCommand::SendXMessage(XMessage::OnPrefsFileUpdated)
            ),
            button("Open preferences directory")
                .width(292.into())
                .on_press(SettingsViewCommand::OnOpenPrefsDirButtonClicked),
            button("Open KeyCode references")
                .width(292.into())
                .on_press(SettingsViewCommand::OnOpenKeycodeReferencesButtonClicked),
            row![
                "Theme: ",
                pick_list(
                    &[AppTheme::Light, AppTheme::Dark][..],
                    Some(self.get_state().theme),
                    SettingsViewCommand::OnThemeSelected,
                ),
            ]
            .align_items(iced::alignment::Alignment::Center),
        ]
        .spacing(8);
        view.into()
    }

    fn view_size(&self) -> (u32, u32) {
        (300, 260)
    }
}

fn open_prefs(state: &ViewState) {
    if let Ok(visual) = std::env::var("VISUAL") {
        debug!(%visual, "use VISUAL");

        let visual_result = std::process::Command::new(visual)
            .arg(state.config_file_path.as_os_str())
            .spawn();

        if visual_result.is_ok() {
            debug!("succeeded");
            return;
        }
    }

    let filer = get_filer();

    debug!(%filer, "use filer");
    let filer_result = std::process::Command::new(filer)
        .arg(state.config_file_path.as_os_str())
        .spawn();

    if filer_result.is_ok() {
        debug!("succeeded");
        return;
    }

    warn!(?filer_result, %filer, "failed to open preferences");
}

fn open_prefs_directory(state: &ViewState) {
    let filer = get_filer();

    let dir = match state.config_file_path.parent() {
        Some(data) => data,
        None => return,
    };

    let ret = std::process::Command::new(filer)
        .arg(dir.as_os_str())
        .spawn();

    match ret {
        Ok(_) => debug!("succeeded"),
        Err(e) => warn!(?e, %filer, "failed to open directory"),
    }
}

fn get_filer() -> &'static str {
    if cfg!(target_os = "windows") {
        "explorer"
    } else if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    }
}

fn open_keycode_references() {
    let filer = get_filer();

    let ret = std::process::Command::new(filer)
        .arg("https://developer.android.com/reference/android/view/KeyEvent#constants")
        .spawn();

    match ret {
        Ok(_) => debug!("succeeded"),
        Err(e) => warn!(?e, %filer, "failed to open keycode references"),
    }
}

async fn save_theme<Repo: PreferencesRepository>(
    repo: Arc<Mutex<Repo>>,
    theme: AppTheme,
) -> Fallible<()> {
    let repo = repo.lock().await;
    let mut prefs = match repo.load().await {
        Ok(data) => data,
        Err(e) => return Err(e),
    };
    prefs.theme = theme;
    repo.save(prefs).await
}

impl Display for AppTheme {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AppTheme::Light => write!(f, "Light"),
            AppTheme::Dark => write!(f, "Dark"),
        }
    }
}
