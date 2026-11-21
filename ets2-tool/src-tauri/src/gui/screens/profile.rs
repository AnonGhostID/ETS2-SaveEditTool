//! Safety sector screen: Profile Clone & Rename, Profile Sharing,
//! Save Health and Backups & Restore.
//!
//! Only shared components and typed data wrappers are used here. Anything
//! belonging to the shell (breadcrumb, overlays, toasts, Issues, Save) is
//! requested through a `ShellCommand`.

use eframe::egui;

use crate::gui::components as comp;
use crate::gui::screens::{IssuesTab, Route, ScreenCtx, ShellCommand, SubView};
use crate::gui::theme;

// ---------------------------------------------------------------- view enums

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Tile {
    #[default]
    Clone,
    Sharing,
    Health,
    Backups,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum CloneView {
    #[default]
    CloneForm,
    RenameForm,
    CopyControls,
    CopyMods,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ShareView {
    #[default]
    ShareExport,
    ShareImport,
    SharePreview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum HealthView {
    #[default]
    HealthScan,
    HealthReport,
    HealthFix,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum BackupView {
    #[default]
    BackupList,
    BackupPreview,
    BackupConfirm,
}

impl CloneView {
    fn sub_view(self) -> SubView {
        match self {
            CloneView::CloneForm => SubView::CloneForm,
            CloneView::RenameForm => SubView::RenameForm,
            CloneView::CopyControls => SubView::CopyControls,
            CloneView::CopyMods => SubView::CopyMods,
        }
    }
}

impl ShareView {
    fn sub_view(self) -> SubView {
        match self {
            ShareView::ShareExport => SubView::ShareExport,
            ShareView::ShareImport => SubView::ShareImport,
            ShareView::SharePreview => SubView::SharePreview,
        }
    }
}

impl HealthView {
    fn sub_view(self) -> SubView {
        match self {
            HealthView::HealthScan => SubView::HealthScan,
            HealthView::HealthReport => SubView::HealthReport,
            HealthView::HealthFix => SubView::HealthFix,
        }
    }
}

impl BackupView {
    fn sub_view(self) -> SubView {
        match self {
            BackupView::BackupList => SubView::BackupList,
            BackupView::BackupPreview => SubView::BackupPreview,
            BackupView::BackupConfirm => SubView::BackupConfirm,
        }
    }
}

// ------------------------------------------------------------ local DTO views

#[derive(Debug, Clone)]
struct ProfileChoice {
    label: String,
    path: String,
}

#[derive(Debug, Clone)]
struct ArchivePreview {
    archive_path: String,
    detected_name: String,
    final_name: String,
    file_count: usize,
    conflict: bool,
    target_path: String,
    has_manifest: bool,
}

#[derive(Debug, Clone)]
struct HealthProblemView {
    title: String,
    severity: String,
    description: String,
    suggestion: String,
    fix_id: Option<String>,
}

#[derive(Debug, Clone)]
struct HealthReportView {
    status: String,
    summary: String,
    problem_count: usize,
    fixable_count: usize,
    problems: Vec<HealthProblemView>,
}

#[derive(Debug, Clone)]
struct BackupRow {
    id: String,
    created_at: String,
    backup_type: String,
    reason: String,
    file_count: usize,
}

#[derive(Debug, Clone)]
struct BackupPreviewView {
    id: String,
    created_at: String,
    reason: String,
    files: Vec<(String, String, usize)>,
    notes: Vec<String>,
}

// ------------------------------------------------------------ pending work

/// A single deferred wrapper call. Clicking a control schedules an operation;
/// it runs on the next frame so the Detail Pane can show its loading state.
#[derive(Debug, Clone)]
enum Operation {
    RefreshProfiles,
    ValidateClone {
        source: String,
        name: String,
        then_clone: Option<(bool, bool, bool)>,
    },
    ShareContext,
    PickExportDir,
    ApplyExport {
        profile_path: String,
        dir_override: Option<String>,
    },
    PickImportArchive,
    InspectArchive {
        archive: String,
        name_override: Option<String>,
    },
    RefreshHealth,
    ApplyHealthFix {
        fix_id: String,
    },
    RefreshBackups,
    PreviewBackup {
        id: String,
    },
    ApplyClone {
        source: String,
        name: String,
        backup: bool,
        replace_hex: bool,
        replace_text: bool,
    },
    ApplyRename {
        name: String,
    },
    ApplyCopyControls {
        source: String,
        target: String,
    },
    ApplyCopyMods {
        target: String,
    },
    ApplyRestore {
        id: String,
    },
    ApplyImport {
        archive: String,
        name_override: Option<String>,
    },
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
    tile: Tile,
    clone_view: CloneView,
    share_view: ShareView,
    health_view: HealthView,
    backup_view: BackupView,

    // Shared profile list used by clone and by both copy actions.
    profiles: Vec<ProfileChoice>,
    profiles_loaded: bool,
    profiles_error: Option<String>,
    clone_source_label: String,
    target_profile_label: String,

    // Clone form.
    clone_new_name: String,
    clone_backup: Option<bool>,
    clone_replace_hex: bool,
    clone_replace_text: bool,
    clone_validation: Option<(bool, String, Option<String>)>,
    clone_error: Option<String>,
    clone_message: Option<String>,

    // Rename and copy forms.
    rename_name: String,
    rename_error: Option<String>,
    rename_message: Option<String>,
    copy_controls_error: Option<String>,
    copy_controls_message: Option<String>,
    copy_mods_error: Option<String>,
    copy_mods_message: Option<String>,

    // Profile sharing.
    share_context_loaded: bool,
    share_context_error: Option<String>,
    share_profile_path: Option<String>,
    share_profile_name: Option<String>,
    share_selected_game: String,
    share_default_dir: String,
    share_can_export: bool,
    share_can_import: bool,
    share_path_error: Option<String>,
    export_dir: String,
    export_error: Option<String>,
    export_message: Option<String>,
    import_archive: String,
    import_name_override: String,
    import_preview: Option<ArchivePreview>,
    import_error: Option<String>,
    import_message: Option<String>,
    health_fix_candidate: Option<(String, String)>,

    // Save health.
    health_report: Option<HealthReportView>,
    health_error: Option<String>,
    health_message: Option<String>,

    // Backups and restore.
    backups_tried: bool,
    backups: Vec<BackupRow>,
    selected_backup: Option<usize>,
    backup_preview_tried: bool,
    backup_preview: Option<BackupPreviewView>,
    backup_error: Option<String>,
    backup_message: Option<String>,

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

fn validation(ctx: &mut ScreenCtx<'_>, message: String) {
    // The shell routes the `Validation: ` prefix into the drawer's
    // Validation tab (docs/components.md §1.3).
    toast(ctx, format!("Validation: {message}"), false);
}

fn labels(state: &ScreenState) -> Vec<String> {
    state.profiles.iter().map(|p| p.label.clone()).collect()
}

fn path_for_label(state: &ScreenState, label: &str) -> Option<String> {
    state
        .profiles
        .iter()
        .find(|p| p.label == label)
        .map(|p| p.path.clone())
}

fn optional_override(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

// ------------------------------------------------------------------ rendering

fn route_for_tile(tile: Tile) -> Route {
    match tile {
        Tile::Clone => Route::safety_profile_clone_rename,
        Tile::Sharing => Route::safety_profile_sharing,
        Tile::Health => Route::safety_save_health,
        Tile::Backups => Route::safety_backups_restore,
    }
}

fn default_view(tile: Tile) -> SubView {
    match tile {
        Tile::Clone => SubView::CloneForm,
        Tile::Sharing => SubView::ShareExport,
        Tile::Health => SubView::HealthScan,
        Tile::Backups => SubView::BackupList,
    }
}

fn navigate(route: Route, sub_view: SubView, ctx: &mut ScreenCtx<'_>) {
    ctx.commands.push(ShellCommand::Navigate {
        route,
        sub_view: Some(sub_view),
    });
}

/// Exact screen contract: one state instance and a render function.
pub fn render(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    // Deferred work runs at frame start; the click frame itself shows the
    // loading row rendered below.
    if let Some(operation) = state.pending.take() {
        execute(operation, state, ctx);
    }
    if let Some(operation) = state.confirmed.take() {
        execute(operation, state, ctx);
    }

    if let Some(tile) = comp::tabs(
        ui,
        state.tile,
        &[
            (Tile::Clone, "Clone & Rename"),
            (Tile::Sharing, "Profile Sharing"),
            (Tile::Health, "Save Health"),
            (Tile::Backups, "Backups & Restore"),
        ],
    ) {
        if tile != state.tile {
            state.tile = tile;
            navigate(route_for_tile(tile), default_view(tile), ctx);
        }
    }

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        if comp::button(ui, "Open Issues").clicked() {
            ctx.commands.push(ShellCommand::OpenIssues(IssuesTab::Errors));
        }
        ui.label(
            egui::RichText::new("Edits stage until Save; backups are created by the backend.")
                .size(10.5)
                .color(theme::MUTED),
        );
    });

    if let Some(message) = state.loading {
        comp::loading_state(ui, message);
    }

    match state.tile {
        Tile::Clone => render_clone(ui, state, ctx),
        Tile::Sharing => render_sharing(ui, state, ctx),
        Tile::Health => render_health(ui, state, ctx),
        Tile::Backups => render_backups(ui, state, ctx),
    }

    render_confirm(ui, state, ctx);

    // Keep a freshly scheduled load visible on its own frame.
    if state.pending.is_none() && state.confirmed.is_none() {
        state.loading = None;
    }
}

// ----------------------------------------------------------- operation runner

fn execute(operation: Operation, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    match operation {
        Operation::RefreshProfiles => {
            let profiles = crate::gui::data::context::find_profiles(ctx.data);
            state.profiles = profiles
                .into_iter()
                .map(|p| {
                    let path = p.path;
                    let label = p
                        .name
                        .filter(|name| !name.is_empty())
                        .unwrap_or_else(|| path.clone());
                    ProfileChoice { label, path }
                })
                .collect();
            state.profiles_loaded = true;
            state.profiles_error = None;
            match crate::gui::data::context::read_last_profile(ctx.data) {
                Ok(Some(path)) => {
                    if state.clone_source_label.is_empty() {
                        state.clone_source_label = state
                            .profiles
                            .iter()
                            .find(|profile| profile.path == path)
                            .map(|profile| profile.label.clone())
                            .unwrap_or(path);
                    }
                }
                Ok(None) => {}
                Err(err) => {
                    state.profiles_error = Some(err.message.clone());
                    error(ctx, err.message);
                }
            }
        }

        Operation::ValidateClone {
            source,
            name,
            then_clone,
        } => {
            match crate::gui::data::safety::validate_clone_target(
                ctx.data,
                source.clone(),
                name.clone(),
            ) {
                Ok(status) => {
                    let message = status.message.clone();
                    let valid = status.valid;
                    state.clone_validation = Some((valid, message.clone(), status.target_path));
                    if !valid {
                        validation(ctx, message);
                    } else if let Some((backup, replace_hex, replace_text)) = then_clone {
                        let target = format!("Clone profile `{name}`? A new profile folder will be created and the backend will create a backup first.");
                        state.confirm(
                            "Clone profile",
                            target,
                            Operation::ApplyClone {
                                source,
                                name,
                                backup,
                                replace_hex,
                                replace_text,
                            },
                        );
                    }
                }
                Err(err) => {
                    state.clone_error = Some(err.message.clone());
                    error(ctx, err.message);
                }
            }
        }

        Operation::ShareContext => {
            match crate::gui::data::safety::share_context(ctx.data, None) {
                Ok(context) => {
                    state.share_profile_path = context.profile_path;
                    state.share_profile_name = context.profile_name;
                    state.share_selected_game = context.selected_game;
                    state.share_default_dir = context.default_export_dir.unwrap_or_default();
                    state.share_can_export = context.can_export;
                    state.share_can_import = context.can_import;
                    state.share_path_error = context.path_resolution_error;
                    state.share_context_loaded = true;
                    state.share_context_error = None;
                    if state.export_dir.is_empty() {
                        state.export_dir = state.share_default_dir.clone();
                    }
                }
                Err(err) => {
                    state.share_context_error = Some(err.message.clone());
                    error(ctx, err.message);
                }
            }
        }

        Operation::PickExportDir => {
            match crate::gui::data::safety::pick_export_directory(ctx.data) {
                Ok(Some(dir)) => state.export_dir = dir,
                Ok(None) => {}
                Err(err) => {
                    state.export_error = Some(err.message.clone());
                    error(ctx, err.message);
                }
            }
        }

        Operation::ApplyExport {
            profile_path,
            dir_override,
        } => {
            match crate::gui::data::safety::export_shared_profile(
                ctx.data,
                profile_path,
                dir_override,
            ) {
                Ok(result) => {
                    let message = format!(
                        "Exported {} ({} files) to {}",
                        result.profile_name, result.exported_files, result.archive_path
                    );
                    state.export_message = Some(message.clone());
                    state.export_error = None;
                    info(ctx, message);
                }
                Err(err) => {
                    state.export_error = Some(err.message.clone());
                    error(ctx, err.message);
                }
            }
        }

        Operation::PickImportArchive => {
            match crate::gui::data::safety::pick_import_archive(ctx.data) {
                Ok(Some(archive)) => state.import_archive = archive,
                Ok(None) => {}
                Err(err) => {
                    state.import_error = Some(err.message.clone());
                    error(ctx, err.message);
                }
            }
        }

        Operation::InspectArchive {
            archive,
            name_override,
        } => {
            match crate::gui::data::safety::inspect_archive(
                ctx.data,
                archive.clone(),
                name_override.clone(),
            ) {
                Ok(preview) => {
                    state.import_preview = Some(ArchivePreview {
                        archive_path: preview.archive_path,
                        detected_name: preview.detected_profile_name,
                        final_name: preview.final_profile_name,
                        file_count: preview.file_count,
                        conflict: preview.profile_name_conflict,
                        target_path: preview.target_profile_path,
                        has_manifest: preview.has_manifest,
                    });
                    state.import_error = None;
                    state.share_view = ShareView::SharePreview;
                    navigate(
                        Route::safety_profile_sharing,
                        SubView::SharePreview,
                        ctx,
                    );
                }
                Err(err) => {
                    state.import_error = Some(err.message.clone());
                    error(ctx, err.message);
                }
            }
        }

        Operation::RefreshHealth => {
            match crate::gui::data::safety::scan_save_health(ctx.data) {
                Ok(report) => {
                    state.health_report = Some(HealthReportView {
                        status: report.status,
                        summary: report.summary,
                        problem_count: report.problem_count,
                        fixable_count: report.fixable_count,
                        problems: report
                            .problems
                            .into_iter()
                            .map(|problem| HealthProblemView {
                                title: problem.title,
                                severity: problem.severity,
                                description: problem.description,
                                suggestion: problem.suggestion,
                                fix_id: problem.fix_id,
                            })
                            .collect(),
                    });
                    state.health_error = None;
                }
                Err(err) => {
                    state.health_error = Some(err.message.clone());
                    error(ctx, err.message);
                }
            }
        }

        Operation::ApplyHealthFix { fix_id } => {
            match crate::gui::data::safety::apply_health_fix(ctx.data, fix_id.clone(), true) {
                Ok(result) => {
                    let message = if result.applied {
                        format!("Fix {} applied: {}", result.fix_id, result.message)
                    } else {
                        format!("Fix {} was not applied: {}", result.fix_id, result.message)
                    };
                    state.health_message = Some(message.clone());
                    state.health_fix_candidate = None;
                    if result.applied {
                        info(ctx, message);
                    } else {
                        error(ctx, message);
                    }
                    state.schedule(Operation::RefreshHealth, "Re-scanning save health…");
                }
                Err(err) => {
                    state.health_error = Some(err.message.clone());
                    error(ctx, err.message);
                }
            }
        }

        Operation::RefreshBackups => {
            match crate::gui::data::safety::list_backups(ctx.data) {
                Ok(list) => {
                    state.backups = list
                        .into_iter()
                        .map(|backup| BackupRow {
                            id: backup.backup_id,
                            created_at: backup.created_at_utc,
                            backup_type: backup.backup_type,
                            reason: backup.action_reason,
                            file_count: backup.file_count,
                        })
                        .collect();
                    if state.selected_backup.is_some_and(|index| index >= state.backups.len()) {
                        state.selected_backup = None;
                    }
                    state.backups_tried = true;
                    state.backup_error = None;
                }
                Err(err) => {
                    state.backup_error = Some(err.message.clone());
                    error(ctx, err.message);
                }
            }
        }

        Operation::PreviewBackup { id } => {
            match crate::gui::data::safety::preview_backup(ctx.data, id.clone()) {
                Ok(preview) => {
                    state.backup_preview = Some(BackupPreviewView {
                        id: preview.backup_id,
                        created_at: preview.created_at_utc,
                        reason: preview.action_reason,
                        files: preview
                            .files
                            .into_iter()
                            .map(|file| {
                                (
                                    file.relative_path,
                                    file.status,
                                    file.change_count,
                                )
                            })
                            .collect(),
                        notes: preview.notes,
                    });
                    state.backup_preview_tried = true;
                    state.backup_error = None;
                    state.backup_view = BackupView::BackupPreview;
                    navigate(Route::safety_backups_restore, SubView::BackupPreview, ctx);
                }
                Err(err) => {
                    state.backup_error = Some(err.message.clone());
                    error(ctx, err.message);
                }
            }
        }

        Operation::ApplyClone {
            source,
            name,
            backup,
            replace_hex,
            replace_text,
        } => {
            match crate::gui::data::safety::clone_profile(
                ctx.data,
                source,
                name,
                backup,
                replace_hex,
                replace_text,
            ) {
                Ok(message) => {
                    state.clone_message = Some(message.clone());
                    state.clone_error = None;
                    state.clone_validation = None;
                    info(ctx, message);
                    state.schedule(Operation::RefreshProfiles, "Refreshing profiles…");
                }
                Err(err) => {
                    state.clone_error = Some(err.message.clone());
                    error(ctx, err.message);
                }
            }
        }

        Operation::ApplyRename { name } => {
            match crate::gui::data::safety::rename_profile(ctx.data, name) {
                Ok(message) => {
                    state.rename_message = Some(message.clone());
                    state.rename_error = None;
                    info(ctx, message);
                    state.schedule(Operation::RefreshProfiles, "Refreshing profiles…");
                }
                Err(err) => {
                    state.rename_error = Some(err.message.clone());
                    error(ctx, err.message);
                }
            }
        }

        Operation::ApplyCopyControls { source, target } => {
            match crate::gui::data::safety::copy_controls(ctx.data, source, target) {
                Ok(message) => {
                    state.copy_controls_message = Some(message.clone());
                    state.copy_controls_error = None;
                    info(ctx, message);
                }
                Err(err) => {
                    state.copy_controls_error = Some(err.message.clone());
                    error(ctx, err.message);
                }
            }
        }

        Operation::ApplyCopyMods { target } => {
            match crate::gui::data::safety::copy_mods(ctx.data, target) {
                Ok(message) => {
                    state.copy_mods_message = Some(message.clone());
                    state.copy_mods_error = None;
                    info(ctx, message);
                }
                Err(err) => {
                    state.copy_mods_error = Some(err.message.clone());
                    error(ctx, err.message);
                }
            }
        }

        Operation::ApplyRestore { id } => {
            match crate::gui::data::safety::restore_backup(ctx.data, id.clone(), true) {
                Ok(result) => {
                    let message = format!(
                        "Restored {} files from backup {}.",
                        result.restored_file_count, result.backup_id
                    );
                    state.backup_message = Some(message.clone());
                    state.backup_error = None;
                    state.backup_preview = None;
                    info(ctx, message);
                    state.schedule(Operation::RefreshBackups, "Refreshing backups…");
                }
                Err(err) => {
                    state.backup_error = Some(err.message.clone());
                    error(ctx, err.message);
                }
            }
        }

        Operation::ApplyImport {
            archive,
            name_override,
        } => {
            match crate::gui::data::safety::import_shared_profile(
                ctx.data,
                archive,
                name_override,
            ) {
                Ok(result) => {
                    let message = format!(
                        "Imported profile `{}` ({} files).",
                        result.profile_name, result.imported_files
                    );
                    state.import_message = Some(message.clone());
                    state.import_error = None;
                    state.import_preview = None;
                    info(ctx, message);
                    state.schedule(Operation::RefreshProfiles, "Refreshing profiles…");
                }
                Err(err) => {
                    state.import_error = Some(err.message.clone());
                    error(ctx, err.message);
                }
            }
        }
    }
}

// --------------------------------------------------------------- clone tile

fn render_clone(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    if let Some(view) = comp::tabs(
        ui,
        state.clone_view,
        &[
            (CloneView::CloneForm, "Clone"),
            (CloneView::RenameForm, "Rename"),
            (CloneView::CopyControls, "Copy controls"),
            (CloneView::CopyMods, "Copy mods"),
        ],
    ) {
        if view != state.clone_view {
            state.clone_view = view;
            navigate(
                Route::safety_profile_clone_rename,
                view.sub_view(),
                ctx,
            );
        }
    }
    ui.add_space(6.0);

    if !state.profiles_loaded && state.profiles_error.is_none() && state.loading.is_none() {
        state.schedule(Operation::RefreshProfiles, "Loading profiles…");
    }

    let options = labels(state);
    comp::section_card(ui, "Source and target profiles", |ui| {
        if state.clone_source_label.is_empty() {
            if let Some(first) = options.first() {
                state.clone_source_label = first.clone();
            }
        }
        comp::form_field(ui, "Source profile", |ui| {
            comp::select(ui, "profile_clone_source", &mut state.clone_source_label, &options);
        });
        comp::form_field(ui, "Target profile for copy actions", |ui| {
            comp::select(
                ui,
                "profile_copy_target",
                &mut state.target_profile_label,
                &options,
            );
        });
        if comp::button(ui, "Refresh profiles").clicked() {
            state.schedule(Operation::RefreshProfiles, "Loading profiles…");
        }
        if let Some(err) = &state.profiles_error {
            if comp::error_state(ui, &format!("Profile load failed: {err}"), Some("Retry")) {
                state.schedule(Operation::RefreshProfiles, "Loading profiles…");
            }
        }
        if state.profiles_loaded && options.is_empty() {
            comp::empty_state(
                ui,
                "No profiles found",
                "Choose a profile in the Context Column first.",
            );
        }
    });

    match state.clone_view {
        CloneView::CloneForm => render_clone_form(ui, state, ctx),
        CloneView::RenameForm => render_rename_form(ui, state, ctx),
        CloneView::CopyControls => render_copy_controls(ui, state, ctx),
        CloneView::CopyMods => render_copy_mods(ui, state, ctx),
    }
}

fn render_clone_form(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    comp::section_card(ui, "Clone active profile", |ui| {
        comp::form_field(ui, "New profile name", |ui| {
            comp::text_input(ui, &mut state.clone_new_name, "Name for the cloned profile");
        });

        let mut backup = state.clone_backup.unwrap_or(true);
        comp::toggle(
            ui,
            "profile_clone_backup",
            "Create backup before cloning (recommended)",
            &mut backup,
        );
        state.clone_backup = Some(backup);
        comp::toggle(
            ui,
            "profile_clone_hex",
            "Update hex folder identifier",
            &mut state.clone_replace_hex,
        );
        comp::toggle(
            ui,
            "profile_clone_text",
            "Update internal profile.sii name",
            &mut state.clone_replace_text,
        );
        ui.label(
            egui::RichText::new(
                "Cloning writes a new profile folder. Validation runs first and the clone must be confirmed before anything is written; the backend creates the pre-write backup.",
            )
            .size(10.5)
            .color(theme::MUTED),
        );

        ui.horizontal(|ui| {
            if comp::button(ui, "Validate target").clicked() {
                let source = path_for_label(state, &state.clone_source_label);
                let name = state.clone_new_name.trim().to_string();
                match (source, name) {
                    (Some(source), name) if !name.is_empty() => {
                        state.schedule(
                            Operation::ValidateClone {
                                source,
                                name,
                                then_clone: None,
                            },
                            "Validating clone target…",
                        );
                    }
                    _ => {
                        let message = "Select a source profile and enter a new profile name."
                            .to_string();
                        state.clone_validation = Some((false, message.clone(), None));
                        validation(ctx, message);
                    }
                }
            }
            if comp::button(ui, "Clone active profile").clicked() {
                let source = path_for_label(state, &state.clone_source_label);
                let name = state.clone_new_name.trim().to_string();
                match (source, name) {
                    (Some(source), name) if !name.is_empty() => {
                        let options = (
                            state.clone_backup.unwrap_or(true),
                            state.clone_replace_hex,
                            state.clone_replace_text,
                        );
                        state.schedule(
                            Operation::ValidateClone {
                                source,
                                name,
                                then_clone: Some(options),
                            },
                            "Validating clone target…",
                        );
                    }
                    _ => {
                        let message = "Select a source profile and enter a new profile name."
                            .to_string();
                        state.clone_validation = Some((false, message.clone(), None));
                        validation(ctx, message);
                    }
                }
            }
        });

        if let Some((valid, message, target)) = &state.clone_validation {
            comp::validation_message(ui, message);
            if *valid {
                ui.label(
                    egui::RichText::new(format!("Target: {}", target.as_deref().unwrap_or("—")))
                        .color(theme::SUCCESS),
                );
            }
        }
        if let Some(err) = &state.clone_error {
            if comp::error_state(ui, &format!("Clone failed: {err}"), Some("Retry")) {
                let source = path_for_label(state, &state.clone_source_label);
                let name = state.clone_new_name.trim().to_string();
                if let (Some(source), true) = (source, !name.is_empty()) {
                    state.schedule(
                        Operation::ValidateClone {
                            source,
                            name,
                            then_clone: None,
                        },
                        "Validating clone target…",
                    );
                }
            }
        }
        if let Some(message) = &state.clone_message {
            ui.label(egui::RichText::new(message).color(theme::SUCCESS));
        }
    });
}

fn render_rename_form(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    comp::section_card(ui, "Rename active profile", |ui| {
        comp::form_field(ui, "New profile name", |ui| {
            comp::text_input(ui, &mut state.rename_name, "New name for the active profile");
        });
        if comp::button(ui, "Rename active profile").clicked() {
            let name = state.rename_name.trim().to_string();
            if name.is_empty() {
                validation(ctx, "A non-empty profile name is required.".to_string());
                state.rename_error = Some("A non-empty profile name is required.".to_string());
            } else {
                state.confirm(
                    "Rename profile",
                    format!(
                        "Rename the active profile to `{name}`? This changes profile metadata and must be confirmed."
                    ),
                    Operation::ApplyRename { name },
                );
            }
        }
        if let Some(err) = &state.rename_error {
            comp::validation_message(ui, err);
        }
        if let Some(message) = &state.rename_message {
            ui.label(egui::RichText::new(message).color(theme::SUCCESS));
        }
    });
}

// ---------------------------------------------------------------- copy tools

fn render_copy_controls(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    comp::section_card(ui, "Copy controls.sii to another profile", |ui| {
        let options = labels(state);
        comp::form_field(ui, "Source profile", |ui| {
            comp::select(
                ui,
                "profile_controls_source",
                &mut state.clone_source_label,
                &options,
            );
        });
        comp::form_field(ui, "Target profile", |ui| {
            comp::select(
                ui,
                "profile_controls_target",
                &mut state.target_profile_label,
                &options,
            );
        });
        ui.label(
            egui::RichText::new("Copying overwrites the target profile's controls; confirmation is required.")
                .size(10.5)
                .color(theme::MUTED),
        );
        if comp::button(ui, "Copy controls to target").clicked() {
            let source = path_for_label(state, &state.clone_source_label);
            let target = path_for_label(state, &state.target_profile_label);
            match (source, target) {
                (Some(source), Some(target)) if source != target => {
                    state.confirm(
                        "Copy controls",
                        format!("Overwrite the target profile's controls.sii with `{source}`? The existing target controls will be replaced."),
                        Operation::ApplyCopyControls { source, target },
                    );
                }
                _ => {
                    let message = "Select two different profiles before copying controls."
                        .to_string();
                    state.copy_controls_error = Some(message.clone());
                    validation(ctx, message);
                }
            }
        }
        if let Some(err) = &state.copy_controls_error {
            comp::validation_message(ui, err);
        }
        if let Some(message) = &state.copy_controls_message {
            ui.label(egui::RichText::new(message).color(theme::SUCCESS));
        }
        if state.profiles_loaded && options.is_empty() {
            comp::empty_state(ui, "No profiles found", "Load a save first.");
        }
    });
}

fn render_copy_mods(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    comp::section_card(ui, "Copy active mods to another profile", |ui| {
        let options = labels(state);
        comp::form_field(ui, "Target profile", |ui| {
            comp::select(
                ui,
                "profile_mods_target",
                &mut state.target_profile_label,
                &options,
            );
        });
        ui.label(
            egui::RichText::new("Copying the mod list replaces the target profile's mod selection; confirmation is required.")
                .size(10.5)
                .color(theme::MUTED),
        );
        if comp::button(ui, "Copy mod list to target").clicked() {
            let source = path_for_label(state, &state.clone_source_label);
            let target = path_for_label(state, &state.target_profile_label);
            match (source, target) {
                (Some(_), Some(target)) => {
                    state.confirm(
                        "Copy mods",
                        format!("Copy the active profile's mod list into the selected target profile ({target})? The target's current mod selection will be replaced."),
                        Operation::ApplyCopyMods { target },
                    );
                }
                _ => {
                    let message = "Select a source and a target profile before copying mods."
                        .to_string();
                    state.copy_mods_error = Some(message.clone());
                    validation(ctx, message);
                }
            }
        }
        if let Some(err) = &state.copy_mods_error {
            comp::validation_message(ui, err);
        }
        if let Some(message) = &state.copy_mods_message {
            ui.label(egui::RichText::new(message).color(theme::SUCCESS));
        }
        if state.profiles_loaded && options.is_empty() {
            comp::empty_state(ui, "No profiles found", "Load a save first.");
        }
    });
}

fn render_sharing(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    if let Some(view) = comp::tabs(
        ui,
        state.share_view,
        &[
            (ShareView::ShareExport, "Export ZIP"),
            (ShareView::ShareImport, "Import ZIP"),
            (ShareView::SharePreview, "Archive preview"),
        ],
    ) {
        if view != state.share_view {
            state.share_view = view;
            navigate(Route::safety_profile_sharing, view.sub_view(), ctx);
        }
    }
    ui.add_space(6.0);

    if !state.share_context_loaded && state.share_context_error.is_none() && state.loading.is_none()
    {
        state.schedule(Operation::ShareContext, "Loading profile share context…");
    }

    comp::section_card(ui, "Profile share context", |ui| {
        if state.share_context_loaded {
            ui.label(format!(
                "Game: {} | Profile: {}",
                state.share_selected_game,
                state.share_profile_name.as_deref().unwrap_or("—")
            ));
            ui.label(format!(
                "Export enabled: {} | Import enabled: {}",
                state.share_can_export, state.share_can_import
            ));
            if let Some(path) = &state.share_path_error {
                comp::validation_message(ui, path);
            }
        }
        if comp::button(ui, "Reload share context").clicked() {
            state.schedule(Operation::ShareContext, "Loading profile share context…");
        }
        if let Some(err) = &state.share_context_error {
            if comp::error_state(ui, &format!("Share context failed: {err}"), Some("Retry")) {
                state.schedule(Operation::ShareContext, "Loading profile share context…");
            }
        }
        if state.share_context_loaded && !state.share_can_export && !state.share_can_import {
            comp::empty_state(
                ui,
                "Sharing unavailable",
                "Select a profile in the Context Column to enable export and import.",
            );
        }
    });

    match state.share_view {
        ShareView::ShareExport => render_share_export(ui, state, ctx),
        ShareView::ShareImport => render_share_import(ui, state, ctx),
        ShareView::SharePreview => render_share_preview(ui, state),
    }
}

fn render_share_export(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    comp::section_card(ui, "Export active profile as a ZIP archive", |ui| {
        comp::form_field(ui, "Export folder", |ui| {
            comp::text_input(ui, &mut state.export_dir, "Choose a destination folder");
        });
        if comp::button(ui, "Browse…").clicked() {
            state.schedule(
                Operation::PickExportDir,
                "Opening export directory chooser…",
            );
        }
        if state.export_dir.trim().is_empty() {
            comp::empty_state(
                ui,
                "No export directory chosen",
                "Browse for a folder or type one above before exporting.",
            );
        }
        if comp::button(ui, "Export profile archive").clicked() {
            match state.share_profile_path.clone() {
                Some(profile_path) => {
                    let override_dir = optional_override(&state.export_dir);
                    state.schedule(
                        Operation::ApplyExport {
                            profile_path,
                            dir_override: override_dir,
                        },
                        "Exporting profile archive…",
                    );
                }
                None => validation(
                    ctx,
                    "Load a profile share context before exporting.".to_string(),
                ),
            }
        }
        if let Some(err) = &state.export_error {
            if comp::error_state(ui, &format!("Export failed: {err}"), Some("Retry")) {
                if let Some(profile_path) = state.share_profile_path.clone() {
                    let override_dir = optional_override(&state.export_dir);
                    state.schedule(
                        Operation::ApplyExport {
                            profile_path,
                            dir_override: override_dir,
                        },
                        "Exporting profile archive…",
                    );
                }
            }
        }
        if let Some(message) = &state.export_message {
            ui.label(egui::RichText::new(message).color(theme::SUCCESS));
        }
    });
}

fn render_share_import(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    comp::section_card(ui, "Import a shared profile archive", |ui| {
        comp::form_field(ui, "Archive (.zip)", |ui| {
            comp::text_input(ui, &mut state.import_archive, "Path to a shared profile ZIP");
        });
        if comp::button(ui, "Browse…").clicked() {
            state.schedule(
                Operation::PickImportArchive,
                "Opening archive chooser…",
            );
        }
        comp::form_field(ui, "Profile name override (optional)", |ui| {
            comp::text_input(
                ui,
                &mut state.import_name_override,
                "Use the name from the archive",
            );
        });
        ui.horizontal(|ui| {
            if comp::button(ui, "Inspect archive").clicked() {
                let archive = state.import_archive.trim().to_string();
                if archive.is_empty() {
                    validation(ctx, "Choose an archive before inspecting it.".to_string());
                } else {
                    let name_override = optional_override(&state.import_name_override);
                    state.schedule(
                        Operation::InspectArchive {
                            archive,
                            name_override,
                        },
                        "Inspecting archive…",
                    );
                }
            }
            if comp::button(ui, "Import profile now").clicked() {
                match state.import_preview.clone() {
                    Some(preview) => {
                        state.confirm(
                            "Import shared profile",
                            format!(
                                "Import `{}` from {}? This creates profile `{}` with {} files.",
                                preview.detected_name,
                                preview.archive_path,
                                preview.final_name,
                                preview.file_count
                            ),
                            Operation::ApplyImport {
                                archive: preview.archive_path,
                                name_override: optional_override(&state.import_name_override),
                            },
                        );
                    }
                    None => validation(
                        ctx,
                        "Inspect the archive first; import requires a preview.".to_string(),
                    ),
                }
            }
        });
        if state.import_archive.trim().is_empty() {
            comp::empty_state(
                ui,
                "No archive chosen",
                "Browse for a shared profile ZIP, then inspect it before importing.",
            );
        }
        if let Some(err) = &state.import_error {
            if comp::error_state(ui, &format!("Import failed: {err}"), Some("Retry")) {
                let archive = state.import_archive.trim().to_string();
                if !archive.is_empty() {
                    state.schedule(
                        Operation::InspectArchive {
                            archive,
                            name_override: optional_override(&state.import_name_override),
                        },
                        "Inspecting archive…",
                    );
                }
            }
        }
        if let Some(message) = &state.import_message {
            ui.label(egui::RichText::new(message).color(theme::SUCCESS));
        }
    });
}

fn render_share_preview(ui: &mut egui::Ui, state: &mut ScreenState) {
    comp::section_card(ui, "Archive inspection", |ui| {
        match state.import_preview.clone() {
            Some(preview) => {
                ui.label(format!("Archive: {}", preview.archive_path));
                ui.label(format!(
                    "Detected profile: {} → final name: {}",
                    preview.detected_name, preview.final_name
                ));
                ui.label(format!(
                    "Files: {} | Manifest: {} | Target: {}",
                    preview.file_count,
                    if preview.has_manifest { "yes" } else { "no" },
                    preview.target_path
                ));
                if preview.conflict {
                    comp::validation_message(
                        ui,
                        "A profile with this name already exists; the imported profile will be renamed.",
                    );
                }
                if comp::button(ui, "Import this archive").clicked() {
                    state.confirm(
                        "Import shared profile",
                        format!(
                            "Import `{}` from {}? This creates profile `{}` with {} files.",
                            preview.detected_name,
                            preview.archive_path,
                            preview.final_name,
                            preview.file_count
                        ),
                        Operation::ApplyImport {
                            archive: preview.archive_path,
                            name_override: optional_override(&state.import_name_override),
                        },
                    );
                }
            }
            None => comp::empty_state(
                ui,
                "No archive inspected",
                "Open the Import ZIP tab and inspect an archive to see its preview here.",
            ),
        }
        if let Some(err) = &state.import_error {
            comp::validation_message(ui, err);
        }
    });
}

// ------------------------------------------------------------- health tile

fn render_health(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    if let Some(view) = comp::tabs(
        ui,
        state.health_view,
        &[
            (HealthView::HealthScan, "Scan"),
            (HealthView::HealthReport, "Report"),
            (HealthView::HealthFix, "Fix"),
        ],
    ) {
        if view != state.health_view {
            state.health_view = view;
            navigate(Route::safety_save_health, view.sub_view(), ctx);
        }
    }
    ui.add_space(6.0);

    if state.health_report.is_none() && state.health_error.is_none() && state.loading.is_none() {
        state.schedule(Operation::RefreshHealth, "Scanning active save health…");
    }

    match state.health_view {
        HealthView::HealthScan => {
            comp::section_card(ui, "Active save health scan", |ui| {
                ui.label(
                    "Checks the currently loaded save for repairable problems. This is read-only until a fix is confirmed.",
                );
                if comp::button(ui, "Scan active save health").clicked() {
                    state.schedule(Operation::RefreshHealth, "Scanning active save health…");
                }
                if let Some(err) = &state.health_error {
                    if comp::error_state(ui, &format!("Scan failed: {err}"), Some("Retry")) {
                        state.schedule(Operation::RefreshHealth, "Scanning active save health…");
                    }
                }
                if let Some(message) = &state.health_message {
                    ui.label(egui::RichText::new(message).color(theme::SUCCESS));
                }
            });
        }

        HealthView::HealthReport => {
            comp::section_card(ui, "Health report", |ui| {
                if let Some(report) = &state.health_report {
                    ui.label(
                        egui::RichText::new(format!("{} — {}", report.status, report.summary))
                            .strong(),
                    );
                    ui.label(format!(
                        "Problems: {} | Auto-fixable: {}",
                        report.problem_count, report.fixable_count
                    ));
                }
                if comp::button(ui, "Re-scan").clicked() {
                    state.schedule(Operation::RefreshHealth, "Scanning active save health…");
                }
                if let Some(err) = &state.health_error {
                    if comp::error_state(ui, &format!("Scan failed: {err}"), Some("Retry")) {
                        state.schedule(Operation::RefreshHealth, "Scanning active save health…");
                    }
                }
            });

            let problems: Vec<HealthProblemView> = state
                .health_report
                .as_ref()
                .map(|report| report.problems.clone())
                .unwrap_or_default();
            if problems.is_empty() && state.health_error.is_none() && state.loading.is_none() {
                comp::empty_state(ui, "No issues found", "The active save passed its health check.");
            }
            for problem in problems {
                comp::section_card(ui, &problem.title, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(&problem.severity).color(theme::WARNING).strong(),
                        );
                        if let Some(fix_id) = problem.fix_id.clone() {
                            if comp::button(ui, "Apply auto-fix").clicked() {
                                state.health_fix_candidate =
                                    Some((fix_id, problem.title.clone()));
                                state.health_view = HealthView::HealthFix;
                                navigate(Route::safety_save_health, SubView::HealthFix, ctx);
                                state.confirm(
                                    "Apply save health fix",
                                    format!("Apply the automatic fix for `{}`? The backend will create a backup first.", problem.title),
                                    Operation::ApplyHealthFix { fix_id },
                                );
                            }
                        }
                    });
                    ui.label(&problem.description);
                    ui.label(format!("Suggestion: {}", problem.suggestion));
                });
            }
        }

        HealthView::HealthFix => {
            comp::section_card(ui, "Confirmed fix", |ui| {
                match state.health_fix_candidate.clone() {
                    Some((fix_id, title)) => {
                        ui.label(format!("Selected fix: {title} ({fix_id})"));
                        if comp::danger_button(ui, "Confirm and apply fix").clicked() {
                            state.confirm(
                                "Apply save health fix",
                                format!("Apply the automatic fix for `{title}`? The backend will create a backup first."),
                                Operation::ApplyHealthFix { fix_id },
                            );
                        }
                    }
                    None => comp::empty_state(
                        ui,
                        "No fix selected",
                        "Open the Report tab and choose Apply auto-fix on a problem.",
                    ),
                }
                if let Some(err) = &state.health_error {
                    if comp::error_state(ui, &format!("Fix failed: {err}"), Some("Retry")) {
                        if let Some((fix_id, title)) = state.health_fix_candidate.clone() {
                            state.confirm(
                                "Apply save health fix",
                                format!("Retry the automatic fix for `{title}`? The backend will create a backup first."),
                                Operation::ApplyHealthFix { fix_id },
                            );
                        }
                    }
                }
                if let Some(message) = &state.health_message {
                    ui.label(egui::RichText::new(message).color(theme::SUCCESS));
                }
            });
        }
    }
}

