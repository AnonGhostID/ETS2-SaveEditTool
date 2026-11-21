//! Shared navigation contract for every Orbit Hub screen.
//!
//! This file is deliberately free of backend access. Screens receive a
//! [`ScreenCtx`] and push [`ShellCommand`]s for anything that belongs to the
//! shell (navigation, overlays, toasts, saving, context changes).

// Exact 30 snake_case tile variants from docs/ui-spec.md §1.2.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    overview_dashboard,
    overview_quick_actions,
    overview_recent_activity,
    save_data_economy,
    save_data_coaches,
    save_data_config,
    save_data_custom_keys,
    vehicles_player_truck,
    vehicles_player_trailer,
    vehicles_truck_switcher,
    vehicles_trailer_switcher,
    vehicles_powertrain,
    vehicles_fleet_inventory,
    operations_garages,
    operations_ai_drivers,
    operations_live_telemetry,
    operations_dispatcher,
    operations_logbook_analytics,
    operations_vtc_account,
    library_mod_manager,
    library_presets,
    library_sandboxes,
    library_conflict_analyzer,
    safety_save_health,
    safety_backups_restore,
    safety_profile_clone_rename,
    safety_profile_sharing,
    system_paths_data,
    system_logs_diagnostics,
    system_about,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sector {
    Overview,
    SaveData,
    Vehicles,
    Operations,
    Library,
    Safety,
    System,
}

impl Default for Sector {
    fn default() -> Self {
        Self::Overview
    }
}

