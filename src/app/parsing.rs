use super::{colors, colors::FromBackgroundExt};
use egui::{
    text::{LayoutJob, LayoutSection},
    Color32, FontId, FontSelection, Style, TextFormat, TextStyle,
};
use regex::Regex;
use regex_syntax::ast::{parse::Parser, Ast, Span};
use std::{
    fmt::{Display, Formatter},
    ops::Range,
};

pub trait GetRangeExt {
    fn range(&self) -> Range<usize>;
}

impl GetRangeExt for Span {
    fn range(&self) -> Range<usize> {
        self.start.offset..self.end.offset
    }
}

#[derive(Debug)]
pub enum RegexError {
    Parse(regex_syntax::ast::Error),
    Compile(regex::Error),
}

impl From<regex_syntax::ast::Error> for RegexError {
    fn from(e: regex_syntax::ast::Error) -> Self {
        Self::Parse(e)
    }
}

impl From<regex::Error> for RegexError {
    fn from(e: regex::Error) -> Self {
        Self::Compile(e)
    }
}

impl Display for RegexError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RegexError::Parse(e) => e.fmt(f),
            RegexError::Compile(e) => e.fmt(f),
        }
    }
}

/// Parses and compiles a regular expression, returning the parsed AST and compiled regex.
pub fn compile_regex(pattern: &str) -> Result<(Ast, Regex), RegexError> {
    Ok((Parser::new().parse(pattern)?, Regex::new(pattern)?))
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
    previous_layout: Option<&RegexLayout>,
) -> RegexLayout {
    if regex.is_empty() {
        return Default::default();
    }

    // Find the spans of each of the capture groups in the regular expression
    let ranges = ast_find_capture_groups(ast);
    let mut sections = Vec::with_capacity(ranges.len());

    let font_id = FontSelection::from(TextStyle::Monospace).resolve(style);

    let max_depth = ranges.iter().map(|(d, _)| *d).max().unwrap_or_default();

    // Convert the byte ranges into char ranges, to later be used to index into the glyphs of the layed out galley
    let capture_group_chars = ranges
        .iter()
        .cloned()
        .map(|(d, r)| {
            (
                (0..=max_depth).nth_back(d).unwrap(),
                convert_byte_range_to_char_range(r, &regex).unwrap(),
            )
        })
        .collect();

    // Calculate the color that each capture group will have
    // Capture groups are 1-indexed, so prepend a placeholder color for the 0th index
    let capture_group_colors = std::iter::once(Color32::TRANSPARENT)
        .chain(
            colors::BACKGROUND_COLORS
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
    while let Some(&[l, r]) = iter.next() {
        if l != r {
            sections.push(LayoutSection {
                leading_space: 0.0,
                byte_range: head..len,
                format: TextFormat::background(font_id.clone(), capture_group_colors[l]),
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

fn ast_find_capture_groups(ast: &Ast) -> Vec<(usize, Range<usize>)> {
    let mut vec = Vec::new();
    ast_find_capture_groups_recurse(0, ast, &mut vec);
    vec
}

fn ast_find_capture_groups_recurse(depth: usize, ast: &Ast, vec: &mut Vec<(usize, Range<usize>)>) {
    match ast {
        Ast::Repetition(r) => ast_find_capture_groups_recurse(depth + 1, &r.ast, vec),
        Ast::Group(g) => {
            if let Some(i) = g.capture_index() {
                assert_eq!(
                    vec.len() + 1,
                    i as usize,
                    "Regex capture group indexes are not consecutive (Expected: {}, Got: {})",
                    vec.len() + 1,
                    i
                );

                vec.push((depth, g.span.range()));
                ast_find_capture_groups_recurse(depth + 1, &g.ast, vec);
            }
        }
        Ast::Alternation(a) => {
            for ast in &a.asts {
                ast_find_capture_groups_recurse(depth + 1, ast, vec);
            }
        }
        Ast::Concat(c) => {
            for ast in &c.asts {
                ast_find_capture_groups_recurse(depth + 1, ast, vec);
            }
        }
        _ => {}
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
fn str_glyph_count(s: &str) -> usize {
    s.chars().count() - s.matches('\n').count()
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
            format: TextFormat::simple(font_id, colors::FG_RED),
        }
    }

    fn highlight(byte_range: Range<usize>, font_id: FontId) -> LayoutSection {
        LayoutSection {
            leading_space: 0.0,
            byte_range,
            format: TextFormat {
                font_id,
                color: Color32::WHITE,
                background: colors::BG_RED,
                ..Default::default()
            },
        }
    }

    let sections = match (span, aux) {
        (None, _) => vec![highlight(0..regex.len(), font_id)],
        (Some(e), None) => vec![
            plaintext(0..e.start.offset, font_id.clone()),
            highlight(e.range(), font_id.clone()),
            plaintext(e.end.offset..regex.len(), font_id),
        ],
        (Some(e), Some(a)) => {
            let min = e.min(a);
            let max = e.max(a);

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
            .map(|m| m.map(|m| m.range()))
            .collect::<Vec<_>>();

        // Mark each byte of the text with the index of the capture group it corresponds to
        for (index, range) in ranges
            .iter()
            .cloned()
            .enumerate()
            .filter_map(|(i, r)| Some(i).zip(r))
        {
            section_indexes[range].fill(index + 1);
        }

        // Convert the byte ranges into char ranges, to later be used to index into the glyphs of the layed out galley
        let char_ranges = ranges
            .into_iter()
            .map(|r| r.map(|r| convert_byte_range_to_char_range(r, &text).unwrap()))
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
    while let Some(&[l, r]) = iter.next() {
        if l != r {
            sections.push(LayoutSection {
                leading_space: 0.0,
                byte_range: head..len,
                format: TextFormat::background(font_id.clone(), capture_group_colors[l]),
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
