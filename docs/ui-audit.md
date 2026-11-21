# Native UI Audit — ETS2 / ATS Save-Edit Tool (Phase 0)

Read-only audit of the current native GUI and its backend call surface. Scope: `AGENTS.md`, `README.md`, `ets2-tool/src-tauri/Cargo.toml`, `src/main.rs`, all 11 files under `src/gui/`, `src/commands.rs`, all `src/features/*/commands.rs` reachable from the UI, `src/state.rs`, `src/shared/save_session.rs`, `src-tauri/locales/en.json`. No source file was modified. All paths below are relative to `ets2-tool/src-tauri/` unless stated otherwise.

---

## 1. Detected stack (with evidence)

**Result: native Rust desktop app — `eframe`/`egui` 0.29 with the `glow` renderer and `rfd` native dialogs. There is no WebView, no browser runtime, and no Node/npm frontend.**

Evidence:

| Claim | Evidence |
|---|---|
| `eframe` 0.29, `egui` 0.29, `egui_extras` 0.29, `glow`, `rfd` 0.15 | `Cargo.toml` `[dependencies]`: `eframe = { version = "0.29", default-features = false, features = ["default_fonts", "glow", "persistence"] }`, `egui = "0.29"`, `egui_extras = "0.29"`, `rfd = "0.15"`. No `tauri`, `wry`, `webview`, `dioxus`, `iced`, or JS bundler crate. |
| Native entry point | `src/main.rs:173-177`: `eframe::run_native("Save-Edit Tool", native_options, Box::new(move \|cc\| Ok(Box::new(gui::Ets2NativeApp::new(cc, app)))))`; `NativeOptions` sets viewport title, 1280×820 inner size and 960×640 minimum. `main.rs:26-153` `initialize_backend()` only wires state/SQLite/telemetry/services. |
| "src-tauri" is a legacy folder name | `AGENTS.md` §1: "Pure-Rust eframe/egui (0.29, glow renderer) — zero WebView, zero Node.js/npm frontend"; `README.md`: "zero WebView or browser runtime". `src/tauri_compat.rs` supplies `AppHandle`, `State<T>`, emitter and `rfd` dialog shims; it is not a Tauri runtime. |
| Async boundary is explicit blocking | `src/gui/` contains **47** `async_runtime::block_on(` call sites (career_tab 2, garages_tab 4, mod.rs 13, mods_tab 2, profile_tools_tab 4, settings_tab 3, vehicles_tab 19). Sync backend functions are called directly. `BackgroundLoad<T>` (`gui/mod.rs:141-183`) is the only thread-based loader. |
| Persistence / dialogs | `tokio`, `sqlx`/`rusqlite` bundled SQLite and `rfd` are used by backend commands; the GUI never opens a browser. |
| Localization is present but not wired to the UI | `locales/en.json` (master; top-level groups: `coaches, app, tabs, buttons, profile, game, hub, launcher, update, career_mode, career, editor, status_text, toasts, tools, errors, modals, garage_manager, label, ui, diagnostics, ui_refinement, user_log_report, simple_logging`); six other locale files exist (de, es, fr, it, pl, pt-BR). A grep for `translator::t(` / `t("…")` in `src/gui/` returns **no** UI calls: every visible string is hardcoded English. The top bar shows only a saved-language indicator with an explanatory tooltip (`gui/mod.rs:1288-1306`). |
| Stale web tool | `ets2-tool/scripts/check_ui.mjs` targets a deleted `src/index.html` fixture and is not a gate for this native UI. |

---

## 2. Inventory of every screen, tab, dialog, modal and flow

### 2.1 Shell and navigation (`gui/mod.rs`, `src/main.rs`)

| Screen / element | File : rendering symbol | Contents |
|---|---|---|
| Process startup | `src/main.rs::main`, `::initialize_backend` | Panic hook, diagnostic log runtime, SQLite init, state registration, translator, career/auth/VTC/ETS2-data migration; `save_session::set_staging_enabled(true)` at `main.rs:159`; constructs the render root. |
| App shell / state container | `gui/mod.rs::Ets2NativeApp` (`:205`), `::new` (`:386`), `eframe::App::update` (`:1891`) | Owns all navigation, selection, editor caches and toasts; renders top panel, sidebar, central panel, toasts, prompt, exit handling. |
| Top command bar | `gui/mod.rs::render_top_bar` (`:1066`) | Brand badge, GAME combo, PROFILE combo, SAVE combo, Reload, Undo, dirty label + Save, LANG indicator. |
| Left navigation rail | `gui/mod.rs::render_sidebar` (`:1327`) | Flat two-level rail: sections WORKSPACE (Dashboard, Save Editor, Garages & Drivers), VEHICLES (5 sub-tabs), CAREER (4 sub-tabs + Coaches), LIBRARY (3 sub-tabs), SAFETY (4 sub-tabs), Settings; version footer. |
| Toast surface | `gui/mod.rs::update` (`:1891`, toast block `:1927-1962`) | Top-right stacked Area; info TTL 5 s, error 10 s; max 4; manual × dismiss; repaint timer. `Toast` struct at `:132`. |
| Unsaved-changes modal | `gui/mod.rs::render_unsaved_prompt` (`:694`), `PendingTopBarAction` (`:193`), `perform_top_bar_action` (`:637`), `resolve_unsaved_prompt` (`:674`) | Centered non-resizable window: Save / Discard / Cancel. Deferred actions: SwitchSave, SwitchProfile, SwitchGame{new,old}, Reload, Exit. Cancel restores the prior game value. |
| Exit guard | `gui/mod.rs::update` (`:1894-1904`), `close_after_prompt` field | `CancelClose` when dirty → prompt → `ViewportCommand::Close`. |
| Background event drain | `gui/mod.rs::drain_background_events` (`:1048`) | Consumes `NativeEvent`s from `app.subscribe_events()`; caps recent activity at 50; converts `career://plugin_install_error` into an error toast. |
| Generic background loader | `gui/mod.rs::BackgroundLoad<T>` (`:141`) | `start`/`poll`/`is_loading` via `std::sync::mpsc`; used by garages and mods screens. |
| Dashboard / Launcher | `gui/mod.rs::render_launcher_tab` (`:1435`) | Profile/save header card; 8 clickable stat tiles → Save Editor / Vehicles / Garages; Current Setup card (truck, fuel/wear bars, trailer, HQ); QUICK ACTIONS (Repair + Refuel, Max All Skills, Scan Save Health, Generate Market Jobs); OPEN A SCREEN action cards; RECENT ACTIVITY (last 5 events); staging reminder footer. |
| Router | `gui/mod.rs::update` central panel (`:1975-1991`) | `WorkspaceTab` (9 variants, `:85`) dispatch; four sub-tab enums (`:98`, `:107`, `:115`, `:122`) are mutated by the sidebar, not by the screens. |

### 2.2 Feature screens and their sub-flows

| Screen | File : rendering symbols | Sub-screens / flows |
|---|---|---|
| Save Editor | `gui/save_editor_tab.rs::render` (`:12`) | Player Economy & Experience (money, XP, level, skill sliders, custom reset), Project Coaches / Bus Career link, config.cfg + save config keys (Traffic, Developer, Console, Convoy, Parking Doubles), arbitrary key/value rows for both configs, Refresh Save Data. |
| Vehicles | `gui/vehicles_tab.rs::render` (`:38`) → `render_player_editor` (`:80`), `render_truck_switcher` (`:315`), `render_trailer_switcher` (`:524`), `render_powertrain_swapper` (`:716`), `render_fleet_inventory` (`:839`), `fleet_sort_header` (`:1112`) | Repair/refuel, plate, odometer, fuel & wear sliders; truck/trailer switch = initialize session → searchable table → preview (warnings, `can_apply`) → apply (optional persistent backup); official powertrain catalog → engine/transmission preview; searchable fleet tables with sorting. |
| Garages & AI Drivers | `gui/garages_tab.rs::render` (`:44`), `render_garage_detail` (`:462`), `render_garage_confirm` (`:623`), `run_garage_confirm` (`:665`), `run_garage_command` (`:781`), `start_garages_load` (`:858`) | Garage table with filters/sorting, buy/upgrade/sell/set-HQ, bulk buy-all and sell-empty, AI driver generation (count, assign trucks), driver dismissal, random resource assignment, per-garage warnings. |
| Career | `gui/career_tab.rs::render` (`:33`), `refresh_career_data` (`:78`), `render_live_telemetry` (`:99`), `render_dispatcher` (`:188`), `render_job_detail` (`:400`), `render_logbook_and_analytics` (`:568`), `render_vtc_and_account` (`:646`) | Live SCS telemetry (500 ms repaint), dispatcher market/active jobs with search & sort, generate standard/universal jobs, accept/cancel, assign + prepare + write to quicksave, cleanup expired, analytics scan/CSV export, logbook/job stats, auth sign in/out, VTC company creation. |
| Coaches — Bus Career | `gui/coaches_tab.rs::render` (`:20`), `reload` (`:367`), `apply_bus_xp` (`:378`) | Bus level/XP editor, bus statistics, job history, searchable/filterable discovered bus stops (`CoachesStopFilter` at `:13`), name-formatting helpers. Uses the only in-screen `components::sub_tabs` call (`:303`). |
| Mods | `gui/mods_tab.rs::render` (`:26`), `render_mod_manager` (`:179`), `render_presets_and_sandboxes` (`:256`), `render_conflict_analyzer` (`:462`), `start_mods_scan` (`:111`), `render_mod_confirm` (`:124`), `reload_preset_list` (`:172`) | Light/deep mod scan, manual Workshop directory, Steam console/page links, preset create/compare/export/import/delete, sandbox create/add-mod/apply/delete, standard & deep conflict diagnostics with report export. |
| Profile Tools | `gui/profile_tools_tab.rs::render` (`:20`), `render_clone_and_rename` (`:50`), `render_profile_sharing` (`:143`), `render_save_health` (`:259`), `render_backups_and_restore` (`:316`), `render_backup_confirm` (`:384`) | Clone (backup, hex/text replacement options), validate clone target, rename, copy controls / mod list to target profile, ZIP export/import with archive preview, save-health scan + one-click fixes, backup list/preview/restore. |
| Settings | `gui/settings_tab.rs::render` (`:15`) | Homedir/data-location status with custom path chooser/reset/mode, SQLite + ETS2 dataset import, user log status/list, diagnostic preview, export user logs / support bundle / log file, clear user logs. |
| Shared widgets | `gui/components.rs` — `UiExt::card` (`:11`), `page_header` (`:25`), `section_label` (`:36`), `nav_section` (`:45`), `nav_item` (`:52`), `primary_button` (`:91`), `sub_tabs` (`:101`), `stat_tile` (`:125`), `clickable_stat_tile` (`:145`), `action_card` (`:173`), `game_badge` (`:199`), `empty_state` (`:214`), `status_dot` (`:232`), `confirm_dialog` (`:240`) | Single shared vocabulary; no backend access. |
| Theme tokens | `gui/theme.rs` — constants (`:11-33`), `apply` (`:36`), `visuals` (`:68`), `widget` (`:120`) | Global `Style`, palette, widget visuals; no layout logic. |

