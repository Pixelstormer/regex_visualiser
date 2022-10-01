mod about;
mod menu_bar;

use self::about::about;
use self::menu_bar::menu_bar;
use super::{editor::editor, inspector::inspector, tab_bar::tab_bar};
use crate::app::state::AppState;
use egui::Context;

/// Displays and updates the entire ui
///
/// Will call `close_fn` if the application should be closed
pub fn root(ctx: &Context, state: &mut AppState, close_fn: impl FnOnce()) {
    menu_bar(ctx, state, close_fn);
    if state.widgets.about_visible {
        about(ctx, state);
    } else {
        tab_bar(ctx, state);
        inspector(ctx, state);
        editor(ctx, state);
    }
}
