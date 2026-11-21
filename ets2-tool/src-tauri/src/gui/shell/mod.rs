//! Orbit Hub application shell.
//!
//! Four regions are rendered every frame: Context Column, Sector Map, Detail
//! Pane and the always-visible Safety Spine. All navigation goes through
//! [`AppShell::navigate`]; all save-context changes go through
//! [`AppShell::request_context_change`].

pub mod context_column;
pub mod detail_pane;
pub mod issues_drawer;
pub mod overlays;
pub mod safety_spine;
pub mod sector_map;

use std::collections::VecDeque;
use std::sync::mpsc::Receiver;

use eframe::egui;
use eframe::egui::Id;

use crate::gui::components::{self, Toast};
use crate::gui::data::DataContext;
use crate::gui::screens::{
    ContextAction, IssuesTab, Route, ScreenCtx, ScreenStates, Sector, ShellCommand, SubView,
};
use crate::gui::theme;
use crate::tauri::{AppHandle, NativeEvent};

/// A context selector option kept in shell-local form so no screen or shell
/// file has to import backend model types.
#[derive(Debug, Clone)]
pub struct ContextOption {
    pub path: String,
    pub label: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmAction {
    /// Safety Spine Undo is a backup restore, so it is confirmed.
    UndoLast,
}

/// Layers the shell renders itself. Any other visible `Order::Middle` layer
/// is a modal opened from a screen (currently `components::confirm_dialog`).
/// The toast Areas share the Middle order but are not modal windows.
fn is_shell_layer(id: Id) -> bool {
    id == Id::new("Unsaved changes")
        || id == Id::new("Confirm undo")
        || id == Id::new("Context")
        || id == Id::new("orbit_toast_stack")
        || id == Id::new("toasts")
}

pub struct AppShell {
    pub app: AppHandle,
    pub data: DataContext,
    pub event_rx: Receiver<NativeEvent>,
    pub recent_events: VecDeque<(String, String)>,

    // Navigation
    pub sector: Option<Sector>,
    pub route: Route,
    /// Sector Map display mode: seven sectors instead of one sector's tiles.
    pub sectors_overview: bool,

    pub sub_view: SubView,
    pub screen_states: ScreenStates,
    pub scroll_to_top: bool,

    // Context Column
    pub selected_game: String,
    pub game_options: Vec<String>,
    pub profiles: Vec<ContextOption>,
    pub selected_profile_path: Option<String>,
    pub saves: Vec<ContextOption>,
    pub selected_save_path: Option<String>,
    pub available_languages: Vec<(String, String)>,
    pub language: String,
    pub context_popover_open: bool,
    // Values shown in the selectors while the unsaved prompt is open.
    pub pending_game: Option<String>,
    pub pending_profile: Option<String>,
    pub pending_save: Option<String>,


    // Safety Spine and Issues drawer
    pub dirty_paths: usize,
    pub staging_enabled: bool,
    pub backup_count: usize,
    pub backup_outcome: String,
    pub last_save_outcome: String,
    pub can_undo: bool,
    pub undo_label: Option<String>,
    pub validation_issues: Vec<String>,
    pub errors: Vec<String>,
    pub user_logs: Vec<String>,
    pub user_log_filter: String,
    pub user_log_loaded: bool,

    pub toasts: VecDeque<Toast>,

