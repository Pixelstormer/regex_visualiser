use super::{
    color,
    color::FromBackgroundExt,
    parsing::{ast_find_capture_groups, RegexError},
};
use eframe::epaint::text::Row;
use egui::{
    text::{LayoutJob, LayoutSection},
    Color32, FontId, Rect, Style, TextFormat, TextStyle,
};
use regex::Regex;
use regex_syntax::ast::{Ast, Span};
use std::ops::{ControlFlow, Range};

pub trait GetRangeExt {
    fn range(&self) -> Range<usize>;
}

impl GetRangeExt for Span {
    fn range(&self) -> Range<usize> {
        self.start.offset..self.end.offset
    }
}

fn convert_byte_range_to_char_range(range: Range<usize>, text: &str) -> Option<Range<usize>> {
    let head = text.get(0..range.start)?;
    let tail = text.get(range)?;
    let head_offset = str_glyph_count(head);
    let tail_offset = head_offset + str_glyph_count(tail);
    Some(head_offset..tail_offset)
}

/// Counts the number of chars in the given string, excluding newlines (`\n`),
/// as egui excludes those when laying out text into glyphs
fn str_glyph_count(text: &str) -> usize {
    text.chars().count() - text.matches('\n').count()
}

#[derive(Default, Clone)]
pub struct TextLayoutJob {
    text: String,
    mapping: Vec<usize>,
    formats: Vec<TextFormat>,
}

impl TextLayoutJob {
    pub fn new(text: String, mapping: Vec<usize>, formats: Vec<TextFormat>) -> Self {
        Self {
            text,
            mapping,
            formats,
        }
    }

    pub fn substring(&self, range: Range<usize>) -> Self {
        Self {
            text: self.text[range.clone()].into(),
            mapping: self.mapping[range].into(),
            formats: self.formats.clone(),
        }
    }

    pub fn replace(&mut self, from: u8, to: &str) {
        let from: char = from.into();

        let mut offset = 0;
        for (index, _) in self.text.match_indices(from) {
            let index = index + offset;
            self.mapping.splice(
                index..=index,
                std::iter::repeat(self.mapping[index]).take(to.len()),
            );
            offset += to.len() - 1;
        }

        self.text = self.text.replace(from, to);
    }

    pub fn replace_format(&mut self, pattern: char, format: TextFormat) {
        let new_index = self.formats.len();
        self.formats.push(format);
        for (index, _) in self.text.match_indices(pattern) {
            self.mapping[index] = new_index;
        }
    }

    pub fn convert_to_layout_job(self) -> LayoutJob {
        let sections = self.build_layout_sections();
        LayoutJob {
            text: self.text,
            sections,
            ..Default::default()
        }
    }

    fn build_layout_sections(&self) -> Vec<LayoutSection> {
        // Empty strings have no layout sections
        if self.text.is_empty() {
            return Default::default();
        }

        // This is a lower bound for how many sections there will be, assuming that each TextFormat will be used at least once
        let mut sections = Vec::with_capacity(self.formats.len());

        // Derived from the `Slice::group_by` method;
        // Find consecutive runs of bytes with equal marked indexes, and create a layout section for each run
        let mut head = 0;
        let mut len = 1;
        let mut iter = self.mapping.windows(2);
        while let Some(&[left, right]) = iter.next() {
            if left != right {
                sections.push(LayoutSection {
                    leading_space: 0.0,
                    byte_range: head..len,
                    format: self.formats[left].clone(),
                });

                head = len;
            }
            len += 1;
        }

        let i = self.mapping[head];
        sections.push(LayoutSection {
            leading_space: 0.0,
            byte_range: head..len,
            format: self.formats[i].clone(),
        });

        sections
    }
}

/// Information about how a regular expression should be rendered
#[derive(Default)]
pub struct RegexLayout {
    /// The layout job describing how to render the regular expression text
    pub job: LayoutJob,
    /// A mapping from capture groups in the regex to ranges of chars in the regular expression text that
    /// correspond to those capture groups, as well as the depth of the capture group in the regex ast
    pub capture_group_chars: Vec<(usize, Range<usize>)>,
    /// The colors used to highlight each capture group in the regex
    pub capture_group_colors: Vec<Color32>,
}

