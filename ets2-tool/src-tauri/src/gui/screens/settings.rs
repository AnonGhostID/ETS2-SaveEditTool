//! System sector screen: Paths & Data, Logs & Diagnostics and About.
//!
//! Only shared components and typed data wrappers are used here. Anything that
//! belongs to the shell (route, sub-view, overlays, toasts, Issues, context
//! reload) is requested through a [`ShellCommand`].

use std::path::Path;

use eframe::egui;

use crate::gui::components as comp;
use crate::gui::screens::{
    ContextAction, IssuesTab, Route, ScreenCtx, ShellCommand, SubView,
};
use crate::gui::theme;

// ------------------------------------------------- shell-owned view helpers

/// Validate the sub-view supplied by the shell. An invalid or unrelated view
/// falls back to the tile's default sub-view (docs/ui-spec.md §1.3 rule 3).
fn normalized_sub_view(route: Route, view: SubView) -> SubView {
    match route {
        Route::system_paths_data => match view {
            SubView::SqliteInfo | SubView::DatasetImport => view,
            _ => SubView::PathsStatus,
        },
        Route::system_logs_diagnostics => match view {
            SubView::DiagnosticsPreview | SubView::LogExports => view,
            _ => SubView::LogsList,
        },
        Route::system_about => SubView::AboutInfo,
        _ => SubView::PathsStatus,
    }
}

/// Sub-view buttons are navigation *requests*: the shell owns `ctx.route` and
/// `ctx.sub_view` and decides what renders on the next frame.
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
                navigate(route, *view, ctx);
            }
        }
    });
    ui.add_space(6.0);
}

fn navigate(route: Route, sub_view: SubView, ctx: &mut ScreenCtx<'_>) {
    ctx.commands.push(ShellCommand::Navigate {
        route,
        sub_view: Some(sub_view),
    });
}

fn toast(ctx: &mut ScreenCtx<'_>, message: String, is_error: bool) {
    ctx.commands.push(ShellCommand::Toast { message, is_error });
}

fn error(ctx: &mut ScreenCtx<'_>, message: String) {
    toast(ctx, message, true);
}

fn info(ctx: &mut ScreenCtx<'_>, message: String) {
    toast(ctx, message, false);
}

fn validation(ctx: &mut ScreenCtx<'_>, message: String) {
    // The shell routes the `Validation: ` prefix into the drawer's
    // Validation tab (docs/components.md §1.3).
    toast(ctx, format!("Validation: {message}"), false);
}

fn open_issues(ctx: &mut ScreenCtx<'_>, tab: IssuesTab) {
    ctx.commands.push(ShellCommand::OpenIssues(tab));
}

// ------------------------------------------------------------ local DTO views

/// Display-only projection of `Ets2DataLocationDto`; no backend type is
/// imported into the screen layer.
#[derive(Debug, Clone, Default)]
struct PathView {
    lines: Vec<String>,
    validation_message: String,
}

#[derive(Debug, Clone, Default)]
struct SqliteView {
    info_lines: Vec<String>,
    count_lines: Vec<String>,
}

#[derive(Debug, Clone)]
struct LogRow {
    cells: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
enum ExportKind {
    UserLogs,
    Bundle,
    LogFile,
}

impl ExportKind {
    fn label(self) -> &'static str {
        match self {
            Self::UserLogs => "user logs",
            Self::Bundle => "support bundle",
            Self::LogFile => "diagnostic log file",
        }
    }
}

#[derive(Debug, Clone)]
enum Operation {
    RefreshAll,
    RefreshPaths,
    ChoosePath,
    ResetPath,
    ApplyMode {
        mode: String,
        path: Option<String>,
    },
    RefreshSqlite,
    ImportDatasets,
    RefreshLogs,
    RefreshPreview,
    Export(ExportKind),
    RefreshAboutActivity,
    ClearLogs,
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
    // Paths & Data.
    paths_tried: bool,
    paths: Option<PathView>,
    paths_error: Option<String>,
    custom_path_input: String,

    sqlite_tried: bool,
    sqlite: Option<SqliteView>,
    sqlite_error: Option<String>,

