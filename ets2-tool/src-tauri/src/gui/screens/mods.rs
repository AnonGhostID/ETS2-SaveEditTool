//! Library sector screen: Mod Manager, Presets, Sandboxes and Conflict Analyzer.
//!
//! Only shared components and typed data wrappers are used here. Anything
//! belonging to the shell (breadcrumb, overlays, toasts, Issues, Save) is
//! requested through a `ShellCommand`. Backend DTOs are converted into local
//! view structs immediately, so this module never names a backend type.

use std::time::Duration;

use eframe::egui;

use crate::gui::components as comp;
use crate::gui::data::loading::BackgroundLoad;
use crate::gui::screens::{IssuesTab, Route, ScreenCtx, ShellCommand, SubView};
use crate::gui::theme;

/// Captures the typed conflict report at analysis time. The report type is
/// never named here; the closure calls the data wrapper directly at export.
type ReportExport = Box<
    dyn Fn(
        &crate::gui::data::DataContext,
    ) -> Result<Option<String>, crate::gui::data::UiError>,
>;

// ------------------------------------------------- shell-owned view helpers

fn normalized_sub_view(route: Route, view: SubView) -> SubView {
    match route {
        Route::library_mod_manager => match view {
            SubView::ModList | SubView::ModWorkshop => view,
            _ => SubView::ModScan,
        },
        Route::library_presets => match view {
            SubView::PresetCompare => view,
            _ => SubView::PresetList,
        },
        Route::library_sandboxes => SubView::SandboxList,
        Route::library_conflict_analyzer => SubView::ConflictReport,
        _ => SubView::None,
    }
}

/// Sub-view buttons are navigation *requests*: the shell owns
/// `ctx.route` / `ctx.sub_view` and decides what renders next frame.
fn sub_view_nav(
    ui: &mut egui::Ui,
    ctx: &mut ScreenCtx<'_>,
    route: Route,
    current: SubView,
    items: &[(SubView, &str)],
) {
    ui.horizontal(|ui| {
        for (view, label) in items {
            if *view == current {
                ui.label(egui::RichText::new(*label).strong().color(theme::ACCENT));
            } else if comp::button(ui, label).clicked() {
                ctx.commands.push(ShellCommand::Navigate {
                    route,
                    sub_view: Some(*view),
                });
            }
        }
    });
    ui.add_space(6.0);
}

// ------------------------------------------------------------ local DTO views

#[derive(Debug, Clone, Default)]
struct ModRow {
    order: String,
    name: String,
    source: String,
    version_author: String,
    status: String,
    workshop_id: Option<String>,
    warnings: String,
}

#[derive(Debug, Clone, Default)]
struct PresetRow {
    id: String,
    name: String,
    game: String,
    mods_count: usize,
    notes: String,
    load_order_source: String,
}

#[derive(Debug, Clone, Default)]
struct ManagerView {
    game: String,
    scan_mode: String,
    local_mods: usize,
    workshop_mods: usize,
    active_mods: usize,
    load_order_source: String,
    manual_workshop_path: Option<String>,
    timed_out: bool,
    warnings: Vec<String>,
    mods: Vec<ModRow>,
}

/// Convert a manager state without naming its backend type. Used both for
/// the synchronous load and inside the background-scan closure.
macro_rules! manager_view {
    ($value:expr) => {{
        let value = $value;
        let summary = &value.summary;
        let game = summary.selected_game.as_str().to_string();
        let scan_mode = summary.scan_mode.clone();
        let local_mods = summary.local_mods_found;
        let workshop_mods = summary.steam_workshop_mods_found;
        let active_mods = summary.active_mods_count;
        let load_order_source = summary.load_order_source.clone();
        let manual_workshop_path = summary.manual_workshop_path.clone();
        let timed_out = summary.scan_timed_out;
        let warnings = value.warnings;
        let mods = value
            .mods
            .into_iter()
            .map(|m| ModRow {
                order: m
                    .load_order_index
                    .map(|index| index.to_string())
                    .unwrap_or_else(|| "-".to_string()),
                name: m.name,
                source: format!("{:?}", m.source),
                version_author: format!(
                    "{} / {}",
                    m.version.as_deref().unwrap_or("-"),
                    m.author.as_deref().unwrap_or("-")
                ),
                status: m.status,
                workshop_id: m.workshop_id,
                warnings: m.warnings.join("; "),
            })
            .collect();
        ManagerView {
            game,
            scan_mode,
            local_mods,
            workshop_mods,
            active_mods,
            load_order_source,
            manual_workshop_path,
            timed_out,
            warnings,
            mods,
        }
    }};
}

#[derive(Debug, Clone, Default)]
struct SandboxView {
    id: String,
    title: String,
    description: String,
    mods: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct CompareView {
    preset_name: String,
    missing: usize,
    extra: usize,
    load_order_diffs: usize,
    changed_paths: usize,
    workshop_links: usize,
    warnings: Vec<String>,
    load_order_source: String,
}

#[derive(Debug, Clone, Default)]
struct SuspectedView {
    score: String,
    confidence: String,
    name: String,
    active_state: String,
    reasons: String,
}

#[derive(Debug, Clone, Default)]
struct ConflictView {
    status: String,
    summary: String,
    confidence_note: String,
    disclaimer: String,
    crash_detected: bool,
    errors: usize,
    warnings: usize,
    indexed_mods: usize,
    active_mods: usize,
    analysis_mode: String,
    timed_out: bool,
    suspected: Vec<SuspectedView>,
    missing_references: Vec<String>,
    error_lines: Vec<String>,
    limitations: Vec<String>,
}

// ------------------------------------------------------------ pending work

#[derive(Debug, Clone)]
enum Operation {
    LoadManager,
    Scan { deep: bool },
    OpenSteamConsole,
    SetWorkshopDir,
    ClearWorkshopDir,
    OpenWorkshopPage { id: String },
    ListPresets,
    ImportPreset,
    CreatePreset { name: String, notes: Option<String> },
    ComparePreset { id: String },
    ExportPreset { id: String },
    DeletePreset { id: String, name: String },
    LoadSandboxes,
    CreateSandbox { title: String, description: String },
    AddModToSandbox { sandbox_id: String, input: String },
    ApplySandbox { id: String },
    DeleteSandbox { id: String, title: String },
    AnalyzeConflicts { deep: bool },
    ExportConflict,
}

#[derive(Debug, Clone)]
struct Confirm {
    title: &'static str,
    message: String,
    action: Operation,
}

/// All local UI state. Navigation swaps this instance in and out but never
/// clears it (docs/ui-spec.md §1.3 rule 5).
#[derive(Default)]
pub struct ScreenState {
    // Mod manager and background scan.
    manager: Option<ManagerView>,
    manager_tried: bool,
    manager_error: Option<String>,
    manager_message: Option<String>,
    scan: BackgroundLoad<Result<ManagerView, String>>,
    scan_error: Option<String>,
    selected_mod: Option<usize>,
    workshop_error: Option<String>,
    workshop_message: Option<String>,

