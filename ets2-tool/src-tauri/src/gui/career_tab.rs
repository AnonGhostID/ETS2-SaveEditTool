use std::time::Duration;

use eframe::egui::{self, Color32, RichText};

use crate::commands::ets_snapshot_refresh_active_save;
use crate::features::auth::commands::{auth_login, auth_logout};
use crate::features::career::commands::{
    career_export_analytics_csv, career_get_analytics_summary, career_get_job_log,
    career_get_job_stats, career_get_overview, career_get_status, career_list_trips,
    career_scan_profile_job_history, dispatcher_accept_job,
    dispatcher_assign_and_prepare_and_write, dispatcher_cancel_job,
    dispatcher_cleanup_expired_jobs, dispatcher_generate_jobs, dispatcher_generate_universal_jobs,
    dispatcher_get_active_jobs, dispatcher_get_dispatcher_overview,
    dispatcher_get_generation_status, dispatcher_get_market_jobs, get_plugin_status,
};
use crate::features::vtc::commands::{
    create_company, get_company_members, get_company_overview, get_current_user_profile,
};
use crate::features::vtc::models::CreateCompanyInput;
use crate::features::career::dispatcher::DispatcherMarketJob;
use crate::gui::components::{self, UiExt};
use crate::gui::{CareerSubTab, Ets2NativeApp, theme};
use crate::tauri::async_runtime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DispatcherSort {
    #[default]
    Reward,
    Distance,
    Rate,
}

pub fn render(app: &mut Ets2NativeApp, ui: &mut egui::Ui, ctx: &egui::Context) {
    // Request periodic repaint on Career tab for live telemetry updates
    ctx.request_repaint_after(Duration::from_millis(500));

    // Load career data on first visit (the sidebar now owns sub-tab
    // switching, so there is no switch event to hook anymore).
    if app.career_overview.is_none() {
        refresh_career_data(app);
    }

    let (title, subtitle) = match app.career_sub_tab {
        CareerSubTab::LiveTelemetry => (
            "Live Telemetry",
            "What the game is doing right now: speed, fuel, trip and company.",
        ),
        CareerSubTab::Dispatcher => (
            "Freight Dispatcher",
            "Generate and manage market jobs for the active profile.",
        ),
        CareerSubTab::LogbookAndAnalytics => (
            "Logbook & Analytics",
            "Your delivery history, earnings and performance over time.",
        ),
        CareerSubTab::VtcAndAccount => (
            "VTC & Account",
            "Your virtual trucking company and account.",
        ),
    };
    components::page_header(ui, title, subtitle);
    ui.add_space(8.0);
    ui.horizontal_wrapped(|ui| {
        if components::primary_button(ui, "Refresh Career & Dispatcher").clicked() {
            refresh_career_data(app);
        }
    });
    ui.add_space(8.0);

    match app.career_sub_tab {
        CareerSubTab::LiveTelemetry => render_live_telemetry(app, ui),
        CareerSubTab::Dispatcher => render_dispatcher(app, ui),
        CareerSubTab::LogbookAndAnalytics => render_logbook_and_analytics(app, ui),
        CareerSubTab::VtcAndAccount => render_vtc_and_account(app, ui),
    }
}

pub fn refresh_career_data(app: &mut Ets2NativeApp) {
    app.career_status = career_get_status(app.career_state()).ok();
    app.plugin_installed = get_plugin_status(app.profile_state()).unwrap_or(false);
    app.career_overview = career_get_overview(app.career_state()).ok();
    app.dispatcher_overview =
        dispatcher_get_dispatcher_overview(app.career_state(), app.profile_state()).ok();
    app.dispatcher_gen_status =
        dispatcher_get_generation_status(app.career_state(), app.profile_state()).ok();
    app.market_jobs =
        dispatcher_get_market_jobs(None, app.career_state(), app.profile_state()).unwrap_or_default();
    app.active_dispatcher_jobs =
        dispatcher_get_active_jobs(app.career_state(), app.profile_state()).unwrap_or_default();
    app.trip_log = career_list_trips(app.career_state()).unwrap_or_default();
    app.job_log = career_get_job_log(app.career_state()).unwrap_or_default();
    app.job_stats = career_get_job_stats(app.career_state()).ok();
    app.analytics_summary = career_get_analytics_summary(None, app.career_state()).ok();
    app.vtc_user_profile = get_current_user_profile(app.auth_state()).ok();
    app.vtc_company = get_company_overview(app.auth_state()).ok();
    app.vtc_members = get_company_members(app.auth_state()).unwrap_or_default();
}