pub fn regex_parse_ast(
    regex: String,
    ast: &Ast,
    style: &Style,
    _previous_layout: Option<&RegexLayout>,
) -> RegexLayout {
    if regex.is_empty() {
        return Default::default();
    }

    // Find the spans of each of the capture groups in the regular expression
    let (depths, ranges) = ast_find_capture_groups(ast);

    // Calculate the color that each capture group will have
    // Capture groups are 1-indexed, so prepend a placeholder color for the 0th index
    let capture_group_colors = std::iter::once(Color32::TRANSPARENT)
        .chain(
            color::BACKGROUND_COLORS
                .into_iter()
                .cycle()
                .take(ranges.len()),
        )
        .collect::<Vec<_>>();

    let sections = build_layout_sections(
        &mut vec![0; regex.len()],
        ranges.iter().cloned().enumerate(),
        TextStyle::Monospace.resolve(style),
        &capture_group_colors,
    );

    let max_depth = *depths.iter().max().unwrap_or(&0);

    // Convert the byte ranges into char ranges, to later be used to index into the glyphs of the layed out galley
    let capture_group_chars = depths
        .into_iter()
        .zip(ranges)
        .map(|(depth, range)| {
            (
                // Invert the depth value, as it will eventually be used as the thickness of the connecting line,
                // so shallower lines should be thicker than deeper lines that may be rendered ontop of them
                (0..=max_depth).nth_back(depth).unwrap(),
                convert_byte_range_to_char_range(range, &regex).unwrap(),
            )
        })
        .collect();

    RegexLayout {
        job: LayoutJob {
            text: regex,
            sections,
            ..Default::default()
        },
        capture_group_chars,
        capture_group_colors,
    }
}

/// Returns information about how a malformed regular expression string should be rendered
pub fn layout_regex_err(regex: String, style: &Style, err: &RegexError) -> RegexLayout {
    let (span, aux) = match err {
        RegexError::Parse(e) => (Some(e.span()), e.auxiliary_span()),
        RegexError::Compile(_) => (None, None),
    };

    fn plaintext(byte_range: Range<usize>, font_id: FontId) -> LayoutSection {
        LayoutSection {
            leading_space: 0.0,
            byte_range,
            format: TextFormat::simple(font_id, color::FG_RED),
        }
    }

    fn highlight(byte_range: Range<usize>, font_id: FontId) -> LayoutSection {
        LayoutSection {
            leading_space: 0.0,
            byte_range,
            format: TextFormat {
                font_id,
                color: Color32::WHITE,
                background: color::BG_RED,
                ..Default::default()
            },
        }
    }

    let font_id = TextStyle::Monospace.resolve(style);

    let sections = match (span, aux) {
        (None, _) => vec![highlight(0..regex.len(), font_id)],
        (Some(span), None) => vec![
            plaintext(0..span.start.offset, font_id.clone()),
            highlight(span.range(), font_id.clone()),
            plaintext(span.end.offset..regex.len(), font_id),
        ],
        (Some(span), Some(aux)) => {
            let min = span.min(aux);
            let max = span.max(aux);

            vec![
                plaintext(0..min.start.offset, font_id.clone()),
                highlight(min.range(), font_id.clone()),
                plaintext(min.end.offset..max.start.offset, font_id.clone()),
                highlight(max.range(), font_id.clone()),
                plaintext(max.end.offset..regex.len(), font_id),
            ]
        }
    };

    RegexLayout {
        job: LayoutJob {
            text: regex,
            sections,
            ..Default::default()
        },
        capture_group_chars: vec![],
        capture_group_colors: vec![],
    }
}

/// Information about how text that was matched against a regex should be rendered
#[derive(Default)]
pub struct MatchedTextLayout {
    /// The layout job describing how to render the matched text
    pub job: TextLayoutJob,
    /// A vec of mappings from the indexes of capture groups in the regex to the parts of the text that were
    /// matched by that capture group, with one mapping for each overall match in the text
    pub capture_group_chars: Vec<Vec<Option<Range<usize>>>>,
}