### 2.3 Dialogs and modals (complete list)

1. Unsaved changes — `gui/mod.rs::render_unsaved_prompt` (hand-rolled `egui::Window`, `:708`).
2. Generic destructive confirmation — `gui/components.rs::confirm_dialog` (`:240`), instantiated by:
   - `gui/garages_tab.rs::render_garage_confirm` (`:655`) — SellGarage, SellEmpty, BuyAll, DismissDriver.
   - `gui/mods_tab.rs::render_mod_confirm` (`:138`) — DeletePreset, DeleteSandbox.
   - `gui/profile_tools_tab.rs::render_backup_confirm` (`:391`) — RestoreBackup.
3. Toasts — not modal, but the only other blocking-visibility surface (`gui/mod.rs::update`).

### 2.4 Cross-screen flows

| Flow | Entry → steps → exit |
|---|---|
| Game/profile/save selection | `render_top_bar` combos → `select_profile` (`:815`) → `find_profile_saves` → `select_save` (`:860`) → `load_profile` + `set_current_save` → `reload_active_save_data` (`:890`) → populates Save Editor, Vehicles, Coaches caches. |
| Staged edit → Save | any `edit_*` writer stages into `save_session` → `is_dirty()` → top-bar `Save` → `flush_pending_changes` (`:581`) → `save_pending_changes` (backup + write) → reload caches, refresh garages, invalidate switchers → toast. |
| Unsaved switch / exit | combo/Reload/close → `is_dirty()` → `PendingTopBarAction` → prompt → Save (abort if still dirty) or Discard → deferred action; Cancel leaves state untouched. |
| Truck/trailer switch | `initialize_*_session` → searchable list → `preview_*` (`expected_file_hash`, warnings) → `apply_*` (optional persistent backup, `create_backup_on_switch`) → `invalidate_switcher_state`. |
| Backup restore | `list_active_save_backups` → `preview_backup_restore` → `render_backup_confirm` → `restore_backup(backup_id, confirmed=true, …)` → reload. |
| Save health | `get_active_save_health` → report → `apply_save_health_fix(fix_id, confirmed, …)`. Reachable from Profile Tools and from Dashboard quick action (which navigates to Profile Tools/Save Health). |
| Dispatcher write | market job → accept → `dispatcher_assign_and_prepare_and_write(job_id, auto_write, app, …)` → reports `write_applied` plus staged-vs-written status. |
| Garage mutation | load (`BackgroundLoad`) → detail → confirm (when destructive) → `run_garage_command` → reload list + drivers + toast; optimistic concurrency via `expected_save_hash`. |
| Mod scan | `start_mods_scan(app, deep)` → `BackgroundLoad<Result<ModProfileManagerState,String>>` → manager/presets/analyzer; diagnostics are blocking `block_on` on the UI thread. |
| Log export / support bundle | Settings buttons → `export_user_logs`, `export_logs_bundle`, `export_log_file` (async) → path toast. |

---

## 3. Backend call surface — every call site

Notation: `State<P>` = `State<'_, AppProfileState>`, `State<C>` = `State<'_, ProfileCache>`, `State<D>` = `State<'_, DecryptCache>`, `State<A>` = `State<'_, AppState>`, `State<CS>` = `State<'_, CareerState>`, `State<Auth>` = `State<'_, AuthState>`, `State<DS>` = `State<'_, DiagnosticsState>`, `State<ETS>` = `State<'_, EtsDbState>`. All `State` values come from `Ets2NativeApp::{profile_state, profile_cache, decrypt_cache, app_state, career_state, auth_state, ets_db_state, diagnostics_state}` (`gui/mod.rs:740-776`). **[BO]** marks a `crate::tauri::async_runtime::block_on(...)` call site. There are 47 such sites; every one appears in a row below.

### 3.1 Shell / router (`gui/mod.rs`)

