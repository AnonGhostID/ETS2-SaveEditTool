//! Sector Map: first navigation level. Seven sector tiles, then the
//! capability tiles of the selected sector.

use eframe::egui::{self, RichText, Ui};

use crate::gui::screens::{ALL_SECTORS, Route, Sector};
use crate::gui::shell::AppShell;
use crate::gui::theme;

pub fn render(shell: &mut AppShell, ctx: &egui::Context, compact: bool) {
    let width = if compact { 210.0 } else { 360.0 };
    egui::SidePanel::left("sector_map")
        .resizable(false)
        .exact_width(width)
        .frame(
            egui::Frame::none()
                .fill(theme::BG)
                .inner_margin(egui::Margin::symmetric(10.0, 10.0))
                .stroke(egui::Stroke::new(1.0, theme::BORDER)),
        )
        .show(ctx, |ui| render_body(shell, ui, compact));
}

fn render_body(shell: &mut AppShell, ui: &mut Ui, compact: bool) {
    ui.horizontal(|ui| {
        let title = if shell.sectors_overview {
            "SECTORS".to_string()
        } else {
            format!("SECTOR: {}", shell.sector.unwrap_or_default().label().to_uppercase())
        };
        ui.label(RichText::new(title).size(10.5).strong().color(theme::FAINT));
        if !shell.sectors_overview && ui.small_button("◀ All sectors").clicked() {
            shell.show_sector_overview();
        }
    });
    ui.add_space(8.0);

    egui::ScrollArea::vertical()
        .id_salt("sector_map_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            if shell.sectors_overview {
                let mut grid = egui::Grid::new("sector_tiles").num_columns(if compact { 1 } else { 2 });
                grid = grid.spacing(egui::vec2(8.0, 8.0));
                grid.show(ui, |ui| {
                    for sector in ALL_SECTORS {
                        let label = format!(
                            "{}\n{} capability tile(s)",
                            sector.label(),
                            sector.routes().len()
                        );
                        if tile(ui, &label, compact) {
                            shell.navigate(sector.default_route(), None);
                        }
                    }
                });
            } else if let Some(sector) = shell.sector {
                let mut grid =
                    egui::Grid::new("capability_tiles").num_columns(if compact { 1 } else { 3 });
                grid = grid.spacing(egui::vec2(8.0, 8.0));
                grid.show(ui, |ui| {
                    for route in sector.routes() {
                        if tile(ui, route.label(), compact) {
                            shell.navigate(*route, Some(route.default_sub_view()));
                        }
                    }
                });
            }
        });
}

fn tile(ui: &mut Ui, label: &str, compact: bool) -> bool {
    let height = if compact { 34.0 } else { 66.0 };
    let response = ui.add(
        egui::Button::new(RichText::new(label).size(11.5).wrap())
            .rounding(egui::Rounding::same(8.0))
            .min_size(egui::vec2(if compact { 186.0 } else { 104.0 }, height)),
    );
    response.on_hover_cursor(egui::CursorIcon::PointingHand).clicked()
}

/// Kept for documentation/deep links: route → sector metadata lives in the
/// shared screen contract so screens can deep-link without a shell borrow.
#[allow(dead_code)]
fn _route_metadata(route: Route) -> Sector {
    route.sector()
}
