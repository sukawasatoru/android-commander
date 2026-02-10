/*
 * Copyright 2026 sukawasatoru
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

use iced::widget::button;
use iced::{Background, Color, Theme, border};

/// A secondary button style compatible with iced 0.13's color scheme.
///
/// iced 0.14 changed the palette generation logic for secondary colors.
/// This function reproduces the iced 0.13 secondary button appearance.
pub fn button_secondary(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.palette();

    let mix = |a: Color, b: Color, factor: f32| Color {
        r: a.r * (1.0 - factor) + b.r * factor,
        g: a.g * (1.0 - factor) + b.g * factor,
        b: a.b * (1.0 - factor) + b.b * factor,
        a: a.a * (1.0 - factor) + b.a * factor,
    };

    let base_color = mix(palette.background, palette.text, 0.2);
    let strong_color = mix(base_color, palette.text, 0.3);

    let base = button::Style {
        background: Some(Background::Color(base_color)),
        text_color: palette.text,
        border: border::rounded(2),
        ..button::Style::default()
    };

    match status {
        button::Status::Active | button::Status::Pressed => base,
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(strong_color)),
            ..base
        },
        button::Status::Disabled => button::Style {
            background: base
                .background
                .map(|background| background.scale_alpha(0.5)),
            text_color: base.text_color.scale_alpha(0.5),
            ..base
        },
    }
}