    import_summary: Option<Vec<String>>,
    import_error: Option<String>,

    // Logs & Diagnostics.
    log_status_lines: Vec<String>,
    logs: Vec<LogRow>,
    logs_tried: bool,
    logs_error: Option<String>,
    log_filter: String,
    max_lines_input: String,
    log_filter_validation: Option<String>,

    preview_tried: bool,
    preview: Option<String>,
    preview_error: Option<String>,

    export_message: Option<String>,
    export_error: Option<String>,
    last_export: Option<ExportKind>,
    about_activity_tried: bool,
    about_activity: Vec<LogRow>,
    about_activity_error: Option<String>,

    pending: Option<Operation>,
    confirmed: Option<Operation>,
    confirm: Option<Confirm>,
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

    fn ensure_defaults(&mut self) {
        if self.log_filter.is_empty() {
            self.log_filter = "ALL".to_string();
        }
        if self.max_lines_input.is_empty() {
            self.max_lines_input = "100".to_string();
        }
    }
}

// ------------------------------------------------------------------ rendering

/// Exact screen contract: one state instance and one render function.
pub fn render(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    if let Some(operation) = state.pending.take() {
        execute(operation, state, ctx);
    }
    if let Some(operation) = state.confirmed.take() {
        execute(operation, state, ctx);
    }
    state.ensure_defaults();

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        if comp::button(ui, "Errors & Logs").clicked() {
            open_issues(ctx, IssuesTab::Errors);
        }
        if comp::button(ui, "Refresh all").clicked() {
            state.schedule(Operation::RefreshAll, "Refreshing paths, database and logs…");
        }
        ui.label(
            egui::RichText::new("Path and log changes never discard staged save edits.")
                .size(10.5)
                .color(theme::MUTED),
        );
    });
    ui.add_space(4.0);

    if !state.paths_tried && !state.logs_tried && state.pending.is_none() {
        state.schedule(Operation::RefreshAll, "Loading system status…");
    }
    if ctx.recent_events.is_empty() && !state.about_activity_tried && state.pending.is_none() {
        state.schedule(Operation::RefreshAboutActivity, "Reading recent activity…");
    }
    if let Some(message) = state.loading {
        comp::loading_state(ui, message);
    }

    let view = normalized_sub_view(ctx.route, ctx.sub_view);
    match ctx.route {
        Route::system_paths_data => render_paths(ui, state, ctx, view),
        Route::system_logs_diagnostics => render_logs(ui, state, ctx, view),
        Route::system_about => render_about(ui, state, ctx),
        _ => render_paths(ui, state, ctx, SubView::PathsStatus),
    }

    render_confirm(ui, state, ctx);

    if state.pending.is_none() && state.confirmed.is_none() {
        state.loading = None;
    }
}

fn render_paths(
    ui: &mut egui::Ui,
    state: &mut ScreenState,
    ctx: &mut ScreenCtx<'_>,
    view: SubView,
) {
    sub_view_nav(
        ui,
        ctx,
        Route::system_paths_data,
        view,
        &[
            (SubView::PathsStatus, "Paths status"),
            (SubView::SqliteInfo, "SQLite info"),
            (SubView::DatasetImport, "Dataset import"),
        ],
    );
    match view {
        SubView::SqliteInfo => render_sqlite_info(ui, state, ctx),
        SubView::DatasetImport => render_dataset_import(ui, state, ctx),
        _ => render_path_status(ui, state, ctx),
    }
}

