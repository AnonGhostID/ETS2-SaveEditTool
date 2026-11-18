#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/ets2-tool/src-tauri"
echo "[ETS2-SaveEditTool Native] Building native Linux release binary (eframe/egui, zero WebView)..."
cargo build --release
echo "[ETS2-SaveEditTool Native] Done: $(pwd)/target/release/ets2-tool"
