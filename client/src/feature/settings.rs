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

use crate::model::app_command::AppCommand;
use crate::model::ButtonStyle;
use iced::widget::{button, column, text, Column};
use iced::{Command, Element};
use std::path::PathBuf;
use tokio::sync::broadcast::Sender;
use tracing::{debug, warn};

pub struct SettingsView {
    common_command_tx: Sender<AppCommand>,
    config_file_path: PathBuf,
}

impl SettingsView {
    pub fn new(common_command_tx: Sender<AppCommand>, config_file_path: PathBuf) -> Self {
        Self {
            common_command_tx,
            config_file_path,
        }
    }
}

#[derive(Clone, Debug)]
pub enum SettingsViewCommand {
    OnOpenKeycodeReferencesButtonClicked,
    OnOpenPrefsButtonClicked,
    OnOpenPrefsDirButtonClicked,
    OnReloadPrefsButtonClicked,
}

impl SettingsView {
    pub fn update(&mut self, command: SettingsViewCommand) -> Command<SettingsViewCommand> {
        match command {
            SettingsViewCommand::OnOpenPrefsButtonClicked => self.open_prefs(),
            SettingsViewCommand::OnOpenPrefsDirButtonClicked => self.open_prefs_directory(),
            SettingsViewCommand::OnOpenKeycodeReferencesButtonClicked => open_keycode_references(),
            SettingsViewCommand::OnReloadPrefsButtonClicked => {
                self.common_command_tx
                    .send(AppCommand::OnPrefsFileUpdated)
                    .ok();
            }
        }

        Command::none()
    }

    pub fn view<'a, Theme>(&'a self) -> Element<'a, SettingsViewCommand, iced::Renderer<Theme>>
    where
        Theme: button::StyleSheet<Style = ButtonStyle> + 'a,
        Theme: text::StyleSheet,
    {
        let view: Column<SettingsViewCommand, iced::Renderer<Theme>> = column![
            button("Reload preferences")
                .width(292.into())
                .on_press(SettingsViewCommand::OnReloadPrefsButtonClicked),
            button("Open preferences directory")
                .width(292.into())
                .on_press(SettingsViewCommand::OnOpenPrefsDirButtonClicked),
            button("Open KeyCode references")
                .width(292.into())
                .on_press(SettingsViewCommand::OnOpenKeycodeReferencesButtonClicked),
        ]
        .spacing(8);
        view.into()
    }

    pub fn view_size(&self) -> (u32, u32) {
        (300, 260)
    }

    fn open_prefs(&self) {
        if let Ok(visual) = std::env::var("VISUAL") {
            debug!(%visual, "use VISUAL");

            let visual_result = std::process::Command::new(visual)
                .arg(self.config_file_path.as_os_str())
                .spawn();

            if visual_result.is_ok() {
                debug!("succeeded");
                return;
            }
        }

        let filer = get_filer();

        debug!(%filer, "use filer");
        let filer_result = std::process::Command::new(filer)
            .arg(self.config_file_path.as_os_str())
            .spawn();

        if filer_result.is_ok() {
            debug!("succeeded");
            return;
        }

        warn!(?filer_result, %filer, "failed to open preferences");
    }

    fn open_prefs_directory(&self) {
        let filer = get_filer();

        let dir = match self.config_file_path.parent() {
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