fn render_path_status(
    ui: &mut egui::Ui,
    state: &mut ScreenState,
    ctx: &mut ScreenCtx<'_>,
) {
    comp::section_card(ui, "ETS2 / ATS homedir & data location", |ui| {
        if comp::button(ui, "Refresh status").clicked() {
            state.schedule(Operation::RefreshPaths, "Reading data location…");
        }
        if let Some(message) = state.paths_error.clone() {
            if comp::error_state(ui, message, Some("Retry")) {
                state.schedule(Operation::RefreshPaths, "Reading data location…");
            }
            return;
        }
        if !state.paths_tried {
            return;
        }
        let Some(paths) = state.paths.clone() else {
            comp::empty_state(
                ui,
                "No path information",
                "Choose a custom ETS2 homedir or use the default location.",
            );
            return;
        };
        for line in &paths.lines {
            ui.label(line.as_str());
        }
        if !paths.validation_message.is_empty() {
            comp::validation_message(ui, &paths.validation_message);
        }

        ui.add_space(8.0);
        ui.label(egui::RichText::new("Custom homedir path").strong());
        comp::text_input(ui, &mut state.custom_path_input, "Absolute ETS2 homedir folder");
        let trimmed = state.custom_path_input.trim().to_string();
        if !trimmed.is_empty() && !Path::new(trimmed.as_str()).is_dir() {
            comp::validation_message(ui, "The entered folder does not exist.");
        }
        ui.horizontal_wrapped(|ui| {
            if comp::button(ui, "Browse custom homedir…").clicked() {
                state.confirm(
                    "Change ETS2 data location",
                    "The custom homedir picker changes the active data location and resets the save context. Continue?".to_string(),
                    Operation::ChoosePath,
                );
            }
            if comp::button(ui, "Set custom path").clicked() {
                if trimmed.is_empty() {
                    let message = "Enter an ETS2 homedir folder first.".to_string();
                    validation(ctx, message);
                } else if !Path::new(trimmed.as_str()).is_dir() {
                    let message = "The entered folder does not exist.".to_string();
                    validation(ctx, message);
                } else {
                    state.confirm(
                        "Change ETS2 data location",
                        format!("Use `{trimmed}` as the custom ETS2 homedir?"),
                        Operation::ApplyMode {
                            mode: "custom".to_string(),
                            path: Some(trimmed.to_string()),
                        },
                    );
                }
            }
            if comp::button(ui, "Use default mode").clicked() {
                state.confirm(
                    "Change ETS2 data location mode",
                    "Switch back to the OS default ETS2 location?".to_string(),
                    Operation::ApplyMode {
                        mode: "default".to_string(),
                        path: None,
                    },
                );
            }
            if comp::danger_button(ui, "Reset to default location").clicked() {
                state.confirm(
                    "Reset ETS2 data location",
                    "Reset the saved location and mode to the OS default?".to_string(),
                    Operation::ResetPath,
                );
            }
        });
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new(
                "A location change reloads the save context; the shell prompts first when edits are staged.",
            )
            .size(10.5)
            .color(theme::MUTED),
        );

    });
}

fn render_sqlite_info(ui: &mut egui::Ui, state: &mut ScreenState, _ctx: &mut ScreenCtx<'_>) {
    comp::section_card(ui, "Central SQLite database & ETS2 datasets", |ui| {
        if comp::button(ui, "Refresh counts").clicked() {
            state.schedule(Operation::RefreshSqlite, "Reading SQLite information…");
        }
        if let Some(message) = state.sqlite_error.clone() {
            if comp::error_state(ui, message, Some("Retry")) {
                state.schedule(Operation::RefreshSqlite, "Reading SQLite information…");
            }
            return;
        }
        if !state.sqlite_tried {
            return;
        }
        let Some(sqlite) = state.sqlite.clone() else {
            comp::empty_state(
                ui,
                "No SQLite data",
                "Refresh to read the database path and table counts.",
            );
            return;
        };
        for line in &sqlite.info_lines {
            ui.label(line.as_str());
        }
        for line in &sqlite.count_lines {
            ui.label(line.as_str());
        }
        ui.add_space(6.0);
        if comp::button(ui, "Re-import ETS2 datasets into SQLite").clicked() {
            state.confirm(
                "Re-import ETS2 datasets",
                "Dataset import replaces the cached ETS2 reference data. Continue?".to_string(),
                Operation::ImportDatasets,
            );
        }
        ui.label(
            egui::RichText::new("Switch to Dataset import for the last import summary.")
                .size(10.5)
                .color(theme::MUTED),
        );

    });
}