| # | Function (module) | Arguments (names : types) | Return shape | Calling screen (file:line) |
|---|---|---|---|---|
| 1 | `get_available_languages_command` (`features/language/commands.rs:24`) | `app: AppHandle` | `Vec<LanguageInfo>` | Shell `::new` — mod.rs:389 |
| 2 | `get_current_language_command` (`features/language/commands.rs:51`) | `app: AppHandle` | `String` | Shell `::new` — mod.rs:390 |
| 3 | `save_pending_changes` (`features/save_editor/commands.rs:196`) | `profile_state: State<P>, profile_cache: State<C>, decrypt_cache: State<D>` | `Result<Vec<String>, String>` (written paths) | Shell `flush_pending_changes` — mod.rs:586 |
| 4 | `set_selected_game` (`features/profile_manager/commands.rs:36`) | `game: String, state: State<P>` | `Result<String, String>` | Shell top bar / refresh / deferred action — mod.rs:646, 778, 1133 |
| 5 | `discard_pending_changes` (`features/save_editor/commands.rs:298`) | `profile_cache: State<C>, decrypt_cache: State<D>` | `Result<(), String>` | Shell `resolve_unsaved_prompt` — mod.rs:684 |
| 6 | `find_ets2_profiles` (`features/profile_manager/commands.rs:247`) | `state: State<P>` | `Vec<ProfileInfo>` | Shell `refresh_profiles_and_saves` — mod.rs:781 |
| 7 | `read_last_profile` (`features/profile_manager/commands.rs:79`) | `app: AppHandle` | `Result<Option<String>, String>` | Shell `refresh_profiles_and_saves` — mod.rs:783 |
| 8 | `save_last_profile` (`features/profile_manager/commands.rs:71`) | `app: AppHandle, profile_path: String` | `Result<String, String>` | Shell `select_profile` — mod.rs:820 |
| 9 | `set_active_profile` (`features/profile_manager/commands.rs:94`) | `profile_path: String, profile_state: State<P>, cache: State<D>, profile_cache: State<C>` | `Result<(), String>` | Shell `select_profile` — mod.rs:823 |
| 10 | `find_profile_saves` **async** (`features/profile_manager/commands.rs:153`) | `profile_path: String` | `Result<Vec<SaveInfo>, String>` | Shell `select_profile` — mod.rs:836 **[BO]** |
| 11 | `load_profile` (`features/profile_manager/commands.rs:378`) | `profile_path: String, save_path: Option<String>, profile_state: State<P>, cache: State<D>, profile_cache: State<C>` | `Result<String, String>` | Shell `select_save` — mod.rs:864 |
| 12 | `set_current_save` (`features/profile_manager/commands.rs:122`) | `save_path: String, state: State<P>, cache: State<D>, profile_cache: State<C>` | `Result<(), String>` | Shell `select_save` — mod.rs:875 |
| 13 | `get_undo_status` (`features/save_editor/commands.rs:776`) | `profile_state: State<P>` | `Result<UndoStatusDto, String>` | Shell `reload_active_save_data` — mod.rs:896 |
| 14 | `read_base_config` (`features/settings/game_config.rs:15`) | `cache: State<C>` | `Result<BaseGameConfig, String>` | Shell `reload_active_save_data` — mod.rs:897 |
| 15 | `read_save_config` (`features/settings/game_config.rs:81`) | `profile_path: &str, _cache: State<C>` | `Result<SaveGameConfig, String>` | Shell `reload_active_save_data` — mod.rs:906 |
| 16 | `read_all_save_data` **async** (`features/save_analysis/reader.rs:69`) | `profile_state: State<P>, cache: State<C>, decrypt_cache: State<D>` | `Result<SaveGameData, String>` | Shell `reload_active_save_data` — mod.rs:912 **[BO]** |
| 17 | `try_read_truck_xp` (`features/save_editor/commands.rs:44`) | `profile_state: State<P>` | `Result<Option<i64>, String>` | Shell `reload_active_save_data` — mod.rs:941 |
| 18 | `quicksave_game_info` **async** (`features/save_analysis/quicksave.rs:10`) | `profile_state: State<P>, profile_cache: State<C>, decrypt_cache: State<D>` | `Result<GameDataQuicksave, String>` | Shell `reload_active_save_data` — mod.rs:963 **[BO]** |
| 19 | `get_current_truck_summary` **async** (`features/save_analysis/quicksave.rs:50`) | same as #18 | `Result<Option<CurrentTruckSummary>, String>` | Shell `reload_active_save_data` — mod.rs:986 **[BO]** |
| 20 | `get_all_trucks` **async** (`features/vehicles/trucks.rs:18`) | `profile_path: String, profile_state: State<P>, profile_cache: State<C>, decrypt_cache: State<D>` | `Result<Vec<ParsedTruck>, String>` | Shell `reload_active_save_data` — mod.rs:995 **[BO]** |
| 21 | `get_player_truck` **async** (`features/vehicles/trucks.rs:56`) | same as #20 | `Result<ParsedTruck, String>` | Shell `reload_active_save_data` — mod.rs:1004 **[BO]** |
| 22 | `get_all_trailers` **async** (`features/vehicles/trailers.rs:274`) | same as #20 | `Result<Vec<ParsedTrailer>, String>` | Shell `reload_active_save_data` — mod.rs:1013 **[BO]** |
| 23 | `get_player_trailer` **async** (`features/vehicles/trailers.rs:20`) | same as #20 | `Result<PlayerTrailerResult, String>` | Shell `reload_active_save_data` — mod.rs:1022 **[BO]** |
| 24 | `get_ets2_data_location` (`features/settings/ets2_paths.rs:13`) | — (none) | `Result<Ets2DataLocationDto, String>` | Shell `refresh_settings_and_location` — mod.rs:1034 |
| 25 | `get_sqlite_info` **async** (`commands.rs:136`) | `app_state: State<A>` | `Result<SqliteInfoDto, String>` | Shell `refresh_settings_and_location` — mod.rs:1040 **[BO]** |
| 26 | `get_sqlite_table_counts` **async** (`commands.rs:141`) | `profile_state: State<P>, app_state: State<A>` | `Result<SqliteTableCountsDto, String>` | Shell `refresh_settings_and_location` — mod.rs:1041 **[BO]** |
| 27 | `undo_last_save_change` (`features/save_editor/commands.rs:962`) | `profile_state: State<P>, profile_cache: State<C>, decrypt_cache: State<D>` | `Result<BackupRestoreResultDto, String>` | Shell top bar Undo — mod.rs:1266 |
| 28 | `repair_player_truck` **async** (`features/vehicles/editor.rs:460`) | `profile_state: State<P>, decrypt_cache: State<D>, profile_cache: State<C>` | `Result<(), String>` | Launcher QUICK ACTIONS — mod.rs:1702 **[BO]** |
| 29 | `refuel_player_truck` **async** (`features/vehicles/editor.rs:520`) | same as #28 | `Result<(), String>` | Launcher QUICK ACTIONS — mod.rs:1707 **[BO]** |
| 30 | `edit_skill_value` (`features/save_editor/commands.rs:1202`) | `skill: String, value: i64, profile_state: State<P>, profile_cache: State<C>, decrypt_cache: State<D>` | `Result<(), String>` | Launcher "Max All Skills" (6 calls in one loop) — mod.rs:1735 |
| 31 | `get_active_save_health` **async** (`features/health_monitor/commands.rs:10`) | `profile_state: State<P>, decrypt_cache: State<D>` | `Result<SaveHealthReportDto, String>` | Launcher "Scan Save Health" — mod.rs:1765 **[BO]** |
| 32 | `dispatcher_generate_jobs` (`features/career/commands.rs:428`) | `career: State<CS>, profile: State<P>` | `Result<DispatcherGenerationStatus, String>` | Launcher "Generate Market Jobs" — mod.rs:1778 |
| 33 | `calculate_level` (`xp/command.rs:79`) | `total_xp: u64` | `LevelProgress` (`.level`) | Shell reload/dashboard — mod.rs:945, 950, 1522 |
| 34 | `save_session::{is_dirty, dirty_paths}` (`shared/save_session.rs`) | — (none) | `bool`, `Vec<PathBuf>` | Shell: mod.rs:585, 609, 680, 705, 808, 1124, 1184, 1235, 1245, 1256, 1286, 1448, 1896 (dirty indicator, prompt guards, exit guard) |

### 3.2 Vehicles (`gui/vehicles_tab.rs`)

| # | Function (module) | Arguments (names : types) | Return shape | Calling screen (file:line) |
|---|---|---|---|---|
| 35 | `repair_player_truck` **async** | `profile_state: State<P>, decrypt_cache: State<D>, profile_cache: State<C>` | `Result<(), String>` | Player Vehicle Editor — vehicles_tab.rs:110 **[BO]** |
| 36 | `refuel_player_truck` **async** | same | `Result<(), String>` | Player Vehicle Editor — :123 **[BO]** |
| 37 | `set_player_truck_fuel` **async** (`editor.rs:539`) | `profile_state: State<P>, decrypt_cache: State<D>, profile_cache: State<C>, level: f32` | `Result<(), String>` | Player Vehicle Editor — :142 **[BO]** |
| 38 | `set_player_truck_wear` **async** (`editor.rs:559`) | same + `wear_type: String` | `Result<(), String>` | Player Vehicle Editor — :164 **[BO]** |
| 39 | `set_player_truck_license_plate` **async** (`editor.rs:431`) | `plate: String, profile_state: State<P>, decrypt_cache: State<D>, profile_cache: State<C>` | `Result<(), String>` | Player Vehicle Editor — :194 **[BO]** |
| 40 | `edit_truck_odometer` **async** (`editor.rs:616`) | `value: i64, …same State trio` | `Result<(), String>` | Player Vehicle Editor — :213 **[BO]** |
| 41 | `repair_player_trailer` **async** (`editor.rs:636`) | State trio | `Result<(), String>` | Player Trailer — :257 **[BO]** |
| 42 | `set_player_trailer_license_plate` **async** (`editor.rs:584`) | `plate: String, …State trio` | `Result<(), String>` | Player Trailer — :274 **[BO]** |
| 43 | `set_player_trailer_cargo_mass` **async** (`editor.rs:722`) | `mass: f32, …State trio` | `Result<(), String>` | Player Trailer — :297 **[BO]** |
| 44 | `initialize_truck_change_session` **async** (`truck_change/commands.rs:24`) | `save_path: Option<String>, profile_state: State<P>, decrypt_cache: State<D>, truck_change_cache: State<TruckChangeSessionCache>` | `Result<TruckChangeSession, String>` | Truck Switcher — :319 **[BO]** |
| 45 | `list_owned_trucks_for_switch` **async** (`:17`) | `save_path: Option<String>, profile_state: State<P>, decrypt_cache: State<D>` | `Result<TruckSwitchList, String>` | Truck Switcher — :327 **[BO]** |
| 46 | `preview_active_truck_switch` **async** (`:37`) | `save_path: Option<String>, target_truck_id: String, expected_file_hash: String, profile_state: State<P>, decrypt_cache: State<D>` | `Result<TruckChangePreview, String>` | Truck Switcher — :428 **[BO]** |
| 47 | `apply_active_truck_switch` **async** (`:58`) | `save_path, target_truck_id, expected_file_hash, create_persistent_backup: Option<bool>, profile_state: State<P>, profile_cache: State<C>, decrypt_cache: State<D>, truck_change_cache` | `Result<ApplyTruckChangeResult, String>` | Truck Switcher — :488 **[BO]** |
| 48 | `initialize_trailer_change_session` **async** (`trailer_change/commands.rs:20`) | `save_path: Option<String>, profile_state: State<P>, decrypt_cache: State<D>, trailer_change_cache: State<TrailerChangeSessionCache>` | `Result<TrailerChangeSession, String>` | Trailer Switcher — :527 **[BO]** |
| 49 | `list_owned_trailers_for_switch` **async** (`:13`) | `save_path: Option<String>, profile_state: State<P>, decrypt_cache: State<D>` | `Result<TrailerSwitchList, String>` | Trailer Switcher — :535 **[BO]** |
| 50 | `preview_active_trailer_switch` **async** (`:33`) | `save_path, target_trailer_id: String, expected_file_hash: String, profile_state, decrypt_cache` | `Result<TrailerChangePreview, String>` | Trailer Switcher — :624 **[BO]** |
| 51 | `apply_active_trailer_switch` **async** (`:54`) | `save_path, target_trailer_id, expected_file_hash, create_persistent_backup: Option<bool>, profile_state, profile_cache, decrypt_cache, trailer_change_cache` | `Result<ApplyTrailerChangeResult, String>` | Trailer Switcher — :683 **[BO]** |
| 52 | `get_official_powertrain_catalog` **async** (`truck_change/commands.rs:79`) | `game: Option<String>, game_version: Option<String>` | `Result<PowertrainCatalog, String>` | Powertrain Swapper — :718 **[BO]** |
| 53 | `preview_truck_powertrain_change` **async** (`:87`) | `save_path: Option<String>, truck_id: String, engine_data_path: Option<String>, transmission_data_path: Option<String>, game: Option<String>, game_version: Option<String>, profile_state: State<P>, decrypt_cache: State<D>` | `Result<TruckPowertrainPreview, String>` | Powertrain Swapper — :789 **[BO]** |

