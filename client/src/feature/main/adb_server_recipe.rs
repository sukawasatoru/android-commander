/*
 * Copyright 2022, 2025, 2026 sukawasatoru
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
use crate::prelude::*;
use iced::Subscription;
use iced::futures::SinkExt;
use iced::futures::channel::mpsc::Sender;
use iced::stream::channel;
use std::hash::Hash;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::process::Command as TokioCommand;
use tokio::select;
use tokio::sync::watch::Receiver;

#[derive(Clone, Debug)]
pub enum AdbServerRecipeEvent {
    Connected,
    Disconnected,
    Error,
}

#[derive(Clone)]
pub struct AdbServerRecipe {
    device: Arc<AndroidDevice>,
    rx: Receiver<String>,
}

macro_rules! deploy_server_path {
    () => {
        "/data/local/tmp/android-commander-server"
    };
}

impl AdbServerRecipe {
    pub fn new(device: Arc<AndroidDevice>, rx: Receiver<String>) -> Self {
        Self { device, rx }
    }

    pub fn subscribe(self) -> Subscription<AdbServerRecipeEvent> {
        Subscription::run_with(self, |data| {
            let data = data.clone();
            channel(3, |output| data.execute(output))
        })
    }

    #[instrument(skip_all, fields(device = %self.device.serial))]
    async fn execute(mut self, mut output: Sender<AdbServerRecipeEvent>) {
        use AdbServerRecipeEvent as YieldValue;

        let mut child = match self.prepare_server().await {
            Ok(data) => data,
            Err(e) => {
                warn!(?e, "failed to prepare server");
                output.send(YieldValue::Error).await.ok();
                return;
            }
        };

        let mut stdin = child
            .stdin
            .take()
            .expect("prepare_server should be set stdin");

        if output.send(YieldValue::Connected).await.is_err() {
            return;
        }

        loop {
            select! {
                result = self.rx.changed() => {
                    if result.is_err() {
                        break;
                    }

                    let line = {
                        let data = &(*self.rx.borrow_and_update());
                        debug!(?data, "send data");

                        // for ignore init value.
                        if data.is_empty() {
                            continue;
                        }

                        format!("{data}\n")
                    };
                    if let Err(e) = stdin.write_all(line.as_bytes()).await {
                        warn!(?e);
                        child.kill().await.ok();
                        output.send(YieldValue::Error).await.ok();
                        return;
                    }
                    if let Err(e) = stdin.flush().await {
                        warn!(?e);
                        child.kill().await.ok();
                        output.send(YieldValue::Error).await.ok();
                        return;
                    }
                }
                status = child.wait() => {
                    debug!(?status, "child process exited");
                    output.send(YieldValue::Disconnected).await.ok();
                    return;
                }
            }
        }

        debug!("channel closed");
        child.kill().await.ok();
        output.send(YieldValue::Disconnected).await.ok();
    }

    async fn prepare_server(&self) -> Fallible<tokio::process::Child> {
        let temp_dir = tempdir().context("failed to prepare temporary directory")?;
        let server_path = temp_dir.path().join("android-commander-server");

        info!(?server_path);

        let server_file = File::create(&server_path)
            .await
            .context("failed to create temporary file")?;

        let server_bin = Asset::get("android-commander-server").context("failed to get asset")?;

        let mut buf = BufWriter::new(server_file);
        buf.write_all(&server_bin.data)
            .await
            .context("failed to write server data")?;
        buf.flush().await.context("failed to flush server data")?;

        let status = adb_command()
            .args([
                "-s",
                &self.device.serial,
                "push",
                server_path.to_str().unwrap(),
                deploy_server_path!(),
            ])
            .status()
            .await
            .context("failed to execute adb")?;

        if !status.success() {
            bail!("adb push failed with status: {:?}", status);
        }

        let child = adb_command()
            .args([
                "-s",
                &self.device.serial,
                "shell",
                concat!(
                    "CLASSPATH=",
                    deploy_server_path!(),
                    " app_process / jp.tinyport.androidcommander.server.MainKt"
                ),
            ])
            .stdin(std::process::Stdio::piped())
            .spawn()?;
        Ok(child)
    }
}

impl Hash for AdbServerRecipe {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::any::TypeId::of::<Self>().hash(state);
        self.device.serial.hash(state);
    }
}

fn find_adb_path() -> String {
    if cfg!(target_os = "macos") {
        // for terminal.
        if let Ok(data) =
            std::env::var("ANDROID_HOME").or_else(|_| std::env::var("ANDROID_SDK_ROOT"))
        {
            debug!(%data, "use adb from env");
            return format!("{data}/platform-tools/adb");
        }

        // for finder.
        if let Ok(data) = std::process::Command::new("whoami").output() {
            let whoami = String::from_utf8_lossy(&data.stdout);
            let whoami = whoami.trim();
            let path_adb = format!("/Users/{}/Library/Android/sdk/platform-tools/adb", whoami);
            if std::fs::metadata(&path_adb).is_ok() {
                info!(path_adb, "use adb from whoami");
                return path_adb;
            }
        }

        // for finder.
        if std::fs::metadata("/opt/android-sdk-macosx/platform-tools/adb").is_ok() {
            info!("use adb from /opt/android-sdk-macosx/platform-tools/adb");
            return "/opt/android-sdk-macosx/platform-tools/adb".into();
        }

        // terminal can refer adb if it is in PATH.
        "adb".into()
    } else {
        "adb".into()
    }
}

#[cfg(target_os = "windows")]
pub fn adb_command() -> TokioCommand {
    let mut cmd = TokioCommand::new(find_adb_path());
    // CREATE_NO_WINDOW
    cmd.creation_flags(0x08000000);
    cmd
}

#[cfg(not(target_os = "windows"))]
pub fn adb_command() -> TokioCommand {
    TokioCommand::new(find_adb_path())
}
