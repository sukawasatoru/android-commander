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

use crate::feature::migrate::migrate_functions::{load_toml, write_toml};
use crate::model::FileVersion;
use crate::prelude::*;
use std::path::Path;
use tracing::info;

pub fn migrate_0_1_2(preferences_dir: &Path) -> Fallible<()> {
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

    if "0.1.2".parse::<FileVersion>()? <= prefs_version {
        info!(%prefs_version, "skip migration");
        return Ok(());
    }

    info!("set version to preferences.toml");

    let prefs_table = preferences
        .as_table_mut()
        .context("failed to parse to table")?;

    prefs_table.insert("version".into(), toml::Value::String("0.1.2".into()));

    info!("set key_map to preferences.toml");

    let key_map_table = prefs_table
        .get_mut("key_map")
        .context("preferences.key_map")?
        .as_table_mut()
        .context("failed to parse to key_map table")?;

    (0..=9).for_each(|num| {
        key_map_table.insert(
            format!("numpad_{}", num),
            toml::Value::String(format!("KEYCODE_NUMPAD_{}", num)),
        );
    });

    write_toml(&preferences_path, &preferences)?;

    info!("succeeded set version to preferences.toml");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::migrate::migrate_functions::tests::{check_version, prepare_preferences};
    use tempfile::tempdir;

    #[test]
    fn migrate_0_1_2() {
        // tracing_subscriber::fmt()
        //     .with_max_level(tracing::Level::TRACE)
        //     .init();

        let old_preferences = r#"
version = "0.1.1"

[key_map]
color_red = "red"
color_green = "green"
color_blue = "blue"
color_yellow = "yellow"
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

        super::migrate_0_1_2(prefs_dir).unwrap();

        let preferences_toml = load_toml(&prefs_dir.join("preferences.toml")).unwrap();

        check_version(&preferences_toml, "0.1.2");

        let actual_key_map = preferences_toml["key_map"]
            .as_table()
            .context("new preferences.key_map")
            .unwrap();

        assert_eq!(
            "KEYCODE_NUMPAD_1",
            actual_key_map["numpad_1"].as_str().unwrap()
        );
        assert_eq!(
            "KEYCODE_NUMPAD_2",
            actual_key_map["numpad_2"].as_str().unwrap()
        );
        assert_eq!(
            "KEYCODE_NUMPAD_3",
            actual_key_map["numpad_3"].as_str().unwrap()
        );
        assert_eq!(
            "KEYCODE_NUMPAD_4",
            actual_key_map["numpad_4"].as_str().unwrap()
        );
        assert_eq!(
            "KEYCODE_NUMPAD_5",
            actual_key_map["numpad_5"].as_str().unwrap()
        );
        assert_eq!(
            "KEYCODE_NUMPAD_6",
            actual_key_map["numpad_6"].as_str().unwrap()
        );
        assert_eq!(
            "KEYCODE_NUMPAD_7",
            actual_key_map["numpad_7"].as_str().unwrap()
        );
        assert_eq!(
            "KEYCODE_NUMPAD_8",
            actual_key_map["numpad_8"].as_str().unwrap()
        );
        assert_eq!(
            "KEYCODE_NUMPAD_9",
            actual_key_map["numpad_9"].as_str().unwrap()
        );
    }

    #[test]
    fn skip_migrate() {
        // tracing_subscriber::fmt()
        //     .with_max_level(tracing::Level::TRACE)
        //     .init();

        let preferences_str = r#"
version = "0.1.3"

[key_map]
back = "KEYCODE_f"
color_red = "red"
color_green = "green"
color_blue = "blue"
color_yellow = "yellow"
dpad_up = "KEYCODE_a"
dpad_down = "KEYCODE_b"
dpad_left = "KEYCODE_c"
dpad_right = "KEYCODE_d"
dpad_ok = "KEYCODE_e"
numpad_1 = "KEYCODE_NUMPAD_1"
numpad_2 = "KEYCODE_NUMPAD_2"
numpad_3 = "KEYCODE_NUMPAD_3"
numpad_4 = "KEYCODE_NUMPAD_4"
numpad_5 = "KEYCODE_NUMPAD_5"
numpad_6 = "KEYCODE_NUMPAD_6"
numpad_7 = "KEYCODE_NUMPAD_7"
numpad_8 = "KEYCODE_NUMPAD_8"
numpad_9 = "KEYCODE_NUMPAD_9"
home = "KEYCODE_g"
"#;

        let temp_dir = tempdir().context("prepare tempfile::tempdir()").unwrap();
        let prefs_dir = temp_dir.path();
        info!(?prefs_dir);

        prepare_preferences(prefs_dir, preferences_str);

        super::migrate_0_1_2(prefs_dir).unwrap();

        let new_prefs = load_toml(&prefs_dir.join("preferences.toml")).unwrap();

        check_version(&new_prefs, "0.1.3");
    }
}
