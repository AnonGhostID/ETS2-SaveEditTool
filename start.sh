#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")" && pwd)"
BIN_PATH="$ROOT_DIR/ets2-tool/src-tauri/target/release/ets2-tool"

if [ ! -x "$BIN_PATH" ]; then
    "$ROOT_DIR/build-linux.sh"
fi

exec "$BIN_PATH" "$@"
