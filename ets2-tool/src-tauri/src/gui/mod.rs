#![allow(dead_code)]

pub mod career_tab;
pub mod garages_tab;
pub mod mods_tab;
pub mod profile_tools_tab;
pub mod save_editor_tab;
pub mod settings_tab;
pub mod vehicles_tab;

use std::collections::VecDeque;
use std::sync::mpsc::Receiver;

use eframe::egui::{self, Color32, RichText};

use crate::db::sqlite::{SqliteInfoDto, SqliteTableCountsDto};
use crate::features::backup::models::{BackupRestorePreviewDto, BackupVersionDto};
use crate::features::career::analytics::AnalyticsSummary;
use crate::features::career::dispatcher::{
    DispatcherGenerationStatus, DispatcherMarketJob, DispatcherOverview,
};
use crate::features::career::job_log::{JobLogEntry, JobStats};
use crate::features::career::logbook::TripSummary;
use crate::features::career::overview::CareerOverview;
use crate::features::garages::models::{AiDriverManagementResult, GarageListResult};
use crate::features::health_monitor::models::SaveHealthReportDto;
use crate::features::hub::events::CareerStatus;
use crate::features::language::commands::{
    LanguageInfo, get_available_languages_command, get_current_language_command,
    set_language_command,
};
use crate::shared::user_log::{UserLogEntry, UserLogStatus};
use crate::features::mod_profile_manager::models::{
    ModPreset, ModProfileManagerState, PresetCompareResult, SandboxCollection,
};
use crate::features::profile_manager::commands::{
    find_ets2_profiles, find_profile_saves, load_profile, read_last_profile, save_last_profile,
    set_active_profile, set_current_save, set_selected_game,
};
use crate::features::profile_sharing::models::{ProfileShareContext, ProfileShareImportPreview};
use crate::features::save_analysis::models::ModConflictAnalysisReport;
use crate::features::save_analysis::quicksave::{get_current_truck_summary, quicksave_game_info};
use crate::features::save_analysis::reader::read_all_save_data;
use crate::features::save_editor::coaches::CoachesSnapshot;
use crate::features::save_editor::commands::{
    UndoStatusDto, get_undo_status, undo_last_save_change,
};
use crate::features::settings::ets2_paths::get_ets2_data_location;
use crate::features::settings::game_config::{read_base_config, read_save_config};
use crate::features::trailer_change::models::{
    TrailerChangePreview, TrailerChangeSession, TrailerSwitchList,
};
use crate::features::truck_change::models::{
    PowertrainCatalog, TruckChangePreview, TruckChangeSession, TruckPowertrainPreview,
    TruckSwitchList,
};
use crate::features::vehicles::trailers::get_all_trailers;
use crate::features::vehicles::trucks::get_all_trucks;
use crate::features::vtc::models::{CompanyMember, CompanyOverview, UserProfile};
use crate::models::global_config_info::BaseGameConfig;
use crate::models::profile_info::ProfileInfo;
use crate::models::quicksave_game_info::{CurrentTruckSummary, GameDataQuicksave};
use crate::models::save_game_config::SaveGameConfig;
use crate::models::save_game_data::SaveGameData;
use crate::models::save_info::SaveInfo;
use crate::models::trailers::{ParsedTrailer, PlayerTrailerResult};
use crate::models::trucks::ParsedTruck;
use crate::shared::paths::Ets2DataLocationDto;
use crate::state::{
    AppProfileState, AppState, AuthState, CareerState, DecryptCache, DiagnosticsState, EtsDbState,
    HubState, ProfileCache,
};
use crate::tauri::{AppHandle, NativeEvent, State, async_runtime};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceTab {
    Launcher,
    SaveEditor,
    Vehicles,
    Garages,
    Career,
    Mods,
    ProfileTools,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VehiclesSubTab {
    PlayerVehicleEditor,
    TruckSwitcher,
    TrailerSwitcher,
    PowertrainSwapper,
    FleetInventory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CareerSubTab {
    LiveTelemetry,
    Dispatcher,
    LogbookAndAnalytics,
    VtcAndAccount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModsSubTab {
    ModManager,
    PresetsAndSandboxes,
    ConflictAnalyzer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileToolsSubTab {
    CloneAndRename,
    ProfileSharing,
    SaveHealth,
    BackupsAndRestore,
}

pub struct Ets2NativeApp {
    pub app: AppHandle,
    pub event_rx: Receiver<NativeEvent>,
    pub recent_events: VecDeque<(String, String)>,

    // Navigation & Status
    pub active_tab: WorkspaceTab,
    pub vehicles_sub_tab: VehiclesSubTab,
    pub career_sub_tab: CareerSubTab,
    pub mods_sub_tab: ModsSubTab,
    pub profile_tools_sub_tab: ProfileToolsSubTab,
    pub status_banner: Option<(String, bool)>,

    // Global Game / Profile / Save selection
    pub selected_game: String,
    pub available_languages: Vec<LanguageInfo>,
    pub current_language: String,
    pub profiles: Vec<ProfileInfo>,
    pub selected_profile_path: Option<String>,
    pub saves: Vec<SaveInfo>,
    pub selected_save_path: Option<String>,
    pub undo_status: Option<UndoStatusDto>,

    // Save Editor State
    pub save_data: Option<SaveGameData>,
    pub quicksave_data: Option<GameDataQuicksave>,
    pub current_truck_summary: Option<CurrentTruckSummary>,
    pub base_config: Option<BaseGameConfig>,
    pub save_config: Option<SaveGameConfig>,
    pub coaches_snapshot: Option<CoachesSnapshot>,

    pub edit_money_input: i64,
    pub edit_xp_input: i64,
    pub edit_level_input: i64,
    pub skill_adr: i64,
    pub skill_long_dist: i64,
    pub skill_heavy: i64,
    pub skill_fragile: i64,
    pub skill_urgent: i64,
    pub skill_mechanical: i64,
    pub cfg_traffic: i64,
    pub cfg_developer: i64,
    pub cfg_console: i64,
    pub cfg_convoy: i64,
    pub cfg_parking_doubles: i64,
    pub custom_cfg_key: String,
    pub custom_cfg_value: String,
    pub custom_save_cfg_key: String,
    pub custom_save_cfg_value: String,

    // Vehicles State
    pub player_truck: Option<ParsedTruck>,
    pub all_trucks: Vec<ParsedTruck>,
    pub player_trailer_result: Option<PlayerTrailerResult>,
    pub all_trailers: Vec<ParsedTrailer>,
    pub truck_plate_input: String,
    pub truck_odometer_input: i64,
    pub truck_fuel_slider: f32,
    pub truck_wear_slider: f32,
    pub trailer_plate_input: String,
    pub trailer_cargo_mass_input: f32,
    pub truck_switch_list: Option<TruckSwitchList>,
    pub truck_change_session: Option<TruckChangeSession>,
    pub selected_switch_truck_id: String,
    pub truck_switch_preview: Option<TruckChangePreview>,
    pub create_backup_on_switch: bool,
    pub trailer_switch_list: Option<TrailerSwitchList>,
    pub trailer_change_session: Option<TrailerChangeSession>,
    pub selected_switch_trailer_id: String,
    pub trailer_switch_preview: Option<TrailerChangePreview>,
    pub powertrain_catalog: Option<PowertrainCatalog>,
    pub selected_engine_path: String,
    pub selected_transmission_path: String,
    pub powertrain_preview: Option<TruckPowertrainPreview>,

    // Garages & AI Drivers State
    pub garage_list: Option<GarageListResult>,
    pub ai_driver_mgmt: Option<AiDriverManagementResult>,
    pub garage_filter_text: String,
    pub garage_only_owned: bool,
    pub generate_drivers_count: usize,
    pub generate_drivers_assign_trucks: bool,
    pub selected_garage_for_drivers: String,

    // Career / Telemetry / Dispatcher / VTC State
    pub career_status: Option<CareerStatus>,
    pub plugin_installed: bool,
    pub career_overview: Option<CareerOverview>,
    pub dispatcher_overview: Option<DispatcherOverview>,
    pub dispatcher_gen_status: Option<DispatcherGenerationStatus>,
    pub market_jobs: Vec<DispatcherMarketJob>,
    pub active_dispatcher_jobs: Vec<DispatcherMarketJob>,
    pub selected_market_job_id: String,
    pub job_search_filter: String,
    pub trip_log: Vec<TripSummary>,
    pub job_log: Vec<JobLogEntry>,
    pub job_stats: Option<JobStats>,
    pub analytics_summary: Option<AnalyticsSummary>,
    pub vtc_user_profile: Option<UserProfile>,
    pub vtc_company: Option<CompanyOverview>,
    pub vtc_members: Vec<CompanyMember>,
    pub auth_email_input: String,
    pub auth_password_input: String,
    pub auth_username_input: String,
    pub new_company_name: String,
    pub new_company_location: String,

    // Mods & Conflict Analyzer State
    pub mod_manager_state: Option<ModProfileManagerState>,
    pub mod_presets: Vec<ModPreset>,
    pub new_preset_name: String,
    pub new_preset_notes: String,
    pub selected_preset_id: String,
    pub preset_compare_result: Option<PresetCompareResult>,
    pub mod_sandboxes: Option<SandboxCollection>,
    pub new_sandbox_title: String,
    pub new_sandbox_desc: String,
    pub selected_sandbox_id: String,
    pub workshop_mod_input: String,
    pub conflict_report: Option<ModConflictAnalysisReport>,

    // Profile Tools & Safety State
    pub clone_new_name: String,
    pub clone_backup: bool,
    pub clone_replace_hex: bool,
    pub clone_replace_text: bool,
    pub rename_profile_input: String,
    pub target_profile_for_copy: String,
    pub share_context: Option<ProfileShareContext>,
    pub share_export_dir: String,
    pub share_import_archive: String,
    pub share_import_name_override: String,
    pub share_import_preview: Option<ProfileShareImportPreview>,
    pub save_health_report: Option<SaveHealthReportDto>,
    pub backups_list: Vec<BackupVersionDto>,
    pub selected_backup_id: String,
    pub backup_preview: Option<BackupRestorePreviewDto>,

    // Settings & Diagnostics State
    pub data_location: Option<Ets2DataLocationDto>,
    pub custom_homedir_input: String,
    pub sqlite_info: Option<SqliteInfoDto>,
    pub sqlite_counts: Option<SqliteTableCountsDto>,
    pub log_status: Option<UserLogStatus>,
    pub user_logs: Vec<UserLogEntry>,
    pub diagnostic_log_preview: String,
}

impl Ets2NativeApp {
    pub fn new(cc: &eframe::CreationContext<'_>, app: AppHandle) -> Self {
        Self::configure_theme(&cc.egui_ctx);
        let event_rx = app.subscribe_events();
        let available_languages = get_available_languages_command(app.clone());
        let current_language = get_current_language_command(app.clone());

        let mut instance = Self {
            app,
            event_rx,
            recent_events: VecDeque::with_capacity(64),
            active_tab: WorkspaceTab::Launcher,
            vehicles_sub_tab: VehiclesSubTab::PlayerVehicleEditor,
            career_sub_tab: CareerSubTab::LiveTelemetry,
            mods_sub_tab: ModsSubTab::ModManager,
            profile_tools_sub_tab: ProfileToolsSubTab::CloneAndRename,
            status_banner: None,

            selected_game: "ets2".to_string(),
            available_languages,
            current_language,
            profiles: Vec::new(),
            selected_profile_path: None,
            saves: Vec::new(),
            selected_save_path: None,
            undo_status: None,

            save_data: None,
            quicksave_data: None,
            current_truck_summary: None,
            base_config: None,
            save_config: None,
            coaches_snapshot: None,

            edit_money_input: 1_000_000,
            edit_xp_input: 50_000,
            edit_level_input: 50,
            skill_adr: 6,
            skill_long_dist: 6,
            skill_heavy: 6,
            skill_fragile: 6,
            skill_urgent: 6,
            skill_mechanical: 6,
            cfg_traffic: 1,
            cfg_developer: 1,
            cfg_console: 1,
            cfg_convoy: 8,
            cfg_parking_doubles: 1,
            custom_cfg_key: String::new(),
            custom_cfg_value: String::new(),
            custom_save_cfg_key: String::new(),
            custom_save_cfg_value: String::new(),

            player_truck: None,
            all_trucks: Vec::new(),
            player_trailer_result: None,
            all_trailers: Vec::new(),
            truck_plate_input: String::new(),
            truck_odometer_input: 0,
            truck_fuel_slider: 1.0,
            truck_wear_slider: 0.0,
            trailer_plate_input: String::new(),
            trailer_cargo_mass_input: 0.0,
            truck_switch_list: None,
            truck_change_session: None,
            selected_switch_truck_id: String::new(),
            truck_switch_preview: None,
            create_backup_on_switch: true,
            trailer_switch_list: None,
            trailer_change_session: None,
            selected_switch_trailer_id: String::new(),
            trailer_switch_preview: None,
            powertrain_catalog: None,
            selected_engine_path: String::new(),
            selected_transmission_path: String::new(),
            powertrain_preview: None,

            garage_list: None,
            ai_driver_mgmt: None,
            garage_filter_text: String::new(),
            garage_only_owned: false,
            generate_drivers_count: 3,
            generate_drivers_assign_trucks: true,
            selected_garage_for_drivers: String::new(),

            career_status: None,
            plugin_installed: false,
            career_overview: None,
            dispatcher_overview: None,
            dispatcher_gen_status: None,
            market_jobs: Vec::new(),
            active_dispatcher_jobs: Vec::new(),
            selected_market_job_id: String::new(),
            job_search_filter: String::new(),
            trip_log: Vec::new(),
            job_log: Vec::new(),
            job_stats: None,
            analytics_summary: None,
            vtc_user_profile: None,
            vtc_company: None,
            vtc_members: Vec::new(),
            auth_email_input: String::new(),
            auth_password_input: String::new(),
            auth_username_input: String::new(),
            new_company_name: String::new(),
            new_company_location: "Berlin".to_string(),

            mod_manager_state: None,
            mod_presets: Vec::new(),
            new_preset_name: String::new(),
            new_preset_notes: String::new(),
            selected_preset_id: String::new(),
            preset_compare_result: None,
            mod_sandboxes: None,
            new_sandbox_title: String::new(),
            new_sandbox_desc: String::new(),
            selected_sandbox_id: String::new(),
            workshop_mod_input: String::new(),
            conflict_report: None,

            clone_new_name: String::new(),
            clone_backup: true,
            clone_replace_hex: true,
            clone_replace_text: true,
            rename_profile_input: String::new(),
            target_profile_for_copy: String::new(),
            share_context: None,
            share_export_dir: String::new(),
            share_import_archive: String::new(),
            share_import_name_override: String::new(),
            share_import_preview: None,
            save_health_report: None,
            backups_list: Vec::new(),
            selected_backup_id: String::new(),
            backup_preview: None,

            data_location: None,
            custom_homedir_input: String::new(),
            sqlite_info: None,
            sqlite_counts: None,
            log_status: None,
            user_logs: Vec::new(),
            diagnostic_log_preview: String::new(),
        };

        instance.refresh_profiles_and_saves();
        instance.refresh_settings_and_location();
        instance
    }

    fn configure_theme(ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::from_rgb(17, 20, 26);
        visuals.window_fill = Color32::from_rgb(22, 26, 34);
        visuals.extreme_bg_color = Color32::from_rgb(12, 14, 18);
        visuals.selection.bg_fill = Color32::from_rgb(214, 142, 28);
        visuals.selection.stroke = egui::Stroke::new(1.0_f32, Color32::from_rgb(255, 210, 120));
        ctx.set_visuals(visuals);
    }

    pub fn set_status(&mut self, message: impl Into<String>, is_error: bool) {
        self.status_banner = Some((message.into(), is_error));
    }

    pub fn profile_state(&self) -> State<'_, AppProfileState> {
        self.app.state::<AppProfileState>()
    }

    pub fn decrypt_cache(&self) -> State<'_, DecryptCache> {
        self.app.state::<DecryptCache>()
    }

    pub fn profile_cache(&self) -> State<'_, ProfileCache> {
        self.app.state::<ProfileCache>()
    }

    pub fn app_state(&self) -> State<'_, AppState> {
        self.app.state::<AppState>()
    }

    pub fn career_state(&self) -> State<'_, CareerState> {
        self.app.state::<CareerState>()
    }

    pub fn auth_state(&self) -> State<'_, AuthState> {
        self.app.state::<AuthState>()
    }

    pub fn ets_db_state(&self) -> State<'_, EtsDbState> {
        self.app.state::<EtsDbState>()
    }

    pub fn hub_state(&self) -> State<'_, HubState> {
        self.app.state::<HubState>()
    }

    pub fn diagnostics_state(&self) -> State<'_, DiagnosticsState> {
        self.app.state::<DiagnosticsState>()
    }

    pub fn refresh_profiles_and_saves(&mut self) {
        let _ = set_selected_game(self.selected_game.clone(), self.profile_state());
        self.profiles = find_ets2_profiles(self.profile_state());

        let last_profile = read_last_profile(self.app.clone()).ok().flatten();
        if self.selected_profile_path.is_none() {
            if let Some(last) = last_profile {
                if self.profiles.iter().any(|p| p.path == last) {
                    self.selected_profile_path = Some(last);
                }
            }
        }
        if self.selected_profile_path.is_none() {
            self.selected_profile_path = self.profiles.first().map(|p| p.path.clone());
        }

        if let Some(profile_path) = self.selected_profile_path.clone() {
            self.select_profile(profile_path);
        } else {
            self.saves.clear();
            self.selected_save_path = None;
        }
    }

    pub fn select_profile(&mut self, profile_path: String) {
        self.selected_profile_path = Some(profile_path.clone());
        let _ = save_last_profile(self.app.clone(), profile_path.clone());
        let _ = set_active_profile(
            profile_path.clone(),
            self.profile_state(),
            self.decrypt_cache(),
            self.profile_cache(),
        );

        match async_runtime::block_on(find_profile_saves(profile_path.clone())) {
            Ok(saves) => {
                self.saves = saves;
                let preferred = self
                    .saves
                    .iter()
                    .find(|s| s.folder.eq_ignore_ascii_case("quicksave"))
                    .or_else(|| self.saves.first())
                    .map(|s| s.path.clone());

                if let Some(save_path) = preferred {
                    self.select_save(save_path);
                } else {
                    self.selected_save_path = None;
                }
            }
            Err(err) => {
                self.saves.clear();
                self.selected_save_path = None;
                self.set_status(format!("Failed to list saves: {err}"), true);
            }
        }
    }

    pub fn select_save(&mut self, save_path: String) {
        self.selected_save_path = Some(save_path.clone());
        if let Some(profile_path) = self.selected_profile_path.clone() {
            let _ = load_profile(
                profile_path,
                Some(save_path.clone()),
                self.profile_state(),
                self.decrypt_cache(),
                self.profile_cache(),
            );
        }
        let _ = set_current_save(
            save_path,
            self.profile_state(),
            self.decrypt_cache(),
            self.profile_cache(),
        );
        self.reload_active_save_data();
    }

    pub fn reload_active_save_data(&mut self) {
        self.undo_status = get_undo_status(self.profile_state()).ok();
        self.base_config = read_base_config(self.profile_cache()).ok();
        if let Some(cfg) = &self.base_config {
            self.cfg_traffic = cfg.traffic.unwrap_or(1);
            self.cfg_developer = cfg.developer.unwrap_or(0);
            self.cfg_console = cfg.console.unwrap_or(0);
            self.cfg_convoy = cfg.max_convoy_size.unwrap_or(8);
        }

        if let Some(profile_path) = &self.selected_profile_path {
            self.save_config = read_save_config(profile_path, self.profile_cache()).ok();
            if let Some(scfg) = &self.save_config {
                self.cfg_parking_doubles = scfg.factor_parking_doubles.unwrap_or(0);
            }
        }

        match async_runtime::block_on(read_all_save_data(
            self.profile_state(),
            self.profile_cache(),
            self.decrypt_cache(),
        )) {
            Ok(data) => {
                if let Some(money) = data.money {
                    self.edit_money_input = money;
                }
                if let Some(xp) = data.xp {
                    self.edit_xp_input = xp;
                }
                self.save_data = Some(data);
            }
            Err(err) => {
                self.save_data = None;
                self.set_status(format!("Save read notice: {err}"), true);
            }
        }

        if let Ok(qs) = async_runtime::block_on(quicksave_game_info(
            self.profile_state(),
            self.profile_cache(),
            self.decrypt_cache(),
        )) {
            self.skill_adr = qs.adr.unwrap_or(0);
            self.skill_long_dist = qs.long_dist.unwrap_or(0);
            self.skill_heavy = qs.heavy.unwrap_or(0);
            self.skill_fragile = qs.fragile.unwrap_or(0);
            self.skill_urgent = qs.urgent.unwrap_or(0);
            self.skill_mechanical = qs.mechanical.unwrap_or(0);
            if let Some(plate) = &qs.license_plate {
                self.truck_plate_input = plate.clone();
            }
            if let Some(odo) = qs.odometer {
                self.truck_odometer_input = odo;
            }
            if let Some(tplate) = &qs.trailer_license_plate {
                self.trailer_plate_input = tplate.clone();
            }
            self.quicksave_data = Some(qs);
        }

        self.current_truck_summary = async_runtime::block_on(get_current_truck_summary(
            self.profile_state(),
            self.profile_cache(),
            self.decrypt_cache(),
        ))
        .ok()
        .flatten();

        if let Some(profile_path) = self.selected_profile_path.clone() {
            if let Ok(trucks) = async_runtime::block_on(get_all_trucks(
                profile_path.clone(),
                self.profile_state(),
                self.profile_cache(),
                self.decrypt_cache(),
            )) {
                self.all_trucks = trucks;
            }
            self.player_truck = async_runtime::block_on(
                crate::features::vehicles::trucks::get_player_truck(
                    profile_path.clone(),
                    self.profile_state(),
                    self.profile_cache(),
                    self.decrypt_cache(),
                ),
            )
            .ok();

            if let Ok(trailers) = async_runtime::block_on(get_all_trailers(
                profile_path.clone(),
                self.profile_state(),
                self.profile_cache(),
                self.decrypt_cache(),
            )) {
                self.all_trailers = trailers;
            }
            self.player_trailer_result = async_runtime::block_on(
                crate::features::vehicles::trailers::get_player_trailer(
                    profile_path,
                    self.profile_state(),
                    self.profile_cache(),
                    self.decrypt_cache(),
                ),
            )
            .ok();
        }
    }

    pub fn refresh_settings_and_location(&mut self) {
        if let Ok(loc) = get_ets2_data_location() {
            if let Some(custom) = &loc.custom_homedir {
                self.custom_homedir_input = custom.clone();
            }
            self.data_location = Some(loc);
        }
        self.sqlite_info = async_runtime::block_on(crate::commands::get_sqlite_info(self.app_state())).ok();
        self.sqlite_counts = async_runtime::block_on(crate::commands::get_sqlite_table_counts(
            self.profile_state(),
            self.app_state(),
        ))
        .ok();
    }

    fn drain_background_events(&mut self) {
        while let Ok(ev) = self.event_rx.try_recv() {
            let summary = ev.payload.to_string();
            if self.recent_events.len() >= 50 {
                self.recent_events.pop_front();
            }
            self.recent_events.push_back((ev.event, summary));
        }
    }

    fn render_top_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.heading(
                RichText::new("ETS2 / ATS Save-Edit Tool")
                    .color(Color32::from_rgb(245, 176, 65))
                    .strong(),
            );
            ui.label(
                RichText::new(format!("v{} (Native GUI)", env!("CARGO_PKG_VERSION"))).weak(),
            );
            ui.separator();

            // Game Selector
            ui.label("Game:");
            let mut game_changed = false;
            egui::ComboBox::from_id_salt("game_selector")
                .selected_text(self.selected_game.to_uppercase())
                .width(75.0)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_value(&mut self.selected_game, "ets2".to_string(), "ETS2")
                        .changed()
                    {
                        game_changed = true;
                    }
                    if ui
                        .selectable_value(&mut self.selected_game, "ats".to_string(), "ATS")
                        .changed()
                    {
                        game_changed = true;
                    }
                });
            if game_changed {
                self.selected_profile_path = None;
                self.selected_save_path = None;
                self.refresh_profiles_and_saves();
            }

            // Profile Selector
            ui.label("Profile:");
            let current_profile_label = self
                .selected_profile_path
                .as_ref()
                .and_then(|path| {
                    self.profiles
                        .iter()
                        .find(|p| &p.path == path)
                        .and_then(|p| p.name.clone())
                        .or_else(|| Some(path.clone()))
                })
                .unwrap_or_else(|| "<No Profile>".to_string());

            let mut newly_selected_profile = None;
            egui::ComboBox::from_id_salt("profile_selector")
                .selected_text(current_profile_label)
                .width(190.0)
                .show_ui(ui, |ui| {
                    for profile in &self.profiles {
                        let label = profile
                            .name
                            .clone()
                            .unwrap_or_else(|| profile.path.clone());
                        let is_selected =
                            self.selected_profile_path.as_deref() == Some(profile.path.as_str());
                        if ui.selectable_label(is_selected, label).clicked() {
                            newly_selected_profile = Some(profile.path.clone());
                        }
                    }
                });
            if let Some(profile_path) = newly_selected_profile {
                self.select_profile(profile_path);
            }

            // Save Selector
            ui.label("Save:");
            let current_save_label = self
                .selected_save_path
                .as_ref()
                .and_then(|path| {
                    self.saves
                        .iter()
                        .find(|s| &s.path == path)
                        .map(|s| {
                            s.name
                                .clone()
                                .unwrap_or_else(|| s.folder.clone())
                        })
                })
                .unwrap_or_else(|| "<No Save>".to_string());

            let mut newly_selected_save = None;
            egui::ComboBox::from_id_salt("save_selector")
                .selected_text(current_save_label)
                .width(180.0)
                .show_ui(ui, |ui| {
                    for save in &self.saves {
                        let label = format!(
                            "{} ({})",
                            save.name.as_deref().unwrap_or(&save.folder),
                            save.folder
                        );
                        let is_selected =
                            self.selected_save_path.as_deref() == Some(save.path.as_str());
                        if ui.selectable_label(is_selected, label).clicked() {
                            newly_selected_save = Some(save.path.clone());
                        }
                    }
                });
            if let Some(save_path) = newly_selected_save {
                self.select_save(save_path);
            }

            if ui.button("⟳ Reload").clicked() {
                self.refresh_profiles_and_saves();
                self.set_status("Profiles and active save reloaded.", false);
            }

            // Undo Button
            let can_undo = self
                .undo_status
                .as_ref()
                .map(|u| u.can_undo)
                .unwrap_or(false);
            if ui
                .add_enabled(can_undo, egui::Button::new("↶ Undo"))
                .clicked()
            {
                match undo_last_save_change(
                    self.profile_state(),
                    self.profile_cache(),
                    self.decrypt_cache(),
                ) {
                    Ok(res) => {
                        self.reload_active_save_data();
                        self.set_status(
                            format!(
                                "Undid last save change (restored {} files from {}).",
                                res.restored_file_count, res.backup_id
                            ),
                            false,
                        );
                    }
                    Err(err) => self.set_status(format!("Undo failed: {err}"), true),
                }
            }

            // Language Selector
            if !self.available_languages.is_empty() {
                ui.separator();
                ui.label("Lang:");
                let mut next_lang = None;
                egui::ComboBox::from_id_salt("lang_selector")
                    .selected_text(self.current_language.to_uppercase())
                    .width(65.0)
                    .show_ui(ui, |ui| {
                        for lang in &self.available_languages {
                            let label = format!("{} ({})", lang.name, lang.code.to_uppercase());
                            if ui
                                .selectable_label(self.current_language == lang.code, label)
                                .clicked()
                            {
                                next_lang = Some(lang.code.clone());
                            }
                        }
                    });
                if let Some(code) = next_lang {
                    if let Ok(msg) = set_language_command(self.app.clone(), code.clone()) {
                        self.current_language = code;
                        self.set_status(msg, false);
                    }
                }
            }
        });

        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            let tabs = [
                (WorkspaceTab::Launcher, "🏠 Workspace Hub"),
                (WorkspaceTab::SaveEditor, "💰 Save Editor"),
                (WorkspaceTab::Vehicles, "🚛 Vehicles & Powertrain"),
                (WorkspaceTab::Garages, "🏢 Garages & AI Drivers"),
                (WorkspaceTab::Career, "📊 Career, Telemetry & VTC"),
                (WorkspaceTab::Mods, "🧩 Mods & Conflict Analyzer"),
                (WorkspaceTab::ProfileTools, "🛡 Profile Tools & Backups"),
                (WorkspaceTab::Settings, "⚙ Settings & Diagnostics"),
            ];
            for (tab, label) in tabs {
                let selected = self.active_tab == tab;
                if ui.selectable_label(selected, RichText::new(label).strong()).clicked() {
                    self.active_tab = tab;
                }
            }
        });
    }

    fn render_launcher_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Select a Workspace");
        ui.label("Native cross-platform desktop interface for Euro Truck Simulator 2 and American Truck Simulator (Zero WebView).");
        ui.add_space(12.0);

        ui.columns(2, |cols| {
            cols[0].group(|ui| {
                ui.heading(RichText::new("🛠 Save Editor & Fleet Workspace").color(Color32::from_rgb(245, 176, 65)));
                ui.label("Decrypt SII saves, edit player money, XP, level, skills, base/save configs, repair/refuel trucks & trailers, swap active vehicles, and manage garages & AI drivers.");
                ui.add_space(8.0);
                if let Some(data) = &self.save_data {
                    ui.label(format!("• Active Balance: €{}", data.money.unwrap_or(0)));
                    ui.label(format!("• Experience: {} XP", data.xp.unwrap_or(0)));
                    ui.label(format!("• Visited Cities: {}", data.visited_cities.unwrap_or(0)));
                } else {
                    ui.label("• Select a profile and save slot in the top bar to begin editing.");
                }
                if let Some(truck) = &self.current_truck_summary {
                    ui.label(format!(
                        "• Current Truck: {} ({} km)",
                        truck.display_name.as_deref().unwrap_or("Unknown"),
                        truck.odometer_km.unwrap_or(0)
                    ));
                }
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Open Save Editor →").clicked() {
                        self.active_tab = WorkspaceTab::SaveEditor;
                    }
                    if ui.button("Open Vehicles →").clicked() {
                        self.active_tab = WorkspaceTab::Vehicles;
                    }
                    if ui.button("Open Garages →").clicked() {
                        self.active_tab = WorkspaceTab::Garages;
                    }
                });
            });

            cols[1].group(|ui| {
                ui.heading(RichText::new("📡 Career, Freight Dispatcher & VTC").color(Color32::from_rgb(88, 214, 141)));
                ui.label("Monitor live SCS shared-memory telemetry, generate and inject freight market jobs directly into your quicksave, track logbook trips, and manage your virtual company.");
                ui.add_space(8.0);
                if ui.button("Open Career & Dispatcher →").clicked() {
                    self.active_tab = WorkspaceTab::Career;
                }
                ui.add_space(8.0);
                ui.separator();
                ui.heading("🧩 Mod & Profile Safety Suite");
                ui.label("Scan local & Steam Workshop mods, manage mod presets/sandboxes, analyze crash/conflict logs, clone/share profiles, and restore automatic save backups.");
                ui.horizontal(|ui| {
                    if ui.button("Open Mod Manager →").clicked() {
                        self.active_tab = WorkspaceTab::Mods;
                    }
                    if ui.button("Open Profile & Backup Tools →").clicked() {
                        self.active_tab = WorkspaceTab::ProfileTools;
                    }
                });
            });
        });
    }
}