### 3.3 Garages & AI drivers (`gui/garages_tab.rs`)

| # | Function (module) | Arguments (names : types) | Return shape | Calling screen (file:line) |
|---|---|---|---|---|
| 54 | `get_all_garages` **async** (`garages/commands.rs:25`) | `profile_state: State<P>, decrypt_cache: State<D>, app_state: State<A>` | `Result<GarageListResult, String>` | `start_garages_load` — :837 **[BO]**, `reload_garages_and_drivers` — :859 **[BO]** |
| 55 | `get_ai_driver_management` **async** (`:46`) | same as #54 | `Result<AiDriverManagementResult, String>` | `start_garages_load` — :838 **[BO]**, `reload_garages_and_drivers` — :877 **[BO]** |
| 56 | `generate_ai_drivers` (`:66`) | `request: AiDriverGenerateRequest, profile_state, profile_cache, decrypt_cache, truck_change_cache, trailer_change_cache, app_state` | `Result<AiDriverMutationResult, String>` | Garage detail — :352 |
| 57 | `purchase_garage` (`:153`) | `request: GarageMutationRequest, …same 6 State args` | `Result<GarageMutationResult, String>` | Garage detail — :531 |
| 58 | `upgrade_owned_garage` (`:182`) | same as #57 | `Result<GarageMutationResult, String>` | Garage detail — :550 |
| 59 | `assign_random_garage_resources` (`:298`) | `request: GarageResourceAssignmentRequest, …same State args` | `Result<GarageMutationResult, String>` | Garage detail — :568 |
| 60 | `update_garage` (`:211`) | `request: GarageUpdateRequest, …same State args` (HQ / size) | `Result<GarageMutationResult, String>` | Garage detail — :589 |
| 61 | `relinquish_garage_ownership` (`:327`) | `request: GarageMutationRequest, …same State args` | `Result<GarageMutationResult, String>` | `run_garage_confirm` — :680 |
| 62 | `relinquish_empty_garages` (`:269`) | `request: GarageRelinquishEmptyRequest, …same State args` | `Result<GarageRelinquishEmptyResult, String>` | Bulk action — :705 |
| 63 | `buy_all_garages` (`:240`) | `request: GarageBuyAllRequest, …same State args` | `Result<GarageBuyAllResult, String>` | Bulk action — :742 |
| 64 | `remove_ai_drivers` (`:124`) | `request: AiDriverRemoveRequest, …same State args` | `Result<AiDriverMutationResult, String>` | `run_garage_confirm` — :765 |

### 3.4 Career (`gui/career_tab.rs`)

| # | Function (module) | Arguments (names : types) | Return shape | Calling screen (file:line) |
|---|---|---|---|---|
| 65 | `career_get_status` (`career/commands.rs:105`) | `career: State<CS>` | `Result<CareerStatus, String>` | `refresh_career_data` — :79, Live Telemetry — :100 |
| 66 | `get_plugin_status` (`:121`) | `profile: State<P>` | `Result<bool, String>` | `refresh_career_data` — :80 |
| 67 | `career_get_overview` (`:131`) | `career: State<CS>` | `Result<CareerOverview, String>` | `refresh_career_data` — :81 |
| 68 | `dispatcher_get_dispatcher_overview` (`:420`) | `career: State<CS>, profile: State<P>` | `Result<DispatcherOverview, String>` | `refresh_career_data` — :83 |
| 69 | `dispatcher_get_generation_status` (`:457`) | `career: State<CS>, profile: State<P>` | `Result<DispatcherGenerationStatus, String>` | `refresh_career_data` — :85 |
| 70 | `dispatcher_get_market_jobs` (`:300`) | `filter: Option<DispatcherJobFilter>, career: State<CS>, profile: State<P>` | `Result<Vec<DispatcherMarketJob>, String>` | `refresh_career_data` — :87 |
| 71 | `dispatcher_get_active_jobs` (`:348`) | `career: State<CS>, profile: State<P>` | `Result<Vec<DispatcherMarketJob>, String>` | `refresh_career_data` — :89 |
| 72 | `career_list_trips` (`:135`) | `career: State<CS>` | `Result<Vec<TripSummary>, String>` | `refresh_career_data` — :90 |
| 73 | `career_get_job_log` (`:189`) | `career: State<CS>` | `Result<Vec<JobLogEntry>, String>` | `refresh_career_data` — :91 |
| 74 | `career_get_job_stats` (`:209`) | `career: State<CS>` | `Result<JobStats, String>` | `refresh_career_data` — :92 |
| 75 | `career_get_analytics_summary` (`:227`) | `filters: Option<AnalyticsFilters>, career: State<CS>` | `Result<AnalyticsSummary, String>` | `refresh_career_data` — :93 |
| 76 | `get_current_user_profile` (`vtc/commands.rs:37`) | `auth: State<Auth>` | `Result<UserProfile, String>` | `refresh_career_data` — :94 |
| 77 | `get_company_overview` (`vtc/commands.rs:105`) | `auth: State<Auth>` | `Result<CompanyOverview, String>` | `refresh_career_data` — :95 |
| 78 | `get_company_members` (`vtc/commands.rs:116`) | `auth: State<Auth>` | `Result<Vec<CompanyMember>, String>` | `refresh_career_data` — :96 |
| 79 | `dispatcher_generate_jobs` (`:428`) | `career: State<CS>, profile: State<P>` | `Result<DispatcherGenerationStatus, String>` | Dispatcher — :191 |
| 80 | `dispatcher_generate_universal_jobs` (`:439`) | `force: Option<bool>, config: Option<DispatcherGenerationConfigInput>, career: State<CS>, profile: State<P>` | `Result<DispatcherGenerationStatus, String>` | Dispatcher — :208 |
| 81 | `ets_snapshot_refresh_active_save` **async** (`commands.rs:51`) | `app: AppHandle, profile_state: State<P>, db: State<ETS>` | `Result<SaveSnapshotDto, AppError>` | Live Telemetry — :228 **[BO]** |
| 82 | `dispatcher_cleanup_expired_jobs` (`:465`) | `career: State<CS>, profile: State<P>` | `Result<DispatcherGenerationStatus, String>` | Dispatcher — :246 |
| 83 | `dispatcher_accept_job` (`:336`) | `job_id: String, career: State<CS>, profile: State<P>` | `Result<DispatcherJobDetails, String>` | Job detail — :469 |
| 84 | `dispatcher_assign_and_prepare_and_write` **async** (`:527`) | `job_id: String, auto_write: bool, app: AppHandle, career: State<CS>, profile: State<P>, db: State<ETS>` | `Result<DispatcherAssignPrepareWriteDto, String>` | Job detail — :482 **[BO]** |
| 85 | `dispatcher_cancel_job` (`:356`) | `job_id: String, career: State<CS>, profile: State<P>` | `Result<DispatcherJobDetails, String>` | Job detail — :525 |
| 86 | `career_scan_profile_job_history` (`:243`) | `profile_path: Option<String>, career: State<CS>, profile: State<P>` | `Result<AnalyticsScanResult, String>` | Logbook & Analytics — :571 |
| 87 | `career_export_analytics_csv` (`:267`) | `app: AppHandle, filters: Option<AnalyticsFilters>, career: State<CS>` | `Result<Option<String>, String>` | Logbook & Analytics — :593 |
| 88 | `auth_logout` (`auth/commands.rs:69`) | `auth: State<Auth>` | `Result<(), String>` | VTC & Account — :658 |
| 89 | `auth_login` (`auth/commands.rs:57`) | `email: String, password: String, remember_me: bool, auth: State<Auth>` | `Result<AuthLoginResult, String>` | VTC & Account — :674 |
| 90 | `create_company` (`vtc/commands.rs:98`) | `input: CreateCompanyInput, auth: State<Auth>` | `Result<CompanyOverview, String>` | VTC & Account — :714 |
| 91 | `save_session::is_staging_enabled` (`shared/save_session.rs`) | — (none) | `bool` | Dispatcher job detail — career_tab.rs:496 (staged-vs-written label) |

