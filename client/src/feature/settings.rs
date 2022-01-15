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
use iced::{Command, Element, Text};

pub struct SettingsView;

#[derive(Clone, Debug)]
pub enum SettingsViewCommand {
    A,
    B,
}

impl SettingsView {
    pub fn update(&mut self) -> Command<SettingsViewCommand> {
        // TODO:
        Command::none()
    }

    pub fn view(&mut self) -> Element<'_, SettingsViewCommand> {
        Column::new()
            .push(Row::new().push(Text::new("Hello!")))
            .into()
    }

    pub fn view_size(&self) -> (u32, u32) {
        (270, 200)
    }
}
