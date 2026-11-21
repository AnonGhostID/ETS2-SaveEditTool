# Shared UI Contract — Orbit Hub foundation (Phase 2)

This document is the source of truth for screen agents. It describes the
primitives in `gui/components.rs` + `gui/components/`, the shell/screen
contract in `gui/shell/` and `gui/screens/mod.rs`, and the data-wrapper names
implemented in `gui/data/`. No backend call belongs in a screen.

## 0. Integration requirements (Phase 4 owner)

`ets2-tool/src-tauri/src/gui/mod.rs` must declare the new modules:

```rust
pub mod data;
pub mod screens;
pub mod shell;
```

`src/main.rs` keeps `initialize_backend()` untouched; Phase 4 swaps the render
root to `gui::shell::AppShell::new(cc, app)`. Until then the old and new UI
trees coexist.

## 1. Screen contract (`gui/screens/mod.rs`)

### 1.1 `Route` (30 exact snake_case variants)

`#[allow(non_camel_case_types)] pub enum Route`:

`overview_dashboard`, `overview_quick_actions`, `overview_recent_activity`,
`save_data_economy`, `save_data_coaches`, `save_data_config`,
`save_data_custom_keys`, `vehicles_player_truck`, `vehicles_player_trailer`,
`vehicles_truck_switcher`, `vehicles_trailer_switcher`, `vehicles_powertrain`,
`vehicles_fleet_inventory`, `operations_garages`, `operations_ai_drivers`,
`operations_live_telemetry`, `operations_dispatcher`,
`operations_logbook_analytics`, `operations_vtc_account`,
`library_mod_manager`, `library_presets`, `library_sandboxes`,
`library_conflict_analyzer`, `safety_save_health`, `safety_backups_restore`,
`safety_profile_clone_rename`, `safety_profile_sharing`, `system_paths_data`,
`system_logs_diagnostics`, `system_about`.

`Route` also exposes `sector()`, `label()`, `default_sub_view()`, `accepts()`.
`Sector` is `Overview | SaveData | Vehicles | Operations | Library | Safety |
System` with `label()`, `default_route()`, `routes()`; `ALL_SECTORS` is
exported for the Sector Map.

### 1.2 `SubView`

`pub enum SubView` has the exact variants from spec §1.4:

`None`, `DashboardHeader`, `DashboardCurrentSetup`, `DashboardStatTiles`,
`DashboardSectorCards`, `DashboardStagingNote`, `QuickActionsList`,
`ActivityFeed`, `EconomyFields`, `EconomySkills`, `EconomyCustomReset`,
`EconomyLevelPreview`, `CoachesOverview`, `CoachesStats`, `CoachesJobHistory`,
`CoachesStops`, `ConfigValues`, `BaseConfigKeys`, `SaveConfigKeys`,
`TruckFields`, `TruckFuelWear`, `TruckPlateOdometer`, `TrailerFields`,
`SwitchSession`, `SwitchList`, `SwitchPreview`, `SwitchApply`, `SwitchResult`,
`PowertrainCatalog`, `PowertrainPreview`, `FleetTrucks`, `FleetTrailers`,
`GarageList`, `GarageDetail`, `GarageWarnings`, `DriverGenerate`,
`DriverAssign`, `DriverDismiss`, `TelemetryLive`, `TelemetrySnapshot`,
`DispatcherJobs`, `DispatcherJobDetail`, `DispatcherGeneration`,
`LogbookTrips`, `LogbookJobLog`, `LogbookStats`, `LogbookAnalytics`,
`VtcSignIn`, `VtcCompany`, `VtcMembers`, `ModScan`, `ModList`, `ModWorkshop`,
`PresetList`, `PresetCompare`, `SandboxList`, `ConflictReport`, `HealthScan`,
`HealthReport`, `HealthFix`, `BackupList`, `BackupPreview`, `BackupConfirm`,
`CloneForm`, `RenameForm`, `CopyControls`, `CopyMods`, `ShareExport`,
`ShareImport`, `SharePreview`, `PathsStatus`, `SqliteInfo`, `DatasetImport`,
`LogsList`, `DiagnosticsPreview`, `LogExports`, `AboutInfo`.

