mod color;
mod parsing;
mod shape;
mod state;
mod text;
mod ui;

use self::state::AppState;
use eframe::{CreationContext, Frame};
use egui::{Context, FontData, FontDefinitions, FontFamily, Style, Vec2};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Default, serde::Deserialize, serde::Serialize)]
#[serde(default)] // If we add new fields, give them default values when deserializing old state
pub struct Application {
    #[serde(skip)]
    state: AppState,
}

impl Application {
    /// Called once before the first frame.
    pub fn new(cc: &CreationContext<'_>) -> Self {
        // Update the default fonts and font sizes
        cc.egui_ctx.set_fonts(get_font_definitions());

        // Update the style
        cc.egui_ctx
            .set_style(update_style(cc.egui_ctx.style().as_ref().clone()));

        // Load previous app state (if any).
        cc.storage
            .and_then(|storage| eframe::get_value(storage, eframe::APP_KEY))
            .unwrap_or_default()
    }
}

impl eframe::App for Application {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second. (Native)
    #[cfg(not(target_arch = "wasm32"))]
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        ui::native::root(&mut self.state, ctx, || frame.close());
    }

    /// Called each time the UI needs repainting, which may be many times per second. (Wasm)
    #[cfg(target_arch = "wasm32")]
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        ui::wasm::root(&mut self.state, ctx);
    }
}

fn get_font_definitions() -> FontDefinitions {
    // Use Atkinson Hyperlegible for legibility
    let font_name = "Atkinson-Hyperlegible-Regular".to_string();

    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        font_name.clone(),
        FontData::from_static(include_bytes!(
            "../assets/fonts/Atkinson-Hyperlegible-Regular-102.ttf"
        )),
    );

    // Insert it first, for highest priority
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, font_name);

    // Make all text a bit larger
    for data in fonts.font_data.values_mut() {
        data.tweak.scale *= 1.15;
    }

    fonts
}

fn update_style(mut style: Style) -> Style {
    style.spacing.item_spacing = Vec2::new(16.0, 6.0);
    style
}
