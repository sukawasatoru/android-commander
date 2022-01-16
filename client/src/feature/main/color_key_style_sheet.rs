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

use iced::button::{Style as ButtonStyle, StyleSheet as ButtonStyleSheet};
use iced::{Color, Vector};

pub struct ColorKeyStyleSheet(pub Color);

impl ButtonStyleSheet for ColorKeyStyleSheet {
    fn active(&self) -> ButtonStyle {
        ButtonStyle {
            shadow_offset: Vector::new(0.0, 0.0),
            background: Some(self.0.into()),
            border_radius: 2.0,
            border_width: 1.0,
            border_color: [0.7, 0.7, 0.7].into(),
            text_color: Color::BLACK,
        }
    }
}
