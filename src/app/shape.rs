use eframe::epaint::CubicBezierShape;
use egui::{Color32, Pos2, Stroke, Vec2};

#[derive(Clone, Copy)]
pub enum Orientation {
    #[allow(dead_code)]
    Horizontal,
    Vertical,
}

/// Returns a bezier curve that connects the given points
pub fn curve_between(
    from: Pos2,
    to: Pos2,
    stroke: impl Into<Stroke>,
    orientation: Orientation,
) -> CubicBezierShape {
    let control_offset = match orientation {
        Orientation::Horizontal => Vec2::X * ((to.x - from.x) / 2.0),
        Orientation::Vertical => Vec2::Y * ((to.y - from.y) / 2.0),
    };

    let from_control = from + control_offset;
    let to_control = to - control_offset;

    CubicBezierShape::from_points_stroke(
        [from, from_control, to_control, to],
        false,
        Color32::TRANSPARENT,
        stroke,
    )
}
