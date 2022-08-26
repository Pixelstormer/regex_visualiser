use eframe::epaint::{text::Glyph, CubicBezierShape};
use egui::{Color32, Galley, Pos2, Rect, Stroke, Vec2};

/// Returns a vec of bounding rects for all of the glyphs in each layout section in the given galley, relative to the galley position
///
/// Layout sections that do not have any corresponding glyphs will be represented by `Rect::NAN`
pub fn galley_section_bounds(galley: &Galley) -> Vec<Rect> {
    // One rect for each layout section
    let mut bounds: Vec<Rect> = Vec::with_capacity(galley.job.sections.len());

    for row in &galley.rows {
        // The `row_section_bounds` method is what actually calculates the bounding rects
        let (index, rects) = match row_section_bounds(&row.glyphs) {
            Some((index, rects)) => (index as usize, rects),
            // Skip empty rows
            None => continue,
        };

        // If there's already a bounding rect for the first layout section in this row (i.e. if the layout section spans multiple rows)
        if let Some(r) = bounds.get_mut(index) {
            // When `row_section_bounds` returns Some the vec is always non-empty, so unwrapping is fine
            let (&first, tail) = rects.split_first().unwrap();

            // Replace the preexisting rect for the layout section with the new one, if the new one is wider
            if first.width() >= r.width() {
                *r = first;
            }

            // Append the remaining rects for the rest of the layout sections in this row
            bounds.extend(tail);
        } else {
            // Else, there isn't a preexisting bounding rect for the first layout section in this row

            // Newlines don't get rendered into glyphs, but do get layed out into layout sections,
            // so layout sections consisting of only newlines won't have any corresponding glyphs,
            // and thus cannot be meaningfully represented by a bounding rect;
            // In these cases, resort to `Rect::NAN` as a kind of placeholder
            bounds.resize(index, Rect::NAN);

            // Append the bounding rects for all of the layout sections in this row
            bounds.extend(rects);
        }
    }

    bounds
}

/// Returns the index of the layout section of the first glyph in the given row,
/// and a vec of bounding rects for all of the glyphs in each layout section in the given row
///
/// Returns None if the given row is empty
pub fn row_section_bounds(row: &[Glyph]) -> Option<(u32, Vec<Rect>)> {
    let first_section_index = row.first()?.section_index;

    let mut bounds: Vec<Rect> = Vec::new();
    for glyph in row {
        let rect = glyph.logical_rect();
        match bounds.get_mut((glyph.section_index - first_section_index) as usize) {
            Some(r) => *r = r.union(rect),
            None => bounds.push(rect),
        }
    }

    Some((first_section_index, bounds))
}

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