fn render_dataset_import(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    comp::section_card(ui, "ETS2 dataset import", |ui| {
        if comp::button(ui, "Import ETS2 datasets (force)").clicked() {
            state.confirm(
                "Re-import ETS2 datasets",
                "Dataset import replaces the cached ETS2 reference data. Continue?".to_string(),
                Operation::ImportDatasets,
            );
        }
        if let Some(message) = state.import_error.clone() {
            if comp::error_state(ui, message, Some("Retry")) {
                state.confirm(
                    "Re-import ETS2 datasets",
                    "Dataset import replaces the cached ETS2 reference data. Continue?".to_string(),
                    Operation::ImportDatasets,
                );
            }
            return;
        }
        let Some(summary) = state.import_summary.clone() else {
            comp::empty_state(
                ui,
                "No import yet",
                "Import the bundled ETS2 datasets to populate SQLite reference tables.",
            );
            return;
        };
        for line in summary {
            ui.label(line.as_str());
        }
        if comp::button(ui, "View import errors").clicked() {
            open_issues(ctx, IssuesTab::Errors);
        }
    });
}

fn render_logs(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>, view: SubView) {
    sub_view_nav(
        ui,
        ctx,
        Route::system_logs_diagnostics,
        view,
        &[
            (SubView::LogsList, "User logs"),
            (SubView::DiagnosticsPreview, "Diagnostic preview"),
            (SubView::LogExports, "Exports"),
        ],
    );
    match view {
        SubView::DiagnosticsPreview => render_preview(ui, state, ctx),
        SubView::LogExports => render_exports(ui, state, ctx),
        _ => render_log_list(ui, state, ctx),
    }
}

fn render_log_list(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    comp::section_card(ui, "Application logs & support bundle", |ui| {
        if comp::button(ui, "Refresh logs").clicked() {
            state.schedule(Operation::RefreshLogs, "Reading user logs…");
        }
        if let Some(message) = state.logs_error.clone() {
            if comp::error_state(ui, message, Some("Retry")) {
                state.schedule(Operation::RefreshLogs, "Reading user logs…");
            }
            if comp::button(ui, "View error in Issues").clicked() {
                open_issues(ctx, IssuesTab::Errors);
            }
            return;
        }

        for line in &state.log_status_lines {
            ui.label(line.as_str());
        }

        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            ui.label("Level");
            let options = vec![
                "ALL".to_string(),
                "INFO".to_string(),
                "WARN".to_string(),
                "ERROR".to_string(),
                "DEBUG".to_string(),
            ];
            comp::select(ui, "system_log_level", &mut state.log_filter, &options);
            ui.label("Max lines");
            comp::text_input(ui, &mut state.max_lines_input, "1-1500");
            if comp::button(ui, "Apply filter").clicked() {
                let parsed = state.max_lines_input.trim().parse::<u32>().ok();
                if parsed.map_or(true, |lines| lines == 0 || lines > 1500) {
                    let message = "Max lines must be a number between 1 and 1500.".to_string();
                    state.log_filter_validation = Some(message.clone());
                    validation(ctx, message);
                } else {
                    state.log_filter_validation = None;
                    state.schedule(Operation::RefreshLogs, "Applying log filter…");
                }
            }
        });
        if let Some(message) = &state.log_filter_validation {
            comp::validation_message(ui, message);
        }

        ui.add_space(6.0);
        if !state.logs_tried {
            return;
        }
        if state.logs.is_empty() {
            comp::empty_state(ui, "No user log entries", "No entries match the current filter.");
            return;
        }
        let rows: Vec<Vec<String>> = state
            .logs
            .iter()
            .map(|row| row.cells.clone())
            .collect();
        comp::table(
            ui,
            "system_user_logs",
            &["Time", "Level", "Module", "Message"],
            &rows,
            None,
        );

        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            if comp::button(ui, "Full log view").clicked() {
                open_issues(ctx, IssuesTab::UserLog);
            }
            if comp::danger_button(ui, "Clear user logs").clicked() {
                state.confirm(
                    "Clear user logs",
                    "Clear the user-visible log? The backend keeps a backup when it can.".to_string(),
                    Operation::ClearLogs,
                );
            }
        });
    });
}