    // Overlays and focus
    pub issues_open: bool,
    pub issues_tab: IssuesTab,
    pub unsaved_prompt: Option<ContextAction>,
    pub confirm: Option<ConfirmAction>,
    pub close_after_prompt: bool,
    pub focus_return: Option<Id>,
    pub focus_index: usize,
    /// True while a screen-owned modal window (e.g. a confirm dialog) is visible.
    pub screen_modal_open: bool,
    /// Focus to hand back to the opener when that modal closes.
    pub screen_modal_focus_return: Option<Id>,
}

impl AppShell {
    pub fn new(cc: &eframe::CreationContext<'_>, app: AppHandle) -> Self {
        theme::apply(&cc.egui_ctx);
        let event_rx = app.subscribe_events();
        let data = DataContext { app: app.clone() };
        let selected_game =
            crate::gui::data::context::selected_game(&data).unwrap_or_else(|_| "ets2".into());
        let mut shell = Self {
            app: app.clone(),
            data,
            event_rx,
            recent_events: VecDeque::with_capacity(64),
            sector: Some(Sector::Overview),
            route: Route::overview_dashboard,
            sub_view: Route::overview_dashboard.default_sub_view(),
            screen_states: ScreenStates::default(),
            scroll_to_top: false,
            sectors_overview: true,

            selected_game,
            game_options: vec!["ets2".to_string(), "ats".to_string()],
            profiles: Vec::new(),
            selected_profile_path: None,
            saves: Vec::new(),
            selected_save_path: None,
            available_languages: Vec::new(),
            language: String::new(),
            context_popover_open: false,
            pending_game: None,
            pending_profile: None,
            pending_save: None,

            dirty_paths: 0,
            staging_enabled: false,
            backup_count: 0,
            backup_outcome: "not loaded".to_string(),
            last_save_outcome: "no save yet".to_string(),
            can_undo: false,
            undo_label: None,
            validation_issues: Vec::new(),
            errors: Vec::new(),
            user_logs: Vec::new(),
            user_log_filter: String::new(),
            user_log_loaded: false,

            toasts: VecDeque::new(),
            issues_open: false,
            issues_tab: IssuesTab::Validation,
            unsaved_prompt: None,
            confirm: None,
            close_after_prompt: false,
            focus_return: None,
            focus_index: 0,
            screen_modal_open: false,
            screen_modal_focus_return: None,
        };
        shell.refresh_languages();
        shell.refresh_context();
        shell.refresh_safety();
        shell
    }

    // ---------------------------------------------------------------- routing

    /// Single navigation authority. Sets sector, tile and a validated
    /// sub-view, and requests a scroll-to-top on the Detail Pane. Never
    /// prompts and never discards staged edits.
    pub fn navigate(&mut self, route: Route, sub_view: Option<SubView>) {
        self.sector = Some(route.sector());
        self.route = route;
        self.sub_view = match sub_view {
            Some(view) if route.accepts(view) => view,
            _ => route.default_sub_view(),
        };
        self.sectors_overview = false;
        self.scroll_to_top = true;

    }

    /// Return the Sector Map to the seven-sector overview. This is map UI
    /// state; the active route and sub-view stay untouched.
    pub fn show_sector_overview(&mut self) {
        self.sectors_overview = true;
    }

    /// Open the Issues drawer without changing the route.
    pub fn open_issues(&mut self, tab: IssuesTab) {
        self.issues_open = true;
        self.issues_tab = tab;
        self.focus_index = 0;
        self.focus_return = Some(Id::new("safety_issues_button"));
    }

    /// Context changes (GAME / PROFILE / SAVE), Reload and Exit are the only
    /// operations that may prompt about staged edits.
    pub fn request_context_change(&mut self, action: ContextAction) {
        if crate::gui::data::session::is_dirty(&self.data) {
            self.unsaved_prompt = Some(action);
        } else {
            self.perform_context_change(action);
        }
    }
    /// Clear the optimistic selector values once a context change resolves.
    pub fn clear_pending(&mut self) {
        self.pending_game = None;
        self.pending_profile = None;
        self.pending_save = None;
    }


    fn perform_context_change(&mut self, action: ContextAction) {
        self.clear_pending();
        match action {

            ContextAction::SwitchGame(game) => {
                if let Err(err) = crate::gui::data::context::set_game(&self.data, game.clone()) {
                    self.push_error(format!("Failed to switch game: {}", err.message));
                    return;
                }
                self.selected_game = game;
                self.selected_profile_path = None;
                self.selected_save_path = None;
                self.refresh_context();
                self.refresh_safety();
                self.push_toast("Game switched.", false);
            }
            ContextAction::SwitchProfile(path) => {
                self.select_profile(path);
                self.refresh_safety();
            }
            ContextAction::SwitchSave(path) => {
                self.select_save(path);
                self.refresh_safety();
            }
            ContextAction::Reload => {
                self.refresh_context();
                self.refresh_safety();
                self.refresh_languages();
                self.push_toast("Profiles and active save reloaded.", false);
            }
            ContextAction::Exit => {
                self.close_after_prompt = true;
            }
        }
    }

