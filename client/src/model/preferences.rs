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

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Preferences {
    pub key_map: KeyMap,
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
    pub numpad_0: String,
    pub numpad_1: String,
    pub numpad_2: String,
    pub numpad_3: String,
    pub numpad_4: String,
    pub numpad_5: String,
    pub numpad_6: String,
    pub numpad_7: String,
    pub numpad_8: String,
    pub numpad_9: String,
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
            numpad_0: "KEYCODE_NUMPAD_0".into(),
            numpad_1: "KEYCODE_NUMPAD_1".into(),
            numpad_2: "KEYCODE_NUMPAD_2".into(),
            numpad_3: "KEYCODE_NUMPAD_3".into(),
            numpad_4: "KEYCODE_NUMPAD_4".into(),
            numpad_5: "KEYCODE_NUMPAD_5".into(),
            numpad_6: "KEYCODE_NUMPAD_6".into(),
            numpad_7: "KEYCODE_NUMPAD_7".into(),
            numpad_8: "KEYCODE_NUMPAD_8".into(),
            numpad_9: "KEYCODE_NUMPAD_9".into(),
            back: "KEYCODE_BACK".into(),
            home: "KEYCODE_HOME".into(),
        }
    }
}
