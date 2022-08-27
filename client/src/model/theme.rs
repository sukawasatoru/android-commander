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

impl From<&AppTheme> for iced::Theme {
    fn from(value: &AppTheme) -> Self {
        match value {
            AppTheme::Light => iced::Theme::Light,
            AppTheme::Dark => iced::Theme::Dark,
        }
    }
}

impl iced::application::StyleSheet for AppTheme {
    type Style = iced::theme::Application;

    fn appearance(&self, style: Self::Style) -> iced::application::Appearance {
        iced::theme::Theme::from(self).appearance(style)
    }
}

#[derive(Clone, Copy)]
pub enum ButtonStyle {
    Primary,
    Secondary,
    ColorKeyRed,
    ColorKeyGreen,
    ColorKeyBlue,
    ColorKeyYellow,
}

impl ButtonStyle {
    fn create_color(&self) -> Color {
        match self {
            ButtonStyle::Primary => panic!(),
            ButtonStyle::Secondary => panic!(),
            ButtonStyle::ColorKeyRed => Color::new(1.0, 0.0, 0.0, 1.0),
            ButtonStyle::ColorKeyGreen => Color::new(0.0, 1.0, 0.0, 1.0),
            ButtonStyle::ColorKeyBlue => Color::new(0.0, 0.0, 1.0, 1.0),
            ButtonStyle::ColorKeyYellow => Color::new(1.0, 1.0, 0.0, 1.0),
        }
    }
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self::Secondary
    }
}

impl iced::widget::button::StyleSheet for AppTheme {
    type Style = ButtonStyle;

    fn active(&self, style: Self::Style) -> iced::widget::button::Appearance {
        match style {
            ButtonStyle::Primary => {
                iced::theme::Theme::from(self).active(iced::theme::Button::Primary)
            }
            ButtonStyle::Secondary => {
                iced::theme::Theme::from(self).active(iced::theme::Button::Secondary)
            }
            ButtonStyle::ColorKeyRed
            | ButtonStyle::ColorKeyGreen
            | ButtonStyle::ColorKeyBlue
            | ButtonStyle::ColorKeyYellow => iced::widget::button::Appearance {
                shadow_offset: Default::default(),
                background: Some(style.create_color().into()),
                border_radius: 2.0,
                border_width: 1.0,
                border_color: [0.7, 0.7, 0.7].into(),
                text_color: Color::BLACK,
            },
        }
    }
}

impl iced::widget::checkbox::StyleSheet for AppTheme {
    type Style = iced::theme::Checkbox;

    fn active(&self, style: Self::Style, is_checked: bool) -> iced::widget::checkbox::Appearance {
        iced::theme::Theme::from(self).active(style, is_checked)
    }

    fn hovered(&self, style: Self::Style, is_checked: bool) -> iced::widget::checkbox::Appearance {
        iced::theme::Theme::from(self).hovered(style, is_checked)
    }
}

impl iced::widget::container::StyleSheet for AppTheme {
    type Style = iced::theme::Container;

    fn appearance(&self, style: Self::Style) -> iced::widget::container::Appearance {
        iced::theme::Theme::from(self).appearance(style)
    }
}

impl iced_style::menu::StyleSheet for AppTheme {
    type Style = ();

    fn appearance(&self, style: Self::Style) -> iced_style::menu::Appearance {
        iced::theme::Theme::from(self).appearance(style)
    }
}

impl iced::widget::pick_list::StyleSheet for AppTheme {
    type Style = ();

    fn active(&self, style: ()) -> iced::widget::pick_list::Appearance {
        iced::Theme::from(self).active(style)
    }

    fn hovered(&self, style: ()) -> iced::widget::pick_list::Appearance {
        iced::Theme::from(self).hovered(style)
    }
}

impl iced::widget::scrollable::StyleSheet for AppTheme {
    type Style = ();

    fn active(&self, style: Self::Style) -> iced::widget::scrollable::Scrollbar {
        iced::Theme::from(self).active(style)
    }

    fn hovered(&self, style: Self::Style) -> iced::widget::scrollable::Scrollbar {
        iced::Theme::from(self).hovered(style)
    }
}

impl iced::widget::text::StyleSheet for AppTheme {
    type Style = iced::theme::Text;

    fn appearance(&self, style: Self::Style) -> iced::widget::text::Appearance {
        iced::Theme::from(self).appearance(style)
    }
}
