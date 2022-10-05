mod color;
mod loop_vec;
mod parsing;
mod shape;
mod state;
mod text;
mod ui;

use self::{
    state::AppState,
    ui::{create_font_definitions, update_style},
};
use eframe::{App, CreationContext, Frame, Storage};
use egui::Context;
use serde::{Deserialize, Serialize};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Default, Deserialize, Serialize)]
#[serde(default)] // If we add new fields, give them default values when deserializing old state
pub struct Application {
    #[serde(skip)]
    state: AppState,
}

impl Application {
    /// Called once before the first frame.
    pub fn new(cc: &CreationContext<'_>) -> Self {
        // Update the default fonts and font sizes
        cc.egui_ctx.set_fonts(create_font_definitions());

        // Update the style
        cc.egui_ctx
            .set_style(update_style(cc.egui_ctx.style().as_ref().clone()));

        // Load previous app state (if any).
        cc.storage
            .and_then(|storage| eframe::get_value(storage, eframe::APP_KEY))
            .unwrap_or_default()
    }
}

impl App for Application {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second. (Native)
    #[cfg(not(target_arch = "wasm32"))]
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        ui::native::root(ctx, &mut self.state, || frame.close());
    }

    /// Called each time the UI needs repainting, which may be many times per second. (Wasm)
    #[cfg(target_arch = "wasm32")]
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        ui::wasm::root(ctx, &mut self.state);
    }
}
