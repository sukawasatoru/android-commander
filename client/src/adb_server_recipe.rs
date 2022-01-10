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

use crate::data::asset::Asset;
use crate::model::AndroidDevice;
use iced::futures::stream::{unfold, BoxStream};
use iced_futures::subscription::Recipe;
use std::hash::Hash;
use std::io::prelude::*;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::fs::{create_dir_all, File};
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::watch::Receiver;
use tracing::{debug, info, warn};

#[derive(Clone, Debug)]
pub enum AdbServerRecipeEvent {
    Connected,
    Disconnected,
    Error,
}

enum StreamState {
    Init(Receiver<String>, Arc<AndroidDevice>),
    Ready(Receiver<String>, std::process::Child),
    Disconnecting,
    Finish,
}

pub struct AdbServerRecipe {
    pub device: Arc<AndroidDevice>,
    pub rx: Receiver<String>,
}

impl<H, I> Recipe<H, I> for AdbServerRecipe
where
    H: std::hash::Hasher,
{
    type Output = AdbServerRecipeEvent;

    fn hash(&self, state: &mut H) {
        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(self: Box<Self>, _: BoxStream<'_, I>) -> BoxStream<'_, Self::Output> {
        use AdbServerRecipeEvent as YieldValue;

        Box::pin(unfold(
            StreamState::Init(self.rx, self.device),
            |state| async move {
                match state {
                    StreamState::Init(rx, device) => {
                        let server_path = match tempdir() {
                            Ok(data) => data.path().join("android-commander-server"),
                            Err(e) => {
                                warn!(?e, "failed to prepare temporary directory");
                                return Some((YieldValue::Error, StreamState::Finish));
                            }
                        };

                        info!(?server_path);

                        match create_dir_all(&server_path.parent().unwrap()).await {
                            Ok(_) => (),
                            Err(e) => {
                                warn!(?e, "failed to create temporary directory");
                                return Some((YieldValue::Error, StreamState::Finish));
                            }
                        }

                        let server_file = match File::create(&server_path).await {
                            Ok(data) => data,
                            Err(e) => {
                                warn!(?e, "failed to create temporary file");
                                return Some((YieldValue::Error, StreamState::Finish));
                            }
                        };

                        let server_bin = match Asset::get("android-commander-server") {
                            Some(data) => data as rust_embed::EmbeddedFile,
                            None => {
                                warn!("failed to get asset");
                                return Some((YieldValue::Error, StreamState::Finish));
                            }
                        };

                        let mut buf = BufWriter::new(server_file);
                        match buf.write_all(&server_bin.data).await {
                            Ok(_) => (),
                            Err(e) => {
                                warn!(?e, "failed to write server data");
                                return Some((YieldValue::Error, StreamState::Finish));
                            }
                        }

                        match buf.flush().await {
                            Ok(_) => (),
                            Err(e) => {
                                warn!(?e, "failed to flush server data");
                                return Some((YieldValue::Error, StreamState::Finish));
                            }
                        }

                        match std::process::Command::new("adb")
                            .args(&[
                                "-s",
                                &device.serial,
                                "push",
                                server_path.to_str().unwrap(),
                                "/data/local/tmp/android-commander-server",
                            ])
                            .spawn()
                        {
                            Ok(_) => (),
                            Err(e) => {
                                warn!(?e, "failed to push server file");
                                return Some((YieldValue::Error, StreamState::Finish));
                            }
                        }

                        match std::process::Command::new("adb")
                            .args(&[
                                "-s",
                                &device.serial,
                                "shell",
                                "CLASSPATH=/data/local/tmp/android-commander-server app_process / jp.tinyport.androidcommander.server.MainKt"
                            ])
                            .stdin(std::process::Stdio::piped())
                            .spawn()
                        {
                            Ok(mut data) => match &data.stdin {
                                Some(_) => Some((YieldValue::Connected, StreamState::Ready(rx, data))),
                                None => {
                                    warn!("stdin not found");
                                    data.kill().ok();
                                    data.wait().ok();
                                    Some((YieldValue::Error, StreamState::Finish))
                                }
                            },
                            Err(e) => {
                                warn!("{:?}", e);
                                Some((YieldValue::Error, StreamState::Finish))
                            }
                        }
                    }
                    StreamState::Ready(mut rx, mut child) => {
                        loop {
                            if rx.changed().await.is_err() {
                                break;
                            }

                            let data = rx.borrow();
                            debug!("send data: {}", data.as_str());

                            // for ignore init value.
                            if data.is_empty() {
                                continue;
                            }

                            let ret = writeln!(child.stdin.as_mut().unwrap(), "{}", data.as_str());
                            if let Err(e) = ret {
                                warn!("{:?}", e);
                                child.kill().ok();
                                child.wait().ok();
                                return Some((YieldValue::Error, StreamState::Disconnecting));
                            }
                        }

                        debug!("channel closed");
                        child.kill().ok();
                        child.wait().ok();
                        Some((YieldValue::Disconnected, StreamState::Finish))
                    }
                    StreamState::Disconnecting => {
                        Some((YieldValue::Disconnected, StreamState::Finish))
                    }
                    StreamState::Finish => {
                        debug!("finish");
                        None
                    }
                }
            },
        ))
    }
}