### 3.5 Coaches (`gui/coaches_tab.rs`)

| # | Function (module) | Arguments (names : types) | Return shape | Calling screen (file:line) |
|---|---|---|---|---|
| 92 | `read_project_coaches` (`save_editor/commands.rs:20`) | `profile_state: State<P>` | `Result<CoachesSnapshot, String>` | Coaches `reload` — :368 |
| 93 | `edit_career_xp` (`save_editor/commands.rs:63`) | `xp: i64, career: Career::Bus, expected_save_hash: String, profile_state: State<P>, profile_cache: State<C>, decrypt_cache: State<D>` | `Result<(), String>` | Coaches `apply_bus_xp` — :379 |
| 94 | `display_job_stop_name` (`save_editor/coaches.rs:91`) | `raw: &str` | `String` | Job history — :227, :228 |
| 95 | `display_stop_name` (`save_editor/coaches.rs:72`) | `id: &str` | `String` | stop search/row — :326, :348 |
| 96 | `display_bus_name` (`save_editor/coaches.rs:85`) | `id: &str` | `String` | bus labels (imported and used in job/stop rendering) |
| 97 | `calculate_level` (`xp/command.rs:79`) | `total_xp: u64` | `LevelProgress` | Bus level display — :63 |

### 3.6 Mods (`gui/mods_tab.rs`)

| # | Function (module) | Arguments (names : types) | Return shape | Calling screen (file:line) |
|---|---|---|---|---|
| 98 | `load_mod_profile_manager_state` (`mod_profile_manager/commands.rs:140`) | `app: AppHandle, profile_state: State<P>, game: Option<String>` | `Result<ModProfileManagerState, String>` | `render` — :64 |
| 99 | `open_steam_console` (`:609`) | — (none) | `Result<(), String>` | Mod Manager — :95 |
| 100 | `scan_mods_deep` (`:260`) | `app: AppHandle, profile_state: State<P>, game: Option<String>` | `Result<ModProfileManagerState, String>` | `start_mods_scan` — :117 (background thread) |
| 101 | `scan_mods_light` (`:226`) | same as #100 | `Result<ModProfileManagerState, String>` | `start_mods_scan` — :119 (background thread) |
| 102 | `delete_mod_preset` (`:472`) | `app: AppHandle, preset_id: String` | `Result<(), String>` | `render_mod_confirm` — :144 |
| 103 | `delete_mod_sandbox` (`:542`) | `app: AppHandle, sandbox_id: String` | `Result<SandboxCollection, String>` | `render_mod_confirm` — :155 |
| 104 | `list_mod_presets` (`:290`) | `app: AppHandle, game: Option<String>` | `Result<Vec<ModPreset>, String>` | `reload_preset_list` — :173 |
| 105 | `select_manual_workshop_directory` (`:491`) | `app: AppHandle, game: String` | `Result<Option<String>, String>` | Presets & Sandboxes — :182 |
| 106 | `clear_manual_workshop_directory` (`:509`) | `app: AppHandle, game: String` | `Result<(), String>` | Presets & Sandboxes — :190 |
| 107 | `open_workshop_page` (`:612`) | `mod_id: String` | `Result<(), String>` | Mod Manager — :238 |
| 108 | `import_mod_preset` (`:450`) | `app: AppHandle` | `Result<Option<ModPreset>, String>` | Presets & Sandboxes — :270 |
| 109 | `create_mod_preset` (`:299`) | `app: AppHandle, profile_state: State<P>, name: String, game: String, notes: Option<String>, preset_label: Option<String>` | `Result<ModPreset, String>` | Presets & Sandboxes — :302 |
| 110 | `compare_mod_preset` (`:384`) | `app: AppHandle, profile_state: State<P>, preset_id: String, game: String` | `Result<PresetCompareResult, String>` | Presets & Sandboxes — :330 |
| 111 | `export_mod_preset` (`:421`) | `app: AppHandle, preset_id: String` | `Result<String, String>` | Presets & Sandboxes — :347 |
| 112 | `load_mod_sandboxes` (`:525`) | `app: AppHandle` | `Result<SandboxCollection, String>` | Presets & Sandboxes — :381 |
| 113 | `create_mod_sandbox` (`:533`) | `app: AppHandle, title: String, description: String` | `Result<SandboxCollection, String>` | Presets & Sandboxes — :393 |
| 114 | `apply_sandbox_to_active_profile` (`:594`) | `app: AppHandle, profile_state: State<P>, sandbox_id: String, force_clear: bool` | `Result<ApplySandboxResult, String>` | Presets & Sandboxes — :414 |
| 115 | `add_mod_to_sandbox` (`:547`) | `app: AppHandle, sandbox_id: String, workshop_input: String, manual_fallback: Option<bool>` | `Result<SandboxCollection, String>` | Presets & Sandboxes — :439 |
| 116 | `analyze_mod_conflict_diagnostics` **async** (`save_analysis/commands.rs:23`) | `profile_state: State<P>, decrypt_cache: State<D>` | `Result<ModConflictAnalysisReport, String>` | Conflict Analyzer — :465 **[BO]** |
| 117 | `analyze_mod_conflict_diagnostics_deep` **async** (`:109`) | same as #116 | `Result<ModConflictAnalysisReport, String>` | Conflict Analyzer — :477 **[BO]** |
| 118 | `export_mod_conflict_diagnostics_report` (`:185`) | `app: AppHandle, report: ModConflictAnalysisReport, formatted: Option<bool>` | `Result<Option<String>, String>` | Conflict Analyzer — :490 |

### 3.7 Profile tools & safety (`gui/profile_tools_tab.rs`)

| # | Function (module) | Arguments (names : types) | Return shape | Calling screen (file:line) |
|---|---|---|---|---|
| 119 | `validate_clone_target` (`profile_clone/commands.rs:22`) | `source_profile: String, new_name: String` | `Result<CloneTargetStatus, String>` | Clone & Rename — :65 |
| 120 | `clone_profile_command` (`profile_clone/commands.rs:5`) | `source_profile: String, new_name: String, backup: bool, replace_hex: bool, replace_text: bool` | `Result<String, String>` | Clone & Rename — :67 |
| 121 | `profile_rename` (`profile_rename/commands.rs:8`) | `new_name: String, profile_state: State<P>` | `Result<String, String>` | Clone & Rename — :90 |
| 122 | `copy_profile_controls` (`profile_controls/commands.rs:5`) | `source_profile_path: String, target_profile_path: String` | `Result<String, String>` | Clone & Rename — :123 |
| 123 | `copy_mods_to_profile` (`profile_move_mods/commands.rs:9`) | `target_profile_path: String, profile_state: State<P>` | `Result<String, String>` | Clone & Rename — :130 |
| 124 | `get_profile_share_context` (`profile_sharing/commands.rs:9`) | `profile_path: Option<String>, profile_state: State<P>` | `Result<ProfileShareContext, String>` | Profile Sharing — :146 |
| 125 | `pick_shared_profile_export_directory` **async** (`:21`) | `app: AppHandle, profile_state: State<P>` | `Result<Option<String>, String>` | Profile Sharing — :156-157 **[BO]** |
| 126 | `export_shared_profile` (`:27`) | `profile_path: String, export_dir_override: Option<String>, profile_state: State<P>` | `Result<ProfileShareExportResult, String>` | Profile Sharing — :170 |
| 127 | `pick_shared_profile_import_archive` **async** (`:15`) | `app: AppHandle, profile_state: State<P>` | `Result<Option<String>, String>` | Profile Sharing — :190-191 **[BO]** |
| 128 | `inspect_shared_profile_archive` (`:34`) | `archive_path: String, profile_name_override: Option<String>, profile_state: State<P>` | `Result<ProfileShareImportPreview, String>` | Profile Sharing — :210 |
| 129 | `import_shared_profile` (`:45`) | `archive_path: String, profile_name_override: Option<String>, profile_state: State<P>` | `Result<ProfileShareImportResult, String>` | Profile Sharing — :227 |
| 130 | `get_active_save_health` **async** (`health_monitor/commands.rs:10`) | `profile_state: State<P>, decrypt_cache: State<D>` | `Result<SaveHealthReportDto, String>` | Save Health — :261 **[BO]** |
| 131 | `apply_save_health_fix` (`:69`) | `fix_id: String, confirmed: bool, profile_state: State<P>, profile_cache: State<C>, decrypt_cache: State<D>` | `Result<SaveHealthFixResultDto, String>` | Save Health — :292 |
| 132 | `list_active_save_backups` **async** (`backup/commands.rs:10`) | `profile_state: State<P>` | `Result<Vec<BackupVersionDto>, String>` | Backups & Restore — :318 **[BO]** |
| 133 | `preview_backup_restore` (`:46`) | `backup_id: String, profile_state: State<P>` | `Result<BackupRestorePreviewDto, String>` | Backups & Restore — :352 |
| 134 | `restore_backup` (`:69`) | `backup_id: String, confirmed: bool, profile_state: State<P>, profile_cache: State<C>, decrypt_cache: State<D>` | `Result<BackupRestoreResultDto, String>` | `render_backup_confirm` — :400 |