An invalid sub-view for a route falls back to `route.default_sub_view()` in
`AppShell::navigate` — never a panic, never stale state.

### 1.3 Commands and state

```rust
pub enum IssuesTab { Validation, Errors, UserLog, Activity }

pub enum ContextAction {
    SwitchGame(String), SwitchProfile(String), SwitchSave(String),
    Reload, Exit,
}

pub enum ShellCommand {
    Navigate { route: Route, sub_view: Option<SubView> },
    OpenIssues(IssuesTab),
    Toast { message: String, is_error: bool },
    Save,
    RefreshSafety,
    RequestContextChange(ContextAction),
}

pub struct ScreenCtx<'a> {
    pub data: &'a crate::gui::data::DataContext,
    pub commands: &'a mut Vec<ShellCommand>,
    // Shell-owned, read-only for screens:
    pub route: Route,
    pub sub_view: SubView,
    /// Last five shell events, newest last (allocated by the shell each frame).
    pub recent_events: &'a [(String, String)],
    /// Selected save's display label, falling back to its path; "" if none.
    pub active_save_label: &'a str,
}

`route` and `sub_view` are assigned by `gui/shell/detail_pane.rs::dispatch`
every frame from `AppShell::route` / `AppShell::sub_view`. The shell is the
single authority: screens may read them to follow Sector Map / Safety Spine
deep links, but must never assign to them or push a `Navigate` command just
to switch their own tab. `dispatch` covers all eight screen renderers.

Every screen module exports exactly:

```rust
#[derive(Default)]
pub struct ScreenState { /* screen-local fields */ }

pub fn render(
    ui: &mut egui::Ui,
    state: &mut ScreenState,
    ctx: &mut ScreenCtx<'_>,
);
```

`ScreenStates` holds one `Default` field per module: `profile`, `economy`,
`fleet`, `garages`, `career`, `mods`, `settings`, `launcher`. Navigation swaps
which state is rendered; it never clears the others. Only a context change,
Save/Discard/restore or explicit invalidation resets tile state.

Example screen:

```rust
use crate::gui::components as ui;
use crate::gui::screens::{ScreenCtx, ShellCommand, Route, SubView};