fn render_live_telemetry(app: &mut Ets2NativeApp, ui: &mut egui::Ui) {
    let status = career_get_status(app.career_state()).ok();
    let live_tel = app
        .career_state()
        .runtime
        .last_telemetry
        .lock()
        .ok()
        .and_then(|g| g.clone());

    ui.columns(2, |cols| {
        cols[0].card(|ui| {
            ui.heading("SCS Telemetry Bridge Status");
            if let Some(st) = status {
                ui.label(format!(
                    "ETS2 Running: {} | ATS Running: {} | Active Game: {}",
                    st.ets2_running,
                    st.ats_running,
                    st.active_game.as_deref().unwrap_or("None")
                ));
                ui.label(format!(
                    "Telemetry Active: {} | Bridge Connected: {} | SDK Plugin Installed: {}",
                    st.telemetry_running, st.bridge_connected, app.plugin_installed
                ));
            }
            ui.add_space(6.0);
            if let Some(tel) = live_tel {
                ui.heading(
                    RichText::new(format!(
                        "{:.1} km/h  |  {:.0} RPM  |  Gear {}",
                        tel.speed_kph, tel.rpm, tel.gear
                    ))
                    .color(Color32::from_rgb(88, 214, 141)),
                );
                ui.label(format!(
                    "Fuel: {:.1} / {:.1} L | Engine: {} | Paused: {}",
                    tel.fuel_liters,
                    tel.fuel_capacity_liters,
                    if tel.engine_on { "ON" } else { "OFF" },
                    tel.paused
                ));
            } else {
                ui.label("No live telemetry sample received yet (start ETS2/ATS on Windows with the SCS SDK plugin active).");
            }
        });

        cols[1].card(|ui| {
            ui.heading("Company Economy & Bank Overview");
            if let Some(ov) = &app.career_overview {
                ui.label(format!(
                    "Company: {} | Cash Balance: €{} | Debt: €{}",
                    ov.economy.company_name, ov.bank.cash_balance, ov.bank.debt_balance
                ));
                ui.label(format!(
                    "Reputation: {} (Lv {} — {} pts) | Completed Jobs: {}",
                    ov.reputation.label,
                    ov.reputation.level,
                    ov.reputation.xp_points,
                    ov.reputation.completed_jobs
                ));
                ui.label(format!(
                    "Fleet Assets: {} ({} trucks, {} trailers, avg cond {:.1}%)",
                    ov.fleet_overview.total_assets,
                    ov.fleet_overview.trucks,
                    ov.fleet_overview.trailers,
                    ov.fleet_overview.avg_condition
                ));
                if let Some(trip) = &ov.active_trip {
                    ui.separator();
                    ui.colored_label(
                        Color32::from_rgb(245, 176, 65),
                        format!(
                            "Active Trip #{}: {} -> {} ({}, {:.1} km, avg {:.1} km/h)",
                            trip.trip_id,
                            trip.origin,
                            trip.destination,
                            trip.cargo,
                            trip.distance_km,
                            trip.avg_speed_kph
                        ),
                    );
                }
            } else {
                ui.label("Click 'Refresh Career & Dispatcher' to load company overview.");
            }
        });
    });
}

