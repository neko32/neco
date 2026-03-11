#!/usr/bin/env bash
# neco ローカル実行用（Linux / WSL bash）
# 使用例: ./scripts_local/neco.sh "カレントディレクトリのファイル一覧"
set -e
cd "$(dirname "$0")/../app"
if [ -z "$*" ]; then
  echo "コマンドの説明を引数で指定してください。例: ./scripts_local/neco.sh \"ファイル一覧を表示\"" >&2
  exit 1
fi
cargo run -q --bin neco -- "$@"