    // Presets.
    presets_tried: bool,
    presets: Vec<PresetRow>,
    presets_error: Option<String>,
    preset_name: String,
    preset_notes: String,
    preset_validation: Option<String>,
    preset_message: Option<String>,
    selected_preset: Option<usize>,
    compare: Option<CompareView>,
    compare_error: Option<String>,
    compare_id: String,

    // Sandboxes.
    sandboxes_tried: bool,
    sandboxes: Option<Vec<SandboxView>>,
    sandboxes_error: Option<String>,
    sandbox_title: String,
    sandbox_description: String,
    sandbox_validation: Option<String>,
    sandbox_message: Option<String>,
    sandbox_target: String,
    workshop_mod_input: String,
    add_mod_validation: Option<String>,

    // Conflict analyzer.
    conflict: Option<ConflictView>,
    conflict_error: Option<String>,
    conflict_message: Option<String>,
    conflict_deep: bool,
    export_report: Option<ReportExport>,
    export_validation: Option<String>,

    pending: Option<Operation>,
    confirm: Option<Confirm>,
    confirmed: Option<Operation>,
    loading: Option<&'static str>,
}

impl ScreenState {
    fn schedule(&mut self, operation: Operation, loading: &'static str) {
        self.pending = Some(operation);
        self.loading = Some(loading);
    }

    fn confirm(&mut self, title: &'static str, message: String, action: Operation) {
        self.confirm = Some(Confirm {
            title,
            message,
            action,
        });
    }
}

// ------------------------------------------------------------------ feedback

fn toast(ctx: &mut ScreenCtx<'_>, message: String, is_error: bool) {
    ctx.commands.push(ShellCommand::Toast { message, is_error });
}

fn error(ctx: &mut ScreenCtx<'_>, message: String) {
    toast(ctx, message, true);
}

fn info(ctx: &mut ScreenCtx<'_>, message: String) {
    toast(ctx, message, false);
}

/// Validation messages must reach the Issues drawer as well as inline text.
fn validation(ctx: &mut ScreenCtx<'_>, message: String) {
    toast(ctx, format!("Validation: {message}"), false);
}

fn game_arg(state: &ScreenState) -> Option<String> {
    let game = state
        .manager
        .as_ref()
        .map(|manager| manager.game.clone())
        .unwrap_or_default();
    if game.is_empty() {
        None
    } else {
        Some(game)
    }
}

fn row_text(row: &[String], column: usize) -> Option<String> {
    row.get(column).cloned()
}

// ------------------------------------------------------------------ rendering

/// Exact screen contract: one state instance and a render function.
pub fn render(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    // Deferred work runs at frame start so the click frame can show loading.
    if let Some(operation) = state.pending.take() {
        execute(operation, state, ctx);
    }
    if let Some(operation) = state.confirmed.take() {
        execute(operation, state, ctx);
    }

    if let Some(result) = state.scan.poll() {
        match result {
            Ok(manager) => {
                state.manager = Some(manager);
                state.manager_error = None;
                state.manager_tried = true;
                state.scan_error = None;
                state.selected_mod = None;
                info(ctx, "Mod scan completed.".to_string());
            }
            Err(message) => {
                state.scan_error = Some(message.clone());
                error(ctx, message);
            }
        }
    }
    if state.scan.is_loading() {
        ui.ctx()
            .request_repaint_after(Duration::from_millis(100));
    }

    // One-time loads make each tile useful on a cold start; explicit buttons
    // remain available for retries.
    if state.manager.is_none() && !state.manager_tried && !state.scan.is_loading() {
        state.manager_tried = true;
        state.schedule(Operation::LoadManager, "Loading mod manager state…");
    }
    if ctx.route == Route::library_presets && !state.presets_tried {
        state.presets_tried = true;
        state.schedule(Operation::ListPresets, "Loading mod presets…");
    }
    if ctx.route == Route::library_sandboxes && !state.sandboxes_tried {
        state.sandboxes_tried = true;
        state.schedule(Operation::LoadSandboxes, "Loading mod sandboxes…");
    }

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        if comp::button(ui, "Open Issues").clicked() {
            ctx.commands
                .push(ShellCommand::OpenIssues(IssuesTab::Errors));
        }
        ui.label(
            egui::RichText::new(
                "Scans are read-only; preset/sandbox actions write files and the backend creates a backup.",
            )
            .size(10.5)
            .color(theme::MUTED),
        );
    });

    if let Some(message) = state.loading {
        comp::loading_state(ui, message);
    }
    if state.scan.is_loading() {
        comp::progress_row(ui, "Scanning mods (quick/deep)…", None);
    }

    let view = normalized_sub_view(ctx.route, ctx.sub_view);
    match ctx.route {
        Route::library_mod_manager => render_mod_manager(ui, state, ctx, view),
        Route::library_presets => render_presets(ui, state, ctx, view),
        Route::library_sandboxes => render_sandboxes(ui, state, ctx),
        Route::library_conflict_analyzer => render_conflicts(ui, state, ctx),
        _ => render_mod_manager(ui, state, ctx, SubView::ModScan),
    }

    render_confirm(ui, state, ctx);

    // Keep a freshly scheduled load visible on its own frame.
    if state.pending.is_none() && state.confirmed.is_none() {
        state.loading = None;
    }
}

// ----------------------------------------------------------- operation runner