// ------------------------------------------------------------ backups tile

fn render_backups(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    if let Some(view) = comp::tabs(
        ui,
        state.backup_view,
        &[
            (BackupView::BackupList, "Backup list"),
            (BackupView::BackupPreview, "Preview"),
            (BackupView::BackupConfirm, "Restore"),
        ],
    ) {
        if view != state.backup_view {
            state.backup_view = view;
            navigate(Route::safety_backups_restore, view.sub_view(), ctx);
        }
    }
    ui.add_space(6.0);

    if !state.backups_tried && state.backup_error.is_none() && state.loading.is_none() {
        state.schedule(Operation::RefreshBackups, "Listing active save backups…");
    }

    comp::section_card(ui, "Backup status", |ui| {
        ui.label(format!(
            "Backups for this save: {} | Last outcome: {}",
            state.backups.len(),
            state
                .backup_message
                .as_deref()
                .unwrap_or("none yet")
        ));
        if ui.button("Open Issues").clicked() {
            ctx.commands.push(ShellCommand::OpenIssues(IssuesTab::Errors));
        }
    });

    match state.backup_view {
        BackupView::BackupList => render_backup_list(ui, state),
        BackupView::BackupPreview => render_backup_preview(ui, state),
        BackupView::BackupConfirm => render_backup_confirm_view(ui, state, ctx),
    }
}

