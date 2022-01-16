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

pub fn migrate_0_1_1(preferences_dir: &Path) -> Fallible<()> {
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

    if "0.1.1".parse::<FileVersion>()? <= prefs_version {
        info!(%prefs_version, "skip migration");
        return Ok(());
    }

    info!("set version to preferences.toml");

    let prefs_table = preferences
        .as_table_mut()
        .context("failed to parse to table")?;

    prefs_table.insert("version".into(), toml::Value::String("0.1.1".into()));

    info!("set key_map to preferences.toml");

    let key_map_table = prefs_table
        .get_mut("key_map")
        .context("preferences.key_map")?
        .as_table_mut()
        .context("failed to parse to key_map table")?;

    key_map_table.insert(
        "color_red".into(),
        toml::Value::String("KEYCODE_PROG_RED".into()),
    );
    key_map_table.insert(
        "color_green".into(),
        toml::Value::String("KEYCODE_PROG_GREEN".into()),
    );
    key_map_table.insert(
        "color_blue".into(),
        toml::Value::String("KEYCODE_PROG_BLUE".into()),
    );
    key_map_table.insert(
        "color_yellow".into(),
        toml::Value::String("KEYCODE_PROG_YELLOW".into()),
    );

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
    fn migrate_0_1_1() {
        // tracing_subscriber::fmt()
        //     .with_max_level(tracing::Level::TRACE)
        //     .init();

        let old_preferences = r#"
version = "0.1.0"

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

        super::migrate_0_1_1(prefs_dir).unwrap();

        let preferences_toml = load_toml(&prefs_dir.join("preferences.toml")).unwrap();

        check_version(&preferences_toml, "0.1.1");

        let actual_key_map = preferences_toml["key_map"]
            .as_table()
            .context("new preferences.key_map")
            .unwrap();

        assert_eq!(
            "KEYCODE_PROG_RED",
            actual_key_map["color_red"]
                .as_str()
                .context("preferences.color_red")
                .unwrap()
        );

        assert_eq!(
            "KEYCODE_PROG_GREEN",
            actual_key_map["color_green"]
                .as_str()
                .context("preferences.color_green")
                .unwrap()
        );

        assert_eq!(
            "KEYCODE_PROG_BLUE",
            actual_key_map["color_blue"]
                .as_str()
                .context("preferences.color_blue")
                .unwrap()
        );

        assert_eq!(
            "KEYCODE_PROG_YELLOW",
            actual_key_map["color_yellow"]
                .as_str()
                .context("preferences.color_yellow")
                .unwrap()
        );
    }

    #[test]
    fn skip_migrate() {
        let preferences_str = r#"
version = "0.1.2"

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

        prepare_preferences(prefs_dir, preferences_str);

        super::migrate_0_1_1(prefs_dir).unwrap();

        let new_prefs = load_toml(&prefs_dir.join("preferences.toml")).unwrap();

        check_version(&new_prefs, "0.1.2");
    }
}