pub fn render(ui_: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    if ui::button(ui_, "Save health").clicked() {
        ctx.commands.push(ShellCommand::Navigate {
            route: Route::safety_save_health,
            sub_view: Some(SubView::HealthScan),
        });
    }
    if state.error.is_some() {
        ui::error_state(ui_, "Load failed", Some("Retry"));
    }
}
```

Validation messages must both render inline and reach the Issues drawer:
push a `ShellCommand::Toast { message: "Validation: …", is_error: false }`;
the shell routes the `Validation: ` prefix into the drawer's Validation tab.
Errors use `is_error: true` and are mirrored into the Errors tab.

### 1.4 Shell API

```rust
impl AppShell {
    pub fn new(cc: &eframe::CreationContext<'_>, app: AppHandle) -> Self;
    pub fn navigate(&mut self, route: Route, sub_view: Option<SubView>);
    pub fn open_issues(&mut self, tab: IssuesTab);
    pub fn show_sector_overview(&mut self);
    pub fn request_context_change(&mut self, action: ContextAction);
    pub fn resolve_unsaved_prompt(&mut self, save_first: bool);
    pub fn flush_pending_changes(&mut self);
    pub fn refresh_safety(&mut self);
    pub fn refresh_context(&mut self);
    pub fn push_toast(&mut self, message: impl Into<String>, is_error: bool);
}
```

`navigate()` is the only route/sector/sub-view mutator. In-app navigation
never prompts. Only GAME/PROFILE/SAVE, Reload and Exit go through
`request_context_change`, which checks `session::is_dirty()` and opens the
Save/Discard/Cancel prompt. A Save failure keeps staging and cancels the
pending change; Cancel restores the prior selector value.

`flush_pending_changes()` calls `data::session::flush` and refreshes the
Backups and Staged gauges, then resets screen-local caches.
`refresh_safety()` re-reads `dirty_paths`, staging state, undo status and the
backup list; failed backup listing keeps the last count and writes an Issues
error. A failed undo-status read disables Undo and pushes an error toast that
is mirrored into Issues. Screens refresh these gauges after screen-level
backup operations by pushing `ShellCommand::RefreshSafety`, which the shell
handles by calling `refresh_safety()`.

### 1.5 Regions

| Region | Module | Responsibility |
|---|---|---|
| Context Column | `gui/shell/context_column.rs` | Selectors, Reload, language status, context facts; compact 40 px row below 1280 px |
| Sector Map | `gui/shell/sector_map.rs` | Seven sector tiles → capability tiles; `[◀ All sectors]` |
| Detail Pane | `gui/shell/detail_pane.rs` | Breadcrumb, screen dispatch, scroll-to-top, validation summary |
| Safety Spine | `gui/shell/safety_spine.rs` | Backups, Staged, Save, Issues, Errors & Logs, Undo — every frame |
| Issues drawer | `gui/shell/issues_drawer.rs` | Four tabs; `Open full log view` deep link |
| Overlays | `gui/shell/overlays.rs` | Unsaved prompt, Undo confirm, focus handling |

The Safety Spine is never scrolled out of view. Save is disabled at zero
staged paths; Undo is disabled while staged or when the backend reports no
undo. Undo is confirmed because it is a backup restore.

The Issues drawer is rendered before the Detail Pane so its right `SidePanel`
reserves width for the central panel in the same frame.

## 2. Primitives

All are re-exported from `crate::gui::components`; legacy helpers
(`card`, `page_header`, `section_label`, `nav_section`, `nav_item`,
`primary_button`, `sub_tabs`, `stat_tile`, `clickable_stat_tile`,
`action_card`, `game_badge`, `empty_state`, `status_dot`, `confirm_dialog`)
remain source-compatible for the old tab files until Phase 4 deletes them.

| Primitive | Signature (abridged) | Contract |
|---|---|---|
| `button` | `(&mut Ui, &str) -> Response` | secondary action |
| `danger_button` | `(&mut Ui, &str) -> Response` | destructive styling only; still confirm |
| `text_input` | `(&mut Ui, &mut String, &str) -> Response` | single-line with hint |
| `text_area` | `(&mut Ui, &mut String, &str, usize) -> Response` | multiline notes/preview |
| `select` | `(&mut Ui, &str, &mut String, &[String]) -> bool` | string combo, unique id |
| `toggle` | `(&mut Ui, &str, &str, &mut bool) -> Response` | boolean checkbox |
| `tabs` | `(&mut Ui, T, &[(T, &str)]) -> Option<T>` | segmented control |
| `form_field` | `(&mut Ui, &str, impl FnOnce(&mut Ui)->R) -> R` | label above widget |
| `validation_message` | `(&mut Ui, &str)` | inline warning; empty = no-op |
| `table` | `(&mut Ui, &str, &[&str], &[Vec<String>], Option<usize>) -> Option<usize>` | fixed header, vertical scroll, row select |
| `section_card` | `(&mut Ui, &str, impl FnOnce(&mut Ui)->R) -> InnerResponse<R>` | card + caption |
| `compact_stat_tile` | `(&mut Ui, &str, &str, Color32) -> Response` | grid-safe tile (unlike legacy `stat_tile`) |
| `status_gauge` | `(&mut Ui, &str, &str, &str, Color32)` | Safety Spine chip |
| `progress_row` | `(&mut Ui, &str, Option<f32>)` | background load progress |
| `loading_state` | `(&mut Ui, &str)` | spinner row |
| `error_state` | `(&mut Ui, &str, Option<&str>) -> bool` | error card + optional Retry |
| `toast_stack` | `(&egui::Context, &mut VecDeque<Toast>)` | 5 s info / 10 s error, max 4, manual dismiss |
| `push_toast` | `(&mut VecDeque<Toast>, impl Into<String>, bool)` | dedupe + cap |
| `modal` | `(&egui::Context, &str, impl FnOnce(&mut Ui)->R) -> Option<R>` | centred non-resizable overlay |
| `button_with_id` | `(&mut Ui, egui::Id, &str, bool) -> Response` | button with an exact widget id (egui 0.29 has no `Button::id`); confirm dialog uses it for modal focus slots |
| `confirm_dialog` | `(&egui::Context, &str, &str, &str, &str) -> Option<bool>` | legacy two-button confirm; confirm id = `modal_focus_id(0)`, cancel id = `modal_focus_id(1)`, Escape = cancel |
| `modal_focus_id` | `(usize) -> egui::Id` | focus id for overlay control `index` |
| `trap_focus_step` | `(&egui::Context) -> i32` | consumes Tab/Shift+Tab; returns focus step |

Overlay buttons that need an exact focus target must use
`components::button_with_id(ui, components::modal_focus_id(index), "…", primary)`
because egui 0.29's `Button` builder has no `id` setter and `ui.push_id`
salts the generated id with the enclosing `Ui` stack. Shell overlays still
use `ui.push_id(components::modal_focus_id(index), …)` and therefore retain
their pre-existing focus-target mismatch; only the shared `confirm_dialog`
uses exact ids now.

`components::confirm_dialog` (the root dialog and all screen-owned dialogs)
interacts with `modal_focus_id(0)` for confirm and `modal_focus_id(1)` for
cancel, so `AppShell::handle_tabs` can target them. Escape while the dialog
is open returns `Some(false)` (cancel); the return type and all call sites
are unchanged.

Screen-owned dialogs are still detected from visible `Order::Middle` egui
layers. While one is open, `AppShell::handle_tabs` consumes Tab/Shift+Tab
and cycles the two slots. **Focus-restoration limitation:** egui 0.29 does
not expose the most recently clicked/interested widget id publicly and a
mouse click does not set `Memory::focused()`, so the shell can only capture
whatever was focused when the modal was detected. Exact restoration needs
the opening screen to announce its opener id, which requires screen-side
cooperation and is outside the foundation component's ownership; no
workaround is invented here. The best-effort shell detection is retained.

Loading/empty/error contract: every data view renders `loading_state` while
loading, `empty_state` when the backend returns no rows, and `error_state`
with Retry on failure. Errors are never toast-only — the shell mirrors every
error toast into the Issues drawer.

## 3. Data wrappers (names only)

Data access is synchronous, `Result<T, UiError>`-shaped, and lives in
`gui/data/`; all backend calls use `crate::tauri::async_runtime::block_on`
there. Wrappers are free functions taking `&DataContext` first:

```rust
let dirty = crate::gui::data::session::dirty_paths(&ctx.data);
if let Err(err) = crate::gui::data::economy::edit_money(&ctx.data, 1_000) {
    ctx.commands.push(ShellCommand::Toast {
        message: format!("Validation: {err}"), is_error: true,
    });
}
```

| Module | Wrappers |
|---|---|
| `mod` | `DataContext { pub app: AppHandle }`, `UiError { pub message: String }` |
| `loading` | `BackgroundLoad<T>` (`start`, `poll`, `is_loading`) |
| `session` | `is_dirty`, `dirty_paths`, `is_staging_enabled`, `flush`, `discard`, `undo_last` |
| `context` | `available_languages`, `current_language`, `set_game`, `find_profiles`, `read_last_profile`, `save_last_profile`, `set_active_profile`, `find_saves`, `load_profile`, `set_current_save`, `undo_status`, `read_base_config`, `read_save_config`, `read_all_save_data`, `try_read_truck_xp`, `quicksave_game_info`, `current_truck_summary`, `all_trucks`, `player_truck`, `all_trailers`, `player_trailer`, `calculate_level` |
| `quick_actions` | `repair_truck`, `refuel_truck`, `max_all_skills`, `scan_save_health`, `generate_market_jobs` |
| `economy` | `refresh_coaches_snapshot`, `edit_money`, `edit_experience`, `edit_level`, `apply_custom_reset`, `set_skill`, `set_traffic`, `set_developer`, `set_console`, `set_parking_doubles`, `set_convoy`, `set_base_config_key`, `set_save_config_key`, `level_progress` |
| `coaches` | `snapshot`, `apply_bus_xp`, `display_job_stop_name`, `display_stop_name`, `display_bus_name`, `bus_level` |
| `vehicles` | `all_trucks`, `player_truck`, `all_trailers`, `player_trailer`, `repair_truck`, `refuel_truck`, `set_fuel`, `set_wear`, `set_plate`, `set_odometer`, `repair_trailer`, `set_trailer_plate`, `set_cargo_mass`, `init_truck_switch`, `list_trucks_for_switch`, `preview_truck_switch`, `apply_truck_switch`, `init_trailer_switch`, `list_trailers_for_switch`, `preview_trailer_switch`, `apply_trailer_switch`, `powertrain_catalog`, `preview_powertrain` |
| `garages` | `list_all_garages`, `ai_driver_management`, `generate_drivers`, `purchase_garage`, `upgrade_garage`, `assign_random_resources`, `update_garage`, `sell_garage`, `sell_empty_garages`, `buy_all_garages`, `remove_drivers` |
| `career` | `status`, `plugin_status`, `overview`, `dispatcher_overview`, `generation_status`, `market_jobs`, `active_jobs`, `list_trips`, `job_log`, `job_stats`, `analytics_summary`, `generate_jobs`, `generate_universal_jobs`, `snapshot_refresh_active_save`, `cleanup_expired_jobs`, `accept_job`, `assign_prepare_write`, `cancel_job`, `scan_profile_job_history`, `export_analytics_csv` |
| `vtc` | `current_user_profile`, `company_overview`, `company_members`, `logout`, `login`, `create_company` |
| `library` | `load_manager_state`, `open_steam_console`, `scan_mods_deep`, `scan_mods_light`, `delete_preset`, `delete_sandbox`, `list_presets`, `choose_workshop_directory`, `clear_workshop_directory`, `open_workshop_page`, `import_preset`, `create_preset`, `compare_preset`, `export_preset`, `load_sandboxes`, `create_sandbox`, `apply_sandbox`, `add_mod_to_sandbox`, `analyze_conflicts`, `analyze_conflicts_deep`, `export_conflict_report` |
| `safety` | `validate_clone_target`, `clone_profile`, `rename_profile`, `copy_controls`, `copy_mods`, `share_context`, `pick_export_directory`, `export_shared_profile`, `pick_import_archive`, `inspect_archive`, `import_shared_profile`, `scan_save_health`, `apply_health_fix`, `list_backups`, `preview_backup`, `restore_backup` |
| `system` | `data_location`, `sqlite_info`, `sqlite_table_counts`, `log_status`, `user_logs`, `user_logs_filtered`, `log_preview`, `choose_data_location`, `reset_data_location`, `set_data_location_mode`, `import_datasets`, `export_user_logs`, `export_logs_bundle`, `export_log_file`, `clear_user_logs` |

## 4. Enforcement

No file under `gui/screens/` may import `crate::features`, `crate::commands`
or `async_runtime`; the same applies to `gui/shell/` and the new component
modules. Backend access is exclusively in `gui/data/`.
