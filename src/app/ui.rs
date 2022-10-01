mod editor;
mod inspector;
mod tab_bar;

/// Functions for displaying UI specific to a native build of the app
#[cfg(not(target_arch = "wasm32"))]
pub mod native;

/// Functions for displaying UI specific to a wasm build of the app
#[cfg(target_arch = "wasm32")]
pub mod wasm;

use egui::{FontData, FontDefinitions, FontFamily, Style, Vec2, Visuals};

/// Toggles between light and dark theme
pub fn toggle_theme(visuals: &Visuals) -> Visuals {
    if visuals.dark_mode {
        Visuals::light()
    } else {
        Visuals::dark()
    }
}

pub fn update_style(mut style: Style) -> Style {
    style.spacing.item_spacing = Vec2::new(16.0, 6.0);
    style
}

pub fn create_font_definitions() -> FontDefinitions {
    // Use Atkinson Hyperlegible for legibility
    let font_name = "Atkinson-Hyperlegible-Regular".to_string();

    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        font_name.clone(),
        FontData::from_static(include_bytes!(
            "../../assets/fonts/Atkinson-Hyperlegible-Regular-102.ttf"
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
