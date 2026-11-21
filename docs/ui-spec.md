# UI Specification — Phase 1, Part B (Chosen Concept: Orbit Hub)

## 0. Contract and decisions

**Source of truth:** `docs/ui-concepts.md` Concept 2 (Orbit Hub), `docs/ui-audit.md` (all nine tabs, 13 sub-tabs, launcher flows, dialogs, 159 grouped backend-call rows, save-safety map, PP#1–PP#12), and `docs/status.md` (recorded pick). This spec is the implementation contract for Phases 2–5. No backend code, no new crate, no new command.

**Frozen-backend decisions made in scope:**

- The `LANG` control stays a **status-only indicator** showing `get_current_language_command` and `get_available_languages_command` data; no selector is invented because no setter command exists. Text remains hardcoded English, as today. This is explicitly recorded as unchanged, not a missing feature (PP#9 is knowingly preserved).
- Powertrain Swapper exposes catalog + preview only; the audited surface has no apply command. No fake Apply button is added.
- Three previously unconfirmed destructive actions now use the shared confirm dialog, frontend-only: `Undo` (backup restore), `Clear User Logs`, and `Max All Skills` (six writes in one click). This closes audit gap §6(b)/(c)/(d) and PP#10 without backend work.
- All mutations keep their backend semantics: edits stage in `save_session` until `Save`; a pre-write backup is created by `save_pending_changes`; switchers and clone carry the existing backup toggles. The shell only surfaces these facts; it never reimplements them.

**Naming:** route enum variants are snake_case identifiers as required by this spec; annotate `#[allow(non_camel_case_types)]` on `pub enum Route` in `gui/screens/mod.rs`. Sub-view variants are PascalCase Rust enums owned by their screen module.

---

## 1. Navigation map

### 1.1 Window regions (roles)

| Region | Role | Contents |
|---|---|---|
| Context Column (left) | Global save context, never a navigation authority | GAME / PROFILE / SAVE selectors, `Reload`, language status, context facts (profile, save, level, truck summary, staged count, health summary) |
| Sector Map (center) | First navigation level | Seven sector tiles. Selecting a sector swaps the grid to that sector's capability tiles; `[All sectors]` returns to the seven-sector overview |
| Detail Pane (right) | Second navigation level | Breadcrumb (`SECTOR › TILE › sub-view`), fields / tables / previews, inline validation, actions, progress and empty/error states for **exactly one** capability |
| Safety Spine (far right, always visible) | Save-safety authority | Backups gauge (count + last outcome), Staged gauge (dirty path count), Save staged edits, Issues gauge, Errors & Logs, Undo |
| Global overlays | Modal / transient | Unsaved-changes prompt, destructive confirm dialog, toast stack, Issues drawer |

### 1.2 Sectors, capability tiles and exact route enum variants

Thirty capability tiles; exactly one `Route` variant per tile. The table below is the complete enum to implement in `gui/screens/mod.rs`.

| Sector | Capability tile | Exact `Route` variant | Default sub-view | Tile purpose |
|---|---|---|---|---|
| Overview | Dashboard | `overview_dashboard` | `DashboardHeader` | Profile/save header, Current Setup, eight stat tiles, sector cards, staging note |
| Overview | Quick Actions | `overview_quick_actions` | `QuickActionsList` | Repair + Refuel, Max All Skills, Scan Save Health, Generate Market Jobs |
| Overview | Recent Activity | `overview_recent_activity` | `ActivityFeed` | Last five background events plus event errors |
| Save Data | Economy & Experience | `save_data_economy` | `EconomyFields` | Money, XP, level, level preview, six skills, custom reset |
| Save Data | Project Coaches | `save_data_coaches` | `CoachesOverview` | Bus XP, bus stats, job history, discovered stops with search/filter |
| Save Data | Config Values | `save_data_config` | `ConfigValues` | Traffic, Developer, Console, Convoy, Parking Doubles |
| Save Data | Custom Keys | `save_data_custom_keys` | `BaseConfigKeys` | Base/save config key-value rows, `Refresh Save Data` |
| Vehicles | Player Truck | `vehicles_player_truck` | `TruckFields` | Repair/refuel, plate, odometer, fuel and wear sliders |
| Vehicles | Player Trailer | `vehicles_player_trailer` | `TrailerFields` | Repair, plate, cargo mass |
| Vehicles | Truck Switcher | `vehicles_truck_switcher` | `SwitchSession` | Session init, searchable owned-truck list, preview, apply with backup toggle |
| Vehicles | Trailer Switcher | `vehicles_trailer_switcher` | `SwitchSession` | Session init, searchable owned-trailer list, preview, apply with backup toggle |
| Vehicles | Powertrain Swapper | `vehicles_powertrain` | `PowertrainCatalog` | Official catalog, engine/transmission selection and preview |
| Vehicles | Fleet Inventory | `vehicles_fleet_inventory` | `FleetTrucks` | Searchable, sortable truck and trailer tables |
| Operations | Garages | `operations_garages` | `GarageList` | Filter/sort, warnings, buy/upgrade/HQ, sell/bulk with confirm |
| Operations | AI Drivers | `operations_ai_drivers` | `DriverGenerate` | Generate (count, assign trucks), assign/reassign, dismiss with confirm |
| Operations | Live Telemetry | `operations_live_telemetry` | `TelemetryLive` | Plugin status, 500 ms telemetry stream, snapshot refresh |
| Operations | Dispatcher | `operations_dispatcher` | `DispatcherJobs` | Generate standard/universal jobs, market/active jobs, accept/cancel, assign + prepare + write |
| Operations | Logbook & Analytics | `operations_logbook_analytics` | `LogbookTrips` | Trips, job log, job stats, history scan, analytics CSV export |
| Operations | VTC & Account | `operations_vtc_account` | `VtcSignIn` | Sign in/out, company overview, members, create company |
| Library | Mod Manager | `library_mod_manager` | `ModScan` | Light/deep background scan, manual Workshop directory, installed mods, Steam links |
| Library | Presets | `library_presets` | `PresetList` | Create, compare, export, import, delete with confirm |
| Library | Sandboxes | `library_sandboxes` | `SandboxList` | Create, add mod, apply to active profile, delete with confirm |
| Library | Conflict Analyzer | `library_conflict_analyzer` | `ConflictReport` | Standard/deep analysis, report view and export |
| Safety | Save Health | `safety_save_health` | `HealthScan` | Scan active save, report, one-click fixes with confirm |
| Safety | Backups & Restore | `safety_backups_restore` | `BackupList` | List, preview, restore with confirm |
| Safety | Profile Clone & Rename | `safety_profile_clone_rename` | `CloneForm` | Validate target, clone with backup and hex/text options, rename, copy controls/mods |
| Safety | Profile Sharing | `safety_profile_sharing` | `ShareExport` | ZIP export, archive pick, preview, import with confirm |
| System | Paths & Data | `system_paths_data` | `PathsStatus` | Homedir/data-location status, chooser/reset/mode, SQLite info, dataset import |
| System | Logs & Diagnostics | `system_logs_diagnostics` | `LogsList` | Log status, filterable user logs, diagnostic preview, exports, clear with confirm |
| System | About | `system_about` | `AboutInfo` | Version footer and language status indicator |

### 1.3 Deep-link router contract

Implement in `gui/shell/` and `gui/screens/mod.rs`:

```rust
#[allow(non_camel_case_types)]
pub enum Route { /* the 30 variants from §1.2, verbatim */ }

pub enum SubView { /* the variants listed in §1.4, verbatim */ }

impl Route {
    pub fn sector(self) -> Sector;   // Sector::{Overview,SaveData,Vehicles,Operations,Library,Safety,System}
    pub fn label(self) -> &'static str;
}

impl AppShell {
    /// Single navigation authority. Sets sector + tile + optional sub-view,
    /// replaces Detail Pane state with that tile's state, scrolls to top,
    /// and updates the breadcrumb. Never prompts, never discards staged edits.
    pub fn navigate(&mut self, route: Route, sub_view: Option<SubView>);
    /// Backups gauge, quick actions and dashboard deep links all call navigate().
    pub fn open_issues(&mut self, tab: IssuesTab); // drawer, not a route
}
```

Router rules:

1. `navigate()` is the **only** way any code sets sector, tile or sub-view. There are no separate `active_tab` / sub-tab enum mutations.
2. Every sector tile and stat tile calls `navigate(route, Some(sub_view))`; selecting a sector from the Sector Map calls `navigate(default_route_for_sector, None)`.
3. An invalid `SubView` for a route falls back to that route's default sub-view; no panic and no stale state.
4. In-app navigation **never** blocks on staged edits. Only Context Column changes (GAME/PROFILE/SAVE), `Reload` and Exit go through `request_context_change(action)`, which checks `save_session::is_dirty()` and opens the unsaved prompt (`Save` / `Discard` / `Cancel`). `Save` failure keeps staging and cancels the pending action; `Cancel` restores the prior game selection.
5. Leaving and re-entering a tile preserves that tile's local state (search text, filters, selected row). Only context changes, Save/Discard/restore and explicit invalidation (switcher hash staleness) reset tile state.
6. `open_issues(IssuesTab)` opens the drawer without a route change; its `Open full log view` action calls `navigate(Route::system_logs_diagnostics, Some(SubView::LogsList))`.

### 1.4 `SubView` variants and allowed tiles

`pub enum SubView` variants (PascalCase): `None`, `DashboardHeader`, `DashboardCurrentSetup`, `DashboardStatTiles`, `DashboardSectorCards`, `DashboardStagingNote`, `QuickActionsList`, `ActivityFeed`, `EconomyFields`, `EconomySkills`, `EconomyCustomReset`, `EconomyLevelPreview`, `CoachesOverview`, `CoachesStats`, `CoachesJobHistory`, `CoachesStops`, `ConfigValues`, `BaseConfigKeys`, `SaveConfigKeys`, `TruckFields`, `TruckFuelWear`, `TruckPlateOdometer`, `TrailerFields`, `SwitchSession`, `SwitchList`, `SwitchPreview`, `SwitchApply`, `SwitchResult`, `PowertrainCatalog`, `PowertrainPreview`, `FleetTrucks`, `FleetTrailers`, `GarageList`, `GarageDetail`, `GarageWarnings`, `DriverGenerate`, `DriverAssign`, `DriverDismiss`, `TelemetryLive`, `TelemetrySnapshot`, `DispatcherJobs`, `DispatcherJobDetail`, `DispatcherGeneration`, `LogbookTrips`, `LogbookJobLog`, `LogbookStats`, `LogbookAnalytics`, `VtcSignIn`, `VtcCompany`, `VtcMembers`, `ModScan`, `ModList`, `ModWorkshop`, `PresetList`, `PresetCompare`, `SandboxList`, `ConflictReport`, `HealthScan`, `HealthReport`, `HealthFix`, `BackupList`, `BackupPreview`, `BackupConfirm`, `CloneForm`, `RenameForm`, `CopyControls`, `CopyMods`, `ShareExport`, `ShareImport`, `SharePreview`, `PathsStatus`, `SqliteInfo`, `DatasetImport`, `LogsList`, `DiagnosticsPreview`, `LogExports`, `AboutInfo`.

Allowed sets: `overview_dashboard` → Dashboard*; `overview_quick_actions` → QuickActionsList; `overview_recent_activity` → ActivityFeed; `save_data_economy` → Economy*; `save_data_coaches` → Coaches* (its old segmented tabs become the stop filter control inside `CoachesStops`); `save_data_config` → ConfigValues; `save_data_custom_keys` → BaseConfigKeys/SaveConfigKeys; `vehicles_player_truck` → Truck*; `vehicles_player_trailer` → TrailerFields; either switcher → Switch*; `vehicles_powertrain` → Powertrain*; `vehicles_fleet_inventory` → Fleet*; `operations_garages` → Garage*; `operations_ai_drivers` → Driver*; `operations_live_telemetry` → Telemetry*; `operations_dispatcher` → Dispatcher*; `operations_logbook_analytics` → Logbook*; `operations_vtc_account` → Vtc*; `library_mod_manager` → Mod*; `library_presets` → Preset*; `library_sandboxes` → SandboxList; `library_conflict_analyzer` → ConflictReport; `safety_save_health` → Health*; `safety_backups_restore` → Backup*; `safety_profile_clone_rename` → Clone*/Rename*/Copy*; `safety_profile_sharing` → Share*; `system_paths_data` → Paths*/Sqlite*/Dataset*; `system_logs_diagnostics` → Logs*/Diagnostics*/LogExports; `system_about` → AboutInfo.

### 1.5 Deep-link sources (replaces the three scattered navigation mutators)

| Old deep-link source | New call |
|---|---|
| Dashboard money/XP stat tile | `navigate(save_data_economy, Some(EconomyFields))` |
| Dashboard truck stat tile | `navigate(vehicles_player_truck, Some(TruckFields))` |
| Dashboard fuel/wear stat tile | `navigate(vehicles_player_truck, Some(TruckFuelWear))` |
| Dashboard trailer stat tile | `navigate(vehicles_player_trailer, Some(TrailerFields))` |
| Dashboard garages-owned stat tile | `navigate(operations_garages, Some(GarageList))` |
| Dashboard AI-drivers stat tile | `navigate(operations_ai_drivers, Some(DriverGenerate))` |
| Dashboard Save Health quick action | `navigate(safety_save_health, Some(HealthScan))` |
| Dashboard Generate Market Jobs quick action | `navigate(operations_dispatcher, Some(DispatcherGeneration))` |
| Dashboard Repair + Refuel quick action | stays on `overview_quick_actions`, runs `quick_actions::repair/refuel` |
| Dashboard Max All Skills quick action | stays on `overview_quick_actions`, confirm then `quick_actions::max_all_skills` |
| Dashboard OPEN A SCREEN cards | `navigate(default_route_for_sector, None)` with `Some(SubView::None)` |
| Safety Spine Backups gauge | `navigate(safety_backups_restore, Some(BackupList))` |
| Safety Spine Errors & Logs | `open_issues(IssuesTab::Errors)`; drawer tab `Validation` routes to the originating tile |
| Issues drawer `Open full log view` | `navigate(system_logs_diagnostics, Some(LogsList))` |
| Save Health report `Apply fix` | confirm on `safety_save_health` / `HealthFix` |

---

## 2. Page hierarchy, responsive collapse and focus order

### 2.1 Large window (>= 1280 x 820, reference 1280 x 820)

```
┌───────────────┬──────────────────────────────────────────────┬───────────────────────────────┬──────────────┐
│ CONTEXT       │ SECTOR: VEHICLES              [◀ All sectors]│ DETAIL                        │ SAFETY SPINE │
│ ┌───────────┐ │ ┌──────────┐ ┌──────────┐ ┌──────────┐       │ Vehicles › Truck Switcher     │ ◷ Backups 4  │
│ │ GAME   ▾  │ │ │ Player   │ │ Trailer  │ │ Truck    │       │ ───────────────────────────── │ last OK      │
│ │ PROFILE▾  │ │ │ Truck    │ │ Editor   │ │ Switcher │◀      │ search ...                    │ [Backups]    │
│ │ SAVE   ▾  │ │ └──────────┘ └──────────┘ └──────────┘       │ preview / warnings / hash     │ ◍ Staged 3   │
│ └───────────┘ │ ┌──────────┐ ┌──────────┐ ┌──────────┐       │ ☑ create backup on switch     │ [Save staged]│
│ Reload        │ │ Trailer  │ │ Powertrain│ │ Fleet    │       │ inline validation ...         │ ⚠ Issues 2   │
│ Lang: EN      │ │ Switcher │ │ Swapper  │ │ Inventory│       │ [Apply]                       │ [Errors&Logs]│
├───────────────┤ └──────────┘ └──────────┘ └──────────┘       ├───────────────────────────────┤ [Undo]       │
│ Context facts │ Breadcrumb: Vehicles › Truck Switcher        │ validation summary / progress │              │
│ save health   │                                              │                               │              │
└───────────────┴──────────────────────────────────────────────┴───────────────────────────────┴──────────────┘
```

Region widths at 1280 px: Context 200 px, Sector Map 360 px (three 108 px tiles per row), Detail flexible (~540 px), Safety Spine 160 px. At >=1600 px the Detail Pane grows; Sector Map caps at 420 px and Context Column at 220 px. All regions use `ScrollArea` vertically; only wide data tables scroll horizontally. The Safety Spine never scrolls out of view.

### 2.2 Minimum window (960 x 640) collapse rules

1. **Context Column collapses** to a 40 px row above the Sector Map: three compact combos, `Reload`, language badge, and a one-line context fact (`ETS2 · QA Copy · QA Save · 3 staged`). The full fact list is available in a `Context` popover anchored in that row. Selectors and Reload remain keyboard reachable and keep their unsaved guards.
2. **Sector Map narrows** to 210 px with one tile per row; the header keeps `[◀ All sectors]`. Tile labels wrap to at most two lines; no icon-only labels.
3. **Detail Pane** receives the remaining width (~590 px at 960 px after the 120 px collapsed spine). `ui.columns()` is not used; forms use single-column `Grid`/`Label` + widget rows. Tables switch to vertical scroll with a fixed header row. No horizontal page scroll.
4. **Safety Spine stays visible** but collapses from 160 px to 120 px: gauges become stacked compact chips (`Backups 4 OK`, `Staged 3`, `Issues 2`), `Save staged edits`, `Backups`, `Errors & Logs` and `Undo` stay as labelled buttons (icons alone are not sufficient). Tooltips carry the explanatory text that no longer fits.
5. If the viewport drops below 960 px (smaller than the declared minimum), the Detail Pane is the first region to shrink and displays a `Resize the window to at least 960x640` banner; no control is clipped or silently hidden.
6. Height 640 px: the Context row is 40 px, the bottom breadcrumb/progress line 28 px; the rest is split between Sector Map and Detail Pane with independent vertical scrolling. Overlays are centred and internally scrollable.

### 2.3 Focus order and modal behaviour

1. Tab order: Context selectors (Game → Profile → Save) → Reload → Sector tiles (row-major) → capability tiles (row-major) → `[All sectors]` → Detail Pane breadcrumb → Detail fields/tables/previews in visual order → Detail actions in visual order → Safety Spine (Backups gauge → Save staged edits → Issues gauge → Errors & Logs → Undo) → Issues drawer when open.
2. Shift+Tab reverses the same order. Focus must be visible on every interactive control; no focus is lost when a tile swaps the Sector Map or when a drawer opens/closes. After closing the drawer or a modal, focus returns to the control that opened it.
3. Issues drawer and both modal overlays trap Tab/Shift+Tab inside themselves while open; Escape closes the drawer, means `Cancel` in the unsaved prompt, and means `No` in a confirm dialog.
4. Save is disabled (not merely greyed by colour) when `dirty_paths()` is empty. Undo is disabled when `!undo_status.can_undo || is_dirty()`.
5. Background scans show a progress row in the Detail Pane and do not steal focus; completion does not reorder controls.

---

## 3. Capability → new location mapping (complete)

Every capability in `docs/ui-audit.md` has a row: all 159 grouped backend-call rows (`C-001`…`C-159`, numbered identically to the audit) and every screen, launcher flow, sub-tab, dialog and shell surface not already named by a call row (`U-001`…`U-050`). `—` in the Route column means a shell surface or global overlay reached by a shell action, not a route; its new location is still explicit. Unmapped rows: 0.

| ID | Audit ref | Capability | Old home | New location (Sector › Tile › Sub-view) | Route variant | Data wrapper |
|---|---|---|---|---|---|---|
| C-001 | §3.1 #1 | Available languages query | Shell new | Shell › Context Column › Language status | `—` | `context::available_languages` |
| C-002 | §3.1 #2 | Current language query | Top bar LANG | Shell › Context Column › Language status | `—` | `context::current_language` |
| C-003 | §3.1 #3 | Save staged edits (backup + write) | Top bar Save | Shell › Safety Spine › Save action | `—` | `session::flush` |
| C-004 | §3.1 #4 | Set selected game | Top bar GAME | Shell › Context Column › Game selector | `—` | `context::set_game` |
| C-005 | §3.1 #5 | Discard pending changes | Unsaved prompt Discard | Shell › Unsaved prompt › Discard | `—` | `session::discard` |
| C-006 | §3.1 #6 | Find ETS2 profiles | Shell refresh | Shell › Context Column › Profile selector | `—` | `context::find_profiles` |
| C-007 | §3.1 #7 | Read last profile | Shell refresh | Shell › Context Column › Profile selector | `—` | `context::read_last_profile` |
| C-008 | §3.1 #8 | Save last profile | Shell select_profile | Shell › Context Column › Profile selector | `—` | `context::save_last_profile` |
| C-009 | §3.1 #9 | Set active profile | Shell select_profile | Shell › Context Column › Profile selector | `—` | `context::set_active_profile` |
| C-010 | §3.1 #10 | Find profile saves | Shell select_profile | Shell › Context Column › Save selector | `—` | `context::find_saves` |
| C-011 | §3.1 #11 | Load profile | Shell select_save | Shell › Context Column › Save selector | `—` | `context::load_profile` |
| C-012 | §3.1 #12 | Set current save | Shell select_save | Shell › Context Column › Save selector | `—` | `context::set_current_save` |
| C-013 | §3.1 #13 | Undo status | Shell reload | Shell › Safety Spine › Undo | `—` | `context::undo_status` |
| C-014 | §3.1 #14 | Read base config | Shell reload | Save Data › Custom Keys › Base config keys | `save_data_custom_keys` | `context::read_base_config` |
| C-015 | §3.1 #15 | Read save config | Shell reload | Save Data › Custom Keys › Save config keys | `save_data_custom_keys` | `context::read_save_config` |
| C-016 | §3.1 #16 | Read all save data | Shell reload | Shell › Context Column › Context facts | `—` | `context::read_all_save_data` |
| C-017 | §3.1 #17 | Try read truck XP | Shell reload | Overview › Dashboard › Current Setup | `overview_dashboard` | `context::try_read_truck_xp` |
| C-018 | §3.1 #18 | Quicksave game info | Shell reload | Overview › Dashboard › Header | `overview_dashboard` | `context::quicksave_game_info` |
| C-019 | §3.1 #19 | Current truck summary | Shell reload | Overview › Dashboard › Current Setup | `overview_dashboard` | `context::current_truck_summary` |
| C-020 | §3.1 #20 | Get all trucks | Shell reload | Vehicles › Fleet Inventory › Trucks | `vehicles_fleet_inventory` | `vehicles::all_trucks` |
| C-021 | §3.1 #21 | Get player truck | Shell reload | Vehicles › Player Truck › Fields | `vehicles_player_truck` | `vehicles::player_truck` |
| C-022 | §3.1 #22 | Get all trailers | Shell reload | Vehicles › Fleet Inventory › Trailers | `vehicles_fleet_inventory` | `vehicles::all_trailers` |
| C-023 | §3.1 #23 | Get player trailer | Shell reload | Vehicles › Player Trailer › Fields | `vehicles_player_trailer` | `vehicles::player_trailer` |
| C-024 | §3.1 #24 | Get ETS2 data location | Settings refresh | System › Paths & Data › Paths status | `system_paths_data` | `system::data_location` |
| C-025 | §3.1 #25 | SQLite info | Settings refresh | System › Paths & Data › SQLite info | `system_paths_data` | `system::sqlite_info` |
| C-026 | §3.1 #26 | SQLite table counts | Settings refresh | System › Paths & Data › SQLite info | `system_paths_data` | `system::sqlite_table_counts` |
| C-027 | §3.1 #27 | Undo last save change | Top bar Undo | Shell › Safety Spine › Undo (confirm) | `—` | `session::undo_last` |
| C-028 | §3.1 #28 | Repair player truck | Dashboard quick action | Overview › Quick Actions › Actions | `overview_quick_actions` | `quick_actions::repair_truck` |
| C-029 | §3.1 #29 | Refuel player truck | Dashboard quick action | Overview › Quick Actions › Actions | `overview_quick_actions` | `quick_actions::refuel_truck` |
| C-030 | §3.1 #30 | Max All Skills (six writes) | Dashboard quick action | Overview › Quick Actions › Actions (confirm) | `overview_quick_actions` | `quick_actions::max_all_skills` |
| C-031 | §3.1 #31 | Active save health scan | Dashboard quick action | Safety › Save Health › Scan | `safety_save_health` | `safety::scan_save_health` |
| C-032 | §3.1 #32 | Generate market jobs | Dashboard quick action | Operations › Dispatcher › Generation | `operations_dispatcher` | `quick_actions::generate_market_jobs` |
| C-033 | §3.1 #33 | Calculate level (shell) | Shell reload / dashboard | Shell › Context Column › Context facts | `—` | `context::calculate_level` |
| C-034 | §3.1 #34 | Dirty state + dirty paths | Top bar dirty label | Shell › Safety Spine › Staged gauge | `—` | `session::is_dirty`, `session::dirty_paths` |
| C-035 | §3.2 #35 | Repair player truck (vehicle screen) | Vehicles › Player Vehicle | Vehicles › Player Truck › Fields | `vehicles_player_truck` | `vehicles::repair_truck` |
| C-036 | §3.2 #36 | Refuel player truck (vehicle screen) | Vehicles › Player Vehicle | Vehicles › Player Truck › Fields | `vehicles_player_truck` | `vehicles::refuel_truck` |
| C-037 | §3.2 #37 | Set truck fuel level | Vehicles › Player Vehicle | Vehicles › Player Truck › Fuel & wear | `vehicles_player_truck` | `vehicles::set_fuel` |
| C-038 | §3.2 #38 | Set truck wear | Vehicles › Player Vehicle | Vehicles › Player Truck › Fuel & wear | `vehicles_player_truck` | `vehicles::set_wear` |
| C-039 | §3.2 #39 | Set truck license plate | Vehicles › Player Vehicle | Vehicles › Player Truck › Plate & odometer | `vehicles_player_truck` | `vehicles::set_plate` |
| C-040 | §3.2 #40 | Edit truck odometer | Vehicles › Player Vehicle | Vehicles › Player Truck › Plate & odometer | `vehicles_player_truck` | `vehicles::set_odometer` |
| C-041 | §3.2 #41 | Repair player trailer | Vehicles › Player Trailer | Vehicles › Player Trailer › Fields | `vehicles_player_trailer` | `vehicles::repair_trailer` |
| C-042 | §3.2 #42 | Set trailer license plate | Vehicles › Player Trailer | Vehicles › Player Trailer › Fields | `vehicles_player_trailer` | `vehicles::set_trailer_plate` |
| C-043 | §3.2 #43 | Set trailer cargo mass | Vehicles › Player Trailer | Vehicles › Player Trailer › Fields | `vehicles_player_trailer` | `vehicles::set_cargo_mass` |
| C-044 | §3.2 #44 | Init truck change session | Vehicles › Truck Switcher | Vehicles › Truck Switcher › Session | `vehicles_truck_switcher` | `vehicles::init_truck_switch` |
| C-045 | §3.2 #45 | List owned trucks for switch | Vehicles › Truck Switcher | Vehicles › Truck Switcher › List | `vehicles_truck_switcher` | `vehicles::list_trucks_for_switch` |
| C-046 | §3.2 #46 | Preview active truck switch | Vehicles › Truck Switcher | Vehicles › Truck Switcher › Preview | `vehicles_truck_switcher` | `vehicles::preview_truck_switch` |
| C-047 | §3.2 #47 | Apply active truck switch + backup | Vehicles › Truck Switcher | Vehicles › Truck Switcher › Apply | `vehicles_truck_switcher` | `vehicles::apply_truck_switch` |
| C-048 | §3.2 #48 | Init trailer change session | Vehicles › Trailer Switcher | Vehicles › Trailer Switcher › Session | `vehicles_trailer_switcher` | `vehicles::init_trailer_switch` |
| C-049 | §3.2 #49 | List owned trailers for switch | Vehicles › Trailer Switcher | Vehicles › Trailer Switcher › List | `vehicles_trailer_switcher` | `vehicles::list_trailers_for_switch` |
| C-050 | §3.2 #50 | Preview active trailer switch | Vehicles › Trailer Switcher | Vehicles › Trailer Switcher › Preview | `vehicles_trailer_switcher` | `vehicles::preview_trailer_switch` |
| C-051 | §3.2 #51 | Apply active trailer switch + backup | Vehicles › Trailer Switcher | Vehicles › Trailer Switcher › Apply | `vehicles_trailer_switcher` | `vehicles::apply_trailer_switch` |
| C-052 | §3.2 #52 | Official powertrain catalog | Vehicles › Powertrain | Vehicles › Powertrain Swapper › Catalog | `vehicles_powertrain` | `vehicles::powertrain_catalog` |
| C-053 | §3.2 #53 | Preview powertrain change | Vehicles › Powertrain | Vehicles › Powertrain Swapper › Preview | `vehicles_powertrain` | `vehicles::preview_powertrain` |
| C-054 | §3.3 #54 | Get all garages | Garages & Drivers | Operations › Garages › List | `operations_garages` | `garages::list_all_garages` |
| C-055 | §3.3 #55 | AI driver management | Garages & Drivers | Operations › AI Drivers › Assign | `operations_ai_drivers` | `garages::ai_driver_management` |
| C-056 | §3.3 #56 | Generate AI drivers | Garages detail | Operations › AI Drivers › Generate | `operations_ai_drivers` | `garages::generate_drivers` |
| C-057 | §3.3 #57 | Purchase garage | Garages detail | Operations › Garages › Detail | `operations_garages` | `garages::purchase_garage` |
| C-058 | §3.3 #58 | Upgrade owned garage | Garages detail | Operations › Garages › Detail | `operations_garages` | `garages::upgrade_garage` |
| C-059 | §3.3 #59 | Random garage resources | Garages detail | Operations › Garages › Detail | `operations_garages` | `garages::assign_random_resources` |
| C-060 | §3.3 #60 | Update garage (HQ/size) | Garages detail | Operations › Garages › Detail | `operations_garages` | `garages::update_garage` |
| C-061 | §3.3 #61 | Sell garage (confirm) | Garages confirm | Operations › Garages › Detail → global confirm | `operations_garages` | `garages::sell_garage` |
| C-062 | §3.3 #62 | Sell empty garages (confirm) | Garages bulk | Operations › Garages › List → global confirm | `operations_garages` | `garages::sell_empty_garages` |
| C-063 | §3.3 #63 | Buy all garages (confirm) | Garages bulk | Operations › Garages › List → global confirm | `operations_garages` | `garages::buy_all_garages` |
| C-064 | §3.3 #64 | Remove/dismiss AI drivers (confirm) | Garages confirm | Operations › AI Drivers › Dismiss → global confirm | `operations_ai_drivers` | `garages::remove_drivers` |
| C-065 | §3.4 #65 | Career status | Career refresh | Operations › Live Telemetry › Live | `operations_live_telemetry` | `career::status` |
| C-066 | §3.4 #66 | Plugin status | Career refresh | Operations › Live Telemetry › Live | `operations_live_telemetry` | `career::plugin_status` |
| C-067 | §3.4 #67 | Career overview | Career refresh | Operations › Live Telemetry › Live | `operations_live_telemetry` | `career::overview` |
| C-068 | §3.4 #68 | Dispatcher overview | Career refresh | Operations › Dispatcher › Jobs | `operations_dispatcher` | `career::dispatcher_overview` |
| C-069 | §3.4 #69 | Dispatcher generation status | Career refresh | Operations › Dispatcher › Generation | `operations_dispatcher` | `career::generation_status` |
| C-070 | §3.4 #70 | Market jobs | Career Dispatcher | Operations › Dispatcher › Jobs | `operations_dispatcher` | `career::market_jobs` |
| C-071 | §3.4 #71 | Active dispatcher jobs | Career Dispatcher | Operations › Dispatcher › Jobs | `operations_dispatcher` | `career::active_jobs` |
| C-072 | §3.4 #72 | List trips | Career Logbook | Operations › Logbook & Analytics › Trips | `operations_logbook_analytics` | `career::list_trips` |
| C-073 | §3.4 #73 | Job log | Career Logbook | Operations › Logbook & Analytics › Job log | `operations_logbook_analytics` | `career::job_log` |
| C-074 | §3.4 #74 | Job stats | Career Logbook | Operations › Logbook & Analytics › Stats | `operations_logbook_analytics` | `career::job_stats` |
| C-075 | §3.4 #75 | Analytics summary | Career Analytics | Operations › Logbook & Analytics › Analytics | `operations_logbook_analytics` | `career::analytics_summary` |
| C-076 | §3.4 #76 | Current user profile | Career VTC | Operations › VTC & Account › Sign in | `operations_vtc_account` | `vtc::current_user_profile` |
| C-077 | §3.4 #77 | Company overview | Career VTC | Operations › VTC & Account › Company | `operations_vtc_account` | `vtc::company_overview` |
| C-078 | §3.4 #78 | Company members | Career VTC | Operations › VTC & Account › Members | `operations_vtc_account` | `vtc::company_members` |
| C-079 | §3.4 #79 | Generate standard jobs | Career Dispatcher | Operations › Dispatcher › Generation | `operations_dispatcher` | `career::generate_jobs` |
| C-080 | §3.4 #80 | Generate universal jobs | Career Dispatcher | Operations › Dispatcher › Generation | `operations_dispatcher` | `career::generate_universal_jobs` |
| C-081 | §3.4 #81 | ETS2 snapshot refresh | Career Telemetry | Operations › Live Telemetry › Snapshot | `operations_live_telemetry` | `career::snapshot_refresh_active_save` |
| C-082 | §3.4 #82 | Cleanup expired jobs | Career Dispatcher | Operations › Dispatcher › Jobs | `operations_dispatcher` | `career::cleanup_expired_jobs` |
| C-083 | §3.4 #83 | Accept dispatcher job | Career job detail | Operations › Dispatcher › Job detail | `operations_dispatcher` | `career::accept_job` |
| C-084 | §3.4 #84 | Assign, prepare and write job | Career job detail | Operations › Dispatcher › Job detail | `operations_dispatcher` | `career::assign_prepare_write` |
| C-085 | §3.4 #85 | Cancel dispatcher job | Career job detail | Operations › Dispatcher › Job detail | `operations_dispatcher` | `career::cancel_job` |
| C-086 | §3.4 #86 | Scan profile job history | Career Analytics | Operations › Logbook & Analytics › Analytics | `operations_logbook_analytics` | `career::scan_profile_job_history` |
| C-087 | §3.4 #87 | Export analytics CSV | Career Analytics | Operations › Logbook & Analytics › Analytics | `operations_logbook_analytics` | `career::export_analytics_csv` |
| C-088 | §3.4 #88 | Auth logout | Career VTC | Operations › VTC & Account › Sign in | `operations_vtc_account` | `vtc::logout` |
| C-089 | §3.4 #89 | Auth login | Career VTC | Operations › VTC & Account › Sign in | `operations_vtc_account` | `vtc::login` |
| C-090 | §3.4 #90 | Create company | Career VTC | Operations › VTC & Account › Company | `operations_vtc_account` | `vtc::create_company` |
| C-091 | §3.4 #91 | Staging enabled state | Career job detail label | Shell › Safety Spine › Staged gauge | `—` | `session::is_staging_enabled` |
| C-092 | §3.5 #92 | Read project coaches snapshot | Coaches reload | Save Data › Project Coaches › Overview | `save_data_coaches` | `coaches::snapshot` |
| C-093 | §3.5 #93 | Edit bus career XP | Coaches apply | Save Data › Project Coaches › Overview | `save_data_coaches` | `coaches::apply_bus_xp` |
| C-094 | §3.5 #94 | Display job stop name | Coaches job history | Save Data › Project Coaches › Job history | `save_data_coaches` | `coaches::display_job_stop_name` |
| C-095 | §3.5 #95 | Display stop name | Coaches stops | Save Data › Project Coaches › Stops | `save_data_coaches` | `coaches::display_stop_name` |
| C-096 | §3.5 #96 | Display bus name | Coaches labels | Save Data › Project Coaches › Overview | `save_data_coaches` | `coaches::display_bus_name` |
| C-097 | §3.5 #97 | Calculate bus level | Coaches level display | Save Data › Project Coaches › Overview | `save_data_coaches` | `coaches::bus_level` |
| C-098 | §3.6 #98 | Load mod manager state | Mods Manager | Library › Mod Manager › Scan | `library_mod_manager` | `library::load_manager_state` |
| C-099 | §3.6 #99 | Open Steam console | Mods Manager | Library › Mod Manager › Workshop | `library_mod_manager` | `library::open_steam_console` |
| C-100 | §3.6 #100 | Deep mod scan | Mods Manager | Library › Mod Manager › Scan | `library_mod_manager` | `library::scan_mods_deep` |
| C-101 | §3.6 #101 | Light mod scan | Mods Manager | Library › Mod Manager › Scan | `library_mod_manager` | `library::scan_mods_light` |
| C-102 | §3.6 #102 | Delete mod preset (confirm) | Mods confirm | Library › Presets › List → global confirm | `library_presets` | `library::delete_preset` |
| C-103 | §3.6 #103 | Delete mod sandbox (confirm) | Mods confirm | Library › Sandboxes › List → global confirm | `library_sandboxes` | `library::delete_sandbox` |
| C-104 | §3.6 #104 | List mod presets | Mods Presets | Library › Presets › List | `library_presets` | `library::list_presets` |
| C-105 | §3.6 #105 | Choose manual Workshop directory | Mods Presets | Library › Mod Manager › Workshop | `library_mod_manager` | `library::choose_workshop_directory` |
| C-106 | §3.6 #106 | Clear manual Workshop directory | Mods Presets | Library › Mod Manager › Workshop | `library_mod_manager` | `library::clear_workshop_directory` |
| C-107 | §3.6 #107 | Open Workshop page | Mods Manager | Library › Mod Manager › Workshop | `library_mod_manager` | `library::open_workshop_page` |
| C-108 | §3.6 #108 | Import mod preset | Mods Presets | Library › Presets › List | `library_presets` | `library::import_preset` |
| C-109 | §3.6 #109 | Create mod preset | Mods Presets | Library › Presets › List | `library_presets` | `library::create_preset` |
| C-110 | §3.6 #110 | Compare mod preset | Mods Presets | Library › Presets › Compare | `library_presets` | `library::compare_preset` |
| C-111 | §3.6 #111 | Export mod preset | Mods Presets | Library › Presets › List | `library_presets` | `library::export_preset` |
| C-112 | §3.6 #112 | Load mod sandboxes | Mods Presets | Library › Sandboxes › List | `library_sandboxes` | `library::load_sandboxes` |
| C-113 | §3.6 #113 | Create mod sandbox | Mods Presets | Library › Sandboxes › List | `library_sandboxes` | `library::create_sandbox` |
| C-114 | §3.6 #114 | Apply sandbox to active profile | Mods Presets | Library › Sandboxes › List (confirm) | `library_sandboxes` | `library::apply_sandbox` |
| C-115 | §3.6 #115 | Add mod to sandbox | Mods Presets | Library › Sandboxes › List | `library_sandboxes` | `library::add_mod_to_sandbox` |
| C-116 | §3.6 #116 | Standard conflict analysis | Mods Analyzer | Library › Conflict Analyzer › Report | `library_conflict_analyzer` | `library::analyze_conflicts` |
| C-117 | §3.6 #117 | Deep conflict analysis | Mods Analyzer | Library › Conflict Analyzer › Report | `library_conflict_analyzer` | `library::analyze_conflicts_deep` |
| C-118 | §3.6 #118 | Export conflict report | Mods Analyzer | Library › Conflict Analyzer › Report | `library_conflict_analyzer` | `library::export_conflict_report` |
| C-119 | §3.7 #119 | Validate clone target | Profile Clone | Safety › Profile Clone & Rename › Clone | `safety_profile_clone_rename` | `safety::validate_clone_target` |
| C-120 | §3.7 #120 | Clone profile with backup options | Profile Clone | Safety › Profile Clone & Rename › Clone | `safety_profile_clone_rename` | `safety::clone_profile` |
| C-121 | §3.7 #121 | Rename active profile | Profile Clone | Safety › Profile Clone & Rename › Rename | `safety_profile_clone_rename` | `safety::rename_profile` |
| C-122 | §3.7 #122 | Copy profile controls | Profile Clone | Safety › Profile Clone & Rename › Copy controls | `safety_profile_clone_rename` | `safety::copy_controls` |
| C-123 | §3.7 #123 | Copy mods to profile | Profile Clone | Safety › Profile Clone & Rename › Copy mods | `safety_profile_clone_rename` | `safety::copy_mods` |
| C-124 | §3.7 #124 | Profile share context | Profile Sharing | Safety › Profile Sharing › Export | `safety_profile_sharing` | `safety::share_context` |
| C-125 | §3.7 #125 | Pick ZIP export directory | Profile Sharing | Safety › Profile Sharing › Export | `safety_profile_sharing` | `safety::pick_export_directory` |
| C-126 | §3.7 #126 | Export shared profile ZIP | Profile Sharing | Safety › Profile Sharing › Export | `safety_profile_sharing` | `safety::export_shared_profile` |
| C-127 | §3.7 #127 | Pick import archive | Profile Sharing | Safety › Profile Sharing › Import | `safety_profile_sharing` | `safety::pick_import_archive` |
| C-128 | §3.7 #128 | Inspect archive preview | Profile Sharing | Safety › Profile Sharing › Preview | `safety_profile_sharing` | `safety::inspect_archive` |
| C-129 | §3.7 #129 | Import shared profile (confirm) | Profile Sharing | Safety › Profile Sharing › Import → global confirm | `safety_profile_sharing` | `safety::import_shared_profile` |
| C-130 | §3.7 #130 | Active save health scan | Profile Tools Health | Safety › Save Health › Scan | `safety_save_health` | `safety::scan_save_health` |
| C-131 | §3.7 #131 | Apply save health fix (confirm) | Profile Tools Health | Safety › Save Health › Fix → global confirm | `safety_save_health` | `safety::apply_health_fix` |
| C-132 | §3.7 #132 | List active save backups | Profile Tools Backups | Safety › Backups & Restore › List | `safety_backups_restore` | `safety::list_backups` |
| C-133 | §3.7 #133 | Preview backup restore | Profile Tools Backups | Safety › Backups & Restore › Preview | `safety_backups_restore` | `safety::preview_backup` |
| C-134 | §3.7 #134 | Restore backup (confirm) | Profile Tools Backups | Safety › Backups & Restore › Confirm | `safety_backups_restore` | `safety::restore_backup` |
| C-135 | §3.8 #135 | Refresh coaches from Save Editor | Save Editor Refresh | Save Data › Project Coaches › Overview | `save_data_coaches` | `economy::refresh_coaches_snapshot` |
| C-136 | §3.8 #136 | Edit player money | Save Editor Economy | Save Data › Economy & Experience › Fields | `save_data_economy` | `economy::edit_money` |
| C-137 | §3.8 #137 | Edit player XP | Save Editor Economy | Save Data › Economy & Experience › Fields | `save_data_economy` | `economy::edit_experience` |
| C-138 | §3.8 #138 | Edit level | Save Editor Economy | Save Data › Economy & Experience › Fields | `save_data_economy` | `economy::edit_level` |
| C-139 | §3.8 #139 | Apply custom reset values | Save Editor Economy | Save Data › Economy & Experience › Custom reset | `save_data_economy` | `economy::apply_custom_reset` |
| C-140 | §3.8 #140 | Edit six skill values | Save Editor Skills | Save Data › Economy & Experience › Skills | `save_data_economy` | `economy::set_skill` |
| C-141 | §3.8 #141 | Edit traffic value | Save Editor Config | Save Data › Config Values › Values | `save_data_config` | `economy::set_traffic` |
| C-142 | §3.8 #142 | Edit developer value | Save Editor Config | Save Data › Config Values › Values | `save_data_config` | `economy::set_developer` |
| C-143 | §3.8 #143 | Edit console value | Save Editor Config | Save Data › Config Values › Values | `save_data_config` | `economy::set_console` |
| C-144 | §3.8 #144 | Edit parking doubles value | Save Editor Config | Save Data › Config Values › Values | `save_data_config` | `economy::set_parking_doubles` |
| C-145 | §3.8 #145 | Edit convoy value | Save Editor Config | Save Data › Config Values › Values | `save_data_config` | `economy::set_convoy` |
| C-146 | §3.8 #146 | Edit base config key | Save Editor Custom keys | Save Data › Custom Keys › Base config | `save_data_custom_keys` | `economy::set_base_config_key` |
| C-147 | §3.8 #147 | Edit save config key | Save Editor Custom keys | Save Data › Custom Keys › Save config | `save_data_custom_keys` | `economy::set_save_config_key` |
| C-148 | §3.8 #148 | Calculate level preview | Save Editor level | Save Data › Economy & Experience › Level preview | `save_data_economy` | `economy::level_progress` |
| C-149 | §3.9 #149 | Log status | Settings logs | System › Logs & Diagnostics › Logs | `system_logs_diagnostics` | `system::log_status` |
| C-150 | §3.9 #150 | User logs with filter | Settings logs | System › Logs & Diagnostics › Logs | `system_logs_diagnostics` | `system::user_logs` |
| C-151 | §3.9 #151 | Diagnostic log preview | Settings logs | System › Logs & Diagnostics › Diagnostics | `system_logs_diagnostics` | `system::log_preview` |
| C-152 | §3.9 #152 | Choose ETS2 data location | Settings homedir | System › Paths & Data › Paths status | `system_paths_data` | `system::choose_data_location` |
| C-153 | §3.9 #153 | Reset ETS2 data location | Settings homedir | System › Paths & Data › Paths status | `system_paths_data` | `system::reset_data_location` |
| C-154 | §3.9 #154 | Set data-location mode | Settings homedir | System › Paths & Data › Paths status | `system_paths_data` | `system::set_data_location_mode` |
| C-155 | §3.9 #155 | Import ETS2 datasets | Settings SQLite | System › Paths & Data › Dataset import | `system_paths_data` | `system::import_datasets` |
| C-156 | §3.9 #156 | Export user logs | Settings logs | System › Logs & Diagnostics › Exports | `system_logs_diagnostics` | `system::export_user_logs` |
| C-157 | §3.9 #157 | Export logs bundle | Settings logs | System › Logs & Diagnostics › Exports | `system_logs_diagnostics` | `system::export_logs_bundle` |
| C-158 | §3.9 #158 | Export diagnostic log file | Settings logs | System › Logs & Diagnostics › Exports | `system_logs_diagnostics` | `system::export_log_file` |
| C-159 | §3.9 #159 | Clear user logs (confirm added) | Settings logs | System › Logs & Diagnostics › Logs → global confirm | `system_logs_diagnostics` | `system::clear_user_logs` |
| U-001 | §2.1 | App shell / state container | `Ets2NativeApp` | Shell › AppShell › regions | `—` | shell-owned, no wrapper |
| U-002 | §2.1 | Game / Profile / Save selectors | Top command bar | Shell › Context Column › Selectors | `—` | `context::*` |
| U-003 | §2.1 | Reload command + unsaved guard | Top command bar | Shell › Context Column › Reload | `—` | `context::*` + `session::is_dirty` |
| U-004 | §2.1 | Undo command | Top command bar | Shell › Safety Spine › Undo (confirm) | `—` | `session::undo_last` |
| U-005 | §2.1 | Dirty indicator + Save | Top command bar | Shell › Safety Spine › Staged + Save | `—` | `session::is_dirty`, `dirty_paths`, `flush` |
| U-006 | §1, §2.1 | Language status indicator | Top command bar | Shell › Context Column › Language status | `—` | `context::current_language` |
| U-007 | §2.1 | Left navigation rail | Sidebar | Shell › Sector Map › Sectors | `—` | shell-owned |
| U-008 | §2.1 | Toast stack | Top-right Area | Shell › Toast stack | `—` | shell-owned |
| U-009 | §2.1, §2.3 #1 | Unsaved-changes prompt | Modal Window | Shell › Unsaved prompt › Save/Discard/Cancel | `—` | `session::flush`, `session::discard` |
| U-010 | §2.1 | Exit guard | `update` close handling | Shell › Unsaved prompt › Exit | `—` | `session::is_dirty` |
| U-011 | §2.1 | Background event drain | `drain_background_events` | Shell › Issues drawer › Recent activity + Overview › Recent Activity | `overview_recent_activity` | shell event receiver |
| U-012 | §2.1 | Generic background loader | `BackgroundLoad<T>` | Shell › Detail Pane › progress row | `—` | `loading::*` |
| U-013 | §2.1 | Router | `update` central panel | Shell › Router › `navigate` | `—` | shell-owned |
| U-014 | §2.1 | Profile/save header card | Dashboard | Overview › Dashboard › Header | `overview_dashboard` | `context::quicksave_game_info` |
| U-015 | §2.1 | Current Setup card | Dashboard | Overview › Dashboard › Current Setup | `overview_dashboard` | `context::current_truck_summary` |
| U-016 | §2.1 | Eight stat tiles | Dashboard | Overview › Dashboard › Stat tiles | `overview_dashboard` | shell deep links §1.5 |
| U-017 | §2.1 | QUICK ACTIONS group | Dashboard | Overview › Quick Actions › Actions | `overview_quick_actions` | `quick_actions::*` |
| U-018 | §2.1 | OPEN A SCREEN cards | Dashboard | Overview › Dashboard › Sector cards | `overview_dashboard` | shell sector routes |
| U-019 | §2.1 | RECENT ACTIVITY (last five) | Dashboard | Overview › Recent Activity › Events | `overview_recent_activity` | shell event receiver |
| U-020 | §2.1 | Staging reminder footer | Dashboard | Overview › Dashboard › Staging note + Spine | `overview_dashboard` | `session::dirty_paths` |
| U-021 | §2.2 | Vehicles sub-tab: Player Vehicle | Vehicles | Vehicles › Player Truck › Fields | `vehicles_player_truck` | `vehicles::*` |
| U-022 | §2.2 | Vehicles sub-tab: Truck Switcher | Vehicles | Vehicles › Truck Switcher › Session | `vehicles_truck_switcher` | `vehicles::*` |
| U-023 | §2.2 | Vehicles sub-tab: Trailer Switcher | Vehicles | Vehicles › Trailer Switcher › Session | `vehicles_trailer_switcher` | `vehicles::*` |
| U-024 | §2.2 | Vehicles sub-tab: Powertrain Swapper | Vehicles | Vehicles › Powertrain Swapper › Catalog | `vehicles_powertrain` | `vehicles::*` |
| U-025 | §2.2 | Vehicles sub-tab: Fleet Inventory | Vehicles | Vehicles › Fleet Inventory › Trucks | `vehicles_fleet_inventory` | `vehicles::*` |
| U-026 | §2.2 | Career sub-tab: Live Telemetry | Career | Operations › Live Telemetry › Live | `operations_live_telemetry` | `career::*` |
| U-027 | §2.2 | Career sub-tab: Dispatcher | Career | Operations › Dispatcher › Jobs | `operations_dispatcher` | `career::*` |
| U-028 | §2.2 | Career sub-tab: Logbook & Analytics | Career | Operations › Logbook & Analytics › Trips | `operations_logbook_analytics` | `career::*` |
| U-029 | §2.2 | Career sub-tab: VTC & Account | Career | Operations › VTC & Account › Sign in | `operations_vtc_account` | `vtc::*` |
| U-030 | §2.2 | Mods sub-tab: Mod Manager | Mods | Library › Mod Manager › Scan | `library_mod_manager` | `library::*` |
| U-031 | §2.2 | Mods sub-tab: Presets & Sandboxes | Mods | Library › Presets › List **and** Library › Sandboxes › List | `library_presets`, `library_sandboxes` | `library::*` |
| U-032 | §2.2 | Mods sub-tab: Conflict Analyzer | Mods | Library › Conflict Analyzer › Report | `library_conflict_analyzer` | `library::*` |
| U-033 | §2.2 | Profile Tools sub-tab: Clone & Rename | Profile Tools | Safety › Profile Clone & Rename › Clone | `safety_profile_clone_rename` | `safety::*` |
| U-034 | §2.2 | Profile Tools sub-tab: Profile Sharing | Profile Tools | Safety › Profile Sharing › Export | `safety_profile_sharing` | `safety::*` |
| U-035 | §2.2 | Profile Tools sub-tab: Save Health | Profile Tools | Safety › Save Health › Scan | `safety_save_health` | `safety::*` |
| U-036 | §2.2 | Profile Tools sub-tab: Backups & Restore | Profile Tools | Safety › Backups & Restore › List | `safety_backups_restore` | `safety::*` |
| U-037 | §2.2 | Coaches internal segmented stop tabs | Coaches stops | Save Data › Project Coaches › Stops (filter segmented control) | `save_data_coaches` | `coaches::display_stop_name` |
| U-038 | §2.3 #2 | Garage destructive confirm | Garage confirm Window | Shell › Confirm dialog (Garages) | `operations_garages` | shell confirm + `garages::*` |
| U-039 | §2.3 #2 | Mod delete confirm | Mods confirm Window | Shell › Confirm dialog (Library) | `library_presets`, `library_sandboxes` | shell confirm + `library::*` |
| U-040 | §2.3 #2 | Backup restore confirm | Backup confirm Window | Shell › Confirm dialog (Safety) | `safety_backups_restore` | shell confirm + `safety::restore_backup` |
| U-041 | Concept 2 | Issues drawer (validation/errors/log/activity) | New surface | Shell › Issues drawer › four tabs | `—` | `system::user_logs` + shell events |
| U-042 | Concept 2 | Safety Spine gauges and actions | New surface | Shell › Safety Spine | `—` | `session::*`, `safety::list_backups` |
| U-043 | §7 PP#8 | Explicit load controls (Coaches/Garages/Mods) | Per-screen Load buttons | Save Data › Coaches › Overview, Operations › Garages › List, Library › Mod Manager › Scan | `save_data_coaches`, `operations_garages`, `library_mod_manager` | `coaches::*`, `garages::*`, `library::*` |
| U-044 | §6 gap (b) | Clear User Logs confirm | Settings (no confirm) | System › Logs & Diagnostics › Logs → global confirm | `system_logs_diagnostics` | shell confirm + `system::clear_user_logs` |
| U-045 | §6 gap (c) | Undo restore confirm | Top bar (no confirm) | Shell › Safety Spine › Undo → global confirm | `—` | shell confirm + `session::undo_last` |
| U-046 | §6 gap (d) | Max All Skills confirm | Dashboard (no confirm) | Overview › Quick Actions › Actions → global confirm | `overview_quick_actions` | shell confirm + `quick_actions::max_all_skills` |
| U-047 | §2.4 | Snapshot refresh flow | Career Telemetry | Operations › Live Telemetry › Snapshot | `operations_live_telemetry` | `career::snapshot_refresh_active_save` |
| U-048 | §2.2 | Save Editor `Refresh Save Data` | Save Editor | Save Data › Custom Keys › Refresh | `save_data_custom_keys` | `context::read_all_save_data`, `economy::refresh_coaches_snapshot` |
| U-049 | §6 | Toast dedupe / TTL / manual dismiss | `set_status` | Shell › Toast stack | `—` | shell-owned |
| U-050 | §6 | Plugin install error event surfacing | Toast only | Shell › Issues drawer › Errors + toast | `operations_live_telemetry` | shell event receiver |

---

## 4. Per-tile states and visible save-safety affordance

Every tile below defines all four states. The Safety Spine (Backups, Staged, Save, Issues, Errors & Logs, Undo) is visible on **all** rows without exception; the last column names the additional tile-specific affordance. Inline validation never appears only as a toast: it also writes an entry to the Issues drawer `Validation` tab.

| Tile (route) | Loading | Empty | Error | Validation | Tile-specific safety affordance |
|---|---|---|---|---|---|
| Dashboard (`overview_dashboard`) | Skeleton cards while `quicksave_game_info`/`current_truck_summary` load | `No save selected — choose Game, Profile and Save` | Card-level error with `Retry`; failure also in Issues | Level/context facts show `—`, never stale values | Staging note + pointer to Save in Spine |
| Quick Actions (`overview_quick_actions`) | Per-action busy state with disabled buttons | `No quick actions available for this context` | Per-action error under its button + Issues | Confirm required before Max All Skills; Scan Health deep-links | Confirm dialog; toast outcome; Spine refresh |
| Recent Activity (`overview_recent_activity`) | `Loading recent events…` | `No activity recorded for this save` | `Activity stream unavailable` + Issues | n/a | Spine unchanged; event errors mirrored in Issues |
| Economy (`save_data_economy`) | `Loading save values…` | `No save data loaded` | Field group error + Retry; keep last valid values read-only | Money 0..=999999999999, XP 0..=999999999, level 0..=256, ADR 0..63, other skills 0..6 | Staged gauge increments; Save in Spine; switcher caches invalidated after Save |
| Coaches (`save_data_coaches`) | `Loading bus career…` | `No bus career data`; explicit Reload action | Snapshot error + Retry; warnings rendered in card | Bus XP numeric; expected save hash checked before write | Staged gauge; confirm on XP apply only if backend reports overwrite risk |
| Config Values (`save_data_config`) | `Loading config values…` | `No config values found` | Group error + Retry | Integer range per key; backend clamp reason shown inline | Staged gauge; Save in Spine |
| Custom Keys (`save_data_custom_keys`) | `Loading base/save config…` | `No custom keys yet — add a key` | Read failure + Retry; write failure inline on the row | Non-empty key, duplicate-key warning | Staged gauge; Refresh invalidates caches |
| Player Truck (`vehicles_player_truck`) | Skeleton fields while `player_truck()` loads | `No player truck found` | Parse error card + Retry | Fuel/wear 0..1, plate non-empty, odometer 0..=99999999 | Staged gauge; Save in Spine; repair/refuel immediate outcome toast |
| Player Trailer (`vehicles_player_trailer`) | Skeleton fields | `No player trailer found` | Parse error + Retry | Plate non-empty, cargo mass 0..=200000 | Staged gauge; Save in Spine |
| Truck Switcher (`vehicles_truck_switcher`) | Session/list spinner, preview progress row | `No owned trucks available` | `safe_to_write=false` / stale hash card with `Reinitialize` | Expected hash compared at preview and apply; warnings block Apply | `create_backup_on_switch` toggle beside Apply; backup outcome in Spine/toast |
| Trailer Switcher (`vehicles_trailer_switcher`) | Session/list spinner, preview progress row | `No owned trailers available` | Stale hash card with `Reinitialize` | Expected hash compared at preview and apply | `create_backup_on_switch` toggle beside Apply; outcome in Spine |
| Powertrain Swapper (`vehicles_powertrain`) | `Loading official catalog…` | `No catalog entries for this game/version` | Catalog/preview error + Retry | Selection completeness before Preview; preview warnings shown inline | Preview is read-only; Spine unchanged until user stages elsewhere |
| Fleet Inventory (`vehicles_fleet_inventory`) | Table skeleton with row-count placeholder | `No trucks/trailers found` | Load error + Retry | Search/sort input validation (no regex errors) | Read-only; nothing staged |
| Garages (`operations_garages`) | `BackgroundLoad` progress in Detail Pane | `No garages found for this save` | Load error clears stale list + Retry; per-garage warnings cards | Sell disabled unless `is_sellable`; expected save hash on every mutation | Confirm dialog for sell / sell-empty / buy-all; result toast; Spine refresh |
| AI Drivers (`operations_ai_drivers`) | Driver list loading row | `No AI drivers — generate some` | Load/mutation error + Retry | Count range, truck assignment availability | Confirm dialog for dismiss; result toast |
| Live Telemetry (`operations_live_telemetry`) | `Waiting for plugin/telemetry…` | `Telemetry not running` with plugin status | Status error in Issues + Retry | n/a | Snapshot refresh outcome in Issues + Spine unchanged |
| Dispatcher (`operations_dispatcher`) | Jobs/generation spinner | `No market or active jobs` | Generation/job error with `last_error_message` | Risk/bonus notes shown before accept; write result labelled staged vs written | Confirm for cleanup; write outcome in toast + Issues; staged count in Spine |
| Logbook & Analytics (`operations_logbook_analytics`) | Scan/CSV spinner | `No trips or job log entries` | Scan/export error + Retry | Filter validation | Export path toast; Issues entry on failure |
| VTC & Account (`operations_vtc_account`) | Auth/company spinner | `Not signed in` | Login/company error inline, never toast only | Email/password non-empty; company fields | Sign-out confirm; credential values never logged |
| Mod Manager (`library_mod_manager`) | Scan progress with scanned/total when available | `No mods detected` | Scan failure + Retry; manual Workshop fallback | Manual directory path validation | Scan is read-only; Workshop directory change outcome in toast + Issues |
| Presets (`library_presets`) | Preset list spinner | `No presets — create one` | Load/import/export error + Retry | Name/notes validation; compare needs two presets | Confirm dialog for delete; import preview before write |
| Sandboxes (`library_sandboxes`) | Sandbox list spinner | `No sandboxes — create one` | Load/apply error + Retry | Add-mod input validation; apply needs active profile | Confirm dialog for delete and apply-to-active-profile |
| Conflict Analyzer (`library_conflict_analyzer`) | Analysis progress row | `No conflicts detected` | Analysis error + Retry | Report export requires completed report | Export outcome in toast + Issues |
| Save Health (`safety_save_health`) | Scan progress row | `No issues found` | Scan error + Retry | Fix availability/fix id validated before apply | Confirm dialog per fix; backup count refreshed in Spine after fix |
| Backups & Restore (`safety_backups_restore`) | Backup list spinner | `No backups for this save` | List/preview error + Retry | Restore requires preview + confirmed flag | Confirm dialog naming backup id; Spine refreshed after restore; staged edits warned on restore |
| Profile Clone & Rename (`safety_profile_clone_rename`) | Validation/target spinner | `No clone target entered` | Validation/clone error inline + Issues | `validate_clone_target` must pass before clone; name non-empty | `backup` default true with explicit toggle; confirm before clone/rename/copy |
| Profile Sharing (`safety_profile_sharing`) | Context/archive spinner | `No export directory chosen` | Pick/inspect/import error + Retry | Archive preview must be inspected before import | Confirm dialog before import; export path toast |
| Paths & Data (`system_paths_data`) | Status/count spinner | `No custom path configured` | Read failure writes Issues entry and keeps old values | Custom homedir must exist; mode string restricted to backend values | Confirm before path mode change / reset / dataset import; Reload afterwards goes through unsaved prompt if dirty |
| Logs & Diagnostics (`system_logs_diagnostics`) | Log list/preview spinner | `No user log entries` | Read failure keeps old values + Issues (never toast only) | Filter/max-lines validation | Confirm dialog before Clear User Logs; exports write path to toast + Issues |
| About (`system_about`) | n/a | n/a | n/a | n/a | Version and language status read-only; Spine visible |

### 4.1 Unsaved-prompt guards by edit-capable page

| Pages that can stage or write save data | Guard contract |
|---|---|
| `save_data_economy`, `save_data_coaches`, `save_data_config`, `save_data_custom_keys`, `vehicles_player_truck`, `vehicles_player_trailer`, `vehicles_truck_switcher`, `vehicles_trailer_switcher`, `operations_garages`, `operations_ai_drivers`, `operations_dispatcher`, `library_sandboxes`, `safety_save_health`, `safety_backups_restore`, `safety_profile_clone_rename`, `safety_profile_sharing` | Unsaved prompt on GAME/PROFILE/SAVE change, Reload and Exit **only**; in-app tile navigation is unblocked; destructive actions go through the shared confirm dialog; Spine Save/Staged/Backups/Issues always visible |
| `overview_quick_actions` (repair/refuel/max skills) | Same guards; Max All Skills additionally requires confirm; per-skill failures listed in Issues |
| `system_paths_data`, `system_logs_diagnostics` | Path mode/reset/dataset import and Clear Logs require confirm; Reload/Exit still guarded if save edits are staged; log config changes do not silently discard staged save edits |
| All read-only tiles (Dashboard, Recent Activity, Fleet Inventory, Telemetry, Logbook, VTC, Mod Manager, Presets, Conflict Analyzer, Powertrain, About) | No staging; navigation never prompts; Spine still tracks global staged count from other tiles |

### 4.2 Backup and error visibility rules

1. On shell start and after every Save, restore, switch apply, clone, fix apply and profile/save change, `safety::list_backups()` refreshes the Backups gauge (`N / last outcome`). Failure to list keeps the previous value and writes an Issues error.
2. `session::dirty_paths()` refreshes the Staged gauge after every wrapper that stages edits and after Save/Discard/restore. Save is disabled at zero staged paths.
3. Toasts are transient (info 5 s, error 10 s, max 4, manual dismiss, dedupe). Every error toast also appends to the Issues drawer; settings/log read failures are Issues-first, toast-optional.
4. The Issues drawer has tabs `Validation`, `Errors`, `User logs`, `Recent activity`; `User logs` uses `system::user_logs`, `Recent activity` uses the shell event receiver (last 50 events, last five mirrored on Overview).

---

## 5. Typed data-layer contract (`gui/data/`)

Enforcement (Phase 2): no file under `gui/screens/` may import `crate::features`, `crate::commands` or `async_runtime`. All backend access lives here. Each wrapper is a synchronous `pub fn` on `DataContext` returning `Result<T, UiError>` (`UiError { message: String }`), obtains state with `self.app.state::<T>()` and uses `crate::tauri::async_runtime::block_on(...)` for async commands. Cached save context is refreshed centrally after context changes and after Save/Discard/restore. Long-running loads use `BackgroundLoad<T>` relocated unchanged into `gui/data/loading.rs`.

| Module | Wrappers (audit rows) | Backend call groups covered |
|---|---|---|
| `gui/data/mod.rs` | `DataContext`, `UiError`, shared helpers | n/a — definitions only |
| `gui/data/loading.rs` | `BackgroundLoad<T>` (from `gui/mod.rs:141-183`, unchanged) | n/a — thread/mpsc loader used by garages, mods scan |
| `gui/data/session.rs` | `is_dirty`, `dirty_paths`, `is_staging_enabled`, `flush`, `discard`, `undo_last` | 3, 5, 27, 34, 91 — `save_session` helpers, `save_pending_changes`, `discard_pending_changes`, `undo_last_save_change` |
| `gui/data/context.rs` | `available_languages`, `current_language`, `set_game`, `find_profiles`, `read_last_profile`, `save_last_profile`, `set_active_profile`, `find_saves`, `load_profile`, `set_current_save`, `undo_status`, `read_base_config`, `read_save_config`, `read_all_save_data`, `try_read_truck_xp`, `quicksave_game_info`, `current_truck_summary`, `all_trucks`, `player_truck`, `all_trailers`, `player_trailer`, `calculate_level` | 1, 2, 4, 6–26, 33 — language, profile/save selection, save snapshot/config/vehicle readers, level helper |
| `gui/data/quick_actions.rs` | `repair_truck`, `refuel_truck`, `max_all_skills`, `scan_save_health`, `generate_market_jobs` | 28–32 — launcher quick actions; delegates to `vehicles`, `economy`, `safety`, `career` wrappers |
| `gui/data/economy.rs` | `refresh_coaches_snapshot`, `edit_money`, `edit_experience`, `edit_level`, `apply_custom_reset`, `set_skill`, `set_traffic`, `set_developer`, `set_console`, `set_parking_doubles`, `set_convoy`, `set_base_config_key`, `set_save_config_key`, `level_progress` | 135–148, 148 — save editor economy/skills/config/custom keys + level helper |
| `gui/data/coaches.rs` | `snapshot`, `apply_bus_xp`, `display_job_stop_name`, `display_stop_name`, `display_bus_name`, `bus_level` | 92–97 — coaches snapshot, bus XP, formatting helpers, level helper |
| `gui/data/vehicles.rs` | `all_trucks`, `player_truck`, `all_trailers`, `player_trailer`, `repair_truck`, `refuel_truck`, `set_fuel`, `set_wear`, `set_plate`, `set_odometer`, `repair_trailer`, `set_trailer_plate`, `set_cargo_mass`, `init_truck_switch`, `list_trucks_for_switch`, `preview_truck_switch`, `apply_truck_switch`, `init_trailer_switch`, `list_trailers_for_switch`, `preview_trailer_switch`, `apply_trailer_switch`, `powertrain_catalog`, `preview_powertrain` | 20–23, 35–53 — player vehicle/trailer editors, truck/trailer switch session, powertrain catalog/preview |
| `gui/data/garages.rs` | `list_all_garages`, `ai_driver_management`, `generate_drivers`, `purchase_garage`, `upgrade_garage`, `assign_random_resources`, `update_garage`, `sell_garage`, `sell_empty_garages`, `buy_all_garages`, `remove_drivers` | 54–64 — garage list/mutations, AI driver generate/dismiss |
| `gui/data/career.rs` | `status`, `plugin_status`, `overview`, `dispatcher_overview`, `generation_status`, `market_jobs`, `active_jobs`, `list_trips`, `job_log`, `job_stats`, `analytics_summary`, `generate_jobs`, `generate_universal_jobs`, `snapshot_refresh_active_save`, `cleanup_expired_jobs`, `accept_job`, `assign_prepare_write`, `cancel_job`, `scan_profile_job_history`, `export_analytics_csv` | 65–75, 79–87 — career status/telemetry/dispatcher/logbook/analytics |
| `gui/data/vtc.rs` | `current_user_profile`, `company_overview`, `company_members`, `logout`, `login`, `create_company` | 76–78, 88–90 — VTC/account/auth |
| `gui/data/library.rs` | `load_manager_state`, `open_steam_console`, `scan_mods_deep`, `scan_mods_light`, `delete_preset`, `delete_sandbox`, `list_presets`, `choose_workshop_directory`, `clear_workshop_directory`, `open_workshop_page`, `import_preset`, `create_preset`, `compare_preset`, `export_preset`, `load_sandboxes`, `create_sandbox`, `apply_sandbox`, `add_mod_to_sandbox`, `analyze_conflicts`, `analyze_conflicts_deep`, `export_conflict_report` | 98–118 — mod manager, presets, sandboxes, conflict analyzer |
| `gui/data/safety.rs` | `validate_clone_target`, `clone_profile`, `rename_profile`, `copy_controls`, `copy_mods`, `share_context`, `pick_export_directory`, `export_shared_profile`, `pick_import_archive`, `inspect_archive`, `import_shared_profile`, `scan_save_health`, `apply_health_fix`, `list_backups`, `preview_backup`, `restore_backup` | 31, 119–134 — profile clone/rename/copy, sharing, save health, backups/restore (row 31 reused via `quick_actions`) |
| `gui/data/system.rs` | `data_location`, `sqlite_info`, `sqlite_table_counts`, `log_status`, `user_logs`, `log_preview`, `choose_data_location`, `reset_data_location`, `set_data_location_mode`, `import_datasets`, `export_user_logs`, `export_logs_bundle`, `export_log_file`, `clear_user_logs` | 24–26, 149–159 — homedir/data location, SQLite/datasets, logs/diagnostics/exports |

Module-to-sector mapping for the screen agents: Overview → `quick_actions`, `context`; Save Data → `economy`, `coaches`, `context`; Vehicles → `vehicles`; Operations → `garages`, `career`, `vtc`; Library → `library`; Safety → `safety`, `session`; System → `system`; shell/global → `session`, `context`, `loading`. Every one of the 159 audit rows is covered by exactly one primary wrapper above (row 31 and rows 135/148 have documented delegate wrappers).

---

## 6. Coverage index (verification aid for orchestrator)

- Seven sectors, 30 capability tiles, 30 unique snake_case `Route` variants (§1.2).
- Mapping table: 159 backend-call rows + 50 screen/flow/dialog/shell rows = **209 rows**; unmapped rows = 0.
- Per-tile state table: 30 rows, one per tile; each has loading, empty, error, validation and a visible safety affordance.
- Unsaved-prompt guards specified for all 13 edit-capable tiles, plus the three configuration/log tiles and the read-only group.
- All 47 `block_on` call sites from `docs/ui-audit.md` are represented by their call rows (§3.1 13, §3.2 19, §3.3 4, §3.4 2, §3.6 2, §3.7 4, §3.9 3); wrappers use the same blocking boundary, no new async runtime.
- Every old navigation surface is replaced: flat sidebar → Sector Map, four sub-tab enums → tiles/sub-views, Dashboard tiles and Save Health jump → `navigate()` (§1.5).