fn render_dispatcher(app: &mut Ets2NativeApp, ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        if ui.button("⚡ Generate Market Jobs").clicked() {
            match dispatcher_generate_jobs(app.career_state(), app.profile_state()) {
                Ok(st) => {
                    app.dispatcher_gen_status = Some(st);
                    refresh_career_data(app);
                    app.set_status("Generated new freight market jobs.", false);
                }
                Err(e) => app.set_status(e, true),
            }
        }
        if ui.button("🌍 Generate Save-Synced Universal Jobs").clicked() {
            match dispatcher_generate_universal_jobs(
                Some(true),
                None,
                app.career_state(),
                app.profile_state(),
            ) {
                Ok(st) => {
                    app.dispatcher_gen_status = Some(st);
                    refresh_career_data(app);
                    app.set_status("Generated universal jobs synced to save context.", false);
                }
                Err(e) => app.set_status(e, true),
            }
        }
        if ui.button("📸 Refresh Active Save Snapshot").clicked() {
            match async_runtime::block_on(ets_snapshot_refresh_active_save(
                app.app.clone(),
                app.profile_state(),
                app.ets_db_state(),
            )) {
                Ok(snap) => app.set_status(
                    format!(
                        "Refreshed save snapshot ({} depots, {} visited cities).",
                        snap.depots.len(),
                        snap.visited_city_tokens.len()
                    ),
                    false,
                ),
                Err(e) => app.set_status(format!("Snapshot error: {e}"), true),
            }
        }
        if ui.button("Cleanup Expired Jobs").clicked() {
            if let Ok(st) = dispatcher_cleanup_expired_jobs(app.career_state(), app.profile_state())
            {
                app.dispatcher_gen_status = Some(st);
                refresh_career_data(app);
            }
        }
    });

    if let Some(st) = &app.dispatcher_gen_status {
        ui.add_space(6.0);
        ui.columns(3, |cols| {
            components::stat_tile(
                &mut cols[0],
                "Open Market Jobs",
                &st.open_total_jobs.to_string(),
                theme::ACCENT,
            );
            components::stat_tile(
                &mut cols[1],
                "Generated",
                &st.open_generated_jobs.to_string(),
                theme::MUTED,
            );
            components::stat_tile(
                &mut cols[2],
                "Save Link Active",
                if st.save_link_active { "Yes" } else { "No" },
                if st.save_link_active {
                    theme::SUCCESS
                } else {
                    theme::WARNING
                },
            );
        });
    }

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.label("Search:");
        ui.text_edit_singleline(&mut app.job_search_filter);
    });
    ui.add_space(6.0);

    let filter = app.job_search_filter.trim().to_ascii_lowercase();
    // Clone so the table can sort/filter without borrowing `app` while
    // the sort headers mutate it.
    let market_jobs = app.market_jobs.clone();
    let mut jobs: Vec<&DispatcherMarketJob> = market_jobs.iter()
        .filter(|job| {
            filter.is_empty()
                || job.company_name.to_ascii_lowercase().contains(&filter)
                || job.origin_city.to_ascii_lowercase().contains(&filter)
                || job.destination_city.to_ascii_lowercase().contains(&filter)
                || job.cargo_type.to_ascii_lowercase().contains(&filter)
        })
        .collect();
    let sort_key = app.dispatcher_sort;
    let sort_asc = app.dispatcher_sort_asc;
    jobs.sort_by(|a, b| {
        let ord = match sort_key {
            DispatcherSort::Reward => a.total_reward.cmp(&b.total_reward),
            DispatcherSort::Distance => a
                .distance_km
                .partial_cmp(&b.distance_km)
                .unwrap_or(std::cmp::Ordering::Equal),
            DispatcherSort::Rate => a
                .calculated_rate_per_km
                .partial_cmp(&b.calculated_rate_per_km)
                .unwrap_or(std::cmp::Ordering::Equal),
        };
        if sort_asc {
            ord
        } else {
            ord.reverse()
        }
    });

    ui.heading(format!("Available Market Jobs ({})", jobs.len()));
    if jobs.is_empty() {
        components::empty_state(
            ui,
            "No jobs match",
            "Try a different search, or generate new market jobs.",
        );
        return;
    }

    egui::Grid::new("dispatcher_jobs_grid")
        .striped(true)
        .num_columns(7)
        .show(ui, |ui| {
            ui.strong("Company");
            ui.strong("Route");
            ui.strong("Cargo");
            dispatcher_sort_header(
                &mut app.dispatcher_sort,
                &mut app.dispatcher_sort_asc,
                ui,
                "Distance",
                DispatcherSort::Distance,
            );
            dispatcher_sort_header(
                &mut app.dispatcher_sort,
                &mut app.dispatcher_sort_asc,
                ui,
                "Rate/km",
                DispatcherSort::Rate,
            );
            dispatcher_sort_header(
                &mut app.dispatcher_sort,
                &mut app.dispatcher_sort_asc,
                ui,
                "Reward",
                DispatcherSort::Reward,
            );
            ui.strong("Status");
            ui.end_row();

            for job in &jobs {
                let selected = app.selected_market_job_id == job.id;
                if ui.selectable_label(selected, &job.company_name).clicked() {
                    app.selected_market_job_id = job.id.clone();
                }
                ui.label(format!(
                    "{} ({}) -> {} ({})",
                    job.origin_city, job.origin_country, job.destination_city, job.destination_country
                ));
                ui.label(format!("{} ({:.0} kg)", job.cargo_type, job.cargo_mass_kg));
                ui.label(format!("{:.0} km", job.distance_km));
                ui.label(format!("€{:.2}", job.calculated_rate_per_km));
                ui.label(format!("€{}", job.total_reward));
                ui.label(format!(
                    "{} ({})",
                    job.status,
                    job.ets2_job_link_status.as_deref().unwrap_or("-")
                ));
                ui.end_row();
            }
        });

    // Detail card for the selected job.
    if let Some(job) = app
        .market_jobs
        .iter()
        .find(|job| job.id == app.selected_market_job_id)
        .cloned()
    {
        render_job_detail(app, ui, &job);
    }
}

