# Guidelines

## コーディング規約

- Clippy の警告は可能な限り解消する

## adb 関連

- `find_adb_path()` で OS ごとに adb のパスを解決する
- 子プロセスの終了検出には `tokio::select!` を使用して、コマンド受信と `child.wait()` を同時に監視する
- `adb push` など完了を待つ必要があるコマンドは `.status().await` を使用する
- バックグラウンドで実行するコマンドは `.spawn()` を使用する