/// Exact `SubView` variants from docs/ui-spec.md §1.4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SubView {
    #[default]
    None,
    DashboardHeader,
    DashboardCurrentSetup,
    DashboardStatTiles,
    DashboardSectorCards,
    DashboardStagingNote,
    QuickActionsList,
    ActivityFeed,
    EconomyFields,
    EconomySkills,
    EconomyCustomReset,
    EconomyLevelPreview,
    CoachesOverview,
    CoachesStats,
    CoachesJobHistory,
    CoachesStops,
    ConfigValues,
    BaseConfigKeys,
    SaveConfigKeys,
    TruckFields,
    TruckFuelWear,
    TruckPlateOdometer,
    TrailerFields,
    SwitchSession,
    SwitchList,
    SwitchPreview,
    SwitchApply,
    SwitchResult,
    PowertrainCatalog,
    PowertrainPreview,
    FleetTrucks,
    FleetTrailers,
    GarageList,
    GarageDetail,
    GarageWarnings,
    DriverGenerate,
    DriverAssign,
    DriverDismiss,
    TelemetryLive,
    TelemetrySnapshot,
    DispatcherJobs,
    DispatcherJobDetail,
    DispatcherGeneration,
    LogbookTrips,
    LogbookJobLog,
    LogbookStats,
    LogbookAnalytics,
    VtcSignIn,
    VtcCompany,
    VtcMembers,
    ModScan,
    ModList,
    ModWorkshop,
    PresetList,
    PresetCompare,
    SandboxList,
    ConflictReport,
    HealthScan,
    HealthReport,
    HealthFix,
    BackupList,
    BackupPreview,
    BackupConfirm,
    CloneForm,
    RenameForm,
    CopyControls,
    CopyMods,
    ShareExport,
    ShareImport,
    SharePreview,
    PathsStatus,
    SqliteInfo,
    DatasetImport,
    LogsList,
    DiagnosticsPreview,
    LogExports,
    AboutInfo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssuesTab {
    Validation,
    Errors,
    UserLog,
    Activity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextAction {
    SwitchGame(String),
    SwitchProfile(String),
    SwitchSave(String),
    Reload,
    Exit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellCommand {
    Navigate {
        route: Route,
        sub_view: Option<SubView>,
    },
    OpenIssues(IssuesTab),
    Toast {
        message: String,
        is_error: bool,
    },
    Save,
    RequestContextChange(ContextAction),
}

/// Everything a screen is allowed to use. Backend calls go through `data`;
/// anything affecting global state is queued as a [`ShellCommand`].
pub struct ScreenCtx<'a> {
    pub data: &'a crate::gui::data::DataContext,
    pub commands: &'a mut Vec<ShellCommand>,
    /// Current capability tile, assigned by the shell every frame.
    /// Screens read it to follow deep links; they must never mutate it.
    pub route: Route,
    /// Current sub-view, assigned by the shell every frame.
    /// Screens read it to follow deep links; they must never mutate it.
    pub sub_view: SubView,
    /// Last five shell events (newest last), assigned by the shell every
    /// frame. Screens read this live feed instead of opening a second
    /// event subscription.
    pub recent_events: &'a [(String, String)],
    /// Label of the currently selected save (path or display name), empty
    /// when no save is selected. Assigned by the shell every frame.
    pub active_save_label: &'a str,
}

impl Route {
    pub fn sector(self) -> Sector {
        match self {
            Self::overview_dashboard | Self::overview_quick_actions | Self::overview_recent_activity => Sector::Overview,
            Self::save_data_economy
            | Self::save_data_coaches
            | Self::save_data_config
            | Self::save_data_custom_keys => Sector::SaveData,
            Self::vehicles_player_truck
            | Self::vehicles_player_trailer
            | Self::vehicles_truck_switcher
            | Self::vehicles_trailer_switcher
            | Self::vehicles_powertrain
            | Self::vehicles_fleet_inventory => Sector::Vehicles,
            Self::operations_garages
            | Self::operations_ai_drivers
            | Self::operations_live_telemetry
            | Self::operations_dispatcher
            | Self::operations_logbook_analytics
            | Self::operations_vtc_account => Sector::Operations,
            Self::library_mod_manager
            | Self::library_presets
            | Self::library_sandboxes
            | Self::library_conflict_analyzer => Sector::Library,
            Self::safety_save_health
            | Self::safety_backups_restore
            | Self::safety_profile_clone_rename
            | Self::safety_profile_sharing => Sector::Safety,
            Self::system_paths_data | Self::system_logs_diagnostics | Self::system_about => Sector::System,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::overview_dashboard => "Dashboard",
            Self::overview_quick_actions => "Quick Actions",
            Self::overview_recent_activity => "Recent Activity",
            Self::save_data_economy => "Economy & Experience",
            Self::save_data_coaches => "Project Coaches",
            Self::save_data_config => "Config Values",
            Self::save_data_custom_keys => "Custom Keys",
            Self::vehicles_player_truck => "Player Truck",
            Self::vehicles_player_trailer => "Player Trailer",
            Self::vehicles_truck_switcher => "Truck Switcher",
            Self::vehicles_trailer_switcher => "Trailer Switcher",
            Self::vehicles_powertrain => "Powertrain Swapper",
            Self::vehicles_fleet_inventory => "Fleet Inventory",
            Self::operations_garages => "Garages",
            Self::operations_ai_drivers => "AI Drivers",
            Self::operations_live_telemetry => "Live Telemetry",
            Self::operations_dispatcher => "Dispatcher",
            Self::operations_logbook_analytics => "Logbook & Analytics",
            Self::operations_vtc_account => "VTC & Account",
            Self::library_mod_manager => "Mod Manager",
            Self::library_presets => "Presets",
            Self::library_sandboxes => "Sandboxes",
            Self::library_conflict_analyzer => "Conflict Analyzer",
            Self::safety_save_health => "Save Health",
            Self::safety_backups_restore => "Backups & Restore",
            Self::safety_profile_clone_rename => "Profile Clone & Rename",
            Self::safety_profile_sharing => "Profile Sharing",
            Self::system_paths_data => "Paths & Data",
            Self::system_logs_diagnostics => "Logs & Diagnostics",
            Self::system_about => "About",
        }
    }

    /// Default sub-view from docs/ui-spec.md §1.2.
    pub fn default_sub_view(self) -> SubView {
        match self {
            Self::overview_dashboard => SubView::DashboardHeader,
            Self::overview_quick_actions => SubView::QuickActionsList,
            Self::overview_recent_activity => SubView::ActivityFeed,
            Self::save_data_economy => SubView::EconomyFields,
            Self::save_data_coaches => SubView::CoachesOverview,
            Self::save_data_config => SubView::ConfigValues,
            Self::save_data_custom_keys => SubView::BaseConfigKeys,
            Self::vehicles_player_truck => SubView::TruckFields,
            Self::vehicles_player_trailer => SubView::TrailerFields,
            Self::vehicles_truck_switcher | Self::vehicles_trailer_switcher => SubView::SwitchSession,
            Self::vehicles_powertrain => SubView::PowertrainCatalog,
            Self::vehicles_fleet_inventory => SubView::FleetTrucks,
            Self::operations_garages => SubView::GarageList,
            Self::operations_ai_drivers => SubView::DriverGenerate,
            Self::operations_live_telemetry => SubView::TelemetryLive,
            Self::operations_dispatcher => SubView::DispatcherJobs,
            Self::operations_logbook_analytics => SubView::LogbookTrips,
            Self::operations_vtc_account => SubView::VtcSignIn,
            Self::library_mod_manager => SubView::ModScan,
            Self::library_presets => SubView::PresetList,
            Self::library_sandboxes => SubView::SandboxList,
            Self::library_conflict_analyzer => SubView::ConflictReport,
            Self::safety_save_health => SubView::HealthScan,
            Self::safety_backups_restore => SubView::BackupList,
            Self::safety_profile_clone_rename => SubView::CloneForm,
            Self::safety_profile_sharing => SubView::ShareExport,
            Self::system_paths_data => SubView::PathsStatus,
            Self::system_logs_diagnostics => SubView::LogsList,
            Self::system_about => SubView::AboutInfo,
        }
    }

    /// Allowed sub-view sets from docs/ui-spec.md §1.4. An invalid sub-view
    /// falls back to `default_sub_view()` in `AppShell::navigate`.
    pub fn accepts(self, view: SubView) -> bool {
        match self {
            Self::overview_dashboard => matches!(
                view,
                SubView::DashboardHeader
                    | SubView::DashboardCurrentSetup
                    | SubView::DashboardStatTiles
                    | SubView::DashboardSectorCards
                    | SubView::DashboardStagingNote
            ),
            Self::overview_quick_actions => matches!(view, SubView::QuickActionsList),
            Self::overview_recent_activity => matches!(view, SubView::ActivityFeed),
            Self::save_data_economy => matches!(
                view,
                SubView::EconomyFields
                    | SubView::EconomySkills
                    | SubView::EconomyCustomReset
                    | SubView::EconomyLevelPreview
            ),
            Self::save_data_coaches => matches!(
                view,
                SubView::CoachesOverview
                    | SubView::CoachesStats
                    | SubView::CoachesJobHistory
                    | SubView::CoachesStops
            ),
            Self::save_data_config => matches!(view, SubView::ConfigValues),
            Self::save_data_custom_keys => {
                matches!(view, SubView::BaseConfigKeys | SubView::SaveConfigKeys)
            }
            Self::vehicles_player_truck => matches!(
                view,
                SubView::TruckFields | SubView::TruckFuelWear | SubView::TruckPlateOdometer
            ),
            Self::vehicles_player_trailer => matches!(view, SubView::TrailerFields),
            Self::vehicles_truck_switcher | Self::vehicles_trailer_switcher => matches!(
                view,
                SubView::SwitchSession
                    | SubView::SwitchList
                    | SubView::SwitchPreview
                    | SubView::SwitchApply
                    | SubView::SwitchResult
            ),
            Self::vehicles_powertrain => {
                matches!(view, SubView::PowertrainCatalog | SubView::PowertrainPreview)
            }
            Self::vehicles_fleet_inventory => {
                matches!(view, SubView::FleetTrucks | SubView::FleetTrailers)
            }
            Self::operations_garages => matches!(
                view,
                SubView::GarageList | SubView::GarageDetail | SubView::GarageWarnings
            ),
            Self::operations_ai_drivers => matches!(
                view,
                SubView::DriverGenerate | SubView::DriverAssign | SubView::DriverDismiss
            ),
            Self::operations_live_telemetry => {
                matches!(view, SubView::TelemetryLive | SubView::TelemetrySnapshot)
            }
            Self::operations_dispatcher => matches!(
                view,
                SubView::DispatcherJobs | SubView::DispatcherJobDetail | SubView::DispatcherGeneration
            ),
            Self::operations_logbook_analytics => matches!(
                view,
                SubView::LogbookTrips
                    | SubView::LogbookJobLog
                    | SubView::LogbookStats
                    | SubView::LogbookAnalytics
            ),
            Self::operations_vtc_account => matches!(
                view,
                SubView::VtcSignIn | SubView::VtcCompany | SubView::VtcMembers
            ),
            Self::library_mod_manager => matches!(
                view,
                SubView::ModScan | SubView::ModList | SubView::ModWorkshop
            ),
            Self::library_presets => {
                matches!(view, SubView::PresetList | SubView::PresetCompare)
            }
            Self::library_sandboxes => matches!(view, SubView::SandboxList),
            Self::library_conflict_analyzer => matches!(view, SubView::ConflictReport),
            Self::safety_save_health => matches!(
                view,
                SubView::HealthScan | SubView::HealthReport | SubView::HealthFix
            ),
            Self::safety_backups_restore => matches!(
                view,
                SubView::BackupList | SubView::BackupPreview | SubView::BackupConfirm
            ),
            Self::safety_profile_clone_rename => matches!(
                view,
                SubView::CloneForm
                    | SubView::RenameForm
                    | SubView::CopyControls
                    | SubView::CopyMods
            ),
            Self::safety_profile_sharing => matches!(
                view,
                SubView::ShareExport | SubView::ShareImport | SubView::SharePreview
            ),
            Self::system_paths_data => matches!(
                view,
                SubView::PathsStatus | SubView::SqliteInfo | SubView::DatasetImport
            ),
            Self::system_logs_diagnostics => matches!(
                view,
                SubView::LogsList | SubView::DiagnosticsPreview | SubView::LogExports
            ),
            Self::system_about => matches!(view, SubView::AboutInfo),
        }
    }
}

impl Sector {
    pub fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::SaveData => "Save Data",
            Self::Vehicles => "Vehicles",
            Self::Operations => "Operations",
            Self::Library => "Library",
            Self::Safety => "Safety",
            Self::System => "System",
        }
    }

    pub fn default_route(self) -> Route {
        match self {
            Self::Overview => Route::overview_dashboard,
            Self::SaveData => Route::save_data_economy,
            Self::Vehicles => Route::vehicles_player_truck,
            Self::Operations => Route::operations_garages,
            Self::Library => Route::library_mod_manager,
            Self::Safety => Route::safety_save_health,
            Self::System => Route::system_paths_data,
        }
    }

    pub fn routes(self) -> &'static [Route] {
        match self {
            Self::Overview => &[
                Route::overview_dashboard,
                Route::overview_quick_actions,
                Route::overview_recent_activity,
            ],
            Self::SaveData => &[
                Route::save_data_economy,
                Route::save_data_coaches,
                Route::save_data_config,
                Route::save_data_custom_keys,
            ],
            Self::Vehicles => &[
                Route::vehicles_player_truck,
                Route::vehicles_player_trailer,
                Route::vehicles_truck_switcher,
                Route::vehicles_trailer_switcher,
                Route::vehicles_powertrain,
                Route::vehicles_fleet_inventory,
            ],
            Self::Operations => &[
                Route::operations_garages,
                Route::operations_ai_drivers,
                Route::operations_live_telemetry,
                Route::operations_dispatcher,
                Route::operations_logbook_analytics,
                Route::operations_vtc_account,
            ],
            Self::Library => &[
                Route::library_mod_manager,
                Route::library_presets,
                Route::library_sandboxes,
                Route::library_conflict_analyzer,
            ],
            Self::Safety => &[
                Route::safety_save_health,
                Route::safety_backups_restore,
                Route::safety_profile_clone_rename,
                Route::safety_profile_sharing,
            ],
            Self::System => &[
                Route::system_paths_data,
                Route::system_logs_diagnostics,
                Route::system_about,
            ],
        }
    }
}

pub const ALL_SECTORS: [Sector; 7] = [
    Sector::Overview,
    Sector::SaveData,
    Sector::Vehicles,
    Sector::Operations,
    Sector::Library,
    Sector::Safety,
    Sector::System,
];

// Screen modules are implemented by the Phase 3 agents. Each exposes exactly
// `pub fn render(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>)`
// and a `#[derive(Default)] pub struct ScreenState`.
pub mod profile;
pub mod economy;
pub mod fleet;
pub mod garages;
pub mod career;
pub mod mods;
pub mod settings;
pub mod launcher;

/// One persistent state instance per screen. Navigation swaps which one is
/// rendered; it never clears the others (docs/ui-spec.md §1.3 rule 5).
#[derive(Default)]
pub struct ScreenStates {
    pub profile: profile::ScreenState,
    pub economy: economy::ScreenState,
    pub fleet: fleet::ScreenState,
    pub garages: garages::ScreenState,
    pub career: career::ScreenState,
    pub mods: mods::ScreenState,
    pub settings: settings::ScreenState,
    pub launcher: launcher::ScreenState,
}
