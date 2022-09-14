use crate::app::{state::AppState, ui::toggle_theme};
use egui::{Context, Layout, TopBottomPanel, Ui};

/// Adds a container that displays the menu bar (The thing that is usually toggled by pressing `alt`)
///
/// Will call `close_fn` if the application should be closed
pub fn menu_bar(ctx: &Context, state: &mut AppState, close_fn: impl FnOnce()) {
    TopBottomPanel::top("menu_bar").show(ctx, |ui| menu_bar_ui(ui, state, ctx, close_fn));
}

/// Displays the menu bar (The thing that is usually toggled by pressing `alt`)
///
/// Will call `close_fn` if the application should be closed
pub fn menu_bar_ui(ui: &mut Ui, state: &mut AppState, ctx: &Context, close_fn: impl FnOnce()) {
    egui::menu::bar(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("Quit").clicked() {
                close_fn();
            }
        });

        ui.menu_button("View", |ui| {
            if ui.button("Toggle Theme").clicked() {
                ctx.set_visuals(toggle_theme(&ctx.style().visuals));
            }
        });

        ui.menu_button("Help", |ui| {
            if ui.button("About").clicked() {
                state.widgets.about_visible = true;
            }
        });

        ui.with_layout(
            Layout::right_to_left(egui::Align::Center),
            egui::warn_if_debug_build,
        );
    });
}