    /// Resolve the unsaved prompt. A failed Save keeps staging and cancels
    /// the pending context change; Discard reports its own failure and also
    /// cancels the change.
    pub fn resolve_unsaved_prompt(&mut self, save_first: bool) {
        let Some(action) = self.unsaved_prompt.take() else {
            return;
        };
        if save_first {
            self.flush_pending_changes();
            if crate::gui::data::session::is_dirty(&self.data) {
                self.push_error("Save failed: staged edits remain, context change cancelled.".to_string());
                return;
            }
        } else if let Err(err) = crate::gui::data::session::discard(&self.data) {
            self.push_error(format!("Discard failed: {}", err.message));
            return;
        } else {
            self.push_toast("Staged edits discarded.", false);
        }
        self.perform_context_change(action);
    }

    /// Write staged edits through the backend Save command (which creates
    /// the pre-write backup), then refresh every safety gauge.
    pub fn flush_pending_changes(&mut self) {
        let staged_before = crate::gui::data::session::dirty_paths(&self.data).len();
        match crate::gui::data::session::flush(&self.data) {
            Ok(saved) => {
                let still_staged = crate::gui::data::session::dirty_paths(&self.data).len();
                self.last_save_outcome = if saved.is_empty() {
                    "nothing staged".to_string()
                } else {
                    format!("{} file(s) saved", saved.len())
                };
                if still_staged > 0 {
                    self.push_error(format!(
                        "Partial save: {} file(s) written, {} still staged.",
                        saved.len(),
                        still_staged
                    ));
                } else {
                    self.push_toast(self.last_save_outcome.clone(), false);
                }
                // A flush changes the on-disk hash and the backup list.
                self.screen_states = ScreenStates::default();
                self.refresh_safety();
                self.refresh_context();
            }
            Err(err) => {
                let still_staged = crate::gui::data::session::dirty_paths(&self.data).len();
                let saved = staged_before.saturating_sub(still_staged);
                self.last_save_outcome = "save failed".to_string();
                self.push_error(format!(
                    "Save failed: {} ({} file(s) written, {} still staged.)",
                    err.message, saved, still_staged
                ));
                self.refresh_safety();
            }
        }
    }

    // ------------------------------------------------------------------ data

    fn refresh_languages(&mut self) {
        let languages = crate::gui::data::context::available_languages(&self.data);
        self.available_languages = languages
            .iter()
            .map(|language| (language.code.clone(), language.name.clone()))
            .collect();
        self.language = crate::gui::data::context::current_language(&self.data);
    }

    /// Refresh the profile and save lists from the backend, preserving the
    /// current selections when they are still valid.
    pub fn refresh_context(&mut self) {
        let profiles = crate::gui::data::context::find_profiles(&self.data);
        self.profiles = profiles
            .into_iter()
            .map(|profile| {
                let path = profile.path;
                let label = profile.name.unwrap_or_else(|| path.clone());
                let detail = profile
                    .message
                    .unwrap_or_else(|| if profile.success { "OK" } else { "unavailable" }.to_string());
                ContextOption {
                    path,
                    label,
                    detail,
                }
            })
            .collect();

        if self.selected_profile_path.is_none() {
            let last = crate::gui::data::context::read_last_profile(&self.data)
                .ok()
                .flatten();
            if let Some(last) = last {
                if self.profiles.iter().any(|profile| profile.path == last) {
                    self.selected_profile_path = Some(last);
                }
            }
            if self.selected_profile_path.is_none() {
                self.selected_profile_path = self.profiles.first().map(|profile| profile.path.clone());
            }
        }
        if let Some(path) = self.selected_profile_path.clone() {
            self.select_profile(path);
        } else {
            self.saves.clear();
            self.selected_save_path = None;
        }
    }

