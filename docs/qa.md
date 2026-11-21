# Phase 5 — QA checklist (user-executed)

## 1. Scope and status

- **All verification below is executed by the user.** No sub-agent ran `cargo build`, `cargo test`, `cargo clippy`, `cargo fmt`, or the native app. There is no agent-produced screenshot or test output; the results table in §7 is intentionally empty.
- **Post-checklist static review:** after this checklist was written, a static review found **5 compile errors and 6 save-safety defects**; all have been fixed in frontend code (commits `13791e7`, `33fa552`, `4523835`, `df3d05b`, `b9e9769`, `b03b8ad`, `3c0a00c`, `55449d2`, plus fleet/settings fixes). No builds or tests were run for those fixes. A focused re-review has completed and verified all 12 fix items with no remaining blocking defects. **The user's build remains the authoritative compile check**; record its exact output in §7.
- Static verification completed while writing this document (no compilation):
  - Branch: `ui-rebuild` (initial static snapshot at `d49965d`; the fixes above landed in later commits).
  - Backend gate `git diff --stat -- <backend dirs>`: **empty**; the committed range `git diff --stat main...HEAD -- <backend dirs>` is also empty.
  - `docs/parity.md`: **159 distinct `C-000`…`C-159` IDs, all 203 table rows ticked ✓**, including the referenced `U-014 U-015 U-016 U-018 U-019 U-020 U-037 U-038 U-043 U-048` rows. No unticked data row.
  - Old UI removed: no `*_tab.rs` remains under `ets2-tool/src-tauri/src/gui/`; `gui/mod.rs` contains only module declarations plus `pub use shell::AppShell;`.
  - Entry point swapped: `git diff main...HEAD -- ets2-tool/src-tauri/src/main.rs` contains exactly one hunk, `gui::Ets2NativeApp::new(cc, app)` → `gui::shell::AppShell::new(cc, app)` at line 176.
  - `gui/screens/mod.rs` declares all 30 snake_case `Route` variants from `docs/ui-spec.md` §1.2; `docs/backend-requests.md` contains no requests.
  - Fixture sources exist: `ets2-tool/src-tauri/test-fixtures/decrypt/plain_game.sii` and fallback `ets2-tool/src-tauri/codexHelp/gameInfo.txt`; `docs/screenshots/` exists.
- Static review is not proof of runtime correctness. The user's build/test output is authoritative; record it verbatim in §7 and report any defect instead of silently fixing backend code.
- Known documented limitation (not a QA failure by itself): exact focus restoration after a modal closes is not achievable with egui 0.29's public API; the shell uses best-effort focus capture (see `docs/components.md` §2). Record what is actually observed.

## 2. Exact commands (run from the repo root unless noted)

```bash
cargo fmt --check --manifest-path ets2-tool/src-tauri/Cargo.toml
cargo clippy --manifest-path ets2-tool/src-tauri/Cargo.toml --all-targets
cargo test --manifest-path ets2-tool/src-tauri/Cargo.toml
cargo build --release --manifest-path ets2-tool/src-tauri/Cargo.toml
```

i18n check — **cwd is `ets2-tool`** (the script resolves `../src-tauri/locales` relative to itself, so any cwd works, but use this one for consistency):

```bash
cd ets2-tool
node scripts/check_i18n.mjs
cd ..
```

Backend gate — exact command, must print nothing:

```bash
git diff --stat -- \
  ets2-tool/src-tauri/src/commands.rs ets2-tool/src-tauri/src/state.rs \
  ets2-tool/src-tauri/src/tauri_compat.rs ets2-tool/src-tauri/src/events.rs \
  ets2-tool/src-tauri/src/lib.rs ets2-tool/src-tauri/src/bin \
  ets2-tool/src-tauri/src/db ets2-tool/src-tauri/src/features \
  ets2-tool/src-tauri/src/models ets2-tool/src-tauri/src/shared \
  ets2-tool/src-tauri/src/xp \
  ets2-tool/src-tauri/Cargo.toml ets2-tool/src-tauri/Cargo.lock \
  ets2-tool/src-tauri/build.rs ets2-tool/src-tauri/data \
  ets2-tool/src/data ets2-tool/src-tauri/test-fixtures
```

