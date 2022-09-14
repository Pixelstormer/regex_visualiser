use crate::app::state::AppState;
use egui::{CentralPanel, Context, Ui, Vec2};

/// Adds a container that displays general information about the application
pub fn about(ctx: &Context, state: &mut AppState) {
    CentralPanel::default().show(ctx, |ui| about_ui(ui, state));
}

/// Displays general information about the application
pub fn about_ui(ui: &mut Ui, state: &mut AppState) {
    let mut rect = ui.available_rect_before_wrap();
    let horizontal_shrink = rect.width() / 3.0;
    let vertical_shrink = rect.height() / 3.0;
    rect = rect.shrink2(Vec2::new(horizontal_shrink, vertical_shrink));

    ui.allocate_ui_at_rect(rect, |ui| {
        ui.heading("Regex Visualiser");
        ui.separator();

        ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.label("Open source on ");
            ui.hyperlink_to(
                format!("{} Github", egui::special_emojis::GITHUB),
                env!("CARGO_PKG_REPOSITORY"),
            );
        });

        ui.vertical_centered_justified(|ui| {
            if ui.button("Close").clicked() {
                state.widgets.about_visible = false;
            }
        });
    });
}