fn render_backup_list(ui: &mut egui::Ui, state: &mut ScreenState) {
    comp::section_card(ui, "Active save backups", |ui| {
        if comp::button(ui, "Refresh backup list").clicked() {
            state.schedule(Operation::RefreshBackups, "Listing active save backups…");
        }
        if let Some(err) = &state.backup_error {
            if comp::error_state(ui, &format!("Backup list failed: {err}"), Some("Retry")) {
                state.schedule(Operation::RefreshBackups, "Listing active save backups…");
            }
        }
        if state.backups.is_empty() && state.backup_error.is_none() && state.loading.is_none() {
            comp::empty_state(
                ui,
                "No backups for this save",
                "Backups are created before edits are saved and before clone/switch actions.",
            );
            return;
        }

        let rows: Vec<Vec<String>> = state
            .backups
            .iter()
            .map(|backup| {
                vec![
                    backup.id.clone(),
                    backup.created_at.clone(),
                    backup.backup_type.clone(),
                    backup.reason.clone(),
                    backup.file_count.to_string(),
                ]
            })
            .collect();
        if let Some(index) = comp::table(
            ui,
            "profile_backup_table",
            &["Backup ID", "Created (UTC)", "Type", "Reason", "Files"],
            &rows,
            state.selected_backup,
        ) {
            state.selected_backup = Some(index);
            state.backup_preview = None;
            if let Some(backup) = state.backups.get(index) {
                state.schedule(
                    Operation::PreviewBackup {
                        id: backup.id.clone(),
                    },
                    "Loading backup preview…",
                );
            }
        }
    });
}

