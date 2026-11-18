# ETS2 / ATS Save-Edit Tool (Native GUI Edition)

A pure-Rust, cross-platform native desktop application (`eframe` / `egui`) for editing Euro Truck Simulator 2 and American Truck Simulator profiles, decrypting `game.sii` / `info.sii` saves, managing trucks, trailers, garages, AI drivers, mods, and running a live Career & Freight Dispatcher workspace—with **zero WebView or browser runtime**.

## Features

- **Save Editor:** Edit player money, experience (XP), level, all 6 driver skills (ADR, Long Distance, Heavy Cargo, Fragile, Urgent, Eco Driving), global/profile `config.cfg` settings (Traffic, Developer mode, Console, Convoy size, Double trailer parking), and Project Coaches XP.
- **Vehicles & Powertrain:** Repair and refuel player truck/trailer, edit license plates, odometer, fuel, and component wear, switch active truck/trailer safely with validation, and preview official powertrain (engine/transmission) swaps.
- **Garages & AI Drivers:** Inspect all garages, buy/upgrade/sell garages individually or in bulk, set headquarters, and generate/manage AI drivers.
- **Career, Telemetry & Freight Dispatcher:** Monitor live SCS shared-memory telemetry, generate freight market and universal jobs, inject jobs directly into your active quicksave, view trip logbooks, export CSV analytics, and manage your local VTC.
- **Mod Profile Manager & Conflict Analyzer:** Scan local and Steam Workshop mods, manage mod presets and sandboxes, and run standard/deep crash & mod conflict diagnostics.
- **Profile Tools, Save Health & Backups:** Clone and rename profiles, copy `controls.sii` and mod lists between profiles, export/import shared profile ZIP archives, run the Save Health Monitor with one-click auto-fixes, and preview/restore automatic save backups.

## Building & Running

### Linux (Native Desktop Binary)
Make sure standard GUI/X11/Wayland and GTK3 dialog headers are installed (e.g. `libgtk-3-dev` on Ubuntu/Debian/Codespaces):
```bash
./start.sh
```
Or build only:
```bash
./build-linux.sh
# Binary: ets2-tool/src-tauri/target/release/ets2-tool
```

### Cross-Compiling the Windows `.exe` from Linux
Install `mingw-w64` (`sudo apt install mingw-w64` or `sudo pacman -S mingw-w64-gcc`) and run:
```bash
./build-windows-from-linux.sh
# Binary: ets2-tool/src-tauri/target/x86_64-pc-windows-gnu/release/ets2-tool.exe
```

### Windows (Native `.exe`)
Run `start.bat` or compile with Cargo:
```powershell
cd ets2-tool\src-tauri
cargo run --release
```

## License

MIT License — see [LICENSE](LICENSE).
