#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/ets2-tool/src-tauri"

if ! rustup target list --installed | grep -q "x86_64-pc-windows-gnu"; then
    echo "[ETS2-SaveEditTool Native] Installing Rust target x86_64-pc-windows-gnu..."
    rustup target add x86_64-pc-windows-gnu
fi

if ! command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1; then
    echo "Error: x86_64-w64-mingw32-gcc not found. Install mingw-w64 first (e.g., sudo apt install mingw-w64 or sudo pacman -S mingw-w64-gcc)."
    exit 1
fi

echo "[ETS2-SaveEditTool Native] Cross-compiling standalone Windows .exe from Linux..."
cargo build --release --target x86_64-pc-windows-gnu
echo "[ETS2-SaveEditTool Native] Done: $(pwd)/target/x86_64-pc-windows-gnu/release/ets2-tool.exe"
