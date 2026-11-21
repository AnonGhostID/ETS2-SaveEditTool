# Sub-agent handoff notes

Each sub-agent appends one short note here when its work is done.

## AuditAgent — Phase 0 (read-only audit)

Delivered `docs/ui-audit.md` with all seven required parts: stack evidence, complete GUI inventory (all 11 `gui/` files), backend call table (159 grouped rows / ~188 call sites, all 47 `block_on` sites mapped), shared-state inventory, reusable vs layout-bound widgets, save-safety map, and IA-specific UX pain points. No source files modified; no builds or state-changing commands run. Blocker: none. Gate: backend diff clean (exact output recorded in final report).

## DesignAgent — Phase 1 Part A (information architecture)

Delivered `docs/ui-concepts.md` with three mutually exclusive, independently implementable concepts — Task Cockpit (workflow stepper + Task Switcher + bottom Save Ledger), Orbit Hub (sector map + progressive disclosure + safety spine), and Save Ledger (chronological staged/written audit trail + stations). Each includes rationale, complete sitemap covering every audited capability, main-window wireframe, PP#1–PP#12 grounded pros/cons, and explicit backup, staged/Save, and error/validation/log placement; plus a coverage index. No source or other docs modified; no builds/tests run. Blocker: none. Gate: backend diff clean.

## SpecAgent — Phase 1 Part B (chosen concept spec)

Delivered `docs/ui-spec.md` for Concept 2 (Orbit Hub): seven sectors, 30 capability tiles with 30 unique snake_case `Route` variants, single `navigate()` deep-link router, large/minimum window hierarchy with 960x640 collapse rules and focus order, a 209-row capability mapping (all 159 audit call rows + 50 shell/screen/sub-tab/dialog/flow rows) with zero unmapped rows, per-tile loading/empty/error/validation and safety affordances, unsaved-prompt guards, and the `gui/data/` wrapper contract covering all 159 backend call groups. Decisions recorded in scope: status-only LANG, no invented powertrain apply, and confirm dialogs added for Undo, Clear User Logs and Max All Skills. No source files or builds touched. Blocker: none. Gate: backend diff empty.

## FoundationData — Phase 2 (typed data layer)

Delivered 14 files under `ets2-tool/src-tauri/src/gui/data/`: `mod.rs` (`DataContext`, `UiError`), relocated `loading.rs` (`BackgroundLoad<T>` unchanged), and all 12 spec modules with 164 named wrappers plus `system::user_logs_filtered` for the filtered log view. Every audit row C-001..C-159 is reachable; all async calls use `crate::tauri::async_runtime::block_on` and all backend `String`/`AppError` failures map into `UiError { message }`. `session::dirty_paths` returns `Vec<String>`; `read_last_profile`/`undo_status` keep `Result` so errors are not swallowed (FoundationShell notified). No backend, shell, screen, component or tab files touched; no builds/tests run. Gate: backend diff empty. Note: `gui/mod.rs` must add `pub mod data;` (Phase 4 integration owner, confirmed by Main). Blocker: none.

## FoundationShell — Phase 2 (shell, shared components, screen contract)

Delivered `gui/shell/` (AppShell plus context column, sector map, detail pane, Safety Spine, Issues drawer and overlays), `gui/screens/mod.rs` (exact 30 snake_case `Route` variants, `Sector`, the full `SubView` set, `IssuesTab`, `ContextAction`, `ShellCommand`, `ScreenCtx`, `ScreenStates` and all eight screen module declarations), new primitives in `gui/components.rs` + `gui/components/{form,data,feedback}.rs` (legacy helper names preserved), and `docs/components.md` with the full contract, wrapper list and usage snippets. Shell uses only `crate::gui::data::*` for backend access; grep for `crate::features|crate::commands|async_runtime` in owned files is empty. Save is disabled at zero staged paths, Undo is disabled while staged and confirmed, Safety Spine renders every frame, and the unsaved/exit guard and modal focus trap are implemented. No builds/tests/clippy/fmt run. Integration note for Phase 4: `gui/mod.rs` must declare `pub mod data; pub mod screens; pub mod shell;`; screen agent files must provide `#[derive(Default)] ScreenState` and `render(ui, state, ctx)` for the declared modules. Blocker: none. Gate: backend diff reported in final message.

