mod syntax_guide;

use self::syntax_guide::syntax_guide;
use crate::app::state::{AppState, TabBarState};
use egui::{Context, RichText, ScrollArea, SidePanel, Ui};

/// Adds a container that displays a tab bar of auxiliary information
pub fn tab_bar(ctx: &Context, state: &mut AppState) {
    SidePanel::left("tab_bar")
        .resizable(false)
        .min_width(0.0)
        .show(ctx, |ui| tab_bar_ui(ui, state));

    if state.widgets.tab_bar_state != TabBarState::Collapsed {
        SidePanel::left("tab_bar_contents")
            .max_width(ctx.available_rect().width() - 64.0)
            .show(ctx, |ui| tab_bar_contents(ui, state));
    }
}

/// Displays a tab bar of auxiliary information
pub fn tab_bar_ui(ui: &mut Ui, state: &mut AppState) {
    ui.add_space(ui.style().spacing.item_spacing.y);

    if ui
        .button(RichText::new('ℹ').monospace().size(24.0))
        .on_hover_text("Regex Information")
        .clicked()
    {
        state.widgets.tab_bar_state.toggle(TabBarState::Information);
    }

    if ui
        .button(RichText::new('📖').monospace().size(24.0))
        .on_hover_text("Syntax Guide")
        .clicked()
    {
        state.widgets.tab_bar_state.toggle(TabBarState::SyntaxGuide);
    }
}

fn tab_bar_contents(ui: &mut Ui, state: &AppState) {
    ui.add_space(ui.style().spacing.item_spacing.y);
    match state.widgets.tab_bar_state {
        TabBarState::Collapsed => {}
        TabBarState::SyntaxGuide => syntax_guide(ui),
        TabBarState::Information => regex_info(ui, state),
    }
}

/// Displays information about the regular expression
fn regex_info(ui: &mut Ui, state: &AppState) {
    let wrap = std::mem::replace(&mut ui.style_mut().wrap, Some(false));
    ui.heading("Regex Information");
    ui.separator();
    ui.style_mut().wrap = wrap;

    ScrollArea::vertical().show(ui, |ui| {
        if let Ok(l) = &state.logic {
            ui.monospace(format!("{:#?}", l.ast))
        } else {
            ui.label("The regular expression is malformed. Hover over the red ⊗ to view the error.")
        }
    });
}