fn render_job_detail(
    app: &mut Ets2NativeApp,
    ui: &mut egui::Ui,
    job: &DispatcherMarketJob,
) {
    ui.add_space(10.0);
    ui.card(|ui| {
        ui.heading(format!(
            "{} -> {}",
            job.origin_city, job.destination_city
        ));
        ui.label(format!(
            "{} · {} ({:.0} kg) · {:.0} km · ~{} min",
            job.company_name,
            job.cargo_type,
            job.cargo_mass_kg,
            job.distance_km,
            job.estimated_duration_minutes
        ));
        ui.add_space(4.0);
        egui::Grid::new("job_detail_grid")
            .num_columns(2)
            .spacing([24.0, 4.0])
            .show(ui, |ui| {
                ui.label("Reward:");
                ui.label(format!("€{} (profit est. €{})", job.total_reward, job.profit_estimate));
                ui.end_row();
                ui.label("Rate:");
                ui.label(format!("€{:.2}/km", job.calculated_rate_per_km));
                ui.end_row();
                ui.label("Urgency / Difficulty:");
                ui.label(format!("{} / {}", job.urgency_level, job.difficulty_level));
                ui.end_row();
                ui.label("Equipment:");
                ui.label(format!(
                    "{}{}",
                    job.equipment_type_required,
                    job.trailer_type_required
                        .as_deref()
                        .map(|t| format!(" ({t})"))
                        .unwrap_or_default()
                ));
                ui.end_row();
                ui.label("Status:");
                ui.label(format!(
                    "{} ({})",
                    job.status,
                    job.ets2_job_link_status.as_deref().unwrap_or("-")
                ));
                ui.end_row();
                if let Some(expires) = &job.expires_at_utc {
                    ui.label("Expires:");
                    ui.label(expires);
                    ui.end_row();
                }
            });
        if let Some(note) = &job.risk_note {
            ui.label(RichText::new(format!("Risk: {note}")).size(11.5).color(theme::WARNING));
        }
        if let Some(note) = &job.bonus_note {
            ui.label(RichText::new(format!("Bonus: {note}")).size(11.5).color(theme::SUCCESS));
        }
        if let Some(error) = &job.last_error_message {
            ui.label(RichText::new(format!("Last error: {error}")).size(11.5).color(theme::DANGER));
        }
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            let job_id = job.id.clone();
            if components::primary_button(ui, "Accept Job").clicked() {
                match dispatcher_accept_job(
                    job_id.clone(),
                    app.career_state(),
                    app.profile_state(),
                ) {
                    Ok(_) => {
                        refresh_career_data(app);
                        app.set_status(format!("Accepted job {job_id}."), false);
                    }
                    Err(e) => app.set_status(e, true),
                }
            }
            if ui.button("Write to Quicksave").clicked() {
                match async_runtime::block_on(dispatcher_assign_and_prepare_and_write(
                    job_id.clone(),
                    true,
                    app.app.clone(),
                    app.career_state(),
                    app.profile_state(),
                    app.ets_db_state(),
                )) {
                    Ok(dto) => {
                        refresh_career_data(app);
                        app.set_status(
                            format!(
                                "Job {} written to quicksave (applied={}).",
                                dto.job.job.id, dto.write_applied
                            ),
                            false,
                        );
                    }
                    Err(e) => app.set_status(format!("Write to quicksave failed: {e}"), true),
                }
            }
            if ui.button("Cancel Job").clicked() {
                let _ = dispatcher_cancel_job(
                    job_id.clone(),
                    app.career_state(),
                    app.profile_state(),
                );
                refresh_career_data(app);
                app.set_status(format!("Cancelled job {job_id}."), false);
            }
        });
    });
}