    fn select_profile(&mut self, path: String) {
        if let Err(err) = crate::gui::data::context::save_last_profile(&self.data, path.clone()) {
            self.push_error(format!("Could not remember last profile: {}", err.message));
        }
        if let Err(err) = crate::gui::data::context::set_active_profile(&self.data, path.clone()) {
            self.push_error(format!("Failed to activate profile: {}", err.message));
            return;
        }
        self.selected_profile_path = Some(path.clone());
        self.screen_states = ScreenStates::default();
        match crate::gui::data::context::find_saves(&self.data, path) {
            Ok(saves) => {
                self.saves = saves
                    .into_iter()
                    .map(|save| {
                        let path = save.path;
                        let folder = save.folder;
                        let label = save.name.unwrap_or_else(|| folder.clone());
                        ContextOption {
                            path,
                            label,
                            detail: folder,
                        }
                    })
                    .collect();
                let preferred = self
                    .saves
                    .iter()
                    .find(|save| save.detail.eq_ignore_ascii_case("quicksave"))
                    .or_else(|| self.saves.first())
                    .map(|save| save.path.clone());
                if let Some(save) = preferred {
                    self.select_save(save);
                } else {
                    self.selected_save_path = None;
                }
            }
            Err(err) => {
                self.saves.clear();
                self.selected_save_path = None;
                self.push_error(format!("Failed to list saves: {}", err.message));
            }
        }
    }

    fn select_save(&mut self, path: String) {
        if let Some(profile) = self.selected_profile_path.clone() {
            if let Err(err) =
                crate::gui::data::context::load_profile(&self.data, profile, Some(path.clone()))
            {
                self.push_error(format!("Failed to load profile for save: {}", err.message));
                return;
            }
        }
        if let Err(err) = crate::gui::data::context::set_current_save(&self.data, path.clone()) {
            self.push_error(format!("Failed to select save: {}", err.message));
            return;
        }
        self.selected_save_path = Some(path);
        self.screen_states = ScreenStates::default();
    }

    /// Cheap gauges refreshed on every relevant lifecycle event.
    pub fn refresh_safety(&mut self) {
        self.dirty_paths = crate::gui::data::session::dirty_paths(&self.data).len();
        self.staging_enabled = crate::gui::data::session::is_staging_enabled(&self.data);
        match crate::gui::data::context::undo_status(&self.data) {
            Ok(status) => {
                self.can_undo = !crate::gui::data::session::is_dirty(&self.data) && status.can_undo;
                self.undo_label = status.last_undo_label;
            }
            Err(err) => {
                self.can_undo = false;
                self.undo_label = None;
                self.push_error(format!("Undo status unavailable: {}", err.message));
            }
        }
        match crate::gui::data::safety::list_backups(&self.data) {
            Ok(backups) => {
                self.backup_count = backups.len();
                self.backup_outcome = "OK".to_string();
            }
            Err(err) => {
                // Keep the last good count; surface the failure in Issues.
                self.backup_outcome = "list unavailable".to_string();
                self.push_error(format!("Backup list failed: {}", err.message));
            }
        }
    }

    // --------------------------------------------------------------- feedback

    /// Toast plus Issues entry for errors. Validation messages use the
    /// `Validation: ` prefix and land in the drawer's Validation tab.
    pub fn push_toast(&mut self, message: impl Into<String>, is_error: bool) {
        let message = message.into();
        if let Some(validation) = message.strip_prefix("Validation: ") {
            if !self.validation_issues.iter().any(|item| item.as_str() == validation) {
                self.validation_issues.push(validation.to_string());
            }
        }
        if is_error {
            if !self.errors.iter().any(|item| item == &message) {
                self.errors.push(message.clone());
            }
        }
        components::push_toast(&mut self.toasts, message, is_error);
    }

    pub fn push_error(&mut self, message: impl Into<String>) {
        self.push_toast(message, true);
    }