fn render_preview(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    comp::section_card(ui, "Diagnostic log preview", |ui| {
        if comp::button(ui, "Refresh preview").clicked() {
            state.schedule(Operation::RefreshPreview, "Reading diagnostic log preview…");
        }
        if let Some(message) = state.preview_error.clone() {
            if comp::error_state(ui, message, Some("Retry")) {
                state.schedule(Operation::RefreshPreview, "Reading diagnostic log preview…");
            }
            if comp::button(ui, "View error in Issues").clicked() {
                open_issues(ctx, IssuesTab::Errors);
            }
            return;
        }
        if !state.preview_tried {
            return;
        }
        let Some(preview) = state.preview.clone() else {
            comp::empty_state(
                ui,
                "No diagnostic preview",
                "The diagnostic log has no readable content yet.",
            );
            return;
        };
        if preview.trim().is_empty() {
            comp::empty_state(
                ui,
                "No diagnostic preview",
                "The diagnostic log is currently empty.",
            );
            return;
        }
        egui::ScrollArea::vertical()
            .id_salt("system_diagnostic_preview")
            .max_height(260.0)
            .show(ui, |ui| {
                ui.monospace(preview);
            });
    });
}

fn render_exports(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    comp::section_card(ui, "Log exports", |ui| {
        ui.horizontal_wrapped(|ui| {
            if comp::button(ui, "Export user logs").clicked() {
                state.schedule(Operation::Export(ExportKind::UserLogs), "Exporting user logs…");
            }
            if comp::button(ui, "Export support bundle").clicked() {
                state.schedule(Operation::Export(ExportKind::Bundle), "Exporting support bundle…");
            }
            if comp::button(ui, "Export diagnostic log (.txt)").clicked() {
                state.schedule(
                    Operation::Export(ExportKind::LogFile),
                    "Exporting diagnostic log…",
                );
            }
        });
        if let Some(message) = state.export_error.clone() {
            if comp::error_state(ui, message, Some("Retry")) {
                if let Some(kind) = state.last_export {
                    state.schedule(Operation::Export(kind), "Retrying export…");
                }
            }
            if comp::button(ui, "View error in Issues").clicked() {
                open_issues(ctx, IssuesTab::Errors);
            }
            return;
        }
        if let Some(message) = &state.export_message {
            ui.label(message.as_str());
        } else {
            comp::empty_state(
                ui,
                "No exports yet",
                "Export user logs, a support bundle or the diagnostic log file.",
            );
        }
    });
}

fn render_about(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    comp::section_card(ui, "About Save-Edit Tool", |ui| {
        ui.label(
            egui::RichText::new(format!("Version {}", env!("CARGO_PKG_VERSION"))).strong(),
        );
        let language = crate::gui::data::context::current_language(ctx.data);
        let available = crate::gui::data::context::available_languages(ctx.data).len();
        ui.label(format!("Language: {language} ({available} available)"));
        ui.label(
            egui::RichText::new(
                "Language is a status indicator only: the interface text stays English.",
            )
            .size(10.5)
            .color(theme::MUTED),
        );
        ui.add_space(6.0);
        ui.label("Architecture: native eframe/egui desktop app — no browser runtime.");
        if comp::button(ui, "Errors & Logs").clicked() {
            open_issues(ctx, IssuesTab::Errors);
        }
    });
    let shell_rows: Vec<Vec<String>> = ctx
        .recent_events
        .iter()
        .rev()
        .take(5)
        .map(|(event, payload)| vec![event.clone(), payload.clone()])
        .collect();
    let using_shell_events = !shell_rows.is_empty();
    ui.add_space(8.0);
    comp::section_card(ui, "Recent activity", |ui| {
        if comp::button(ui, "Refresh activity").clicked() {
            state.schedule(Operation::RefreshAboutActivity, "Reading recent activity…");
        }
        if comp::button(ui, "Open full activity").clicked() {
            open_issues(ctx, IssuesTab::Activity);
        }
        if using_shell_events {
            comp::table(
                ui,
                "about_recent_activity",
                &["Event", "Payload"],
                &shell_rows,
                None,
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Live shell events (last five).")
                    .size(10.5)
                    .color(theme::MUTED),
            );
            return;
        }
        if let Some(message) = state.about_activity_error.clone() {
            if comp::error_state(ui, message, Some("Retry")) {
                state.schedule(Operation::RefreshAboutActivity, "Reading recent activity…");
            }
            return;
        }
        if !state.about_activity_tried {
            return;
        }
        if state.about_activity.is_empty() {
            comp::empty_state(
                ui,
                "No activity recorded",
                "No shell events or user-log entries yet.",
            );
            return;
        }
        let rows: Vec<Vec<String>> = state
            .about_activity
            .iter()
            .map(|row| row.cells.clone())
            .collect();
        comp::table(
            ui,
            "about_recent_activity",
            &["Time", "Level", "Module", "Message"],
            &rows,
            None,
        );
    });
}

