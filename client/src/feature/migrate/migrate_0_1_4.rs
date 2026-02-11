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

use crate::feature::migrate::migrate_functions::{load_toml, write_toml};
use crate::model::{FileVersion, default_custom_keys};
use crate::prelude::*;
use std::path::Path;

pub fn migrate_0_1_4(preferences_dir: &Path) -> Fallible<()> {
    let preferences_path = preferences_dir.join("preferences.toml");

    if !preferences_path.exists() {
        info!("preferences.toml not found");
        return Ok(());
    }

    info!("check preferences.toml");

    let mut preferences = load_toml(&preferences_path)?;

    let prefs_version = preferences["version"]
        .as_str()
        .context("preferences.version")?
        .parse::<FileVersion>()?;

    if "0.1.4".parse::<FileVersion>()? <= prefs_version {
        info!(%prefs_version, "skip migration");
        return Ok(());
    }

    let prefs_table = preferences
        .as_table_mut()
        .context("failed to parse to table")?;

    prefs_table.insert("version".into(), toml::Value::String("0.1.4".into()));

    if prefs_table.get("custom_keys").is_none() {
        info!("set custom_keys to preferences.toml");

        let custom_keys = default_custom_keys();
        let mut custom_keys_array = toml::value::Array::new();

        for entry in &custom_keys {
            let mut table = toml::value::Table::new();
            table.insert("label".into(), toml::Value::String(entry.label.clone()));
            table.insert("keycode".into(), toml::Value::String(entry.keycode.clone()));
            if let Some(shortcut) = &entry.shortcut {
                table.insert("shortcut".into(), toml::Value::String(shortcut.clone()));
            }
            custom_keys_array.push(toml::Value::Table(table));
        }

        prefs_table.insert("custom_keys".into(), toml::Value::Array(custom_keys_array));
    }

    write_toml(&preferences_path, &preferences)?;

    info!("succeeded migration to 0.1.4");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::migrate::migrate_functions::tests::{check_version, prepare_preferences};
    use tempfile::tempdir;

    #[test]
    fn migrate_0_1_4_adds_custom_keys() {
        let old_preferences = r#"
version = "0.1.3"

[key_map]
back = "KEYCODE_BACK"
color_red = "KEYCODE_PROG_RED"
color_green = "KEYCODE_PROG_GREEN"
color_blue = "KEYCODE_PROG_BLUE"
color_yellow = "KEYCODE_PROG_YELLOW"
dpad_up = "KEYCODE_DPAD_UP"
dpad_down = "KEYCODE_DPAD_DOWN"
dpad_left = "KEYCODE_DPAD_LEFT"
dpad_right = "KEYCODE_DPAD_RIGHT"
dpad_ok = "KEYCODE_DPAD_CENTER"
num_0 = "KEYCODE_0"
num_1 = "KEYCODE_1"
num_2 = "KEYCODE_2"
num_3 = "KEYCODE_3"
num_4 = "KEYCODE_4"
num_5 = "KEYCODE_5"
num_6 = "KEYCODE_6"
num_7 = "KEYCODE_7"
num_8 = "KEYCODE_8"
num_9 = "KEYCODE_9"
home = "KEYCODE_HOME"
"#;

        let temp_dir = tempdir().context("prepare tempfile::tempdir()").unwrap();
        let prefs_dir = temp_dir.path();
        info!(?prefs_dir);

        prepare_preferences(prefs_dir, old_preferences);

        super::migrate_0_1_4(prefs_dir).unwrap();

        let preferences_toml = load_toml(&prefs_dir.join("preferences.toml")).unwrap();

        check_version(&preferences_toml, "0.1.4");

        let custom_keys = preferences_toml["custom_keys"]
            .as_array()
            .expect("custom_keys should be an array");

        assert_eq!(custom_keys.len(), default_custom_keys().len());

        let first = custom_keys[0].as_table().unwrap();
        assert_eq!(first["label"].as_str().unwrap(), "Power");
        assert_eq!(first["keycode"].as_str().unwrap(), "KEYCODE_POWER");
        assert_eq!(first["shortcut"].as_str().unwrap(), "p");
    }

    #[test]
    fn migrate_0_1_4_preserves_existing_custom_keys() {
        let old_preferences = r#"
version = "0.1.3"

[key_map]
back = "KEYCODE_BACK"
color_red = "KEYCODE_PROG_RED"
color_green = "KEYCODE_PROG_GREEN"
color_blue = "KEYCODE_PROG_BLUE"
color_yellow = "KEYCODE_PROG_YELLOW"
dpad_up = "KEYCODE_DPAD_UP"
dpad_down = "KEYCODE_DPAD_DOWN"
dpad_left = "KEYCODE_DPAD_LEFT"
dpad_right = "KEYCODE_DPAD_RIGHT"
dpad_ok = "KEYCODE_DPAD_CENTER"
num_0 = "KEYCODE_0"
num_1 = "KEYCODE_1"
num_2 = "KEYCODE_2"
num_3 = "KEYCODE_3"
num_4 = "KEYCODE_4"
num_5 = "KEYCODE_5"
num_6 = "KEYCODE_6"
num_7 = "KEYCODE_7"
num_8 = "KEYCODE_8"
num_9 = "KEYCODE_9"
home = "KEYCODE_HOME"

[[custom_keys]]
label = "MyKey"
keycode = "KEYCODE_MY_KEY"
shortcut = "m"
"#;

        let temp_dir = tempdir().context("prepare tempfile::tempdir()").unwrap();
        let prefs_dir = temp_dir.path();
        info!(?prefs_dir);

        prepare_preferences(prefs_dir, old_preferences);

        super::migrate_0_1_4(prefs_dir).unwrap();

        let preferences_toml = load_toml(&prefs_dir.join("preferences.toml")).unwrap();

        check_version(&preferences_toml, "0.1.4");

        let custom_keys = preferences_toml["custom_keys"]
            .as_array()
            .expect("custom_keys should be an array");

        assert_eq!(custom_keys.len(), 1);
        let first = custom_keys[0].as_table().unwrap();
        assert_eq!(first["label"].as_str().unwrap(), "MyKey");
    }

    #[test]
    fn skip_migrate() {
        let preferences_str = r#"
version = "0.1.5"

[key_map]
back = "KEYCODE_BACK"
color_red = "KEYCODE_PROG_RED"
color_green = "KEYCODE_PROG_GREEN"
color_blue = "KEYCODE_PROG_BLUE"
color_yellow = "KEYCODE_PROG_YELLOW"
dpad_up = "KEYCODE_DPAD_UP"
dpad_down = "KEYCODE_DPAD_DOWN"
dpad_left = "KEYCODE_DPAD_LEFT"
dpad_right = "KEYCODE_DPAD_RIGHT"
dpad_ok = "KEYCODE_DPAD_CENTER"
num_0 = "KEYCODE_0"
num_1 = "KEYCODE_1"
num_2 = "KEYCODE_2"
num_3 = "KEYCODE_3"
num_4 = "KEYCODE_4"
num_5 = "KEYCODE_5"
num_6 = "KEYCODE_6"
num_7 = "KEYCODE_7"
num_8 = "KEYCODE_8"
num_9 = "KEYCODE_9"
home = "KEYCODE_HOME"
"#;

        let temp_dir = tempdir().context("prepare tempfile::tempdir()").unwrap();
        let prefs_dir = temp_dir.path();
        info!(?prefs_dir);

        prepare_preferences(prefs_dir, preferences_str);

        super::migrate_0_1_4(prefs_dir).unwrap();

        let new_prefs = load_toml(&prefs_dir.join("preferences.toml")).unwrap();

        check_version(&new_prefs, "0.1.5");
    }
}
