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

use crate::prelude::*;
use std::fs::File;
use std::io::{prelude::*, BufReader, BufWriter};
use std::path::Path;

pub fn load_toml(prefs_file_path: &Path) -> Fallible<toml::Value> {
    let mut buf = String::new();
    BufReader::new(File::open(prefs_file_path)?)
        .read_to_string(&mut buf)
        .context("read preferences.toml")?;
    toml::from_str(&buf).context("parse preferences.toml")
}

pub fn write_toml(file_path: &Path, toml_value: &toml::Value) -> Fallible<()> {
    let mut writer = BufWriter::new(File::create(file_path)?);
    writer.write_all(toml::to_string_pretty(&toml_value)?.as_bytes())?;
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn check_version(toml_value: &toml::Value, version: &str) {
        assert_eq!(
            toml_value["version"]
                .as_str()
                .context("preferences.toml 's version")
                .unwrap(),
            version
        );
    }

    pub fn prepare_preferences(prefs_dir: &Path, file_str: &str) {
        let mut writer = BufWriter::new(
            File::create(prefs_dir.join("preferences.toml"))
                .context("create preferences.toml")
                .unwrap(),
        );
        writer
            .write_all(file_str.as_bytes())
            .context("write preferences.toml")
            .unwrap();
        writer.flush().unwrap();
    }
}