fn execute(operation: Operation, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    use crate::gui::data::library as lib;

    match operation {
        Operation::LoadManager => match lib::load_manager_state(ctx.data, game_arg(state)) {
            Ok(value) => {
                state.manager = Some(manager_view!(value));
                state.manager_error = None;
                state.manager_message = Some("Loaded mod manager state.".to_string());
                state.selected_mod = None;
            }
            Err(err) => {
                let message = err.message;
                state.manager_error = Some(message.clone());
                error(ctx, message);
            }
        },

        Operation::Scan { deep } => {
            let data = ctx.data.clone();
            let game = game_arg(state);
            state.scan.start(move || {
                let result = if deep {
                    lib::scan_mods_deep(&data, game)
                } else {
                    lib::scan_mods_light(&data, game)
                };
                result
                    .map(|value| manager_view!(value))
                    .map_err(|err| err.message)
            });
            state.scan_error = None;
        }

        Operation::OpenSteamConsole => match lib::open_steam_console(ctx.data) {
            Ok(()) => info(ctx, "Opened Steam console.".to_string()),
            Err(err) => {
                let message = err.message;
                state.workshop_error = Some(message.clone());
                error(ctx, message);
            }
        },

        Operation::SetWorkshopDir => {
            let Some(game) = game_arg(state) else {
                let message = "Load mod manager state before setting a Workshop folder.".to_string();
                state.workshop_error = Some(message.clone());
                validation(ctx, message);
                return;
            };
            match lib::choose_workshop_directory(ctx.data, game) {
                Ok(Some(path)) => {
                    if let Some(manager) = state.manager.as_mut() {
                        manager.manual_workshop_path = Some(path.clone());
                    }
                    state.workshop_message = Some(format!("Manual Workshop folder: {path}"));
                    state.workshop_error = None;
                    info(ctx, format!("Manual Workshop folder: {path}"));
                }
                Ok(None) => {}
                Err(err) => {
                    let message = err.message;
                    state.workshop_error = Some(message.clone());
                    error(ctx, message);
                }
            }
        }

        Operation::ClearWorkshopDir => {
            let Some(game) = game_arg(state) else {
                let message = "Load mod manager state before clearing a Workshop folder.".to_string();
                state.workshop_error = Some(message.clone());
                validation(ctx, message);
                return;
            };
            match lib::clear_workshop_directory(ctx.data, game) {
                Ok(()) => {
                    if let Some(manager) = state.manager.as_mut() {
                        manager.manual_workshop_path = None;
                    }
                    state.workshop_message = Some("Cleared manual Workshop folder.".to_string());
                    state.workshop_error = None;
                    info(ctx, "Cleared manual Workshop folder.".to_string());
                }
                Err(err) => {
                    let message = err.message;
                    state.workshop_error = Some(message.clone());
                    error(ctx, message);
                }
            }
        }

        Operation::OpenWorkshopPage { id } => match lib::open_workshop_page(ctx.data, id) {
            Ok(()) => info(ctx, "Opened Steam Workshop page.".to_string()),
            Err(err) => {
                let message = err.message;
                state.workshop_error = Some(message.clone());
                error(ctx, message);
            }
        },

        Operation::ListPresets => match lib::list_presets(ctx.data, game_arg(state)) {
            Ok(list) => {
                state.presets = list
                    .into_iter()
                    .map(|preset| PresetRow {
                        id: preset.id,
                        name: preset.name,
                        game: preset.game.as_str().to_string(),
                        mods_count: preset.mods.len(),
                        notes: preset.notes.unwrap_or_default(),
                        load_order_source: preset.load_order_source,
                    })
                    .collect();
                state.presets_error = None;
                state.selected_preset = None;
            }
            Err(err) => {
                let message = err.message;
                state.presets_error = Some(message.clone());
                error(ctx, message);
            }
        },

        Operation::ImportPreset => match lib::import_preset(ctx.data) {
            Ok(Some(preset)) => {
                let name = preset.name.clone();
                let game = preset.game.as_str().to_string();
                let game_label = preset.game.display_name();
                let current_game = game_arg(state).unwrap_or_default();
                let message = if game == current_game || current_game.is_empty() {
                    format!("Imported mod preset `{name}`.")
                } else {
                    format!(
                        "Imported mod preset `{name}` for {game_label} — switch game in the Context Column to see it."
                    )
                };
                match lib::list_presets(ctx.data, game_arg(state)) {
                    Ok(list) => {
                        state.presets = list
                            .into_iter()
                            .map(|item| PresetRow {
                                id: item.id,
                                name: item.name,
                                game: item.game.as_str().to_string(),
                                mods_count: item.mods.len(),
                                notes: item.notes.unwrap_or_default(),
                                load_order_source: item.load_order_source,
                            })
                            .collect();
                    }
                    Err(err) => {
                        let failure = err.message;
                        state.presets_error = Some(failure.clone());
                        error(ctx, failure);
                    }
                }
                state.preset_message = Some(message.clone());
                state.presets_error = None;
                info(ctx, message);
            }
            Ok(None) => {}
            Err(err) => {
                let message = err.message;
                state.presets_error = Some(message.clone());
                error(ctx, message);
            }
        },

        Operation::CreatePreset { name, notes } => {
            let Some(game) = game_arg(state) else {
                let message = "Load mod manager state before saving a preset.".to_string();
                state.preset_validation = Some(message.clone());
                validation(ctx, message);
                return;
            };
            if name.trim().is_empty() {
                let message = "Preset name is required.".to_string();
                state.preset_validation = Some(message.clone());
                validation(ctx, message);
                return;
            }
            match lib::create_preset(ctx.data, name, game, notes, None) {
                Ok(_) => {
                    state.preset_validation = None;
                    state.preset_message = Some("Saved mod preset.".to_string());
                    info(ctx, "Saved mod preset.".to_string());
                    state.schedule(Operation::ListPresets, "Refreshing mod presets…");
                }
                Err(err) => {
                    let message = err.message;
                    state.preset_validation = Some(message.clone());
                    error(ctx, message);
                }
            }
        }

        Operation::ComparePreset { id } => {
            let Some(game) = game_arg(state) else {
                let message = "Load mod manager state before comparing a preset.".to_string();
                state.compare_error = Some(message.clone());
                validation(ctx, message);
                return;
            };
            state.compare_id = id.clone();
            match lib::compare_preset(ctx.data, id, game) {
                Ok(result) => {
                    state.compare = Some(CompareView {
                        preset_name: result.preset.name,
                        missing: result.summary.missing_mods_count,
                        extra: result.summary.extra_mods_count,
                        load_order_diffs: result.summary.load_order_differences_count,
                        changed_paths: result.summary.changed_paths_count,
                        workshop_links: result.summary.workshop_links_count,
                        warnings: result.warnings,
                        load_order_source: result.load_order_source,
                    });
                    state.compare_error = None;
                    ctx.commands.push(ShellCommand::Navigate {
                        route: Route::library_presets,
                        sub_view: Some(SubView::PresetCompare),
                    });
                }
                Err(err) => {
                    let message = err.message;
                    state.compare_error = Some(message.clone());
                    error(ctx, message);
                }
            }
        }

        Operation::ExportPreset { id } => match lib::export_preset(ctx.data, id) {
            Ok(path) if !path.is_empty() => {
                let message = format!("Exported preset to {path}.");
                state.preset_message = Some(message.clone());
                info(ctx, message);
            }
            Ok(_) => {}
            Err(err) => {
                let message = err.message;
                state.preset_validation = Some(message.clone());
                error(ctx, message);
            }
        },

        Operation::DeletePreset { id, name } => match lib::delete_preset(ctx.data, id) {
            Ok(()) => {
                let message = format!("Deleted mod preset `{name}`.");
                state.preset_message = Some(message.clone());
                info(ctx, message);
                state.schedule(Operation::ListPresets, "Refreshing mod presets…");
            }
            Err(err) => {
                let message = err.message;
                state.preset_validation = Some(message.clone());
                error(ctx, message);
            }
        },

        Operation::LoadSandboxes => match lib::load_sandboxes(ctx.data) {
            Ok(collection) => {
                let sandboxes: Vec<SandboxView> = collection
                    .sandboxes
                    .into_iter()
                    .map(|sandbox| SandboxView {
                        id: sandbox.id,
                        title: sandbox.title,
                        description: sandbox.description,
                        mods: sandbox
                            .mods
                            .into_iter()
                            .map(|mod_entry| format!("{} — {}", mod_entry.id, mod_entry.name))
                            .collect(),
                    })
                    .collect();
                if state.sandbox_target.is_empty() {
                    if let Some(first) = sandboxes.first() {
                        state.sandbox_target = first.title.clone();
                    }
                }
                state.sandboxes = Some(sandboxes);
                state.sandboxes_error = None;
            }
            Err(err) => {
                let message = err.message;
                state.sandboxes_error = Some(message.clone());
                error(ctx, message);
            }
        },

        Operation::CreateSandbox { title, description } => {
            if title.trim().is_empty() {
                let message = "Sandbox title is required.".to_string();
                state.sandbox_validation = Some(message.clone());
                validation(ctx, message);
                return;
            }
            match lib::create_sandbox(ctx.data, title, description) {
                Ok(collection) => {
                    state.sandboxes = Some(
                        collection
                            .sandboxes
                            .into_iter()
                            .map(|sandbox| SandboxView {
                                id: sandbox.id,
                                title: sandbox.title,
                                description: sandbox.description,
                                mods: sandbox
                                    .mods
                                    .into_iter()
                                    .map(|mod_entry| {
                                        format!("{} — {}", mod_entry.id, mod_entry.name)
                                    })
                                    .collect(),
                            })
                            .collect(),
                    );
                    state.sandbox_validation = None;
                    state.sandbox_message = Some("Created sandbox.".to_string());
                    state.sandboxes_error = None;
                    info(ctx, "Created sandbox.".to_string());
                }
                Err(err) => {
                    let message = err.message;
                    state.sandbox_validation = Some(message.clone());
                    error(ctx, message);
                }
            }
        }

        Operation::AddModToSandbox {
            sandbox_id,
            input,
        } => {
            if input.trim().is_empty() {
                let message = "Workshop ID / URL is required.".to_string();
                state.add_mod_validation = Some(message.clone());
                validation(ctx, message);
                return;
            }
            match lib::add_mod_to_sandbox(ctx.data, sandbox_id, input, Some(true)) {
                Ok(collection) => {
                    state.sandboxes = Some(
                        collection
                            .sandboxes
                            .into_iter()
                            .map(|sandbox| SandboxView {
                                id: sandbox.id,
                                title: sandbox.title,
                                description: sandbox.description,
                                mods: sandbox
                                    .mods
                                    .into_iter()
                                    .map(|mod_entry| {
                                        format!("{} — {}", mod_entry.id, mod_entry.name)
                                    })
                                    .collect(),
                            })
                            .collect(),
                    );
                    state.add_mod_validation = None;
                    state.sandbox_message = Some("Added mod to sandbox.".to_string());
                    state.sandboxes_error = None;
                    info(ctx, "Added mod to sandbox.".to_string());
                }
                Err(err) => {
                    let message = err.message;
                    state.add_mod_validation = Some(message.clone());
                    error(ctx, message);
                }
            }
        }

        Operation::ApplySandbox { id } => match lib::apply_sandbox(ctx.data, id, false) {
            Ok(result) => {
                let message = format!(
                    "{}{} Backup: {}",
                    result.message,
                    if result.success { "" } else { " (not successful)" },
                    result.backup_path
                );
                state.sandbox_message = Some(message.clone());
                if result.success {
                    ctx.commands.push(ShellCommand::RefreshSafety);
                    info(ctx, message);
                } else {
                    error(ctx, message);
                }
            }
            Err(err) => {
                let message = err.message;
                state.sandboxes_error = Some(message.clone());
                error(ctx, message);
            }
        },

        Operation::DeleteSandbox { id, title } => match lib::delete_sandbox(ctx.data, id) {
            Ok(collection) => {
                state.sandboxes = Some(
                    collection
                        .sandboxes
                        .into_iter()
                        .map(|sandbox| SandboxView {
                            id: sandbox.id,
                            title: sandbox.title,
                            description: sandbox.description,
                            mods: sandbox
                                .mods
                                .into_iter()
                                .map(|mod_entry| format!("{} — {}", mod_entry.id, mod_entry.name))
                                .collect(),
                        })
                        .collect(),
                );
                let message = format!("Deleted sandbox `{title}`.");
                state.sandbox_message = Some(message.clone());
                state.sandboxes_error = None;
                info(ctx, message);
            }
            Err(err) => {
                let message = err.message;
                state.sandboxes_error = Some(message.clone());
                error(ctx, message);
            }
        },

        Operation::AnalyzeConflicts { deep } => {
            let result = if deep {
                lib::analyze_conflicts_deep(ctx.data)
            } else {
                lib::analyze_conflicts(ctx.data)
            };
            match result {
                Ok(report) => {
                    let view = ConflictView {
                        status: report.overview.status_badge.clone(),
                        summary: report.overview.summary.clone(),
                        confidence_note: report.overview.confidence_note.clone(),
                        disclaimer: report.overview.disclaimer.clone(),
                        crash_detected: report.crash_summary.crash_detected,
                        errors: report.crash_summary.error_count,
                        warnings: report.crash_summary.warning_count,
                        indexed_mods: report.sources.indexed_mods_count,
                        active_mods: report.sources.active_mods_count,
                        analysis_mode: report.sources.analysis_mode.clone(),
                        timed_out: report.sources.analysis_timed_out,
                        suspected: report
                            .suspected_mods
                            .iter()
                            .map(|suspect| SuspectedView {
                                score: suspect.score.to_string(),
                                confidence: suspect.confidence.clone(),
                                name: suspect.name.clone(),
                                active_state: suspect.active_state.clone(),
                                reasons: suspect.reasons.join("; "),
                            })
                            .collect(),
                        missing_references: report
                            .missing_references
                            .iter()
                            .map(|entry| format!("{} — {}", entry.path, entry.reason))
                            .collect(),
                        error_lines: report
                            .errors
                            .iter()
                            .map(|entry| format!("[{}] {}", entry.severity, entry.explanation))
                            .collect(),
                        limitations: report.limitations.clone(),
                    };
                    state.conflict = Some(view);
                    state.conflict_deep = deep;
                    state.conflict_error = None;
                    state.export_validation = None;
                    state.export_report = Some(Box::new(move |data| {
                        lib::export_conflict_report(data, report.clone(), Some(true))
                    }));
                    let message = if deep {
                        "Completed deep mod conflict analysis."
                    } else {
                        "Completed mod conflict analysis."
                    };
                    state.conflict_message = Some(message.to_string());
                    info(ctx, message.to_string());
                }
                Err(err) => {
                    let message = err.message;
                    state.conflict_error = Some(message.clone());
                    error(ctx, message);
                }
            }
        }

        Operation::ExportConflict => {
            if let Some(export) = state.export_report.as_ref() {
                match export(ctx.data) {
                    Ok(Some(path)) => {
                        let message = format!("Exported conflict report to {path}.");
                        state.conflict_message = Some(message.clone());
                        state.export_validation = None;
                        info(ctx, message);
                    }
                    Ok(None) => {}
                    Err(err) => {
                        let message = err.message;
                        state.export_validation = Some(message.clone());
                        error(ctx, message);
                    }
                }
            } else {
                let message = "Run a conflict analysis before exporting its report.".to_string();
                state.export_validation = Some(message.clone());
                validation(ctx, message);
            }
        }
    }
}

