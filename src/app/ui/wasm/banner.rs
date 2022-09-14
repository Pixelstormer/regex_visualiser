use crate::app::ui::toggle_theme;
use egui::{Align, Context, Frame, Layout, RichText, TopBottomPanel, Ui};

/// Adds a container that displays a banner at the top of the window
pub fn banner(ctx: &Context) {
    TopBottomPanel::top("banner").show(ctx, |ui| banner_ui(ui, ctx));
}

/// Displays a banner at the top of the window
pub fn banner_ui(ui: &mut Ui, ctx: &Context) {
    Frame::none().inner_margin(8.0).show(ui, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.heading("Regex Visualiser");

            egui::warn_if_debug_build(ui);

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let spacing = std::mem::replace(&mut ui.spacing_mut().item_spacing.x, 5.0);
                ui.hyperlink_to(
                    format!("{} Github", egui::special_emojis::GITHUB),
                    env!("CARGO_PKG_REPOSITORY"),
                );
                ui.label("Open source on");
                ui.separator();
                ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
                ui.separator();
                ui.spacing_mut().item_spacing.x = spacing;

                let icon = if ctx.style().visuals.dark_mode {
                    '☀'
                } else {
                    '🌙'
                };

                if ui.button(RichText::new(icon).size(20.0)).clicked() {
                    ctx.set_visuals(toggle_theme(&ctx.style().visuals));
                }
            });
        });
    });
}