fn render_backup_preview(ui: &mut egui::Ui, state: &mut ScreenState) {
    comp::section_card(ui, "Backup restore preview", |ui| {
        match state.backup_preview.clone() {
            Some(preview) => {
                ui.label(format!("Backup: {}", preview.id));
                ui.label(format!(
                    "Reason: {} | Created: {}",
                    preview.reason, preview.created_at
                ));
                let files: Vec<Vec<String>> = preview
                    .files
                    .iter()
                    .map(|(path, status, changes)| {
                        vec![path.clone(), status.clone(), changes.to_string()]
                    })
                    .collect();
                if files.is_empty() {
                    comp::empty_state(ui, "No file changes", "This backup has no recorded diffs.");
                } else {
                    comp::table(
                        ui,
                        "profile_backup_preview_table",
                        &["File", "Status", "Changes"],
                        &files,
                        None,
                    );
                }
                for note in &preview.notes {
                    comp::validation_message(ui, note);
                }
                if comp::danger_button(ui, "Restore this backup").clicked() {
                    state.confirm(
                        "Restore backup",
                        format!("Restore backup {}? This overwrites the current save with the backup's files. Your current state is kept as a backup first.", preview.id),
                        Operation::ApplyRestore { id: preview.id },
                    );
                }
            }
            None => comp::empty_state(
                ui,
                "No backup preview",
                "Select a backup in the list to see its restore preview.",
            ),
        }
        if let Some(err) = &state.backup_error {
            if comp::error_state(ui, &format!("Preview failed: {err}"), Some("Retry")) {
                if let Some(index) = state.selected_backup {
                    if let Some(backup) = state.backups.get(index) {
                        state.schedule(
                            Operation::PreviewBackup {
                                id: backup.id.clone(),
                            },
                            "Loading backup preview…",
                        );
                    }
                }
            }
        }
    });
}

