/*
 * Copyright 2022, 2025 sukawasatoru
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

pub use preferences::{KeyMap, Preferences};
pub use x_message::XMessage;

mod file_version;
mod preferences;
pub mod send_event_key;
mod x_message;

pub use file_version::FileVersion;
use std::fmt::{Display, Formatter};

#[derive(Debug, Eq, PartialEq)]
pub struct AndroidDevice {
    pub serial: String,
}

impl Display for AndroidDevice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.serial)
    }
}
