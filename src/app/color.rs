#![allow(dead_code)]

use egui::{Color32, FontId, TextFormat};

pub const FG_BLUE: Color32 = Color32::from_rgb(23, 159, 255);
pub const FG_YELLOW: Color32 = Color32::from_rgb(255, 215, 0);
pub const FG_PINK: Color32 = Color32::from_rgb(218, 112, 214);

pub const FOREGROUND_COLORS: [Color32; 3] = [FG_BLUE, FG_YELLOW, FG_PINK];

pub const BG_BLUE: Color32 = Color32::from_rgb(38, 77, 109);
pub const BG_YELLOW: Color32 = Color32::from_rgb(108, 94, 32);
pub const BG_PINK: Color32 = Color32::from_rgb(97, 63, 97);

pub const BACKGROUND_COLORS: [Color32; 3] = [BG_BLUE, BG_YELLOW, BG_PINK];

pub const FG_RED: Color32 = Color32::RED;
pub const BG_RED: Color32 = Color32::from_rgb(104, 41, 47);

pub trait FromBackgroundExt {
    fn background(font_id: FontId, background: Color32) -> Self;
}

impl FromBackgroundExt for TextFormat {
    /// A parallel to the `TextFormat::simple` function, but for specifying the background color instead of the foreground color
    fn background(font_id: FontId, background: Color32) -> Self {
        Self {
            font_id,
            background,
            color: Color32::WHITE,
            ..Default::default()
        }
    }
}
