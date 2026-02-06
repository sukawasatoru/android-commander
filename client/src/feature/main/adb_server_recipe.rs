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
use tokio::sync::watch::Receiver;

#[derive(Clone, Debug)]
pub enum AdbServerRecipeEvent {
    Connected,
    Disconnected,
    Error,
}

struct AdbServerRecipe {
    device: Arc<AndroidDevice>,
    rx: Receiver<String>,
}

impl Hash for AdbServerRecipe {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::any::TypeId::of::<Self>().hash(state);
        self.device.serial.hash(state);
    }
}

pub fn adb_server(
    device: Arc<AndroidDevice>,
    rx: Receiver<String>,
) -> Subscription<AdbServerRecipeEvent> {
    Subscription::run_with(AdbServerRecipe { device, rx }, |data| {
        let device = data.device.clone();
        let rx = data.rx.clone();
        channel(3, |output| execute(device, rx, output))
    })
}

#[instrument(skip_all, fields(device = %device.serial))]
async fn execute(
    device: Arc<AndroidDevice>,
    mut rx: Receiver<String>,
    mut output: Sender<AdbServerRecipeEvent>,
) {
    use AdbServerRecipeEvent as YieldValue;

    let temp_dir = match tempdir() {
        Ok(data) => data,
        Err(e) => {
            warn!(?e, "failed to prepare temporary directory");
            output.send(YieldValue::Error).await.ok();
            return;
        }
    };
    let server_path = temp_dir.path().join("android-commander-server");

    info!(?server_path);

    let server_file = match File::create(&server_path).await {
        Ok(data) => data,
        Err(e) => {
            warn!(?e, "failed to create temporary file");
            output.send(YieldValue::Error).await.ok();
            return;
        }
    };

    let server_bin = match Asset::get("android-commander-server") {
        Some(data) => data,
        None => {
            warn!("failed to get asset");
            output.send(YieldValue::Error).await.ok();
            return;
        }
    };

    let mut buf = BufWriter::new(server_file);
    if let Err(e) = buf.write_all(&server_bin.data).await {
        warn!(?e, "failed to write server data");
        output.send(YieldValue::Error).await.ok();
        return;
    }

    if let Err(e) = buf.flush().await {
        warn!(?e, "failed to flush server data");
        output.send(YieldValue::Error).await.ok();
        return;
    }

    match adb_command()
        .args([
            "-s",
            &device.serial,
            "push",
            server_path.to_str().unwrap(),
            "/data/local/tmp/android-commander-server",
        ])
        .status()
        .await
    {
        Ok(status) if status.success() => {
            debug!("server file pushed successfully");
        }
        Ok(status) => {
            warn!(?status, "failed to execute adb push");
            output.send(YieldValue::Error).await.ok();
            return;
        }
        Err(e) => {
            warn!(?e, "failed to execute adb");
            output.send(YieldValue::Error).await.ok();
            return;
        }
    }

    let mut child = match adb_command()
        .args([
            "-s",
            &device.serial,
            "shell",
            "CLASSPATH=/data/local/tmp/android-commander-server app_process / jp.tinyport.androidcommander.server.MainKt"
        ])
        .stdin(std::process::Stdio::piped())
        .spawn()
    {
        Ok(data) => data,
        Err(e) => {
            warn!(?e);
            output.send(YieldValue::Error).await.ok();
            return;
        }
    };

    let mut stdin = match child.stdin.take() {
        Some(data) => data,
        None => {
            warn!("stdin not found");
            output.send(YieldValue::Error).await.ok();
            return;
        }
    };

    output.send(YieldValue::Connected).await.ok();

    loop {
        tokio::select! {
            result = rx.changed() => {
                if result.is_err() {
                    break;
                }

                let data = rx.borrow_and_update().clone();
                debug!(?data, "send data");

                // for ignore init value.
                if data.is_empty() {
                    continue;
                }

                let line = format!("{}\n", data.as_str());
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

pub fn find_adb_path() -> String {
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
    use std::os::windows::process::CommandExt;

    let mut cmd = TokioCommand::new(find_adb_path());
    // CREATE_NO_WINDOW
    cmd.creation_flags(0x08000000);
    cmd
}

#[cfg(not(target_os = "windows"))]
pub fn adb_command() -> TokioCommand {
    TokioCommand::new(find_adb_path())
}