// ----------------------------------------------------------- operation runner

fn execute(operation: Operation, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    match operation {
        Operation::RefreshAll => {
            refresh_paths(state, ctx);
            refresh_sqlite(state, ctx);
            refresh_logs(state, ctx);
            refresh_preview(state, ctx);
        }
        Operation::RefreshPaths => refresh_paths(state, ctx),
        Operation::RefreshSqlite => refresh_sqlite(state, ctx),
        Operation::RefreshLogs => refresh_logs(state, ctx),
        Operation::RefreshPreview => refresh_preview(state, ctx),
        Operation::RefreshAboutActivity => refresh_about_activity(state, ctx),
        Operation::ChoosePath => {
            match crate::gui::data::system::choose_data_location(ctx.data, None) {
                Ok(Some(location)) => {
                    apply_location_result(
                        location.mode,
                        location.custom_homedir,
                        "Custom ETS2 data directory updated.",
                        state,
                        ctx,
                    );
                }
                Ok(None) => info(ctx, "No directory was chosen.".to_string()),
                Err(err) => {
                    state.paths_error = Some(err.message.clone());
                    error(ctx, err.message);
                }
            }
        }
        Operation::ResetPath => {
            match crate::gui::data::system::reset_data_location(ctx.data) {
                Ok(location) => apply_location_result(
                    location.mode,
                    location.custom_homedir,
                    "ETS2 data location reset to the OS default.",
                    state,
                    ctx,
                ),
                Err(err) => {
                    state.paths_error = Some(err.message.clone());
                    error(ctx, err.message);
                }
            }
        }
        Operation::ApplyMode { mode, path } => {
            match crate::gui::data::system::set_data_location_mode(ctx.data, mode, path) {
                Ok(location) => apply_location_result(
                    location.mode,
                    location.custom_homedir,
                    "ETS2 data location mode updated.",
                    state,
                    ctx,
                ),
                Err(err) => {
                    state.paths_error = Some(err.message.clone());
                    error(ctx, err.message);
                }
            }
        }
        Operation::ImportDatasets => {
            state.import_error = None;
            match crate::gui::data::system::import_datasets(ctx.data, true) {
                Ok(summary) => {
                    state.import_summary = Some(vec![
                        format!("Dataset version: {}", summary.dataset_version),
                        format!(
                            "Countries: {} · Cities: {} · Companies: {} · Offices: {}",
                            summary.country_count,
                            summary.city_count,
                            summary.company_count,
                            summary.office_count
                        ),
                        format!("Force re-import: {}", summary.force),
                    ]);
                    state
                        .import_summary
                        .as_mut()
                        .expect("import summary just set")
                        .extend(summary.warnings.into_iter().map(|warning| format!("Warning: {warning}")));
                    info(
                        ctx,
                        format!(
                            "Imported datasets ({} cities, {} companies).",
                            summary.city_count, summary.company_count
                        ),
                    );
                    refresh_sqlite(state, ctx);
                }
                Err(err) => {
                    state.import_error = Some(err.message.clone());
                    error(ctx, err.message);
                }
            }
        }
        Operation::Export(kind) => {
            state.export_error = None;
            state.last_export = Some(kind);
            let outcome = match kind {
                ExportKind::UserLogs => crate::gui::data::system::export_user_logs(ctx.data)
                    .map(|path| Some(path)),
                ExportKind::Bundle => crate::gui::data::system::export_logs_bundle(ctx.data),
                ExportKind::LogFile => crate::gui::data::system::export_log_file(ctx.data),
            };
            match outcome {
                Ok(Some(path)) => {
                    let message = format!("Exported {}: {path}", kind.label());
                    state.export_message = Some(message.clone());
                    info(ctx, message);
                }
                Ok(None) => {
                    let message = format!("No {} was produced.", kind.label());
                    state.export_message = Some(message.clone());
                    info(ctx, message);
                }
                Err(err) => {
                    state.export_error = Some(err.message.clone());
                    error(ctx, err.message);
                }
            }
        }
        Operation::ClearLogs => {
            match crate::gui::data::system::clear_user_logs(ctx.data) {
                Ok(Some(backup)) => {
                    state.logs.clear();
                    state.log_status_lines.clear();
                    state.preview = None;
                    let message = format!("Cleared user logs (backup: {backup}).");
                    state.export_message = Some(message.clone());
                    info(ctx, message);
                    refresh_logs(state, ctx);
                    refresh_preview(state, ctx);
                }
                Ok(None) => {
                    state.logs.clear();
                    let message = "User logs were already empty.".to_string();
                    state.export_message = Some(message.clone());
                    info(ctx, message);
                    refresh_logs(state, ctx);
                }
                Err(err) => {
                    state.logs_error = Some(err.message.clone());
                    error(ctx, err.message);
                }
            }
        }
    }
}

