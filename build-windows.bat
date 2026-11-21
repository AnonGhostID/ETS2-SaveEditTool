@echo off
setlocal
cd /d "%~dp0\ets2-tool\src-tauri"

echo [ETS2-SaveEditTool Native] Building native Windows release binary (eframe/egui, zero WebView)...
cargo build --release
if errorlevel 1 exit /b 1
echo [ETS2-SaveEditTool Native] Done: %cd%\target\release\ets2-tool.exe