/// A clickable table header that toggles sorting by `key`.
fn dispatcher_sort_header(
    current: &mut DispatcherSort,
    ascending: &mut bool,
    ui: &mut egui::Ui,
    label: &str,
    key: DispatcherSort,
) {
    let arrow = if *current == key {
        if *ascending {
            " ^"
        } else {
            " v"
        }
    } else {
        ""
    };
    if ui.button(format!("{label}{arrow}")).clicked() {
        if *current == key {
            *ascending = !*ascending;
        } else {
            *current = key;
            *ascending = false;
        }
    }
}

fn render_logbook_and_analytics(app: &mut Ets2NativeApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if ui.button("🔍 Scan Profile Save Job History").clicked() {
            match career_scan_profile_job_history(
                app.selected_profile_path.clone(),
                app.career_state(),
                app.profile_state(),
            ) {
                Ok(_res) => {
                    refresh_career_data(app);
                    app.set_status("Scanned profile job history into SQLite analytics.", false);
                }
                Err(e) => app.set_status(e, true),
            }
        }
        if ui.button("📤 Export Analytics CSV").clicked() {
            match career_export_analytics_csv(app.app.clone(), None, app.career_state()) {
                Ok(Some(path)) => app.set_status(format!("Exported analytics CSV to {path}"), false),
                Ok(None) => {}
                Err(e) => app.set_status(e, true),
            }
        }
    });

    if let Some(sum) = &app.analytics_summary {
        ui.card(|ui| {
            ui.heading("Analytics Summary");
            ui.label(format!(
                "Total Jobs: {} | Revenue: €{} | Costs: €{} | Net Profit: €{} | Distance: {:.1} km",
                sum.total_jobs,
                sum.total_revenue.unwrap_or(0),
                sum.total_costs.unwrap_or(0),
                sum.total_profit.unwrap_or(0),
                sum.total_distance_km.unwrap_or(0.0)
            ));
        });
    }

    ui.add_space(8.0);
    ui.heading(format!("Recorded Trips ({})", app.trip_log.len()));
    egui::Grid::new("trips_grid")
        .striped(true)
        .num_columns(7)
        .show(ui, |ui| {
            ui.strong("Trip ID");
            ui.strong("Route");
            ui.strong("Cargo");
            ui.strong("Distance");
            ui.strong("Avg / Max Speed");
            ui.strong("Income");
            ui.strong("Status");
            ui.end_row();
            for t in &app.trip_log {
                ui.label(format!("#{}", t.id));
                ui.label(format!(
                    "{} -> {}",
                    t.origin.as_deref().unwrap_or("-"),
                    t.destination.as_deref().unwrap_or("-")
                ));
                ui.label(t.cargo.as_deref().unwrap_or("-"));
                ui.label(format!("{:.1} km", t.distance_km));
                ui.label(format!("{:.1} / {:.1} km/h", t.avg_speed_kph, t.max_speed_kph));
                ui.label(format!("€{}", t.income.unwrap_or(0)));
                ui.label(&t.status);
                ui.end_row();
            }
        });
}