// ----------------------------------------------------------- mod manager tile

fn render_mod_manager(
    ui: &mut egui::Ui,
    state: &mut ScreenState,
    ctx: &mut ScreenCtx<'_>,
    view: SubView,
) {
    sub_view_nav(
        ui,
        ctx,
        Route::library_mod_manager,
        view,
        &[
            (SubView::ModScan, "Scan & Summary"),
            (SubView::ModList, "Installed Mods"),
            (SubView::ModWorkshop, "Workshop & Links"),
        ],
    );

    ui.horizontal_wrapped(|ui| {
        if comp::button(ui, "Load State").clicked() {
            state.schedule(Operation::LoadManager, "Loading mod manager state…");
        }
        let scanning = state.scan.is_loading();
        if ui
            .add_enabled(!scanning, egui::Button::new("Quick Scan Mods"))
            .clicked()
        {
            state.schedule(Operation::Scan { deep: false }, "Starting quick mod scan…");
        }
        if ui
            .add_enabled(!scanning, egui::Button::new("Deep Scan Mods"))
            .clicked()
        {
            state.schedule(Operation::Scan { deep: true }, "Starting deep mod scan…");
        }
        if comp::button(ui, "Open Steam Console").clicked() {
            state.schedule(Operation::OpenSteamConsole, "Opening Steam console…");
        }
    });

    if let Some(message) = &state.scan_error {
        if comp::error_state(ui, message, Some("Retry quick scan")) {
            state.schedule(Operation::Scan { deep: false }, "Retrying quick mod scan…");
        }
    }
    if let Some(message) = &state.manager_error {
        if comp::error_state(ui, message, Some("Retry load")) {
            state.schedule(Operation::LoadManager, "Reloading mod manager state…");
        }
    }
    if let Some(message) = &state.manager_message {
        ui.label(egui::RichText::new(message).color(theme::MUTED));
    }

    match view {
        SubView::ModList => render_installed_mods(ui, state, ctx),
        SubView::ModWorkshop => render_workshop(ui, state, ctx),
        _ => render_scan_summary(ui, state),
    }
}