impl eframe::App for Ets2NativeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain_background_events();

        egui::TopBottomPanel::top("native_top_bar").show(ctx, |ui| {
            self.render_top_bar(ui);
        });

        if let Some((msg, is_error)) = self.status_banner.clone() {
            egui::TopBottomPanel::bottom("native_status_bar").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let color = if is_error {
                        Color32::from_rgb(231, 76, 60)
                    } else {
                        Color32::from_rgb(88, 214, 141)
                    };
                    ui.colored_label(color, RichText::new(&msg).strong());
                    if ui.small_button("✖").clicked() {
                        self.status_banner = None;
                    }
                });
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| match self.active_tab {
                    WorkspaceTab::Launcher => self.render_launcher_tab(ui),
                    WorkspaceTab::SaveEditor => save_editor_tab::render(self, ui),
                    WorkspaceTab::Vehicles => vehicles_tab::render(self, ui),
                    WorkspaceTab::Garages => garages_tab::render(self, ui),
                    WorkspaceTab::Career => career_tab::render(self, ui, ctx),
                    WorkspaceTab::Mods => mods_tab::render(self, ui),
                    WorkspaceTab::ProfileTools => profile_tools_tab::render(self, ui),
                    WorkspaceTab::Settings => settings_tab::render(self, ui),
                });
        });
    }
}