## ProfileScreen — Phase 3 pilot (Profile / Safety)

Delivered `ets2-tool/src-tauri/src/gui/screens/profile.rs` (~1776 lines) with the exact `#[derive(Default)] pub struct ScreenState` + `pub fn render(&mut egui::Ui, &mut ScreenState, &mut ScreenCtx<'_>)` contract. All four Safety tiles and their spec sub-views are present: Clone/Rename/CopyControls/CopyMods, Export/Import/Preview sharing, HealthScan/Report/Fix and BackupList/Preview/Confirm. Every wrapper in `gui/data/safety.rs` is called (validate, clone, rename, copy controls/mods, share context, pick export dir, export ZIP, pick archive, inspect, import, health scan/fix, backup list/preview/restore); profile list and staged state come from `data::context` / `data::session`. All destructive actions (clone, rename, copy controls, copy mods, import, health fix, backup restore) go through the shared `components::confirm_dialog`; restore additionally warns when `session::is_dirty`. Loading rows, empty states, inline validation, and `error_state` with Retry are rendered in every data view; every failure also pushes an error toast (mirrored into Issues by the shell) and validation pushes the `Validation: ` toast. No backend/component/shell/screen-contract files touched; no builds/tests run per assignment. Two shared-contract gaps logged in `docs/foundation-requests.md` (route visibility in `ScreenCtx`, focus trapping for screen-owned confirm dialogs). Parity rows appended. Gate: backend diff to be reported by the orchestrator; no backend paths edited.

## FoundationFix — Phase 3 pilot follow-up (shared navigation + modal focus)

Added `route: Route` and `sub_view: SubView` to `ScreenCtx` as shell-owned,
read-only fields; `detail_pane::dispatch` now takes the shell's `SubView` and
assigns both fields before all eight screen render calls. Added shell-side
screen-modal focus trapping: visible `Order::Middle` layers other than the
shell's own windows are treated as screen confirm dialogs, Tab/Shift+Tab are
consumed and cycled over the two modal slots, and focus is restored on close.
Documented both in `docs/components.md`; logged the two residual component-level
items (explicit button ids/Escape opener support) in `docs/foundation-requests.md`.
No builds/tests run per instruction. Gate: backend diff reported clean.

## FoundationData — Phase 3 follow-up (garage data-layer gaps)

Changed `gui/data/garages.rs` model import to `pub use` so Garages/AI Drivers screens can construct request/result types (including `GarageOwnership`, `GarageSize`, `AiDriverTruckAssignmentRequest`) via `crate::gui::data::garages::*` without importing `crate::features`. Added `update_driver_truck(&DataContext, AiDriverTruckAssignmentRequest) -> Result<AiDriverMutationResult, UiError>` delegating to `features::garages::commands::update_ai_driver_truck` with the six State args. No other file or backend path touched; no builds/tests run. Gate: backend diff empty.

## CareerScreen — Phase 3 fan-out (Operations sector)

Delivered `ets2-tool/src-tauri/src/gui/screens/career.rs` with the exact
`#[derive(Default)] pub struct ScreenState` + `pub fn render(&mut egui::Ui,
&mut ScreenState, &mut ScreenCtx<'_>)` contract. All four Operations routes and
their spec sub-views are present: Live/Save snapshot; Jobs/Generation/Job
detail; Trips/Job log/Stats/Analytics; Sign in/Company/Members. Every one of
the 20 `gui/data/career.rs` and six `gui/data/vtc.rs` wrappers is called;
staging state comes from `data::session::is_staging_enabled` and Save is a
`ShellCommand::Save`. The 500 ms repaint, bridge/plugin status, live sample,
snapshot refresh, market/active job search and sort, accept/cancel, confirmed
write/cleanup, risk/bonus/last-error detail, staged-vs-written message, trips,
job log, stats, analytics scan/export, auth sign in/out and VTC company
creation are implemented. Every data view has loading, empty, error+retry and
inline validation states; failures push an error toast and open the Issues
drawer. Destructive actions (write, cancel, cleanup, sign out) use the shared
confirm dialog. Only foundation primitives are used; no `crate::features`,
`crate::commands` or `async_runtime` imports. Three data-layer requests were
logged in `docs/foundation-requests.md`. No builds/tests run per instruction;
gate: backend diff empty (verified by exact command in final report).