fn render_scan_summary(ui: &mut egui::Ui, state: &ScreenState) {
    let Some(manager) = state.manager.as_ref() else {
        comp::empty_state(
            ui,
            "No mod state loaded",
            "Load state or run a quick/deep scan to inspect installed mods.",
        );
        return;
    };

    comp::section_card(ui, "Scan summary", |ui| {
        ui.horizontal_wrapped(|ui| {
            comp::compact_stat_tile(ui, "Game", &manager.game, theme::ACCENT);
            comp::compact_stat_tile(ui, "Mode", &manager.scan_mode, theme::ACCENT);
            comp::compact_stat_tile(
                ui,
                "Local mods",
                &manager.local_mods.to_string(),
                theme::ACCENT,
            );
            comp::compact_stat_tile(
                ui,
                "Workshop mods",
                &manager.workshop_mods.to_string(),
                theme::ACCENT,
            );
            comp::compact_stat_tile(
                ui,
                "Active in profile",
                &manager.active_mods.to_string(),
                theme::ACCENT,
            );
        });
        ui.label(format!("Load order source: {}", manager.load_order_source));
        ui.label(format!("Manual Workshop folder: {}", manager.manual_workshop_path.as_deref().unwrap_or("not set")));
        if manager.timed_out {
            comp::validation_message(ui, "The backend reported a scan timeout; results may be partial.");
        }
        for warning in &manager.warnings {
            comp::validation_message(ui, warning);
        }
    });
}

