/*
 * Copyright 2022, 2025, 2026 sukawasatoru
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

use iced::Theme;
use iced::keyboard::{Key, key};

#[derive(Debug, PartialEq)]
pub struct Preferences {
    pub key_map: KeyMap,
    pub theme: Theme,
    pub custom_keys: Vec<CustomKeyEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CustomKeyEntry {
    pub label: String,
    pub keycode: String,
    pub shortcut: Option<String>,
}

impl CustomKeyEntry {
    pub fn matches_key(&self, key: &Key, modified_key: &Key) -> bool {
        let Some(shortcut) = &self.shortcut else {
            return false;
        };

        if let Some(named) = parse_named_key(shortcut) {
            return matches!(key.as_ref(), Key::Named(k) if k == named)
                || matches!(modified_key.as_ref(), Key::Named(k) if k == named);
        }

        if let Key::Character(c) = key.as_ref()
            && c == shortcut.as_str()
        {
            return true;
        }
        if let Key::Character(c) = modified_key.as_ref()
            && c == shortcut.as_str()
        {
            return true;
        }
        false
    }
}

fn parse_named_key(s: &str) -> Option<key::Named> {
    match s {
        "F1" => Some(key::Named::F1),
        "F2" => Some(key::Named::F2),
        "F3" => Some(key::Named::F3),
        "F4" => Some(key::Named::F4),
        "F5" => Some(key::Named::F5),
        "F6" => Some(key::Named::F6),
        "F7" => Some(key::Named::F7),
        "F8" => Some(key::Named::F8),
        "F9" => Some(key::Named::F9),
        "F10" => Some(key::Named::F10),
        "F11" => Some(key::Named::F11),
        "F12" => Some(key::Named::F12),
        _ => None,
    }
}

impl std::fmt::Display for CustomKeyEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.shortcut {
            Some(s) => write!(f, "{} ({})", self.label, s),
            None => write!(f, "{}", self.label),
        }
    }
}

pub fn default_custom_keys() -> Vec<CustomKeyEntry> {
    vec![
        CustomKeyEntry {
            label: "Power".into(),
            keycode: "KEYCODE_POWER".into(),
            shortcut: Some("p".into()),
        },
        CustomKeyEntry {
            label: "Vol Up".into(),
            keycode: "KEYCODE_VOLUME_UP".into(),
            shortcut: Some("F12".into()),
        },
        CustomKeyEntry {
            label: "Vol Down".into(),
            keycode: "KEYCODE_VOLUME_DOWN".into(),
            shortcut: Some("F11".into()),
        },
        CustomKeyEntry {
            label: "Menu".into(),
            keycode: "KEYCODE_MENU".into(),
            shortcut: None,
        },
        CustomKeyEntry {
            label: "Ch Up".into(),
            keycode: "KEYCODE_CHANNEL_UP".into(),
            shortcut: Some("+".into()),
        },
        CustomKeyEntry {
            label: "Ch Down".into(),
            keycode: "KEYCODE_CHANNEL_DOWN".into(),
            shortcut: Some("-".into()),
        },
    ]
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            key_map: KeyMap::default(),
            theme: Theme::Light,
            custom_keys: default_custom_keys(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyMap {
    pub back: String,
    pub color_red: String,
    pub color_green: String,
    pub color_blue: String,
    pub color_yellow: String,
    pub dpad_up: String,
    pub dpad_down: String,
    pub dpad_left: String,
    pub dpad_right: String,
    pub dpad_ok: String,
    pub num_0: String,
    pub num_1: String,
    pub num_2: String,
    pub num_3: String,
    pub num_4: String,
    pub num_5: String,
    pub num_6: String,
    pub num_7: String,
    pub num_8: String,
    pub num_9: String,
    pub home: String,
}

impl Default for KeyMap {
    fn default() -> Self {
        Self {
            color_red: "KEYCODE_PROG_RED".into(),
            color_green: "KEYCODE_PROG_GREEN".into(),
            color_blue: "KEYCODE_PROG_BLUE".into(),
            color_yellow: "KEYCODE_PROG_YELLOW".into(),
            dpad_up: "KEYCODE_DPAD_UP".into(),
            dpad_down: "KEYCODE_DPAD_DOWN".into(),
            dpad_left: "KEYCODE_DPAD_LEFT".into(),
            dpad_right: "KEYCODE_DPAD_RIGHT".into(),
            dpad_ok: "KEYCODE_DPAD_CENTER".into(),
            num_0: "KEYCODE_0".into(),
            num_1: "KEYCODE_1".into(),
            num_2: "KEYCODE_2".into(),
            num_3: "KEYCODE_3".into(),
            num_4: "KEYCODE_4".into(),
            num_5: "KEYCODE_5".into(),
            num_6: "KEYCODE_6".into(),
            num_7: "KEYCODE_7".into(),
            num_8: "KEYCODE_8".into(),
            num_9: "KEYCODE_9".into(),
            back: "KEYCODE_BACK".into(),
            home: "KEYCODE_HOME".into(),
        }
    }
}