## ModsScreen — Phase 3 fan-out (Library sector)

Delivered `ets2-tool/src-tauri/src/gui/screens/mods.rs` with the exact `#[derive(Default)] pub struct ScreenState` + `pub fn render(&mut egui::Ui, &mut ScreenState, &mut ScreenCtx<'_>)` contract. All four Library routes dispatch from `ctx.route` and their sub-views from a defensive `normalized_sub_view(ctx.route, ctx.sub_view)`; sub-view buttons only enqueue `ShellCommand::Navigate`. Covered audit rows C-098–C-118: manager load, light/deep background scan with a Detail Pane progress row and repaint timer, installed-mod table, Steam console and Workshop page links, manual Workshop folder set/clear, preset create/compare/export/import/delete, sandbox create/add-mod/apply/delete, and standard/deep conflict analysis with report view and export. Delete preset, delete sandbox and apply sandbox use the shared `components::confirm_dialog`; sandbox apply shows the backend backup path and a staged-edits warning. Failures push error toasts (mirrored into Issues) and inline `error_state` with Retry; validation pushes `Validation: ` toasts and inline messages. All data views have loading, empty, error and validation states.

Implementation notes: the screen never names a backend DTO — manager/preset/sandbox/compare/report values are converted into local view structs in the data-callback frame, and the conflict report is captured in a typed `data::UiError`-returning closure so `library::export_conflict_report` can be called without a `crate::features` import. Game arguments come from `ModProfileManagerState::summary.selected_game` captured in local state, with `None` passed to `list_presets` before a state load; create/compare/Workshop actions show inline validation until the active game is known. Scan progress is indeterminate because the backend exposes no scanned/total counters. No new backend capability and no shared-component change was needed, so `docs/foundation-requests.md` was left untouched. Per instruction no build, test, clippy or fmt was run; the file was reviewed by inspection. Grep for `crate::features|crate::commands|async_runtime` in the owned file is empty; backend gate reported clean.

## EconomyScreen — Phase 3 fan-out (Save Data sector)

Delivered `ets2-tool/src-tauri/src/gui/screens/economy.rs` with the exact `#[derive(Default)] pub struct ScreenState` + `pub fn render(&mut egui::Ui, &mut ScreenState, &mut ScreenCtx<'_>)` contract. All four Save Data routes dispatch from `ctx.route` and their sub-views from a defensive `normalized_sub_view`; sub-view buttons only enqueue `ShellCommand::Navigate`. Covered audit rows C-092–C-097, C-135–C-148 and U-037/U-043/U-048: money/XP/level editing with level preview and quick amounts, six skill sliders, confirmed Max All Skills, custom reset batch, five config values, base/save config key rows with duplicate-key confirm, Refresh Save Data, and bus career overview/statistics/job history/stops with search, filter and formatted names. Every data view has loading, empty, error+Retry and inline validation states; failures push an error toast mirrored into Issues by the shell, and validation pushes `Validation:` toasts. Confirm dialogs guard Max All Skills, duplicate-key overwrite and bus XP against an ambiguous snapshot. All mutations stage through `gui/data/`; no backup or Save logic is local. Only foundation primitives and typed wrappers are used; `crate::features`, `crate::commands` and `async_runtime` grep is empty. Decision: `ScreenCtx` exposes no active profile path, so `read_save_config` obtains it from the existing `data::safety::share_context` wrapper with `context::read_last_profile` fallback — no shared-component or data-layer change was needed. No backend requests. No builds/tests run per instruction; backend gate empty (exact command reported to Main).
