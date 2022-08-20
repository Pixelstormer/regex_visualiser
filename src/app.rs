mod colors;
mod layout;
mod parsing;
mod state;
mod ui;

use self::state::AppState;
use eframe::{CreationContext, Frame};
use egui::{Context, FontData, FontDefinitions, FontFamily};

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
        cc.egui_ctx.set_fonts(get_font_definitions());

        // Load previous app state (if any).
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for Application {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        ui::root(&mut self.state, ctx, frame);
    }
}

pub fn get_font_definitions() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();

    // Use Atkinson Hyperlegible for legibility
    fonts.font_data.insert(
        "Atkinson-Hyperlegible-Regular".to_owned(),
        FontData::from_static(include_bytes!(
            "../assets/fonts/Atkinson-Hyperlegible-Regular-102.ttf"
        )),
    );

    // Insert it first, for highest priority
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "Atkinson-Hyperlegible-Regular".to_owned());

    // Make all text a bit larger
    for data in fonts.font_data.values_mut() {
        data.tweak.scale *= 1.15;
    }

    fonts
}
