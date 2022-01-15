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

#[derive(Clone, Debug, Default)]
pub struct Preferences {
    pub key_map: KeyMap,
}

#[derive(Clone, Debug)]
pub struct KeyMap {
    pub dpad_up: String,
    pub dpad_down: String,
    pub dpad_left: String,
    pub dpad_right: String,
    pub dpad_ok: String,
    pub back: String,
    pub home: String,
}

impl Default for KeyMap {
    fn default() -> Self {
        Self {
            dpad_up: "KEYCODE_DPAD_UP".into(),
            dpad_down: "KEYCODE_DPAD_DOWN".into(),
            dpad_left: "KEYCODE_DPAD_LEFT".into(),
            dpad_right: "KEYCODE_DPAD_RIGHT".into(),
            dpad_ok: "KEYCODE_ENTER".into(),
            back: "KEYCODE_BACK".into(),
            home: "KEYCODE_HOME".into(),
        }
    }
}