fn apply_location_result(
    mode: impl std::fmt::Debug,
    custom_homedir: Option<String>,
    success: &'static str,
    state: &mut ScreenState,
    ctx: &mut ScreenCtx<'_>,
) {
    state.paths_error = None;
    info(ctx, success.to_string());
    refresh_paths(state, ctx);
    refresh_sqlite(state, ctx);
    // Context reload goes through the shell's unsaved-changes guard.
    ctx.commands
        .push(ShellCommand::RequestContextChange(ContextAction::Reload));
    let _ = (mode, custom_homedir);
}

fn refresh_paths(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    match crate::gui::data::system::data_location(ctx.data) {
        Ok(location) => {
            let validation = location.validation;
            let mut lines = vec![
                format!("Mode: {:?}", location.mode),
                format!(
                    "Game data dir: {}",
                    location.game_data_dir.as_deref().unwrap_or("-")
                ),
                format!(
                    "Profiles dir: {} (exists: {})",
                    location.profiles_dir.as_deref().unwrap_or("-"),
                    validation.profiles_dir_exists
                ),
                format!(
                    "Local mod dir: {} (exists: {})",
                    location.local_mod_dir.as_deref().unwrap_or("-"),
                    validation.local_mod_dir_exists
                ),
                format!(
                    "Config file: {}",
                    location.config_file.as_deref().unwrap_or("-")
                ),
                format!(
                    "Profiles found: {} · Local mods: {}",
                    validation.profiles_count, validation.local_mods_count
                ),
            ];
            if let Some(path) = location.homedir_root.as_deref() {
                lines.push(format!("Homedir root: {path}"));
            }
            if state.custom_path_input.is_empty() {
                if let Some(path) = location.custom_homedir.as_deref() {
                    state.custom_path_input = path.to_string();
                }
            }
            state.paths = Some(PathView {
                lines,
                validation_message: validation.message_key,
            });
            state.paths_error = None;
        }
        Err(err) => {
            state.paths_error = Some(err.message.clone());
            error(ctx, err.message);
        }
    }
    state.paths_tried = true;
}