pub fn layout_matched_text(
    text: String,
    regex: &Regex,
    style: &Style,
    capture_group_colors: &[Color32],
) -> MatchedTextLayout {
    if text.is_empty() {
        return Default::default();
    }

    if regex.as_str().is_empty() {
        return MatchedTextLayout {
            job: layout_plain_text_job(text, style),
            capture_group_chars: vec![],
        };
    }

    let mut capture_group_chars = Vec::new();
    let mut ranges = Vec::new();

    for captures in regex.captures_iter(&text) {
        // Convert the byte ranges into char ranges, to later be used to index into the glyphs of the layed out galley
        let char_ranges = captures
            .iter()
            .skip(1) // The first (0th) capture group always corresponds to the entire match, not any 'real' capture groups
            .map(|r#match| {
                r#match.map(|r#match| {
                    convert_byte_range_to_char_range(r#match.range(), &text).unwrap()
                })
            })
            .collect();

        capture_group_chars.push(char_ranges);

        // Get the spans of the matched text from each capture group
        let iter = captures
            .iter()
            .enumerate()
            .skip(1) // The first (0th) capture group always corresponds to the entire match, not any 'real' capture groups
            .filter_map(|(index, r#match)| r#match.map(|r#match| (index, r#match.range())));

        ranges.extend(iter);
    }

    let mut section_indexes = vec![0; text.len()];
    for (index, range) in ranges {
        section_indexes[range].fill(index);
    }

    let font_id = TextStyle::Monospace.resolve(style);

    MatchedTextLayout {
        job: TextLayoutJob::new(
            text,
            section_indexes,
            capture_group_colors
                .iter()
                .map(|&color| TextFormat::background(font_id.clone(), color))
                .collect(),
        ),
        capture_group_chars,
    }
}

pub fn layout_plain_text_job(text: String, style: &Style) -> TextLayoutJob {
    let len = text.len();
    TextLayoutJob::new(
        text,
        vec![0; len],
        vec![TextFormat {
            font_id: TextStyle::Monospace.resolve(style),
            ..Default::default()
        }],
    )
}

/// Returns information about how plain text should be rendered
pub fn layout_plain_text(text: String, style: &Style) -> LayoutJob {
    LayoutJob::single_section(
        text,
        TextFormat {
            font_id: TextStyle::Monospace.resolve(style),
            ..Default::default()
        },
    )
}

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
        ControlFlow::Break(result) => result,
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

            Some(row.glyphs[head].iter().fold(Rect::NOTHING, |rect, glyph| {
                rect.union(glyph.logical_rect())
            }))
        })
        // Choose the widest rect out of those that this range produced
        .max_by(|x, y| x.width().partial_cmp(&y.width()).unwrap())
}

/// Builds a vec of layout sections from the given iterator of ranges
fn build_layout_sections(
    section_indexes: &mut [usize],
    ranges: impl ExactSizeIterator<Item = (usize, Range<usize>)>,
    font_id: FontId,
    colors: &[Color32],
) -> Vec<LayoutSection> {
    // This is a lower bound for how many sections there will be, as each range will have at least 1 section,
    // but gaps between ranges or ranges that overlap will result in multiple additional sections
    // Technically there can be less sections than this if some ranges are entirely 'covered' by other ranges,
    // but that is very unlikely, if not impossible, due to how regular expressions are structured
    let mut sections = Vec::with_capacity(ranges.len());

    // Mark each byte of the string with the index it corresponds to
    for (index, range) in ranges {
        section_indexes[range].fill(index + 1);
    }

    // Derived from the `Slice::group_by` method;
    // Find consecutive runs of bytes with equal marked indexes, and create a layout section for each run
    let mut head = 0;
    let mut len = 1;
    let mut iter = section_indexes.windows(2);
    while let Some(&[left, right]) = iter.next() {
        if left != right {
            sections.push(LayoutSection {
                leading_space: 0.0,
                byte_range: head..len,
                format: TextFormat::background(font_id.clone(), colors[left]),
            });

            head = len;
        }
        len += 1;
    }

    let i = section_indexes[head];
    sections.push(LayoutSection {
        leading_space: 0.0,
        byte_range: head..len,
        format: TextFormat::background(font_id, colors[i]),
    });

    sections
}
