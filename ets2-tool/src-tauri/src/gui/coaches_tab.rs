use eframe::egui;

use crate::features::save_editor::coaches::{
    display_bus_name, display_job_stop_name, display_stop_name, Career,
};
use crate::features::save_editor::commands::{edit_career_xp, read_project_coaches};
use crate::gui::components::{self, UiExt};
use crate::gui::theme;
use crate::gui::Ets2NativeApp;
use crate::xp::command::calculate_level;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CoachesStopFilter {
    #[default]
    All,
    Discovered,
    Undiscovered,
}

pub fn render(app: &mut Ets2NativeApp, ui: &mut egui::Ui) {
    components::page_header(
        ui,
        "Coaches — Bus Career",
        "Bus level, XP, job history and discovered stops from the Coaches DLC.",
    );
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if components::primary_button(ui, "Load Coaches Data").clicked() {
            reload(app);
        }
        if let Some(coaches) = &app.coaches_snapshot {
            ui.label(format!(
                "{} stops · {} jobs logged",
                coaches.stops.len(),
                coaches.total_bus_jobs
            ));
        }
    });
    ui.add_space(8.0);

    let Some(coaches) = app.coaches_snapshot.clone() else {
        components::empty_state(
            ui,
            "No coaches data loaded",
            "Load coaches data to see the bus career from the active save.",
        );
        return;
    };

    if !coaches.warnings.is_empty() {
        ui.card(|ui| {
            ui.horizontal(|ui| {
                components::status_dot(ui, theme::WARNING);
                ui.label("The save could not be read unambiguously in places:");
            });
            for warning in &coaches.warnings {
                ui.label(format!("• {warning}"));
            }
        });
        ui.add_space(8.0);
    }

    let bus_level = coaches.bus_xp.map(|xp| calculate_level(xp.max(0) as u64));
    ui.columns(4, |cols| {
        match bus_level {
            Some(progress) => components::stat_tile(
                &mut cols[0],
                "Bus Level",
                &progress.level.to_string(),
                theme::ACCENT,
            ),
            None => components::stat_tile(&mut cols[0], "Bus Level", "—", theme::FAINT),
        }
        components::stat_tile(
            &mut cols[1],
            "Bus XP",
            &coaches
                .bus_xp
                .map(|xp| xp.to_string())
                .unwrap_or_else(|| "—".to_string()),
            theme::TEXT,
        );
        components::stat_tile(
            &mut cols[2],
            "Discovered Stops",
            &format!("{}/{}", coaches.discovered_stops, coaches.stops.len()),
            theme::SUCCESS,
        );
        components::stat_tile(
            &mut cols[3],
            "Jobs Completed",
            &coaches.total_bus_jobs.to_string(),
            theme::TEXT,
        );
    });
    ui.add_space(8.0);
    ui.columns(3, |cols| {
        components::stat_tile(
            &mut cols[0],
            "Total Earned",
            &format!("€{}", coaches.total_bus_reward),
            theme::SUCCESS,
        );
        components::stat_tile(
            &mut cols[1],
            "Passengers Carried",
            &coaches.total_bus_passengers.to_string(),
            theme::TEXT,
        );
        components::stat_tile(
            &mut cols[2],
            "Distance Driven",
            &format!("{:.0} km", coaches.total_bus_distance),
            theme::TEXT,
        );
    });
    ui.add_space(8.0);

    if let Some(progress) = bus_level {
        ui.card(|ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Level {} — bus career progress", progress.level));
                if progress.xp_needed_for_next_level == 0 {
                    ui.label("(max level)");
                }
            });
            ui.add(egui::ProgressBar::new(progress.progress_percent / 100.0).text(format!(
                "{} / {} XP to level {}",
                progress.xp_into_level,
                progress.xp_into_level + progress.xp_needed_for_next_level,
                progress.level + 1
            )));
        });
        ui.add_space(8.0);
    }

    ui.card(|ui| {
        components::section_label(ui, "CAREER");
        ui.horizontal(|ui| {
            ui.label(format!(
                "Mode: {}",
                coaches
                    .my_vehicles_mode
                    .as_deref()
                    .unwrap_or("unknown")
            ));
            ui.separator();
            ui.label(format!(
                "Owned buses: {}",
                coaches.buses.as_deref().unwrap_or("—")
            ));
            ui.separator();
            ui.label(format!(
                "Current bus job: {}",
                match coaches.current_bus_job.as_deref() {
                    Some("null") | None => "none".to_string(),
                    Some(job) => job.to_string(),
                }
            ));
        });
        ui.add_space(4.0);
        if coaches.bus_xp.is_some() {
            ui.horizontal(|ui| {
                ui.label("Bus XP:");
                ui.add(
                    egui::DragValue::new(&mut app.edit_bus_xp_input)
                        .speed(100)
                        .range(0..=9_007_199_254_740_991i64),
                );
                if ui.button("Apply").clicked() {
                    apply_bus_xp(app, app.edit_bus_xp_input, &coaches.save_hash);
                }
                if let Some(truck_xp) = coaches.truck_xp {
                    if ui.button("Set bus XP = truck XP").clicked() {
                        apply_bus_xp(app, truck_xp, &coaches.save_hash);
                    }
                }
            });
            ui.label("Only bus XP are changed. Bus levels are derived from XP.");
        } else {
            ui.label("This save has no bus career yet, so bus XP cannot be edited.");
        }
    });
    ui.add_space(8.0);

    ui.card(|ui| {
        components::section_label(ui, "JOB HISTORY");
        let entries = coaches
            .job_logs
            .iter()
            .flat_map(|log| log.entries_detail.iter())
            .collect::<Vec<_>>();
        if entries.is_empty() {
            ui.label("No bus jobs logged in this save yet.");
        } else {
            egui::Grid::new("coaches_job_history")
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Route");
                    ui.label("Bus");
                    ui.label("Type");
                    ui.label("Reward");
                    ui.label("XP");
                    ui.label("Distance");
                    ui.label("Passengers");
                    ui.end_row();
                    for entry in entries {
                        let route = match (&entry.source_stop, &entry.target_stop) {
                            (Some(from), Some(to)) => format!(
                                "{} -> {}",
                                display_job_stop_name(from),
                                display_job_stop_name(to)
                            ),
                            _ => "—".to_string(),
                        };
                        ui.label(route);
                        ui.label(
                            entry
                                .assigned_bus
                                .as_deref()
                                .map(display_bus_name)
                                .unwrap_or_else(|| "—".to_string()),
                        );
                        ui.label(entry.job_type.as_deref().unwrap_or("—"));
                        ui.label(
                            entry
                                .reward
                                .map(|value| format!("€{value}"))
                                .unwrap_or_else(|| "—".to_string()),
                        );
                        ui.label(
                            entry
                                .exp_delta
                                .map(|value| value.to_string())
                                .unwrap_or_else(|| "—".to_string()),
                        );
                        ui.label(
                            entry
                                .travelled_distance
                                .map(|value| format!("{value:.0} km"))
                                .unwrap_or_else(|| "—".to_string()),
                        );
                        ui.label(
                            entry
                                .total_passengers
                                .map(|value| value.to_string())
                                .unwrap_or_else(|| "—".to_string()),
                        );
                        ui.end_row();
                    }
                });
        }
    });
    ui.add_space(8.0);

    ui.card(|ui| {
        components::section_label(ui, "BUS STATISTICS");
        let present = coaches
            .statistics
            .iter()
            .filter_map(|(field, value)| value.as_ref().map(|value| (field, value)))
            .collect::<Vec<_>>();
        if present.is_empty() {
            ui.label(
                "This save does not store per-career bus statistics yet \
                (distance, job counts, passengers, play time). Totals above come from the job log.",
            );
        } else {
            egui::Grid::new("coaches_statistics")
                .striped(true)
                .show(ui, |ui| {
                    for (field, value) in present {
                        ui.label(prettify_stat_field(field));
                        ui.label(value);
                        ui.end_row();
                    }
                });
        }
    });
    ui.add_space(8.0);

    ui.card(|ui| {
        components::section_label(ui, "BUS STOPS");
        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut app.coaches_stop_search);
            if let Some(filter) = components::sub_tabs(
                ui,
                app.coaches_stop_filter,
                &[
                    (CoachesStopFilter::All, "All"),
                    (CoachesStopFilter::Discovered, "Discovered"),
                    (CoachesStopFilter::Undiscovered, "Undiscovered"),
                ],
            ) {
                app.coaches_stop_filter = filter;
            }
        });
        let query = app.coaches_stop_search.trim().to_lowercase();
        let filtered = coaches
            .stops
            .iter()
            .filter(|stop| match app.coaches_stop_filter {
                CoachesStopFilter::All => true,
                CoachesStopFilter::Discovered => stop.discovered == Some(true),
                CoachesStopFilter::Undiscovered => stop.discovered != Some(true),
            })
            .filter(|stop| {
                query.is_empty()
                    || display_stop_name(&stop.id).to_lowercase().contains(&query)
                    || stop.id.to_lowercase().contains(&query)
            })
            .collect::<Vec<_>>();
        ui.label(format!(
            "Showing {} of {} stops ({} discovered)",
            filtered.len(),
            coaches.stops.len(),
            coaches.discovered_stops
        ));
        let row_height = ui.text_style_height(&egui::TextStyle::Body) + 6.0;
        egui::ScrollArea::vertical()
            .max_height(360.0)
            .show_rows(ui, row_height, filtered.len(), |ui, range| {
                for stop in &filtered[range] {
                    ui.horizontal(|ui| {
                        let color = match stop.discovered {
                            Some(true) => theme::SUCCESS,
                            Some(false) => theme::FAINT,
                            None => theme::WARNING,
                        };
                        components::status_dot(ui, color);
                        ui.label(display_stop_name(&stop.id));
                        ui.label(
                            egui::RichText::new(&stop.id)
                                .small()
                                .color(theme::MUTED),
                        );
                        if !stop.resolved {
                            ui.label(
                                egui::RichText::new("unresolved")
                                    .small()
                                    .color(theme::WARNING),
                            );
                        }
                    });
                }
            });
    });
}

fn reload(app: &mut Ets2NativeApp) {
    match read_project_coaches(app.profile_state()) {
        Ok(snapshot) => {
            app.edit_bus_xp_input = snapshot.bus_xp.unwrap_or(0);
            app.coaches_snapshot = Some(snapshot);
            app.set_status("Coaches data loaded.", false);
        }
        Err(error) => app.set_status(format!("Coaches load: {error}"), true),
    }
}

fn apply_bus_xp(app: &mut Ets2NativeApp, xp: i64, save_hash: &str) {
    match edit_career_xp(
        xp,
        Career::Bus,
        save_hash.to_string(),
        app.profile_state(),
        app.profile_cache(),
        app.decrypt_cache(),
    ) {
        Ok(()) => {
            app.set_status(format!("Bus XP saved: {xp}"), false);
            reload(app);
        }
        Err(error) => app.set_status(format!("Bus XP edit: {error}"), true),
    }
}

fn prettify_stat_field(field: &str) -> String {
    let rest = field.strip_prefix("bus_").unwrap_or(field);
    rest.split('_')
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