### 3.8 Save editor (`gui/save_editor_tab.rs`)

| # | Function (module) | Arguments (names : types) | Return shape | Calling screen (file:line) |
|---|---|---|---|---|
| 135 | `read_project_coaches` | `profile_state: State<P>` | `Result<CoachesSnapshot, String>` | Save Editor Refresh — :22 |
| 136 | `edit_player_money` (`save_editor/commands.rs:1139`) | `value: i64, profile_state: State<P>, profile_cache: State<C>, decrypt_cache: State<D>` | `Result<(), String>` | Economy — :98 |
| 137 | `edit_player_experience` (`:1187`) | `value: i64, …State trio` | `Result<(), String>` | Economy — :152 |
| 138 | `edit_level` (`:1106`) | `level: i64, …State trio` | `Result<(), String>` | Economy — :187 |
| 139 | `apply_custom_reset_values` (`:810`) | `level: Option<u32>, xp: Option<u64>, money: Option<i64>, …State trio` | `Result<ApplyCustomResetValuesResultDto, String>` | Economy — :218 |
| 140 | `edit_skill_value` (`:1202`) | `skill: String, value: i64, …State trio` | `Result<(), String>` | Skills (6 calls) — :293, :321 |
| 141 | `edit_traffic_value` (`:1399`) | `value: i64, …State trio` | `Result<(), String>` | Config — :361 |
| 142 | `edit_developer_value` (`:1267`) | `value: i64, …State trio` | `Result<(), String>` | Config — :380 |
| 143 | `edit_console_value` (`:1311`) | `value: i64, …State trio` | `Result<(), String>` | Config — :399 |
| 144 | `edit_parking_doubles_value` (`:1438`) | `value: i64, …State trio` | `Result<(), String>` | Config — :424 |
| 145 | `edit_convoy_value` (`:1355`) | `value: i64, …State trio` | `Result<(), String>` | Config — :443 |
| 146 | `edit_config_value` (`:1494`) | `payload: KeyValuePayload, …State trio` | `Result<(), String>` | Custom base config key — :466 |
| 147 | `edit_save_config_value` (`:1539`) | `payload: KeyValuePayload, …State trio` | `Result<(), String>` | Custom save config key — :489 |
| 148 | `calculate_level` (`xp/command.rs:79`) | `total_xp: u64` | `LevelProgress` | Level preview — :53 |

### 3.9 Settings & diagnostics (`gui/settings_tab.rs`)

| # | Function (module) | Arguments (names : types) | Return shape | Calling screen (file:line) |
|---|---|---|---|---|
| 149 | `get_log_status` (`logging/commands.rs:112`) | — (none) | `Result<UserLogStatus, String>` | Settings refresh — :27 |
| 150 | `get_user_logs` (`:97`) | `level_filter: Option<String>, max_lines: Option<u32>` | `Result<Vec<UserLogEntry>, String>` | Settings refresh — :31, logs view — :170 |
| 151 | `get_log_preview` **async** (`logging/diagnostics.rs:16`) | `state: State<DS>` | `Result<String, String>` | Settings refresh — :35 **[BO]**, logs view — :174 **[BO]** |
| 152 | `choose_ets2_data_location` (`settings/ets2_paths.rs:16`) | `app: AppHandle, dialog_title: Option<String>, profile_state: State<P>, decrypt_cache: State<D>, profile_cache: State<C>` | `Result<Option<Ets2DataLocationDto>, String>` | Homedir section — :71 |
| 153 | `reset_ets2_data_location` (`:77`) | `app: AppHandle, profile_state: State<P>, decrypt_cache: State<D>, profile_cache: State<C>` | `Result<Ets2DataLocationDto, String>` | Homedir section — :88 |
| 154 | `set_ets2_data_location_mode` (`:44`) | `app: AppHandle, mode: String, custom_homedir: Option<String>, profile_state: State<P>, decrypt_cache: State<D>, profile_cache: State<C>` | `Result<Ets2DataLocationDto, String>` | Homedir section — :110 |
| 155 | `data_import_ets2_datasets` (`commands.rs:166`) | `force: bool, app: AppHandle, career: State<CS>` | `Result<Ets2DataImportSummary, String>` | SQLite/datasets section — :148 |
| 156 | `export_user_logs` (`logging/commands.rs:103`) | `app: AppHandle, profile_state: State<P>` | `Result<String, String>` | Logs section — :182 |
| 157 | `export_logs_bundle` (`:118`) | `app: AppHandle, profile_state: State<P>` | `Result<Option<String>, String>` | Logs section — :188 |
| 158 | `export_log_file` **async** (`logging/diagnostics.rs:34`) | `app: AppHandle, state: State<DS>` | `Result<Option<String>, String>` | Logs section — :195 **[BO]** |
| 159 | `clear_user_logs` (`logging/commands.rs:109`) | — (none) | `Result<Option<String>, String>` | Logs section — :208 |

**Call-table totals:** 159 grouped rows covering **≈188 individual call sites** (including the 47 `block_on` sites) and all 11 `gui/` files. Rows 92-97 and 135-148 are pure formatting/level helpers living in backend modules but called directly from the UI; they are included for completeness.

---

## 4. Shared-state inventory

### 4.1 `Ets2NativeApp` (`gui/mod.rs:205-384`) — GUI-owned mirror state

| Field group (representative fields) | Written by | Read by |
|---|---|---|
| `app`, `event_rx`, `recent_events` | `::new`, `drain_background_events` (mod.rs:1048) | all screens (via `app.*_state()` accessors); Dashboard RECENT ACTIVITY |
| `active_tab`, `vehicles_sub_tab`, `career_sub_tab`, `mods_sub_tab`, `profile_tools_sub_tab` | sidebar (`render_sidebar`), Dashboard tiles/quick actions, Save Health jump | `update` router (`:1975-1991`), every screen header |
| `toasts`, `unsaved_prompt`, `close_after_prompt` | `set_status` (`:562`), top bar / prompt / exit guard | `update` toast area, `render_unsaved_prompt` |
| `selected_game`, `available_languages`, `current_language`, `profiles`, `selected_profile_path`, `saves`, `selected_save_path`, `undo_status` | `refresh_profiles_and_saves`, `select_profile`, `select_save`, `reload_active_save_data`, game combo | top bar combos, Dashboard, Save Editor (empty state), Profile Tools, Settings |
| `save_data`, `quicksave_data`, `current_truck_summary`, `base_config`, `save_config`, `edit_money_input`, `edit_xp_input`, `edit_level_input`, `batch_level_edited_last`, `skill_*`, `cfg_*`, `custom_*_key/value` | `reload_active_save_data`, Save Editor controls, launcher Max All Skills | Save Editor, Dashboard tiles, Coaches (level) |
| `coaches_snapshot`, `coaches_stop_search`, `coaches_stop_filter`, `edit_bus_xp_input` | `reload_active_save_data` (clears on save change), Coaches `reload`, Save Editor Refresh | Coaches screen |
| `player_truck`, `all_trucks`, `player_trailer_result`, `all_trailers`, `truck_plate_input`, `truck_odometer_input`, `truck_fuel_slider`, `truck_wear_slider`, `trailer_plate_input`, `trailer_cargo_mass_input` | `reload_active_save_data`, Vehicles Player Editor | Vehicles Player Editor, Dashboard Current Setup/stat tiles |
| `truck_switch_list`, `truck_change_session`, `selected_switch_truck_id`, `truck_switch_preview`, `create_backup_on_switch`, `trailer_switch_*`, `powertrain_catalog`, `selected_engine_path`, `selected_transmission_path`, `powertrain_preview` | Vehicles switchers/powertrain; cleared by `invalidate_switcher_state` (`:625`) | Vehicles switchers only |
| `fleet_search`, `fleet_*_sort(_asc)`, `truck_switch_search`, `trailer_switch_search` | Vehicles table headers/search boxes | Vehicles tables |
| `garage_list`, `ai_driver_mgmt`, `garage_filter_text`, `garage_only_owned`, `selected_garage_id`, `garage_sort_key`, `garage_sort_asc`, `garage_confirm`, `garages_load`, `generate_drivers_*`, `selected_garage_for_drivers` | `start_garages_load` / `reload_garages_and_drivers`, Garage UI, confirm dialog, shell flush | Garages screen, Dashboard GARAGES OWNED / AI DRIVERS tiles |
| `career_status`, `plugin_installed`, `career_overview`, `dispatcher_overview`, `dispatcher_gen_status`, `market_jobs`, `active_dispatcher_jobs`, `selected_market_job_id`, `job_search_filter`, `dispatcher_sort(_asc)`, `trip_log`, `job_log`, `job_stats`, `analytics_summary` | `refresh_career_data`, Dispatcher, logbook | Career sub-screens, launcher Generate Market Jobs |
| `vtc_user_profile`, `vtc_company`, `vtc_members`, `auth_*_input`, `new_company_*` | VTC & Account form/login | VTC & Account |
| `mod_manager_state`, `mod_presets`, `new_preset_*`, `selected_preset_id`, `preset_compare_result`, `mod_sandboxes`, `new_sandbox_*`, `selected_sandbox_id`, `workshop_mod_input`, `conflict_report`, `mod_confirm`, `mods_scan_load` | Mods screen + background scan | Mods sub-screens |
| `clone_new_name`, `clone_backup`, `clone_replace_hex/text`, `rename_profile_input`, `target_profile_for_copy`, `share_*`, `save_health_report`, `backups_list`, `selected_backup_id`, `backup_preview`, `backup_confirm` | Profile Tools forms/flows; `save_health_report` also set by launcher quick action | Profile Tools sub-screens |
| `data_location`, `custom_homedir_input`, `sqlite_info`, `sqlite_counts`, `log_status`, `user_logs`, `diagnostic_log_preview` | `refresh_settings_and_location`, Settings screen | Settings screen; Dashboard uses sqlite counts indirectly via commands |

