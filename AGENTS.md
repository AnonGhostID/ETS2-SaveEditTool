# Agents.md
**Engineering & Architecture Guide for ETS2 / ATS Save-Edit Tool (Native GUI Edition)**

## 1. Architecture Overview

- **GUI Framework**: Pure-Rust [`eframe` / `egui`](https://github.com/emilk/egui) (`0.29`, `glow` renderer) — zero WebView, zero Node.js/npm frontend.
- **Native File Dialogs**: [`rfd`](https://crates.io/crates/rfd) (`0.15`) for cross-platform Linux (GTK/XDG portal) and Windows (`IFileDialog`) pickers.
- **Async & Database Runtime**: `tokio` multi-threaded runtime + `sqlx` / `rusqlite` bundled SQLite + `decrypt_truck` for native ETS2/ATS SII save decryption.
- **Supported Platforms**: Native Linux (`x86_64-unknown-linux-gnu`) and Native Windows (`x86_64-pc-windows-msvc` / `x86_64-pc-windows-gnu`).

## 2. Directory Structure

```
ETS2-SaveEditTool/
├── start.sh                        # Build (if needed) & launch native Linux binary
├── start.bat                       # Build (if needed) & launch native Windows .exe
├── build-linux.sh                  # Compile native Linux release binary
├── build-windows-from-linux.sh     # Cross-compile standalone Windows .exe from Linux (MinGW)
└── ets2-tool/
    ├── data/                       # ETS2/ATS datasets & powertrain catalogs
    ├── native/                     # SCS telemetry C++ shared-memory plugin source
    ├── sdk/                        # SCS SDK headers
    ├── src/data/level-table.json   # Embedded XP level table
    └── src-tauri/                  # Rust crate root (`ets2-tool`)
        ├── Cargo.toml
        ├── locales/                # JSON translation files (de, en, es, fr, it, pl)
        ├── resources/plugins/      # Bundled telemetry plugin DLL
        └── src/
            ├── main.rs             # Backend initialization & eframe::run_native entrypoint
            ├── lib.rs              # Shared library exports (for ets2_data_builder)
            ├── tauri_compat.rs     # Native AppHandle, State<T>, Emitter, PathResolver, and rfd DialogExt
            ├── state.rs            # Managed application states
            ├── commands.rs         # Top-level dataset, SQLite, and snapshot commands
            ├── features/           # Domain modules (save_editor, vehicles, garages, career, mods, etc.)
            └── gui/                # Native egui desktop views
                ├── mod.rs              # Top bar (Game/Profile/Save/Language/Undo) & Workspace router
                ├── save_editor_tab.rs  # Money, XP, Level, Skills, Config.cfg, Coaches XP
                ├── vehicles_tab.rs     # Truck/Trailer repair, plates, odometer, switcher, powertrain
                ├── garages_tab.rs      # Garages & AI driver management
                ├── career_tab.rs       # Live SCS telemetry, Freight Dispatcher, Logbook, VTC
                ├── mods_tab.rs         # Mod Profile Manager, Presets, Sandboxes, Conflict Analyzer
                ├── profile_tools_tab.rs# Profile Clone/Rename, ZIP Sharing, Save Health, Backups
                └── settings_tab.rs     # Homedir paths, SQLite datasets, Diagnostic logs
```

## 3. Build & Run Commands

From the repository root:

- **Linux (Native):**
  ```bash
  ./start.sh
  # or manually:
  cd ets2-tool/src-tauri && cargo run --release
  ```
- **Cross-compile Windows `.exe` from Linux:**
  ```bash
  ./build-windows-from-linux.sh
  # Output: ets2-tool/src-tauri/target/x86_64-pc-windows-gnu/release/ets2-tool.exe
  ```
- **Windows (Native):**
  ```bat
  start.bat
  ```

## 4. Adding or Modifying Features

1. **Backend Logic**: Implement service/command functions in `ets2-tool/src-tauri/src/features/<feature>/`. Access shared state via `State<'_, T>` or `AppHandle` from `crate::tauri`.
2. **Native GUI Wiring**: Add UI controls in the corresponding tab under `ets2-tool/src-tauri/src/gui/` and invoke the Rust functions directly (use `crate::tauri::async_runtime::block_on(...)` when calling `async fn` handlers from UI callbacks).
3. **Localization**: Add translation keys to `ets2-tool/src-tauri/locales/*.json` and query via `crate::features::language::translator::t("key")`.