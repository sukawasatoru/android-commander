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
use tracing::info;

pub fn migrate_0_1_0(preferences_dir: &Path) -> Fallible<()> {
    let preferences_path = preferences_dir.join("preferences.toml");

    if !preferences_path.exists() {
        info!("preferences.toml not found");
        return Ok(());
    }

    info!("check preferences.toml");

    let mut buf = String::new();
    BufReader::new(File::open(&preferences_path)?)
        .read_to_string(&mut buf)
        .context("read preferences.toml")?;

    let mut preferences =
        toml::from_str::<toml::Value>(&buf).context("toml::from_str for preferences.toml")?;

    if let Some(version) = preferences.get("version") {
        info!(?version, "exists version");
        return Ok(());
    }

    info!("set version to preferences.toml");

    preferences
        .as_table_mut()
        .context("failed to parse to table")?
        .insert("version".into(), toml::Value::String("0.1.0".into()));

    let mut writer = BufWriter::new(File::create(&preferences_path)?);
    writer.write_all(toml::to_string_pretty(&preferences)?.as_bytes())?;
    writer.flush()?;

    info!("succeeded set version to preferences.toml");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn migrate_0_1_0() {
        let old_preferences = r#"
[key_map]
dpad_up = "KEYCODE_a"
dpad_down = "KEYCODE_b"
dpad_left = "KEYCODE_c"
dpad_right = "KEYCODE_d"
dpad_ok = "KEYCODE_e"
back = "KEYCODE_f"
home = "KEYCODE_g"
"#;

        let temp_dir = tempdir().context("prepare tempfile::tempdir()").unwrap();
        let prefs_dir = temp_dir.path();
        info!(?prefs_dir);

        prepare_preferences(prefs_dir, old_preferences);

        super::migrate_0_1_0(prefs_dir).unwrap();

        let preferences_toml = load_preferences(&prefs_dir.join("preferences.toml"));

        assert_eq!(
            preferences_toml["version"]
                .as_str()
                .context("preferences.toml 's version")
                .unwrap(),
            "0.1.0"
        );

        let preferences_toml = load_preferences(&prefs_dir.join("preferences.toml"));

        assert_eq!(
            "0.1.0",
            preferences_toml["version"]
                .as_str()
                .context("new preferences.version")
                .unwrap()
        );

        let actual_key_map = preferences_toml["key_map"]
            .as_table()
            .context("new preferences.key_map")
            .unwrap();

        assert_eq!(
            "KEYCODE_a",
            actual_key_map["dpad_up"]
                .as_str()
                .context("preferences.dpad_up")
                .unwrap()
        );
        assert_eq!(
            "KEYCODE_b",
            actual_key_map["dpad_down"]
                .as_str()
                .context("preferences.dpad_down")
                .unwrap()
        );
        assert_eq!(
            "KEYCODE_c",
            actual_key_map["dpad_left"]
                .as_str()
                .context("preferences.dpad_c")
                .unwrap()
        );
        assert_eq!(
            "KEYCODE_d",
            actual_key_map["dpad_right"]
                .as_str()
                .context("preferences.dpad_right")
                .unwrap()
        );
        assert_eq!(
            "KEYCODE_e",
            actual_key_map["dpad_ok"]
                .as_str()
                .context("preferences.dpad_ok")
                .unwrap()
        );
        assert_eq!(
            "KEYCODE_f",
            actual_key_map["back"]
                .as_str()
                .context("preferences.back")
                .unwrap()
        );
        assert_eq!(
            "KEYCODE_g",
            actual_key_map["home"]
                .as_str()
                .context("preferences.home")
                .unwrap()
        );
    }

    #[test]
    fn skip_migrate() {
        let old_preferences = r#"
version = "0.1.1"
[key_map]
dpad_up = "KEYCODE_a"
dpad_down = "KEYCODE_b"
dpad_left = "KEYCODE_c"
dpad_right = "KEYCODE_d"
dpad_ok = "KEYCODE_e"
back = "KEYCODE_f"
home = "KEYCODE_g"
"#;

        let temp_dir = tempdir().context("prepare tempfile::tempdir()").unwrap();
        let prefs_dir = temp_dir.path();
        info!(?prefs_dir);

        prepare_preferences(prefs_dir, old_preferences);

        let new_prefs = load_preferences(&prefs_dir.join("preferences.toml"));
        assert_eq!(
            "0.1.1",
            new_prefs["version"]
                .as_str()
                .context("preferences.version")
                .unwrap()
        );
    }

    fn prepare_preferences(prefs_dir: &Path, file_str: &str) {
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

    fn load_preferences(prefs_file_path: &Path) -> toml::Value {
        let mut buf = String::new();
        BufReader::new(File::open(prefs_file_path).unwrap())
            .read_to_string(&mut buf)
            .context("read preferences.toml")
            .unwrap();
        toml::from_str(&buf)
            .context("parse preferences.toml")
            .unwrap()
    }
}
