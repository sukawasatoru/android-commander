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

mod migrate_0_1_0;
mod migrate_0_1_1;
mod migrate_0_1_2;
mod migrate_functions;

use crate::model::FileVersion;
use crate::prelude::*;
use migrate_0_1_0::migrate_0_1_0;
use migrate_0_1_1::migrate_0_1_1;
use migrate_0_1_2::migrate_0_1_2;
use std::fs::File;
use std::io::{prelude::*, BufReader, BufWriter};
use std::path::Path;
use tracing::info;

pub fn migrate() -> Fallible<()> {
    let version = env!("CARGO_PKG_VERSION").parse::<FileVersion>()?;

    info!(%version, "start migration");

    let project_dirs = directories::ProjectDirs::from("com", "sukawasatoru", "AndroidCommander")
        .context("directories")?;

    let prefs_dir = project_dirs.config_dir();

    #[allow(clippy::type_complexity)]
    let functions: Vec<(&str, Box<dyn Fn() -> Fallible<()>>)> = vec![
        ("0.1.0", Box::new(|| migrate_0_1_0(prefs_dir))),
        ("0.1.1", Box::new(|| migrate_0_1_1(prefs_dir))),
        ("0.1.2", Box::new(|| migrate_0_1_2(prefs_dir))),
    ];

    for (version_str, migrate) in functions {
        let migrate_version = version_str.parse::<FileVersion>()?;

        info!(%migrate_version, "start migrate");
        migrate()?;
        info!(%migrate_version, "end migrate");
    }

    set_latest_version(project_dirs.config_dir(), version.clone())?;

    info!(%version, "succeeded all migration");
    Ok(())
}

fn set_latest_version(preferences_dir: &Path, new_version: FileVersion) -> Fallible<()> {
    let new_version_string = new_version.to_string();
    let preferences_path = preferences_dir.join("preferences.toml");

    let mut buf = String::new();

    if preferences_path.exists() {
        info!("check preferences.toml");

        buf.clear();

        BufReader::new(File::open(&preferences_path)?)
            .read_to_string(&mut buf)
            .context("read preferences.toml")?;

        let mut preferences =
            toml::from_str::<toml::Value>(&buf).context("toml::from_str for preferences.toml")?;

        let preferences_version_str = preferences["version"]
            .as_str()
            .context("preferences.version")?;
        if preferences_version_str != new_version_string {
            info!("set version to preferences.toml");

            preferences["version"] = toml::Value::try_from(&new_version)?;

            let mut writer = BufWriter::new(File::create(&preferences_path)?);
            writer.write_all(toml::to_string_pretty(&preferences)?.as_bytes())?;
            writer.flush()?;

            info!("succeeded set version to preferences.toml");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tracing::info;

    #[tokio::test]
    async fn test_set_version() {
        // tracing_subscriber::fmt()
        //     .with_max_level(tracing::Level::TRACE)
        //     .init();

        let old_preferences = r#"
version = "0.0.1"

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

        let mut writer = BufWriter::new(
            File::create(prefs_dir.join("preferences.toml"))
                .context("create preferences.toml")
                .unwrap(),
        );
        writer
            .write_all(old_preferences.as_bytes())
            .context("write preferences.toml")
            .unwrap();
        writer.flush().unwrap();
        drop(writer);

        set_latest_version(prefs_dir, "0.1.0".parse().unwrap()).unwrap();

        let mut buf = String::new();
        let mut reader = BufReader::new(File::open(prefs_dir.join("preferences.toml")).unwrap());
        reader
            .read_to_string(&mut buf)
            .context("read preferences.toml")
            .unwrap();
        let preferences_toml = toml::from_str::<toml::Value>(&buf)
            .context("parse preferences.toml")
            .unwrap();

        assert_eq!(
            preferences_toml["version"]
                .as_str()
                .context("preferences.toml 's version")
                .unwrap(),
            "0.1.0"
        );
    }
}
