# Instructions for AI Agents

This guideline provides AI agents working on this codebase.

## Do and Don'ts

- Do: adb の Path を解決する場合はそれぞれの OS を考慮した `find_adb_path()` を使用する
- Do: commit message は英語で記述する
- Do: コードを変更したら reformat と lint と test を実行する

## Project Structure and Module Organization

- client/ Windows や macOS や Linux から Android 上で動作するサーバープログラムにコマンドを送信するリモコンアプリ
- client/src/widget_style.rs iced のウィジェットスタイルのカスタマイズを定義するモジュール
- server/ Android 上で実行する client から stdin でコマンドを受信し実行するサーバープログラム

## Build, Test, and Development Commands

- client build: cd client && make debug
- client unit test: cd client && cargo test
- client lint: cd client && cargo clippy
- client reformat code: cd client && cargo fmt
- server build: cd server && make
