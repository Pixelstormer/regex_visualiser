use eframe::epaint::{text::Row, CubicBezierShape};
use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use std::ops::{ControlFlow, Range};

/// Returns a bounding rect equal to the union of the bounding rects of all of the glyphs in the given rows that
/// are delimited by the given range
///
/// Returns None if the range is entirely out of the bounds of the rows - if the range is only partially out of bounds,
/// it will be truncated to the part that is in bounds
pub fn glyph_bounds(rows: &[Row], range: &Range<usize>) -> Option<Rect> {
    let mut iter = rows.iter();
    let (mut offset, first_row) = match iter.try_fold(0, |offset, row| {
        // Skip to the row that the range starts in
        let new_offset = offset + row.glyphs.len();
        if range.start >= new_offset {
            ControlFlow::Continue(new_offset)
        } else {
            ControlFlow::Break((offset, row))
        }
    }) {
        // If `try_fold` returns `ControlFlow::Continue` that means the entire iterator was exhausted,
        // or in other words the range is out of the bounds of all of the rows
        ControlFlow::Continue(_) => return None,
        ControlFlow::Break(r) => r,
    };

    let mut tail_start = range.start;

    // Manually prepend the first row, as `map_while` would otherwise not see it,
    // because `try_fold` consumes (from the iterator) every element it visits,
    // including the one on which `ControlFlow::Break` is returned, which `first_row` is
    std::iter::once(first_row)
        .chain(iter)
        .map_while(|row| {
            if tail_start >= range.end {
                // Stop iterating once the entire range has been exhausted
                return None;
            }

            let row_end = offset + row.glyphs.len();

            // Split off the part of the range that corresponds to this row,
            // leaving the rest of the range that corresponds to the following rows
            let head = tail_start - offset..range.end.min(row_end) - offset;
            tail_start = row_end;

            // Update the offset for the next row
            offset = row_end;

            Some(
                row.glyphs[head]
                    .iter()
                    .fold(Rect::NOTHING, |a, g| a.union(g.logical_rect())),
            )
        })
        // Choose the widest rect out of those that this range produced
        .max_by(|a, b| a.width().partial_cmp(&b.width()).unwrap())
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