Entry-point check — must show **only** the constructor swap at `main.rs:176`:

```bash
git diff main...HEAD -- ets2-tool/src-tauri/src/main.rs
```

Rules:

- **Do NOT run `ets2-tool/scripts/check_ui.mjs`.** It serves a deleted web/browser fixture and is stale for this native `eframe`/`egui` UI.
- Pre-existing backend warnings or test failures must be **recorded in §7 with file/line and exact text, not fixed**. Backend code is frozen.
- Release binary after a successful build: `ets2-tool/src-tauri/target/release/ets2-tool.exe`.

## 3. Copy-only fixture setup (PowerShell)

Never point the app at a real ETS2/ATS profile. Everything below lives in a fresh temp homedir. The skeleton satisfies `find_ets2_profiles` (a directory under `profiles/` containing `profile.sii` with `profile_name: "…"`) and `find_profile_saves` (an allowed save folder named `quicksave` containing `info.sii` with `name: "…"` and `game.sii`).

```powershell
# Run from the repo root in PowerShell.
$qa    = Join-Path $env:TEMP ('ets2-ui-qa-' + [guid]::NewGuid())
$src   = 'D:\QBT\GitData\ETS2-SaveEditTool\ets2-tool\src-tauri\test-fixtures\decrypt\plain_game.sii'
$cfgDir = Join-Path $env:APPDATA 'save-edit-tool'
$cfg    = Join-Path $cfgDir 'ets2_data_location.json'
$bak    = Join-Path $cfgDir ('ets2_data_location.json.qa-backup-' + [guid]::NewGuid().ToString('N'))

# 1. Temp homedir skeleton: <qa>\profiles\qa-copy\save\quicksave
$save = Join-Path $qa 'profiles\qa-copy\save\quicksave'
New-Item -ItemType Directory -Force -Path $save | Out-Null
Copy-Item -LiteralPath $src -Destination (Join-Path $save 'game.sii')

# 2. Minimal SII files (ASCII avoids BOM issues)
@'
SiiNunit {
    profile : profile.qa {
        profile_name: "QA Copy"
    }
}
'@ | Set-Content -LiteralPath (Join-Path $qa 'profiles\qa-copy\profile.sii') -Encoding ascii
@'
SiiNunit {
    save_info : save.info {
        name: "QA Save"
    }
}
'@ | Set-Content -LiteralPath (Join-Path $save 'info.sii') -Encoding ascii

# 3. Back up the real data-location config, then point the app at the temp root.
New-Item -ItemType Directory -Force -Path $cfgDir | Out-Null
if (Test-Path -LiteralPath $cfg) { Copy-Item -LiteralPath $cfg -Destination $bak -Force }
$json = '{"mode":"custom","customHomedir":"' + $qa.Replace('\','\\') + '"}'
[IO.File]::WriteAllText($cfg, $json, (New-Object Text.UTF8Encoding($false)))
Write-Host "QA root: $qa"; Write-Host "Config backup: $bak"
```

If `plain_game.sii` is ever unavailable, replace step 1's `$src` with
`'D:\QBT\GitData\ETS2-SaveEditTool\ets2-tool\src-tauri\codexHelp\gameInfo.txt'`
and copy it to `game.sii` the same way; keep `profile.sii` and `info.sii` unchanged.

**Restore after QA (mandatory, even if a check fails):**

```powershell
if (Test-Path -LiteralPath $bak) { Copy-Item -LiteralPath $bak -Destination $cfg -Force; Remove-Item -LiteralPath $bak }
else { Remove-Item -LiteralPath $cfg -Force }
Remove-Item -LiteralPath $qa -Recurse -Force
```

## 4. End-to-end flow walk (new Orbit Hub UI)