fn render_vtc_and_account(app: &mut Ets2NativeApp, ui: &mut egui::Ui) {
    ui.columns(2, |cols| {
        cols[0].card(|ui| {
            ui.heading("Local Account & Session");
            if let Some(profile) = &app.vtc_user_profile {
                ui.label(format!(
                    "Signed in as: {} (User ID #{}) | Role: {}",
                    profile.username,
                    profile.user_id,
                    profile.company_role.as_deref().unwrap_or("Driver")
                ));
                if ui.button("Sign Out").clicked() {
                    let _ = auth_logout(app.auth_state());
                    refresh_career_data(app);
                }
            } else {
                ui.label("Sign in to local VTC database:");
                ui.horizontal(|ui| {
                    ui.label("Email:");
                    ui.text_edit_singleline(&mut app.auth_email_input);
                });
                ui.horizontal(|ui| {
                    ui.label("Password:");
                    ui.add(
                        egui::TextEdit::singleline(&mut app.auth_password_input).password(true),
                    );
                });
                if ui.button("Sign In").clicked() {
                    match auth_login(
                        app.auth_email_input.clone(),
                        app.auth_password_input.clone(),
                        true,
                        app.auth_state(),
                    ) {
                        Ok(res) => {
                            refresh_career_data(app);
                            app.set_status(format!("Signed in as {}.", res.user.username), false);
                        }
                        Err(e) => app.set_status(e, true),
                    }
                }
            }
        });

        cols[1].card(|ui| {
            ui.heading("Virtual Trucking Company (VTC)");
            if let Some(comp) = &app.vtc_company {
                ui.label(format!(
                    "Company: {} (ID #{}) | HQ: {} | Members: {}",
                    comp.name, comp.id, comp.location, comp.members_count
                ));
                ui.separator();
                for m in &app.vtc_members {
                    ui.label(format!("• {} — {} (joined {})", m.username, m.role_key, m.joined_at));
                }
            } else {
                ui.label("Create a local VTC company:");
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut app.new_company_name);
                });
                ui.horizontal(|ui| {
                    ui.label("HQ City:");
                    ui.text_edit_singleline(&mut app.new_company_location);
                });
                if ui.button("Create VTC Company").clicked()
                    && !app.new_company_name.trim().is_empty()
                {
                    match create_company(
                        CreateCompanyInput {
                            name: app.new_company_name.trim().to_string(),
                            location: app.new_company_location.trim().to_string(),
                            language: app.current_language.clone(),
                            game: app.selected_game.clone(),
                            description: None,
                            logo_path: None,
                            header_path: None,
                            slogan: None,
                            accent_color: None,
                            public_visibility: Some(true),
                        },
                        app.auth_state(),
                    ) {
                        Ok(_) => {
                            refresh_career_data(app);
                            app.set_status("Created VTC company.", false);
                        }
                        Err(e) => app.set_status(e, true),
                    }
                }
            }
        });
    });
}