**Observation:** screens are coupled through this single struct rather than through events. Examples: `reload_active_save_data` overwrites inputs used by Save Editor, Vehicles and Coaches at once; `flush_pending_changes` triggers `garages_tab::start_garages_load` and `invalidate_switcher_state`; launcher's "Scan Save Health" writes `save_health_report` and mutates `active_tab`/`profile_tools_sub_tab`.

### 4.2 Backend-managed state (`src/state.rs`) and accessors

| State type | Meaningful contents | GUI interaction |
|---|---|---|
| `AppProfileState` (`:269`) | `current_profile: Mutex<Option<String>>`, `current_save`, `selected_game: Mutex<String>` | Written via `set_selected_game`, `set_active_profile`, `set_current_save` from shell combos; read by nearly every save-scoped command. Accessor `Ets2NativeApp::profile_state` (`mod.rs:740`). |
| `DecryptCache` (`:19`) | `files`, `inflight`, `generations` maps; `invalidate_path` (`:39`) | Cleared on profile/save switch and per-edit; read by every decrypt/parse command. Accessor `:744`. |
| `ProfileCache` (`:65`) | private `CachedProfileData`: `profile_path`, `save_path`, `base_config`, `save_config`, `save_game_data`, `quicksave_data`, `current_truck_summary`, `all_trucks`, `player_truck`, `all_trailers`, `player_trailer`; methods `reset_profile` (`:77`), `set_save_path` (`:94`), `invalidate_save_data` (`:77` region), cache getters/setters | Reset on switch; invalidated after Save/Discard/undo; read via `read_all_save_data`, `quicksave_game_info`, vehicle readers, config readers. Accessor `:748`. |
| `AppState` (`:424`) | `sqlite: SqlitePool`, `sqlite_path: PathBuf`, `garage_mutation_lock: Mutex<()>` | Read-only in GUI (`get_sqlite_info`, `get_sqlite_table_counts`, garage mutation guard). Accessor `:752`. |
| `CareerState` / `CareerRuntime` (`:408`, `:305`) | atomics `telemetry_running`, `ets2_running`, `plugin_installed`, `bridge_connected`, `overview_dirty`; `last_telemetry`, `active_job`, `active_trip`, `db_path` | GUI calls `career_*`/`dispatcher_*` commands and reads `last_telemetry` directly in `career_tab::render_live_telemetry` (`:101-106`). Accessor `:756`. |
| `AuthState` (`:440`) | `session: Mutex<Option<AuthSession>>` | VTC sign in/out, company commands. Accessor `:760`. |
| `EtsDbState` (`:420`) | `pool: SqlitePool` | Snapshot refresh and dispatcher write only. Accessor `:764`. |
| `HubState` (`:293`) | `mode: RwLock<AppMode>` (`Utility`/`Career`) | Accessor `hub_state` (`:768`) exists but is unused by the current GUI. |
| `DiagnosticsState` (`:9`) | `Arc<DiagnosticRuntime>` | `get_log_preview`, `export_log_file`; `shutdown()` in `main.rs`. Accessor `:772`. |
| `TruckChangeSessionCache` / `TrailerChangeSessionCache` | session maps | Only via switcher commands in `vehicles_tab.rs`. |
| Process-global `shared::save_session` (`shared/save_session.rs`) | `staging_enabled: bool`; `staged: BTreeMap<PathBuf, String>` | Enabled in `main.rs:159`; GUI reads `is_dirty()`/`dirty_paths()`/`is_staging_enabled()`; writers stage through backend commands; flush/discard via `save_pending_changes`/`discard_pending_changes`. |

---

## 5. Reusable vs layout-bound widgets

### 5.1 `gui/components.rs` — reusable as-is

- `UiExt::card` (`:11-23`) — frame container used by every tab; pure context-free wrapper.
- `page_header` (`:25`), `section_label` (`:36`), `primary_button` (`:91`), `empty_state` (`:214`), `status_dot` (`:232`), `game_badge` (`:199`) — layout-agnostic.
- `confirm_dialog` (`:240-265`) — modal primitive: centered, non-collapsible, non-resizable `egui::Window`, two buttons, returns `Option<bool>`; the only shared confirm mechanism.
- `sub_tabs<T>` (`:101`) — generic segmented control; **currently used only by `coaches_tab.rs:303`** because the sidebar owns sub-navigation everywhere else.
- `theme.rs` (all) — 17 color constants plus `apply`/`visuals`/`widget`; no layout assumptions; globally applied in `Ets2NativeApp::new` via `configure_theme` (`:558`).

### 5.2 Layout-bound / constrained

- `stat_tile` (`:125`) and `clickable_stat_tile` (`:145`) call `ui.set_min_width(ui.available_width())`; the doc comment (`:115-124`) requires `ui.columns()` and forbids `horizontal_wrapped` (first tile would consume the row). `clickable_stat_tile` derives its widget `Id` from `("clickable_stat_tile", label)` — duplicate labels collide.
- `nav_section`/`nav_item` (`:45`, `:52`) — sidebar-only: fixed 34 px rows, painted accent bar and text at fixed offsets, pointing-hand cursor.
- `action_card` (`:173`) — launcher-only: fixed `min_height(104)`, full-width fill, embedded primary button.
- Toast `Area` and unsaved prompt `Window` are implemented inline in `gui/mod.rs::update` / `render_unsaved_prompt`, not factored into `components.rs`.
- Three near-duplicate sort-header helpers: `vehicles_tab.rs::fleet_sort_header` (`:1112`), `garages_tab.rs::sort_header` (`:442`), `career_tab.rs::dispatcher_sort_header` (`:542`).
- Theme bypass: tabs hardcode RGB values (`Color32::from_rgb(88,214,141)`, `(231,76,60)`, `(245,176,65)`) in `vehicles_tab.rs`, `garages_tab.rs`, `career_tab.rs`, `mods_tab.rs`, `profile_tools_tab.rs` instead of `theme::SUCCESS/DANGER/WARNING`.

---

## 6. Save-safety map

