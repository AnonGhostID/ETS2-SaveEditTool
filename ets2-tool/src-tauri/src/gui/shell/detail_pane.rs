//! Detail Pane: breadcrumb plus exactly one capability tile's screen.

use eframe::egui::{self, RichText};

use crate::gui::screens::{self, IssuesTab, Route, ScreenCtx, ScreenStates, ShellCommand};
use crate::gui::shell::AppShell;
use crate::gui::theme;

pub fn render(shell: &mut AppShell, ctx: &egui::Context, viewport_width: f32) {
    let route = shell.route;
    egui::CentralPanel::default()
        .frame(
            egui::Frame::none()
                .fill(theme::BG)
                .inner_margin(egui::Margin::symmetric(16.0, 12.0)),
        )
        .show(ctx, |ui| {
            if viewport_width < 960.0 {
                ui.label(
                    RichText::new("Resize the window to at least 960x640")
                        .size(12.0)
                        .strong()
                        .color(theme::WARNING),
                );
                ui.add_space(6.0);
            }

            let sector = shell.sector.unwrap_or_default().label();
            let tile = route.label();
            let sub_view = format!("{:?}", shell.sub_view);
            ui.label(
                RichText::new(format!("{sector} › {tile} › {sub_view}"))
                    .size(11.0)
                    .color(theme::MUTED),
            );
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(8.0);

            let mut commands: Vec<ShellCommand> = Vec::new();
            let mut scroll = egui::ScrollArea::vertical()
                .id_salt("detail_scroll")
                .auto_shrink([false, false]);
            if shell.scroll_to_top {
                scroll = scroll.vertical_scroll_offset(0.0);
            }
            scroll.show(ui, |ui| {
                let mut screen_ctx = ScreenCtx {
                    data: &shell.data,
                    commands: &mut commands,
                };
                dispatch(route, &mut shell.screen_states, ui, &mut screen_ctx);
            });
            shell.scroll_to_top = false;

            for command in commands {
                shell.handle_command(command);
            }

            ui.add_space(8.0);
            ui.separator();
            ui.horizontal(|ui| {
                let problems = shell.validation_issues.len() + shell.errors.len();
                ui.label(
                    RichText::new(format!("Validation / errors: {problems}"))
                        .size(10.0)
                        .color(if problems == 0 { theme::FAINT } else { theme::WARNING }),
                );
                if ui.small_button("Open Issues").clicked() {
                    shell.open_issues(IssuesTab::Validation);
                }
            });
        });
}

fn dispatch(
    route: Route,
    states: &mut ScreenStates,
    ui: &mut egui::Ui,
    screen_ctx: &mut ScreenCtx<'_>,
) {
    match route {
        Route::overview_dashboard
        | Route::overview_quick_actions
        | Route::overview_recent_activity => {
            screens::launcher::render(ui, &mut states.launcher, screen_ctx)
        }
        Route::save_data_economy
        | Route::save_data_coaches
        | Route::save_data_config
        | Route::save_data_custom_keys => {
            screens::economy::render(ui, &mut states.economy, screen_ctx)
        }
        Route::vehicles_player_truck
        | Route::vehicles_player_trailer
        | Route::vehicles_truck_switcher
        | Route::vehicles_trailer_switcher
        | Route::vehicles_powertrain
        | Route::vehicles_fleet_inventory => {
            screens::fleet::render(ui, &mut states.fleet, screen_ctx)
        }
        Route::operations_garages | Route::operations_ai_drivers => {
            screens::garages::render(ui, &mut states.garages, screen_ctx)
        }
        Route::operations_live_telemetry
        | Route::operations_dispatcher
        | Route::operations_logbook_analytics
        | Route::operations_vtc_account => {
            screens::career::render(ui, &mut states.career, screen_ctx)
        }
        Route::library_mod_manager
        | Route::library_presets
        | Route::library_sandboxes
        | Route::library_conflict_analyzer => {
            screens::mods::render(ui, &mut states.mods, screen_ctx)
        }
        Route::safety_save_health
        | Route::safety_backups_restore
        | Route::safety_profile_clone_rename
        | Route::safety_profile_sharing => {
            screens::profile::render(ui, &mut states.profile, screen_ctx)
        }
        Route::system_paths_data | Route::system_logs_diagnostics | Route::system_about => {
            screens::settings::render(ui, &mut states.settings, screen_ctx)
        }
    }
}
