use egui::{text::LayoutSection, Color32};

pub const FG_YELLOW: Color32 = Color32::from_rgb(255, 215, 0);
pub const FG_PINK: Color32 = Color32::from_rgb(218, 112, 214);
pub const FG_BLUE: Color32 = Color32::from_rgb(23, 159, 255);

pub const FOREGROUND_COLORS: [Color32; 3] = [FG_BLUE, FG_YELLOW, FG_PINK];

pub trait GetColorExt {
    fn get_color(&self) -> Color32;
}

impl GetColorExt for LayoutSection {
    fn get_color(&self) -> Color32 {
        self.format.color
    }
}
