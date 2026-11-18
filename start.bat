@echo off
setlocal
cd /d "%~dp0\ets2-tool\src-tauri"

if exist "target\release\ets2-tool.exe" (
    start "" "target\release\ets2-tool.exe"
    exit /b 0
)

if exist "target\x86_64-pc-windows-gnu\release\ets2-tool.exe" (
    start "" "target\x86_64-pc-windows-gnu\release\ets2-tool.exe"
    exit /b 0
)

echo [ETS2-SaveEditTool Native] Building native Windows release binary...
cargo build --release
if errorlevel 1 exit /b 1
start "" "target\release\ets2-tool.exe"
