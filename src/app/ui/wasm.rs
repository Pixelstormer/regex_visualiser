use self::banner::banner;
use super::{editor::editor, tab_bar::tab_bar};
use crate::app::state::AppState;
use egui::Context;

mod banner;

/// Displays and updates the entire ui
pub fn root(ctx: &Context, state: &mut AppState) {
    banner(ctx);
    tab_bar(ctx, state);
    editor(ctx, state);
}
