use super::{
    color,
    color::FromBackgroundExt,
    parsing::{ast_find_capture_groups, RegexError},
};
use eframe::epaint::text::Row;
use egui::{
    text::{LayoutJob, LayoutSection},
    Color32, FontId, FontSelection, Rect, Style, TextFormat, TextStyle,
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
    let ranges = ast_find_capture_groups(ast);
    let mut sections = Vec::with_capacity(ranges.len());

    let font_id = FontSelection::from(TextStyle::Monospace).resolve(style);

    let max_depth = ranges
        .iter()
        .map(|(depth, _)| *depth)
        .max()
        .unwrap_or_default();

    // Convert the byte ranges into char ranges, to later be used to index into the glyphs of the layed out galley
    let capture_group_chars = ranges
        .iter()
        .cloned()
        .map(|(depth, range)| {
            (
                (0..=max_depth).nth_back(depth).unwrap(),
                convert_byte_range_to_char_range(range, &regex).unwrap(),
            )
        })
        .collect();

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

    // Mark each byte of the regular expression with the index of the capture group it corresponds to
    let mut section_indexes = vec![0; regex.len()];
    for (index, (_, range)) in ranges.into_iter().enumerate() {
        section_indexes[range].fill(index + 1);
    }

    // Derived from the `Slice::group_by` method;
    // Find consecutive runs of bytes with equal marked capture groups, and create a layout section for each run
    let mut head = 0;
    let mut len = 1;
    let mut iter = section_indexes.windows(2);
    while let Some(&[left, right]) = iter.next() {
        if left != right {
            sections.push(LayoutSection {
                leading_space: 0.0,
                byte_range: head..len,
                format: TextFormat::background(font_id.clone(), capture_group_colors[left]),
            });

            head = len;
        }
        len += 1;
    }

    sections.push(LayoutSection {
        leading_space: 0.0,
        byte_range: head..len,
        format: TextFormat::background(font_id, capture_group_colors[section_indexes[head]]),
    });

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
    let font_id = FontSelection::from(TextStyle::Monospace).resolve(style);

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
    pub job: LayoutJob,
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
            job: layout_plain_text(text, style),
            capture_group_chars: vec![],
        };
    }

    let mut capture_group_chars = Vec::new();
    let mut section_indexes = vec![0; text.len()];

    for captures in regex.captures_iter(&text) {
        // Get the spans of the matched text from each capture group
        let ranges = captures
            .iter()
            .skip(1) // The first (0th) capture group always corresponds to the entire match, not any 'real' capture groups
            .map(|r#match| r#match.map(|r#match| r#match.range()))
            .collect::<Vec<_>>();

        // Mark each byte of the text with the index of the capture group it corresponds to
        for (index, range) in ranges
            .iter()
            .cloned()
            .enumerate()
            .filter_map(|(index, range)| Some(index).zip(range))
        {
            section_indexes[range].fill(index + 1);
        }

        // Convert the byte ranges into char ranges, to later be used to index into the glyphs of the layed out galley
        let char_ranges = ranges
            .into_iter()
            .map(|range| range.map(|range| convert_byte_range_to_char_range(range, &text).unwrap()))
            .collect();

        capture_group_chars.push(char_ranges);
    }

    let font_id = FontSelection::from(TextStyle::Monospace).resolve(style);
    let mut sections = Vec::with_capacity(capture_group_chars.len() * regex.captures_len());

    // Derived from the `Slice::group_by` method;
    // Find consecutive runs of bytes with equal marked capture groups, and create a layout section for each run
    let mut head = 0;
    let mut len = 1;
    let mut iter = section_indexes.windows(2);
    while let Some(&[left, right]) = iter.next() {
        if left != right {
            sections.push(LayoutSection {
                leading_space: 0.0,
                byte_range: head..len,
                format: TextFormat::background(font_id.clone(), capture_group_colors[left]),
            });

            head = len;
        }
        len += 1;
    }

    sections.push(LayoutSection {
        leading_space: 0.0,
        byte_range: head..len,
        format: TextFormat::background(font_id, capture_group_colors[section_indexes[head]]),
    });

    MatchedTextLayout {
        job: LayoutJob {
            text,
            sections,
            ..Default::default()
        },
        capture_group_chars,
    }
}

/// Returns information about how plain text should be rendered
pub fn layout_plain_text(text: String, style: &Style) -> LayoutJob {
    LayoutJob::single_section(
        text,
        TextFormat {
            font_id: FontSelection::from(TextStyle::Monospace).resolve(style),
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
