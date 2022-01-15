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

use iced::widget::{Column, Row};
use iced::{button, Button, Command, Element, Text};
use std::path::PathBuf;
use tracing::{debug, warn};

pub struct SettingsView {
    config_file_path: PathBuf,
    widget_state: WidgetState,
}

impl SettingsView {
    pub fn new(config_file_path: PathBuf) -> Self {
        Self {
            config_file_path,
            widget_state: Default::default(),
        }
    }
}

#[derive(Default)]
struct WidgetState {
    config_open_button: button::State,
}

#[derive(Clone, Debug)]
pub enum SettingsViewCommand {
    OnConfigOpenButtonClicked,
}

impl SettingsView {
    pub fn update(&mut self, command: SettingsViewCommand) -> Command<SettingsViewCommand> {
        match command {
            SettingsViewCommand::OnConfigOpenButtonClicked => self.open_prefs(),
        }

        Command::none()
    }

    pub fn view(&mut self) -> Element<'_, SettingsViewCommand> {
        Column::new()
            .push(
                Row::new().push(
                    Button::new(
                        &mut self.widget_state.config_open_button,
                        Text::new("Open preferences"),
                    )
                    .on_press(SettingsViewCommand::OnConfigOpenButtonClicked),
                ),
            )
            .into()
    }

    pub fn view_size(&self) -> (u32, u32) {
        (270, 200)
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

        let filer = if cfg!(target_os = "windows") {
            "explorer"
        } else if cfg!(target_os = "macos") {
            "open"
        } else {
            "xdg-open"
        };

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
}
