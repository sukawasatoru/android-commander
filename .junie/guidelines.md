# Instructions for AI Agents

This guideline provides AI agents working on this codebase.

## Do and Don'ts

- Do: adb の Path を解決する場合はそれぞれの OS を考慮した `find_adb_path()` を使用する
- Do: commit message は英語で記述する
- Do: コードを変更したら reformat と lint と test を実行する
- Do: Windows 固有の依存やコードは `#[cfg(target_os = "windows")]` / `#[cfg(windows)]` で囲み、他 OS に影響を与えないようにする
- Don't: `Asset`（`client/src/data/asset.rs`）に自動生成出力以外のファイルを含めない

## Project Structure and Module Organization

- client/ Windows や macOS や Linux から Android 上で動作するサーバープログラムにコマンドを送信するリモコンアプリ
- client/assets/ ビルド時や CI で使用するファイル
- client/resources/ アプリが実行時に使用するリソースファイルの配置先（`Resource` 構造体で埋め込まれる）
- client/src/data/asset.rs `RustEmbed` で server のビルド出力 (`android-commander-server`) を埋め込む構造体。サーバーバイナリ専用。client/assets とは関係ない。
- client/src/data/resource.rs `RustEmbed` で `client/resources/` フォルダのリソース（SVG、PNG 等）を埋め込む構造体。アプリのリソースはここに配置する。
- client/src/widget_style.rs iced のウィジェットスタイルのカスタマイズを定義するモジュール
- server/ Android 上で実行する client から stdin でコマンドを受信し実行するサーバープログラム

## Build, Test, and Development Commands

- client build: cd client && make debug
- client unit test: cd client && cargo test
- client lint: cd client && cargo clippy
- client reformat code: cd client && cargo fmt
- server build: cd server && make
