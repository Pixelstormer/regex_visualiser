use egui::{Galley, Rect};
use std::ops::Range;

pub enum Bounds<'a> {
    None,
    One(&'a Rect),
    Some(&'a [Rect]),
}

impl<'a> From<&'a [Rect]> for Bounds<'a> {
    fn from(b: &'a [Rect]) -> Self {
        match b {
            [] => Self::None,
            [r] => Self::One(r),
            _ => Self::Some(b),
        }
    }
}

pub enum BoundsMut<'a> {
    None,
    One(&'a mut Rect),
    Some(&'a mut [Rect]),
}

impl<'a> From<&'a mut [Rect]> for BoundsMut<'a> {
    fn from(b: &'a mut [Rect]) -> Self {
        match b {
            [] => Self::None,
            [r] => Self::One(r),
            _ => Self::Some(b),
        }
    }
}

/// Bounding rectangles of all the glyphs of each layout section in a galley
#[derive(Default, Debug)]
pub struct LayoutSectionBounds {
    bounds: Vec<Rect>,
    ranges: Vec<Range<usize>>,
}

impl LayoutSectionBounds {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            bounds: Vec::with_capacity(capacity),
            ranges: Vec::with_capacity(capacity),
        }
    }

    /// Get the bounding rectangles of the layout section of the given index
    pub fn get(&self, index: usize) -> Option<&[Rect]> {
        self.bounds.get(self.ranges.get(index)?.clone())
    }

    /// Mutably get the bounding rectangles of the layout section of the given index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut [Rect]> {
        self.bounds.get_mut(self.ranges.get(index)?.clone())
    }

    /// Get the bounding rectangles of the layout section of the given index
    pub fn get_bounds(&self, index: usize) -> Bounds<'_> {
        match self.get(index) {
            Some(b) => b.into(),
            None => Bounds::None,
        }
    }

    /// Mutably get the bounding rectangles of the layout section of the given index
    pub fn get_bounds_mut(&mut self, index: usize) -> BoundsMut<'_> {
        match self.get_mut(index) {
            Some(b) => b.into(),
            None => BoundsMut::None,
        }
    }

    /// Push a bounding rectangle for the sequentially next layout section
    pub fn push(&mut self, bound: Rect) {
        let l = self.bounds.len();
        self.bounds.push(bound);
        self.ranges.push(l..self.bounds.len());
    }

    /// Extend the last layout section with another bounding rectangle
    pub fn extend(&mut self, bound: Rect) {
        if let Some(r) = self.ranges.last_mut() {
            *r = r.start..r.end + 1;
            self.bounds.push(bound);
        }
    }

    /// Check whether or not we have any bounding rectangles for the layout section of the given index
    pub fn contains(&self, index: usize) -> bool {
        self.get(index).is_some()
    }
}

/// Get the bounding rectangles for all the glyphs of each layout section in the given galley
pub fn get_section_bounds(galley: &Galley) -> LayoutSectionBounds {
    let mut bounds = LayoutSectionBounds::with_capacity(galley.job.sections.len());

    for row in &galley.rows {
        let mut glyphs = row.glyphs.iter();

        if let Some(g) = glyphs.next() {
            let r = g.logical_rect();
            if bounds.contains(g.section_index as usize) {
                bounds.extend(r)
            } else {
                bounds.push(r)
            }
        }

        for glyph in glyphs {
            let r = glyph.logical_rect();
            match bounds.get_mut(glyph.section_index as usize) {
                None => bounds.push(r),
                Some([.., tail]) => *tail = tail.union(r),
                _ => unreachable!(),
            }
        }
    }

    bounds
}