fn render_installed_mods(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    let Some(manager) = state.manager.as_ref() else {
        comp::empty_state(
            ui,
            "No mod state loaded",
            "Load state or run a scan before viewing installed mods.",
        );
        return;
    };
    if manager.mods.is_empty() {
        comp::empty_state(
            ui,
            "No mods detected",
            "Run a deep scan or set a manual Workshop folder.",
        );
        return;
    }

    let rows: Vec<Vec<String>> = manager
        .mods
        .iter()
        .map(|mod_row| {
            vec![
                mod_row.order.clone(),
                mod_row.name.clone(),
                mod_row.source.clone(),
                mod_row.version_author.clone(),
                mod_row.status.clone(),
                mod_row.workshop_id.clone().unwrap_or_else(|| "-".to_string()),
                if mod_row.warnings.is_empty() {
                    "-".to_string()
                } else {
                    mod_row.warnings.clone()
                },
            ]
        })
        .collect();
    if let Some(selected) = comp::table(
        ui,
        "mods_installed_table",
        &["Order", "Mod name", "Source", "Version / author", "Status", "Workshop ID", "Warnings"],
        &rows,
        state.selected_mod,
    ) {
        state.selected_mod = Some(selected);
    }

    let workshop_id = state
        .selected_mod
        .and_then(|index| rows.get(index))
        .and_then(|row| row_text(row, 5))
        .filter(|id| id.as_str() != "-");
    ui.horizontal(|ui| {
        if comp::button(ui, "Open Workshop Page for selected mod").clicked() {
            if let Some(id) = workshop_id {
                state.schedule(
                    Operation::OpenWorkshopPage { id },
                    "Opening Steam Workshop page…",
                );
            } else {
                let message = "Select a mod with a Workshop ID first.".to_string();
                validation(ctx, message.clone());
                state.workshop_error = Some(message);
            }
        }
    });
    if let Some(message) = &state.workshop_error {
        comp::validation_message(ui, message);
    }
}

fn render_workshop(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    comp::section_card(ui, "Steam Workshop", |ui| {
        ui.horizontal_wrapped(|ui| {
            if comp::button(ui, "Set Manual Workshop Folder").clicked() {
                state.schedule(Operation::SetWorkshopDir, "Choosing Workshop folder…");
            }
            if comp::button(ui, "Clear Manual Workshop Folder").clicked() {
                state.schedule(Operation::ClearWorkshopDir, "Clearing Workshop folder…");
            }
            if comp::button(ui, "Open Steam Console").clicked() {
                state.schedule(Operation::OpenSteamConsole, "Opening Steam console…");
            }
        });
        let manual = state
            .manager
            .as_ref()
            .and_then(|manager| manager.manual_workshop_path.clone())
            .unwrap_or_else(|| "None — detected Workshop folders are used".to_string());
        ui.label(format!("Manual Workshop folder: {manual}"));
        comp::validation_message(
            ui,
            state
                .workshop_error
                .as_deref()
                .unwrap_or("A manual folder overrides automatic Workshop discovery for this game."),
        );
        if let Some(message) = &state.workshop_message {
            ui.label(egui::RichText::new(message).color(theme::MUTED));
        }
    });

    let first_workshop_id = state
        .manager
        .as_ref()
        .and_then(|manager| manager.mods.iter().find_map(|mod_row| mod_row.workshop_id.clone()));
    ui.horizontal(|ui| {
        if comp::button(ui, "Open first detected Workshop page").clicked() {
            if let Some(id) = first_workshop_id {
                state.schedule(
                    Operation::OpenWorkshopPage { id },
                    "Opening Steam Workshop page…",
                );
            } else {
                let message = "No Workshop ID found — scan mods first.".to_string();
                state.workshop_error = Some(message.clone());
                validation(ctx, message);
            }
        }
    });
}

// ---------------------------------------------------------------- presets tile

fn render_presets(
    ui: &mut egui::Ui,
    state: &mut ScreenState,
    ctx: &mut ScreenCtx<'_>,
    view: SubView,
) {
    sub_view_nav(
        ui,
        ctx,
        Route::library_presets,
        view,
        &[
            (SubView::PresetList, "Preset list"),
            (SubView::PresetCompare, "Comparison"),
        ],
    );

    if view == SubView::PresetCompare {
        render_compare(ui, state, ctx);
        return;
    }

    ui.horizontal_wrapped(|ui| {
        if comp::button(ui, "Load Presets").clicked() {
            state.schedule(Operation::ListPresets, "Loading mod presets…");
        }
        if comp::button(ui, "Import Preset JSON").clicked() {
            state.schedule(Operation::ImportPreset, "Importing mod preset…");
        }
    });

    comp::section_card(ui, "Save current mods as preset", |ui| {
        comp::form_field(ui, "Preset name", |ui| {
            comp::text_input(ui, &mut state.preset_name, "Name for this mod preset");
        });
        comp::form_field(ui, "Notes (optional)", |ui| {
            comp::text_input(ui, &mut state.preset_notes, "Optional notes");
        });
        if comp::button(ui, "Save Current as Preset").clicked() {
            if state.preset_name.trim().is_empty() {
                let message = "Preset name is required.".to_string();
                state.preset_validation = Some(message.clone());
                validation(ctx, message);
            } else if game_arg(state).is_none() {
                let message = "Load mod manager state before saving a preset.".to_string();
                state.preset_validation = Some(message.clone());
                validation(ctx, message);
            } else {
                let name = state.preset_name.trim().to_string();
                let notes = {
                    let trimmed = state.preset_notes.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed.to_string())
                    }
                };
                state.schedule(
                    Operation::CreatePreset { name, notes },
                    "Saving mod preset…",
                );
            }
        }
        comp::validation_message(ui, state.preset_validation.as_deref().unwrap_or(""));
    });

    if let Some(message) = &state.presets_error {
        if comp::error_state(ui, message, Some("Retry load")) {
            state.schedule(Operation::ListPresets, "Reloading mod presets…");
        }
    }
    if let Some(message) = &state.preset_message {
        ui.label(egui::RichText::new(message).color(theme::MUTED));
    }
    if state.presets.is_empty() && state.presets_error.is_none() {
        comp::empty_state(ui, "No presets — create one", "Save current mods as a preset above, or import a preset JSON file.");
        return;
    }

    let rows: Vec<Vec<String>> = state
        .presets
        .iter()
        .map(|preset| {
            vec![
                preset.name.clone(),
                preset.game.clone(),
                preset.mods_count.to_string(),
                preset.load_order_source.clone(),
                preset.notes.clone(),
            ]
        })
        .collect();
    if let Some(selected) = comp::table(
        ui,
        "mod_presets_table",
        &["Name", "Game", "Mods", "Load order source", "Notes"],
        &rows,
        state.selected_preset,
    ) {
        state.selected_preset = Some(selected);
    }

    let selected = state
        .selected_preset
        .and_then(|index| state.presets.get(index))
        .cloned();
    ui.horizontal_wrapped(|ui| {
        if comp::button(ui, "Compare with active profile").clicked() {
            if let Some(preset) = &selected {
                state.schedule(
                    Operation::ComparePreset {
                        id: preset.id.clone(),
                    },
                    "Comparing mod preset…",
                );
            } else {
                let message = "Select a preset to compare.".to_string();
                validation(ctx, message);
            }
        }
        if comp::button(ui, "Export JSON").clicked() {
            if let Some(preset) = &selected {
                state.schedule(
                    Operation::ExportPreset {
                        id: preset.id.clone(),
                    },
                    "Exporting mod preset…",
                );
            } else {
                let message = "Select a preset to export.".to_string();
                validation(ctx, message);
            }
        }
        if comp::danger_button(ui, "Delete preset").clicked() {
            if let Some(preset) = &selected {
                state.confirm(
                    "Delete preset",
                    format!(
                        "Delete the mod preset \"{}\"? This cannot be undone.",
                        preset.name
                    ),
                    Operation::DeletePreset {
                        id: preset.id.clone(),
                        name: preset.name.clone(),
                    },
                );
            } else {
                let message = "Select a preset to delete.".to_string();
                validation(ctx, message);
            }
        }
    });
}

