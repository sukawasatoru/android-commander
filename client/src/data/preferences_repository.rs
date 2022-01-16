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

use crate::model::{KeyMap, Preferences};
use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs::{create_dir_all, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};

#[async_trait::async_trait]
pub trait PreferencesRepository {
    async fn load(&self) -> Fallible<Preferences>;
}

pub struct PreferencesRepositoryImpl {
    config_file_path: PathBuf,
}

impl PreferencesRepositoryImpl {
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            config_file_path: file_path,
        }
    }

    async fn prepare(&self) -> Fallible<()> {
        if !self.config_file_path.exists() {
            create_dir_all(self.config_file_path.parent().context("file directory")?).await?;

            let mut buf = BufWriter::new(
                File::create(&self.config_file_path)
                    .await
                    .context("failed to create preferences file")?,
            );

            buf.write_all(toml::to_string::<PrefsDto>(&Preferences::default().into())?.as_bytes())
                .await?;

            buf.flush().await.context("failed to flush preferences")?;

            return Ok(());
        }

        // migration if needed.
        Ok(())
    }
}

#[async_trait::async_trait]
impl PreferencesRepository for PreferencesRepositoryImpl {
    async fn load(&self) -> Fallible<Preferences> {
        self.prepare().await?;

        let mut buf = BufReader::new(
            File::open(&self.config_file_path)
                .await
                .context("failed to open preferences file")?,
        );

        let mut prefs_string = String::new();
        buf.read_to_string(&mut prefs_string)
            .await
            .context("failed to load preferences file")?;

        Ok(toml::from_str::<PrefsDto>(&prefs_string)
            .with_context(|| format!("failed to parse preferences: {}", prefs_string))?
            .into())
    }
}

pub struct MockPreferencesRepository;

#[async_trait::async_trait]
impl PreferencesRepository for MockPreferencesRepository {
    async fn load(&self) -> Fallible<Preferences> {
        Ok(Default::default())
    }
}

#[derive(Deserialize, Eq, PartialEq, Serialize)]
struct PrefsDto {
    key_map: PrefsKeyMap,
}

impl From<Preferences> for PrefsDto {
    fn from(value: Preferences) -> Self {
        Self {
            key_map: PrefsKeyMap::from(value.key_map),
        }
    }
}

impl From<PrefsDto> for Preferences {
    fn from(value: PrefsDto) -> Self {
        Self {
            key_map: KeyMap::from(value.key_map),
        }
    }
}

#[derive(Deserialize, Eq, PartialEq, Serialize)]
struct PrefsKeyMap {
    dpad_up: String,
    dpad_down: String,
    dpad_left: String,
    dpad_right: String,
    dpad_ok: String,
    back: String,
    home: String,
}

impl From<PrefsKeyMap> for KeyMap {
    fn from(value: PrefsKeyMap) -> Self {
        Self {
            dpad_up: value.dpad_up,
            dpad_down: value.dpad_down,
            dpad_left: value.dpad_left,
            dpad_right: value.dpad_right,
            dpad_ok: value.dpad_ok,
            back: value.back,
            home: value.home,
        }
    }
}

impl From<KeyMap> for PrefsKeyMap {
    fn from(value: KeyMap) -> Self {
        Self {
            dpad_up: value.dpad_up,
            dpad_down: value.dpad_down,
            dpad_left: value.dpad_left,
            dpad_right: value.dpad_right,
            dpad_ok: value.dpad_ok,
            back: value.back,
            home: value.home,
        }
    }
}