fn render_backup_confirm_view(
    ui: &mut egui::Ui,
    state: &mut ScreenState,
    ctx: &mut ScreenCtx<'_>,
) {
    comp::section_card(ui, "Restore a backup", |ui| {
        if crate::gui::data::session::is_dirty(ctx.data) {
            let paths = crate::gui::data::session::dirty_paths(ctx.data);
            comp::validation_message(
                ui,
                &format!(
                    "{} staged file(s) are pending. Restoring a backup overwrites those edits — Save or discard them first. Staged: {}",
                    paths.len(),
                    paths.join(", ")
                ),
            );
        }
        match state.backup_preview.clone() {
            Some(preview) => {
                if comp::danger_button(ui, "Restore this backup").clicked() {
                    state.confirm(
                        "Restore backup",
                        format!("Restore backup {}? This overwrites the current save with the backup's files. Your current state is kept as a backup first.", preview.id),
                        Operation::ApplyRestore { id: preview.id },
                    );
                }
            }
            None => comp::empty_state(
                ui,
                "No backup selected",
                "Choose a backup in the list and inspect its preview first.",
            ),
        }
        if let Some(message) = &state.backup_message {
            ui.label(egui::RichText::new(message).color(theme::SUCCESS));
        }
    });
}

/// Shared destructive-action confirmation, rendered once per frame.
fn render_confirm(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    let Some(confirmation) = state.confirm.clone() else {
        return;
    };
    let mut message = confirmation.message.clone();
    if matches!(
        confirmation.action,
        Operation::ApplyRestore { .. }
            | Operation::ApplyClone { .. }
            | Operation::ApplyImport { .. }
    ) && crate::gui::data::session::is_dirty(ctx.data)
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
        Some(true) => state.schedule(confirmation.action, "Applying confirmed action…"),
        Some(false) => info(ctx, "Action cancelled.".to_string()),
        None => {}
    }
}