fn render_compare(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    if let Some(message) = &state.compare_error {
        let id = state.compare_id.clone();
        if comp::error_state(ui, message, Some("Retry compare")) {
            if id.is_empty() {
                validation(ctx, "Select a preset from the preset list first.".to_string());
            } else {
                state.schedule(
                    Operation::ComparePreset { id },
                    "Comparing mod preset…",
                );
            }
        }
    }
    let Some(compare) = state.compare.as_ref() else {
        comp::empty_state(
            ui,
            "No comparison yet",
            "Open a preset and choose Compare with active profile.",
        );
        return;
    };
    let compare_title = format!("Comparison: {}", compare.preset_name);
    comp::section_card(ui, &compare_title, |ui| {
        ui.horizontal_wrapped(|ui| {
            comp::compact_stat_tile(ui, "Missing", &compare.missing.to_string(), theme::ACCENT);
            comp::compact_stat_tile(ui, "Extra", &compare.extra.to_string(), theme::ACCENT);
            comp::compact_stat_tile(
                ui,
                "Load order diffs",
                &compare.load_order_diffs.to_string(),
                theme::ACCENT,
            );
            comp::compact_stat_tile(
                ui,
                "Changed paths",
                &compare.changed_paths.to_string(),
                theme::ACCENT,
            );
            comp::compact_stat_tile(
                ui,
                "Workshop links",
                &compare.workshop_links.to_string(),
                theme::ACCENT,
            );
        });
        ui.label(format!("Load order source: {}", compare.load_order_source));
        for warning in &compare.warnings {
            comp::validation_message(ui, warning);
        }
    });
}

// -------------------------------------------------------------- sandboxes tile

fn render_sandboxes(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    let has_staged = crate::gui::data::session::is_dirty(ctx.data);
    ui.horizontal_wrapped(|ui| {
        if comp::button(ui, "Load Sandboxes").clicked() {
            state.schedule(Operation::LoadSandboxes, "Loading mod sandboxes…");
        }
    });

    comp::section_card(ui, "Create sandbox", |ui| {
        comp::form_field(ui, "Title", |ui| {
            comp::text_input(ui, &mut state.sandbox_title, "Sandbox title");
        });
        comp::form_field(ui, "Description", |ui| {
            comp::text_input(ui, &mut state.sandbox_description, "Optional description");
        });
        if comp::button(ui, "Create Sandbox").clicked() {
            if state.sandbox_title.trim().is_empty() {
                let message = "Sandbox title is required.".to_string();
                state.sandbox_validation = Some(message.clone());
                validation(ctx, message);
            } else {
                state.schedule(
                    Operation::CreateSandbox {
                        title: state.sandbox_title.trim().to_string(),
                        description: state.sandbox_description.trim().to_string(),
                    },
                    "Creating sandbox…",
                );
            }
        }
        comp::validation_message(ui, state.sandbox_validation.as_deref().unwrap_or(""));
    });

    if let Some(message) = &state.sandboxes_error {
        if comp::error_state(ui, message, Some("Retry load")) {
            state.schedule(Operation::LoadSandboxes, "Reloading mod sandboxes…");
        }
    }
    if let Some(message) = &state.sandbox_message {
        ui.label(egui::RichText::new(message).color(theme::MUTED));
    }
    let Some(sandboxes) = state.sandboxes.clone() else {
        comp::empty_state(
            ui,
            "No sandboxes loaded",
            "Load sandboxes or create a new one above.",
        );
        return;
    };
    if sandboxes.is_empty() {
        comp::empty_state(
            ui,
            "No sandboxes — create one",
            "Sandboxes hold an isolated mod list you can apply to the active profile.",
        );
        return;
    }

    let options: Vec<String> = sandboxes.iter().map(|sandbox| sandbox.title.clone()).collect();
    if state.sandbox_target.is_empty()
        || !sandboxes
            .iter()
            .any(|sandbox| sandbox.title == state.sandbox_target)
    {
        state.sandbox_target = sandboxes.first().map(|sandbox| sandbox.title.clone()).unwrap_or_default();
    }
    comp::section_card(ui, "Add a Workshop mod", |ui| {
        comp::form_field(ui, "Target sandbox", |ui| {
            comp::select(
                ui,
                "mods_sandbox_target",
                &mut state.sandbox_target,
                &options,
            );
        });
        comp::form_field(ui, "Workshop ID / URL", |ui| {
            comp::text_input(
                ui,
                &mut state.workshop_mod_input,
                "Workshop ID or share URL",
            );
        });
        if comp::button(ui, "Add Mod").clicked() {
            let target = sandboxes
                .iter()
                .find(|sandbox| sandbox.title == state.sandbox_target)
                .map(|sandbox| sandbox.id.clone());
            if target.is_none() {
                let message = "Select a sandbox first.".to_string();
                state.add_mod_validation = Some(message.clone());
                validation(ctx, message);
            } else if state.workshop_mod_input.trim().is_empty() {
                let message = "Workshop ID / URL is required.".to_string();
                state.add_mod_validation = Some(message.clone());
                validation(ctx, message);
            } else {
                state.schedule(
                    Operation::AddModToSandbox {
                        sandbox_id: target.unwrap_or_default(),
                        input: state.workshop_mod_input.trim().to_string(),
                    },
                    "Adding mod to sandbox…",
                );
            }
        }
        comp::validation_message(ui, state.add_mod_validation.as_deref().unwrap_or(""));
    });

    for sandbox in &sandboxes {
        comp::section_card(ui, &sandbox.title, |ui| {
            ui.label(&sandbox.description);
            ui.label(format!("Mods: {}", sandbox.mods.len()));
            for mod_entry in &sandbox.mods {
                ui.label(egui::RichText::new(mod_entry).size(11.0));
            }
            ui.horizontal_wrapped(|ui| {
                if comp::danger_button(ui, "Apply to Active Profile").clicked() {
                    let mut message = format!(
                        "Apply sandbox \"{}\" to the active profile? Existing mods may be replaced.",
                        sandbox.title
                    );
                    if has_staged {
                        message.push_str("\n\nStaged edits are still pending; this action may overwrite them.");
                    }
                    state.confirm(
                        "Apply sandbox",
                        message,
                        Operation::ApplySandbox {
                            id: sandbox.id.clone(),
                        },
                    );
                }
                if comp::danger_button(ui, "Delete sandbox").clicked() {
                    state.confirm(
                        "Delete sandbox",
                        format!(
                            "Delete the sandbox \"{}\" and its mod list? This cannot be undone.",
                            sandbox.title
                        ),
                        Operation::DeleteSandbox {
                            id: sandbox.id.clone(),
                            title: sandbox.title.clone(),
                        },
                    );
                }
            });
        });
    }
}

