/*
 * Copyright 2022, 2025 sukawasatoru
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
use iced::futures::channel::mpsc::Sender;
use iced::futures::SinkExt;
use iced::stream::channel;
use iced::Subscription;
use std::io::prelude::*;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::fs::{create_dir_all, File};
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::watch::Receiver;

#[derive(Clone, Debug)]
pub enum AdbServerRecipeEvent {
    Connected,
    Disconnected,
    Error,
}

struct AdbServerRecipeType;

pub fn adb_server(
    device: Arc<AndroidDevice>,
    rx: Receiver<String>,
) -> Subscription<AdbServerRecipeEvent> {
    Subscription::run_with_id(
        std::any::TypeId::of::<AdbServerRecipeType>(),
        channel(3, move |output| execute(device, rx, output)),
    )
}

#[instrument(skip_all, fields(device = %device.serial))]
async fn execute(
    device: Arc<AndroidDevice>,
    mut rx: Receiver<String>,
    mut output: Sender<AdbServerRecipeEvent>,
) {
    use AdbServerRecipeEvent as YieldValue;

    let server_path = match tempdir() {
        Ok(data) => data.path().join("android-commander-server"),
        Err(e) => {
            warn!(?e, "failed to prepare temporary directory");
            output.send(YieldValue::Error).await.ok();
            return;
        }
    };

    info!(?server_path);

    if let Err(e) = create_dir_all(&server_path.parent().unwrap()).await {
        warn!(?e, "failed to create temporary directory");
        output.send(YieldValue::Error).await.ok();
        return;
    }

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

    if let Err(e) = adb_command()
        .args([
            "-s",
            &device.serial,
            "push",
            server_path.to_str().unwrap(),
            "/data/local/tmp/android-commander-server",
        ])
        .spawn()
    {
        warn!(?e, "failed to push server file");
        output.send(YieldValue::Error).await.ok();
        return;
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
        Ok(mut data) => match &data.stdin {
            Some(_) => {
                output.send(YieldValue::Connected).await.ok();
                data
            },
            None => {
                warn!("stdin not found");
                data.kill().ok();
                data.wait().ok();
                output.send(YieldValue::Error).await.ok();
                return;
            }
        },
        Err(e) => {
            warn!(?e);
            output.send(YieldValue::Error).await.ok();
            return;
        }
    };

    loop {
        if rx.changed().await.is_err() {
            break;
        }

        let data = rx.borrow().clone();
        debug!(?data, "send data");

        // for ignore init value.
        if data.is_empty() {
            continue;
        }

        let ret = writeln!(child.stdin.as_mut().unwrap(), "{}", data.as_str());
        if let Err(e) = ret {
            warn!(?e);
            child.kill().ok();
            child.wait().ok();
            output.send(YieldValue::Error).await.ok();
            return;
        }
    }

    debug!("channel closed");
    child.kill().ok();
    child.wait().ok();
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
pub fn adb_command() -> std::process::Command {
    use std::os::windows::process::CommandExt;

    let mut cmd = std::process::Command::new(find_adb_path());
    // CREATE_NO_WINDOW
    cmd.creation_flags(0x08000000);
    cmd
}

#[cfg(not(target_os = "windows"))]
pub fn adb_command() -> std::process::Command {
    std::process::Command::new(find_adb_path())
}