    fn drain_events(&mut self) {
        while let Ok(event) = self.event_rx.try_recv() {
            let summary = event.payload.to_string();
            if event.event == "career://plugin_install_error" {
                let detail = event.payload.as_str().unwrap_or(summary.as_str());
                self.push_error(format!("Plugin install failed: {detail}"));
            }
            if self.recent_events.len() >= 50 {
                self.recent_events.pop_front();
            }
            self.recent_events.push_back((event.event, summary));
        }
    }

    fn handle_command(&mut self, command: ShellCommand) {
        match command {
            ShellCommand::Navigate { route, sub_view } => self.navigate(route, sub_view),
            ShellCommand::OpenIssues(tab) => self.open_issues(tab),
            ShellCommand::Toast { message, is_error } => self.push_toast(message, is_error),
            ShellCommand::Save => self.flush_pending_changes(),
            ShellCommand::RefreshSafety => self.refresh_safety(),
            ShellCommand::RequestContextChange(action) => self.request_context_change(action),
        }
    }

    /// Detect a screen-owned modal from egui's visible layers. Panels and
    /// tooltips use other layer orders; shell windows are excluded by title.
    fn screen_modal_visible(ctx: &egui::Context) -> bool {
        ctx.memory(|memory| {
            let visible = memory.areas().visible_layer_ids();
            memory.layer_ids().any(|layer| {
                layer.order == egui::Order::Middle
                    && visible.contains(&layer)
                    && !is_shell_layer(layer.id)
            })
        })
    }

    fn handle_tabs(&mut self, ctx: &egui::Context) {
        // Spec 2.3: a screen-owned modal traps Tab/Shift+Tab and gives focus
        // back to the opener when it closes. Detection runs before screen
        // rendering, so the layer set is the one produced by the last frame.
        let screen_modal = Self::screen_modal_visible(ctx);
        if screen_modal != self.screen_modal_open {
            if screen_modal {
                self.screen_modal_focus_return = ctx.memory(|memory| memory.focused());
                self.focus_index = 0;
            } else if let Some(id) = self.screen_modal_focus_return.take() {
                ctx.memory_mut(|memory| memory.request_focus(id));
            }
            self.screen_modal_open = screen_modal;
        }

        let count = if self.confirm.is_some() {
            2
        } else if self.unsaved_prompt.is_some() {
            3
        } else if self.issues_open {
            5
        } else if screen_modal {
            2
        } else {
            0
        };
        if count == 0 {
            return;
        }
        let step = components::trap_focus_step(ctx);
        if step != 0 {
            self.focus_index = (self.focus_index as i32 + step).rem_euclid(count) as usize;
            ctx.memory_mut(|memory| {
                memory.request_focus(components::modal_focus_id(self.focus_index))
            });
        }
    }

    fn restore_focus(&mut self, ctx: &egui::Context) {
        if let Some(id) = self.focus_return.take() {
            ctx.memory_mut(|memory| memory.request_focus(id));
        }
    }
}

impl eframe::App for AppShell {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Exit guard: never lose staged edits to a window close.
        if ctx.input(|input| input.viewport().close_requested())
            && crate::gui::data::session::is_dirty(&self.data)
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            if self.unsaved_prompt.is_none() {
                self.unsaved_prompt = Some(ContextAction::Exit);
            }
        }

        self.drain_events();
        self.dirty_paths = crate::gui::data::session::dirty_paths(&self.data).len();
        self.handle_tabs(ctx);

        if self.issues_open {
            issues_drawer::render(self, ctx);
        }

        let width = ctx.screen_rect().width();
        if width >= 1280.0 {
            context_column::render_expanded(self, ctx);
            sector_map::render(self, ctx, false);
            safety_spine::render(self, ctx, false);
        } else {
            context_column::render_compact(self, ctx);
            sector_map::render(self, ctx, true);
            safety_spine::render(self, ctx, true);
        }
        detail_pane::render(self, ctx, width);

        overlays::render(self, ctx);
        components::toast_stack(ctx, &mut self.toasts);

        if self.close_after_prompt {
            self.close_after_prompt = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}