// --------------------------------------------------------- conflict analyzer

fn render_conflicts(ui: &mut egui::Ui, state: &mut ScreenState, _ctx: &mut ScreenCtx<'_>) {
    ui.horizontal_wrapped(|ui| {
        if comp::button(ui, "Run Standard Conflict Diagnostics").clicked() {
            state.schedule(
                Operation::AnalyzeConflicts { deep: false },
                "Running standard conflict diagnostics…",
            );
        }
        if comp::button(ui, "Run Deep Conflict Diagnostics").clicked() {
            state.schedule(
                Operation::AnalyzeConflicts { deep: true },
                "Running deep conflict diagnostics…",
            );
        }
        let enabled = state.conflict.is_some();
        if ui
            .add_enabled(enabled, egui::Button::new("Export Diagnostic Report"))
            .clicked()
        {
            state.schedule(Operation::ExportConflict, "Exporting conflict report…");
        }
    });
    let export_hint = state.export_validation.as_deref().unwrap_or_else(|| {
        if state.conflict.is_some() {
            ""
        } else {
            "Export requires a completed report."
        }
    });
    comp::validation_message(ui, export_hint);

    if let Some(message) = &state.conflict_error {
        let deep = state.conflict_deep;
        if comp::error_state(ui, message, Some("Retry analysis")) {
            state.schedule(
                Operation::AnalyzeConflicts { deep },
                "Retrying conflict diagnostics…",
            );
        }
    }
    if let Some(message) = &state.conflict_message {
        ui.label(egui::RichText::new(message).color(theme::MUTED));
    }

    let Some(report) = state.conflict.as_ref() else {
        comp::empty_state(
            ui,
            "No conflict analysis yet",
            "Run standard or deep diagnostics to inspect crashes and mod conflicts.",
        );
        return;
    };

    comp::section_card(ui, "Conflict report", |ui| {
        ui.strong(format!("[{}] {}", report.status, report.summary));
        ui.label(&report.confidence_note);
        ui.label(&report.disclaimer);
        ui.label(format!(
            "Crash detected: {} | Errors: {} | Warnings: {} | Indexed mods: {} | Active mods: {}",
            report.crash_detected,
            report.errors,
            report.warnings,
            report.indexed_mods,
            report.active_mods
        ));
        ui.label(format!("Analysis mode: {}", report.analysis_mode));
        if report.timed_out {
            comp::validation_message(ui, "The analysis timed out; results may be partial.");
        }
    });

    if report.suspected.is_empty() && report.missing_references.is_empty() && report.error_lines.is_empty() {
        comp::empty_state(ui, "No conflicts detected", "No suspected mods, missing references or errors were reported.");
    } else {
        let rows: Vec<Vec<String>> = report
            .suspected
            .iter()
            .map(|suspect| {
                vec![
                    suspect.score.clone(),
                    suspect.confidence.clone(),
                    suspect.name.clone(),
                    suspect.active_state.clone(),
                    suspect.reasons.clone(),
                ]
            })
            .collect();
        ui.add_space(6.0);
        ui.strong(format!("Suspected mods ({})", rows.len()));
        comp::table(
            ui,
            "conflict_suspected_table",
            &["Score", "Confidence", "Mod name", "Active state", "Reasons"],
            &rows,
            None,
        );
        if !report.missing_references.is_empty() {
            ui.add_space(6.0);
            ui.strong("Missing references");
            for entry in &report.missing_references {
                ui.label(egui::RichText::new(entry).size(11.0));
            }
        }
        if !report.error_lines.is_empty() {
            ui.add_space(6.0);
            ui.strong("Relevant errors");
            for line in &report.error_lines {
                ui.label(egui::RichText::new(line).size(11.0));
            }
        }
    }
    for limitation in &report.limitations {
        comp::validation_message(ui, limitation);
    }
}

// ------------------------------------------------------- shared confirm dialog

/// Shared destructive-action confirmation, rendered once per frame.
fn render_confirm(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    let Some(confirmation) = state.confirm.clone() else {
        return;
    };
    let mut message = confirmation.message.clone();
    if matches!(confirmation.action, Operation::ApplySandbox { .. })
        && crate::gui::data::session::is_dirty(ctx.data)
    {
        message.push_str("\n\nStaged edits are still pending; this action may overwrite them.");
    }
    match comp::confirm_dialog(
        ui.ctx(),
        confirmation.title,
        &message,
        "Confirm",
        "Cancel",
    ) {
        Some(true) => {
            state.confirm = None;
            state.confirmed = Some(confirmation.action);
        }
        Some(false) => {
            state.confirm = None;
            info(ctx, "Action cancelled.".to_string());
        }
        None => {}
    }
}
