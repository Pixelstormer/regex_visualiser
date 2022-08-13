use egui::{text::LayoutSection, Color32, FontId, TextFormat};

pub const FG_BLUE: Color32 = Color32::from_rgb(23, 159, 255);
pub const FG_YELLOW: Color32 = Color32::from_rgb(255, 215, 0);
pub const FG_PINK: Color32 = Color32::from_rgb(218, 112, 214);

pub const FOREGROUND_COLORS: [Color32; 3] = [FG_BLUE, FG_YELLOW, FG_PINK];

pub const BG_BLUE: Color32 = Color32::from_rgb(19, 122, 191);
pub const BG_YELLOW: Color32 = Color32::from_rgb(191, 156, 0);
pub const BG_PINK: Color32 = Color32::from_rgb(153, 80, 151);

pub const BACKGROUND_COLORS: [Color32; 3] = [BG_BLUE, BG_YELLOW, BG_PINK];

pub trait GetColorExt {
    fn get_color(&self) -> Color32;
}

impl GetColorExt for LayoutSection {
    fn get_color(&self) -> Color32 {
        self.format.background
    }
}

pub trait FromBackgroundExt {
    fn background(font_id: FontId, background: Color32) -> Self;
}

impl FromBackgroundExt for TextFormat {
    fn background(font_id: FontId, background: Color32) -> Self {
        Self {
            font_id,
            background,
            color: Color32::WHITE,
            ..Default::default()
        }
    }
}