1. Launch: `& 'D:\QBT\GitData\ETS2-SaveEditTool\ets2-tool\src-tauri\target\release\ets2-tool.exe'`.
2. In the **Context Column** (left), select GAME `ETS2` if needed; confirm **QA Copy** appears in the profile selector and **QA Save** in the save selector. If either is missing, stop and re-check §3 (`profile.sii` / `info.sii` contents, folder name `quicksave`, config JSON path).
3. Edit one field of each type through the new UI; after each Apply, confirm the Success/Info toast and that the **Safety Spine › Staged** count increases:
   - **Integer:** Sector Map › Save Data › **Economy & Experience** › Fields → `Apply Money` (e.g. 1234567) and `Apply XP`.
   - **Text:** Vehicles › **Player Truck** › Plate & odometer → set plate `QA-1234` → confirm.
   - **Table selection:** Vehicles › **Fleet Inventory** › Trucks → select a row in the table (also check Trailers); or Operations › Dispatcher › Jobs → select a job row and open its detail.
   - **Toggle:** Vehicles › **Truck Switcher** › Apply → `create backup on switch` toggle.
   - **Slider:** Vehicles › **Player Truck** › Fuel & wear → fuel slider `0.75`, wear slider `0.25`, `Apply` → shared **confirm dialog** appears; confirm.
   - **Confirm dialog:** exercise `Cancel`/Esc once (no mutation) and `Confirm` once (mutation applied).
4. Press **Save staged edits** in the **Safety Spine** (far right, always visible). Confirm a success toast, Staged returns to 0, and the **Backups** gauge count grows by one. Open Safety › **Backups & Restore** to see the new backup row.
5. Quit and relaunch the release binary with the same config; re-open the edited tiles and confirm plate, money/XP, fuel/wear and the switcher toggle persisted.
6. Safety › **Backups & Restore** → select the just-created backup → Preview → Restore → shared confirm dialog → confirm. Re-open the edited fields and confirm values rolled back. Check the outcome text in the Safety Spine and in `Errors & Logs` / Issues drawer.
7. Stage a new edit (e.g. money), then:
   - switch PROFILE or SAVE in the Context Column → **unsaved prompt** appears with Save / Discard / Cancel; test `Cancel` (selector reverts), `Discard`, then `Save` at least once;
   - exit the app with staged edits → unsaved prompt appears again.
8. Every failure must surface in a toast **and** the Issues drawer (Errors or Validation tab), never silently. No save file outside `%TEMP%` may be touched.

## 5. UI checks

Window geometry — resize the running app with `user32!MoveWindow` and capture each size to `docs/screenshots/`:

```powershell
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class QaWin {
  [DllImport("user32.dll")] public static extern bool MoveWindow(IntPtr hWnd, int X, int Y, int w, int h, bool repaint);
}
"@
$p = Get-Process ets2-tool | Select-Object -First 1
[QaWin]::MoveWindow($p.MainWindowHandle, 40, 40, 960, 640, $true)   # then 1280x820, 1920x1080
```

Screenshot (run from repo root; repeat for each width):

```powershell
powershell -NoProfile -Command "Add-Type -AssemblyName System.Windows.Forms,System.Drawing; $b=[System.Windows.Forms.Screen]::PrimaryScreen.Bounds; $bmp=New-Object System.Drawing.Bitmap($b.Width,$b.Height); $g=[System.Drawing.Graphics]::FromImage($bmp); $g.CopyFromScreen($b.Location,[System.Drawing.Point]::Empty,$b.Size); $bmp.Save('docs/screenshots/qa-960x640.png',[System.Drawing.Imaging.ImageFormat]::Png)"
```

Check at **960x640, 1280x820, 1920x1080**:

