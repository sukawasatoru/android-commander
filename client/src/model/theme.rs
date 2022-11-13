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

use iced::theme::Theme;
use iced::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTheme {
    Light,
    Dark,
}

impl Default for AppTheme {
    fn default() -> Self {
        Self::Light
    }
}

impl From<&AppTheme> for Theme {
    fn from(value: &AppTheme) -> Self {
        match value {
            AppTheme::Light => Theme::Light,
            AppTheme::Dark => Theme::Dark,
        }
    }
}

impl From<&Theme> for AppTheme {
    fn from(value: &Theme) -> Self {
        match value {
            Theme::Light => AppTheme::Light,
            Theme::Dark => AppTheme::Dark,
            Theme::Custom(_) => todo!(),
        }
    }
}

pub enum ColorKeyButtonStyle {
    ColorKeyRed,
    ColorKeyGreen,
    ColorKeyBlue,
    ColorKeyYellow,
}

impl ColorKeyButtonStyle {
    fn create_color(&self) -> Color {
        match self {
            ColorKeyButtonStyle::ColorKeyRed => Color::new(1.0, 0.0, 0.0, 1.0),
            ColorKeyButtonStyle::ColorKeyGreen => Color::new(0.0, 1.0, 0.0, 1.0),
            ColorKeyButtonStyle::ColorKeyBlue => Color::new(0.0, 0.0, 1.0, 1.0),
            ColorKeyButtonStyle::ColorKeyYellow => Color::new(1.0, 1.0, 0.0, 1.0),
        }
    }
}

impl Default for ColorKeyButtonStyle {
    fn default() -> Self {
        Self::ColorKeyRed
    }
}

impl iced::widget::button::StyleSheet for ColorKeyButtonStyle {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        match self {
            ColorKeyButtonStyle::ColorKeyRed
            | ColorKeyButtonStyle::ColorKeyGreen
            | ColorKeyButtonStyle::ColorKeyBlue
            | ColorKeyButtonStyle::ColorKeyYellow => iced::widget::button::Appearance {
                shadow_offset: Default::default(),
                background: Some(self.create_color().into()),
                border_radius: 2.0,
                border_width: 1.0,
                border_color: [0.7, 0.7, 0.7].into(),
                text_color: Color::BLACK,
            },
        }
    }
}