fn refresh_sqlite(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    let mut view = SqliteView::default();
    let mut failure: Option<String> = None;
    match crate::gui::data::system::sqlite_info(ctx.data) {
        Ok(info) => {
            view.info_lines = vec![
                format!("Database: {}", info.path),
                format!(
                    "Size: {} bytes · Tables: {}",
                    info.size_bytes,
                    info.tables.len()
                ),
                format!("Tables: {}", info.tables.join(", ")),
            ];
        }
        Err(err) => failure = Some(err.message),
    }
    match crate::gui::data::system::sqlite_table_counts(ctx.data) {
        Ok(counts) => {
            view.count_lines = vec![
                format!(
                    "Companies: {} · Save snapshots: {} · Depots: {}",
                    counts.ets2_companies, counts.ets_save_snapshot, counts.ets_save_depots
                ),
                format!(
                    "Visited cities: {} · Cargo: {} · Job links: {}",
                    counts.ets_save_visited_cities,
                    counts.ets_save_transport_cargo,
                    counts.ets_job_links
                ),
                format!(
                    "Dispatcher jobs: {} · VTC companies: {} · VTC members: {}",
                    counts.dispatcher_jobs, counts.vtc_companies, counts.vtc_company_members
                ),
            ];
        }
        Err(err) => {
            if failure.is_none() {
                failure = Some(err.message);
            }
        }
    }
    if let Some(message) = failure {
        state.sqlite_error = Some(message.clone());
        error(ctx, message);
    } else {
        state.sqlite_error = None;
        state.sqlite = Some(view);
    }
    state.sqlite_tried = true;
}

fn refresh_logs(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    let filter = state.log_filter.trim();
    let filter = if filter.is_empty() || filter == "ALL" {
        None
    } else {
        Some(filter.to_string())
    };
    let max_lines = state
        .max_lines_input
        .trim()
        .parse::<u32>()
        .ok()
        .filter(|lines| *lines > 0 && *lines <= 1500);
    let limit = max_lines.unwrap_or(100);
    let mut failure: Option<String> = None;
    match crate::gui::data::system::log_status(ctx.data) {
        Ok(status) => {
            state.log_status_lines = vec![
                format!("Log file: {}", status.log_file_path),
                format!(
                    "Size: {} bytes · Modified: {}",
                    status.log_size_bytes,
                    status.last_modified_utc.unwrap_or_else(|| "-".to_string())
                ),
                format!(
                    "Warnings: {} · Errors: {}",
                    status.warning_count, status.error_count
                ),
            ];
        }
        Err(err) => failure = Some(err.message),
    }
    match crate::gui::data::system::user_logs_filtered(ctx.data, filter, Some(limit)) {
        Ok(entries) => {
            state.logs = entries
                .into_iter()
                .map(|entry| LogRow {
                    cells: vec![entry.timestamp, entry.level, entry.module, entry.message],
                })
                .collect();
        }
        Err(err) => {
            if failure.is_none() {
                failure = Some(err.message);
            }
        }
    }
    if let Some(message) = failure {
        state.logs_error = Some(message.clone());
        error(ctx, message);
    } else {
        state.logs_error = None;
    }
    state.logs_tried = true;
}

fn refresh_about_activity(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    match crate::gui::data::system::user_logs_filtered(ctx.data, None, Some(5)) {
        Ok(entries) => {
            state.about_activity = entries
                .into_iter()
                .map(|entry| LogRow {
                    cells: vec![entry.timestamp, entry.level, entry.module, entry.message],
                })
                .collect();
            state.about_activity_error = None;
        }
        Err(err) => {
            state.about_activity_error = Some(err.message.clone());
            error(ctx, err.message);
        }
    }
    state.about_activity_tried = true;
}
fn refresh_preview(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    match crate::gui::data::system::log_preview(ctx.data) {
        Ok(preview) => {
            state.preview = Some(preview);
            state.preview_error = None;
        }
        Err(err) => {
            state.preview_error = Some(err.message.clone());
            error(ctx, err.message);
        }
    }
    state.preview_tried = true;
}

// ---------------------------------------------------------------- confirm dialog

fn render_confirm(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    let Some(confirmation) = state.confirm.clone() else {
        return;
    };
    let mut message = confirmation.message.clone();
    if matches!(confirmation.action, Operation::ClearLogs)
        && crate::gui::data::session::is_dirty(ctx.data)
    {
        message.push_str("\n\nStaged save edits are still pending; this action does not change them.");
    }
    match comp::confirm_dialog(
        ui.ctx(),
        confirmation.title,
        &message,
        "Confirm",
        "Cancel",
    ) {
        Some(true) => state.schedule(confirmation.action, "Applying confirmed action…"),
        Some(false) => info(ctx, "Action cancelled.".to_string()),
        None => {}
    }
}