- [ ] **Safety Spine stays visible** at every size (Backups, Staged, Save staged edits, Issues, Errors & Logs, Undo labelled, not icon-only); no clipped controls; no horizontal page scroll.
- [ ] At 960x640 the Context Column collapses to the compact row, Sector Map narrows with `[◀ All sectors]` intact, Detail Pane uses single-column forms, and the context popover is reachable.
- [ ] Focus order: Game → Profile → Save → Reload → sector/capability tiles → Detail Pane fields/tables/actions → Safety Spine; Shift+Tab reverses it; focus ring visible; focus survives tile swaps and drawer open/close. Record any lost focus.
- [ ] Confirm dialog and unsaved prompt trap Tab/Shift+Tab inside themselves; **Esc cancels/closes**; focus returns to a sensible control afterwards (best-effort restoration limitation from §1 noted).
- [ ] Status/error/validation text is legible against surface tokens in all four regions; record any low-contrast text.
- [ ] All seven Sector Map tiles and `[All sectors]` work; every one of the 30 routes below opens without panic and shows its default sub-view:
  - Overview: [ ] `overview_dashboard` [ ] `overview_quick_actions` [ ] `overview_recent_activity`
  - Save Data: [ ] `save_data_economy` [ ] `save_data_coaches` [ ] `save_data_config` [ ] `save_data_custom_keys`
  - Vehicles: [ ] `vehicles_player_truck` [ ] `vehicles_player_trailer` [ ] `vehicles_truck_switcher` [ ] `vehicles_trailer_switcher` [ ] `vehicles_powertrain` [ ] `vehicles_fleet_inventory`
  - Operations: [ ] `operations_garages` [ ] `operations_ai_drivers` [ ] `operations_live_telemetry` [ ] `operations_dispatcher` [ ] `operations_logbook_analytics` [ ] `operations_vtc_account`
  - Library: [ ] `library_mod_manager` [ ] `library_presets` [ ] `library_sandboxes` [ ] `library_conflict_analyzer`
  - Safety: [ ] `safety_save_health` [ ] `safety_backups_restore` [ ] `safety_profile_clone_rename` [ ] `safety_profile_sharing`
  - System: [ ] `system_paths_data` [ ] `system_logs_diagnostics` [ ] `system_about`
- [ ] Deep links work: Dashboard stat tiles, Quick Actions (Scan Save Health, Generate Market Jobs), Safety Spine Backups, Issues drawer → full log view.

## 6. Parity confirmation

- [ ] `docs/parity.md` contains **159/159** distinct `C-001`…`C-159` rows, all ticked ✓ (203 ticked rows overall), plus the `U-` rows listed in §1. `docs/ui-spec.md` maps all 209 audit rows with **zero `NO HOME`** entries.
- [ ] Spot-check at least five rows across different sectors (e.g. C-003 Save, C-047 truck switch apply, C-114 sandbox apply, C-134 restore, C-159 clear logs) in the running app: the listed new location exists and the capability works.
- [ ] Report any gap as `parity row ID — expected location — observed behavior` in §7. Do not edit parity ticks to hide a gap.
- [ ] `docs/backend-requests.md` has no entries; if the user finds a missing backend capability, add the exact need there rather than working around it in UI code.

## 7. Results table (fill in during user verification)

| # | Check / exact command | Result (pass/fail) | Notes / evidence (output, file:line, screenshot path) |
|---|---|---|---|
| 1 | `cargo fmt --check --manifest-path ets2-tool/src-tauri/Cargo.toml` |  |  |
| 2 | `cargo clippy --manifest-path ets2-tool/src-tauri/Cargo.toml --all-targets` |  |  |
| 3 | `cargo test --manifest-path ets2-tool/src-tauri/Cargo.toml` |  |  |
| 4 | `cargo build --release --manifest-path ets2-tool/src-tauri/Cargo.toml` |  |  |
| 5 | `cd ets2-tool && node scripts/check_i18n.mjs` |  |  |
| 6 | Backend gate (`git diff --stat -- <backend dirs>`, §2) |  |  |
| 7 | `git diff main...HEAD -- ets2-tool/src-tauri/src/main.rs` (only constructor line) |  |  |
| 8 | Copy-only fixture setup + restore (§3), QA Copy / QA Save visible |  |  |
| 9 | Field types: integer, text, table selection, toggle, slider, confirm dialog (§4.3) |  |  |
| 10 | Save staged edits → toast, Staged 0, backup count grows (§4.4) |  |  |
| 11 | Relaunch → edits persisted (§4.5) |  |  |
| 12 | Backup restore → values rolled back, outcome visible (§4.6) |  |  |
| 13 | Unsaved prompt on profile/save switch and exit (§4.7) |  |  |
| 14 | 960x640 layout + `docs/screenshots/qa-960x640.png` |  |  |
| 15 | 1280x820 layout + `docs/screenshots/qa-1280x820.png` |  |  |
| 16 | 1920x1080 layout + `docs/screenshots/qa-1920x1080.png` |  |  |
| 17 | Focus order, Shift+Tab, modal trap, Esc, contrast (§5) |  |  |
| 18 | 7 sectors + 30 routes + deep links reachable (§5) |  |  |
| 19 | Parity 159/159 + U rows + spot-check (§6) |  |  |
| 20 | Final config restore and temp fixture deletion verified (§3) |  |  |