| Behavior | File : symbol / evidence |
|---|---|
| Staging enabled only for the GUI; edits are in-memory overlays until Save | `src/main.rs:159` `save_session::set_staging_enabled(true)`; `shared/save_session.rs::{set_staging_enabled, stage_save, staged_content, is_dirty, dirty_paths, take_for_flush, discard, remove_staged}`; enabled-state checked in `career_tab.rs:496`. |
| Dirty indicator and Save enablement | `gui/mod.rs::render_top_bar` (`:1286-1300`): `"! Unsaved changes"` label, `add_enabled(dirty, Button::new("Save"))`, tooltip "Write staged save edits to disk". Dirty count also shown in the unsaved prompt (`:705`). |
| Explicit flush with pre-write backup and partial-failure accounting | `gui/mod.rs::flush_pending_changes` (`:581-623`) → `save_editor/commands.rs::save_pending_changes` (`:196`; backend creates a backup before writing, re-stages on failure). GUI counts `dirty_paths()` before/after and reports "N file(s) saved, M still staged". |
| Unsaved-changes prompt before destructive navigation | `gui/mod.rs::render_unsaved_prompt` (`:694`), `PendingTopBarAction` (`:193`), `resolve_unsaved_prompt` (`:674`), `perform_top_bar_action` (`:637`): game/profile/save switch, Reload, Exit; Save failure keeps staged edits and cancels the switch; Cancel restores the old game value. |
| Exit guard | `gui/mod.rs::update` (`:1894-1904`): `CancelClose` when dirty → prompt → `close_after_prompt` → `ViewportCommand::Close`. |
| Discard path | `gui/mod.rs::resolve_unsaved_prompt` (`:684-692`): `discard_pending_changes` + `reload_active_save_data` + `invalidate_switcher_state`. |
| Undo disabled while edits are staged | `gui/mod.rs::render_top_bar` (`:1256`): `can_undo = !save_session::is_dirty() && undo_status.can_undo`; `undo_last_save_change` (`:1266`) restores disk files from a backup. |
| Automatic backup creation on switch / clone | `gui/mod.rs:458` `create_backup_on_switch = true`; `vehicles_tab.rs:477,492,672,687` passes `Some(app.create_backup_on_switch)` to apply-switch commands; `profile_tools_tab.rs:67` passes `clone_backup` (default true, `:455` in mod.rs). |
| Backup listing / preview / confirmed restore | `profile_tools_tab.rs::render_backups_and_restore` (`:316`) → `list_active_save_backups` (`:318`), `preview_backup_restore` (`:352`), `render_backup_confirm` (`:384`) using `components::confirm_dialog` (`:391`) → `restore_backup(backup_id, true, …)` (`:400`). Message states that current state is backed up first. |
| Destructive confirmations | `components.rs::confirm_dialog`; `garages_tab.rs::render_garage_confirm` (sell garage, sell empty, buy all, dismiss driver); `mods_tab.rs::render_mod_confirm` (delete preset/sandbox); `profile_tools_tab.rs::render_backup_confirm` (restore). |
| Pre-write validation | `validate_clone_target` before cloning (`profile_tools_tab.rs:65`); `garages_tab.rs::is_sellable` (`:814`) + explanatory disabled Sell (`:606-620`); skill ranges (ADR 0-63, others 0-6), money `0..=999_999_999_999`, XP `0..=999_999_999`, level `0..=256`, odometer `0..=99_999_999`, cargo mass `0..=200_000.0`, non-empty custom config keys; backend clamps/`validate_*` mirror these. |
| Optimistic concurrency / stale-write prevention | `vehicles_tab.rs:471-492,666-687` (`safe_to_write`, `can_apply`, `expected_file_hash`, preview warnings); garage mutations carry `expected_save_hash` from list results; backend refuses `save_changed_since_load`. |
| Staleness invalidation | `gui/mod.rs::invalidate_switcher_state` (`:625`) on save/profile switch, flush, discard; `flush_pending_changes` restarts garage loads (`:600-604`); `reload_active_save_data` clears `coaches_snapshot` (`:893`); garage load errors clear stale lists (`garages_tab.rs:52-63,851-884`). |
| Validation / error / outcome surfacing | `Ets2NativeApp::set_status` (`:562`, dedupe, max 4, 5 s/10 s TTL) + toast Area in `update`; percentage of mutation results are reported (`set_status(res.message, !res.applied)` in `profile_tools_tab.rs:261`; `run_garage_command` in `garages_tab.rs:781`; `is_no_change_error` at `mod.rs:186` for `coaches.no_change` neutral notes). |
| User log / diagnostic display | `settings_tab.rs::render`: `get_log_status` (`:27`), `get_user_logs` (`:31,170`), `get_log_preview` (`:35,174`), `export_user_logs` (`:182`), `export_logs_bundle` (`:188`), `export_log_file` (`:195`), `clear_user_logs` (`:208`); read failures produce error toasts and keep old values (AS7). |
| Event-based error surfacing | `gui/mod.rs::drain_background_events` (`:1048`) converts `career://plugin_install_error` to an error toast; Dashboard RECENT ACTIVITY (`:1810-1840`) shows last 5 events. |
| Parse/warning display | Garage list + driver + per-garage warning cards (`garages_tab.rs:146-164,498-508`); Coaches warnings (`coaches_tab.rs:48-60`); dispatcher job `risk_note`/`bonus_note`/`last_error_message` (`career_tab.rs:462-470`). |

**Gaps to preserve consciously in the redesign:** (a) backup count/outcome is not continuously visible — it lives only in Profile Tools → Backups; (b) `Clear User Logs` (`settings_tab.rs:208`) is destructive but has no confirm dialog; (c) Undo (`mod.rs:1266`) is a backup restore with no confirm; (d) "Max All Skills" (`mod.rs:1731-1759`) applies six writes in one click with no confirm, though per-skill failures are aggregated.

---

## 7. UX pain points — information architecture only (not color/font/spacing)

1. **God-object state couples every screen.** `Ets2NativeApp` holds ~180 fields; `reload_active_save_data` (`:890-1031`) rewrites Save Editor, Vehicles and Coaches inputs in one pass, and `flush_pending_changes` reaches into garages and switcher state. A screen cannot be reasoned about (or tested) in isolation; hidden cross-screen invalidation rules (`invalidate_switcher_state`, coach snapshot clearing) must be remembered by the caller.
2. **Two-level navigation is flattened into one 214 px rail with 21 destinations.** Nine top-level tabs plus four sets of sub-tabs are all peers in the sidebar (`render_sidebar`, `:1327-1424`). There is no notion of "current task", no progressive disclosure, and no way to see where you are in a workflow beyond the highlighted row.
3. **Navigation is duplicated in three places.** The sidebar sets `active_tab` + sub-tab enums; Dashboard stat tiles and action cards also set them (`render_launcher_tab`, `:1520-1800`); the Save Health quick action additionally jumps to Profile Tools. Deep-link side effects are scattered and easy to desynchronize.
4. **Save safety is under-publicized.** The only always-on signal is a small `"! Unsaved changes"` text in a crowded top bar. Backup count, last backup outcome and staged-file count are absent from any persistent surface; the backup browser is buried two levels deep under SAFETY → Backups & Restore. A user editing money has no path to "show me what will be backed up" without leaving the task.
5. **Top bar mixes global context, navigation actions and save lifecycle.** GAME/PROFILE/SAVE combos, Reload, Undo, dirty label, Save and the inert LANG indicator all share one `horizontal_wrapped` row (`:1066-1325`). At the 960 px minimum window width this row wraps; critical actions (Save / dirty state) can move between rows, weakening the save-safety affordance.
6. **Workflow fragmentation across sub-tabs.** A single "vehicle" task spans Player Vehicle Editor (plates, fuel, wear, odometer), Truck Switcher, Trailer Switcher, Powertrain Swapper and Fleet Inventory. Similarly, "save health" is reachable from Dashboard and Profile Tools, "money/XP" from both Save Editor and Dashboard quick actions, and dispatcher job writing (Career) is separate from verifying the resulting save (Save Editor). There is no task-centric view.
7. **Error and log surfaces are split three ways.** Ephemeral toasts (5 s / 10 s), Dashboard RECENT ACTIVITY (last 5 events only), and Settings → Application Logs & Support Bundle. No notification center, no filterable global error list, no way to revisit a dismissed failure. Settings read failures are toast-only.
8. **Loading and empty states are inconsistent.** Some screens load on first visit (Career `refresh_career_data`), some require an explicit "Load …" button (Coaches, Garages, Mods "Load State"), some use `BackgroundLoad` without progress (garages, mods scan), and several show `"—"`/empty tables with no explanation of what must be loaded first. There is no global loading/error indicator tied to the selected save.
9. **Inert localization affordance.** A LANG label is permanently visible in the top bar but offers no selector and the entire UI is hardcoded English; `locales/*.json` and the language commands (`get_available_languages_command`, `get_current_language_command`) are loaded but unused by any screen. The indicator implies a capability the UI does not have.
10. **Confirm/discoverability inconsistencies.** Three destructive flows use the shared confirm dialog, but `Clear User Logs`, Undo (backup restore) and bulk "Max All Skills" do not; the unsaved-changes prompt is hand-rolled with different geometry and button styling. Users cannot predict which actions ask for confirmation.
11. **Staged-vs-written model is explained only in prose.** The dashboard footer ("Edits stage in memory until you press Save…") and Undo tooltip are the only education; the dispatcher write result has to spell out `was NOT written to quicksave (status: …)` / staged counts in a toast. A dedicated, always-visible save-state model would remove this ambiguity.
12. **Diagnostic and dataset management share one settings page.** Homedir/paths, SQLite table counts, ETS2 dataset import, user logs, diagnostic preview and support-bundle export are stacked vertically with no separation between "configuration", "data" and "troubleshooting", making recovery tasks hard to find under pressure.

### Coverage checks

- All 11 files under `src/gui/` are referenced: `mod.rs` (§2.1, §3.1, §4), `components.rs` (§2.2, §5), `theme.rs` (§2.2, §5), `save_editor_tab.rs` (§2.2, §3.8), `vehicles_tab.rs` (§2.2, §3.2), `garages_tab.rs` (§2.2, §3.3), `career_tab.rs` (§2.2, §3.4), `coaches_tab.rs` (§2.2, §3.5), `mods_tab.rs` (§2.2, §3.6), `profile_tools_tab.rs` (§2.2, §3.7), `settings_tab.rs` (§2.2, §3.9).
- All 47 `async_runtime::block_on(` sites are mapped: mod.rs 13 (rows 10,16,18,19,20,21,22,23,25,26,28,29,31), vehicles_tab.rs 19 (rows 35-53), garages_tab.rs 4 (row 54/55), career_tab.rs 2 (rows 81,84), mods_tab.rs 2 (rows 116,117), profile_tools_tab.rs 4 (rows 125,127,130,132), settings_tab.rs 3 (rows 151,158 and the second call site of row 151).
- Backend gate: see handoff note and final report; no source file was modified by this audit.
