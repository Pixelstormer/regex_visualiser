use super::{colors, colors::GetColorExt};
use egui::{
    text::{LayoutJob, LayoutSection},
    Color32, FontId, FontSelection, Style, TextFormat, TextStyle,
};
use regex::Regex;
use regex_syntax::ast::{parse::Parser, Ast, Span};
use std::ops::Range;

pub trait GetRangeExt {
    fn range(&self) -> Range<usize>;
}

impl GetRangeExt for Span {
    fn range(&self) -> Range<usize> {
        self.start.offset..self.end.offset
    }
}

/// Parses and compiles a regular expression, returning the parsed AST and compiled regex.
pub fn compile_regex(pattern: &str) -> anyhow::Result<(Ast, Regex)> {
    Ok((Parser::new().parse(pattern)?, Regex::new(pattern)?))
}

/// Information about how a regular expression should be rendered
#[derive(Default)]
pub struct RegexLayout {
    /// The layout job describing how to render the regular expression text
    pub job: LayoutJob,
    /// A mapping from the indexes of capture groups in the regular expression to the indexes of
    /// layout sections in the layout job that correspond to those groups
    capture_group_sections: Vec<usize>,
}

impl RegexLayout {
    pub fn new(job: LayoutJob, mut capture_group_sections: Vec<usize>) -> Self {
        // Capture groups are 1-indexed so the element at index 0 is a placeholder that never gets read
        if !matches!(capture_group_sections.first(), Some(0)) {
            capture_group_sections.insert(0, 0);
        }

        Self {
            job,
            capture_group_sections,
        }
    }

    /// Gets the index of the layout section that corresponds to the capture group of the given index
    pub fn get_section_index_from_group(&self, capture_group_index: usize) -> Option<usize> {
        // Index 0 is an implicit capture group that corresponds to the entire match, never a 'real' capture group
        if capture_group_index == 0 {
            return None;
        }

        self.capture_group_sections
            .get(capture_group_index)
            .copied()
    }

    /// Gets the layout section that corresponds to the capture group of the given index
    pub fn get_section_from_group(&self, capture_group_index: usize) -> Option<&LayoutSection> {
        let section_index = self.get_section_index_from_group(capture_group_index)?;
        self.job.sections.get(section_index)
    }
}

fn regex_simple_layout(
    text: String,
    font_id: FontId,
    color: Color32,
    capture_group_sections: Vec<usize>,
) -> RegexLayout {
    RegexLayout::new(
        LayoutJob::single_section(text, TextFormat::simple(font_id, color)),
        capture_group_sections,
    )
}

/// Parses the AST of a regular expression and returns information about how it should be rendered
pub fn layout_regex(
    regex: String,
    ast: &Ast,
    style: &Style,
    previous_layout: Option<&RegexLayout>,
) -> RegexLayout {
    let font_id = FontSelection::from(TextStyle::Monospace).resolve(style);
    let mut colors_iter = colors::FOREGROUND_COLORS.into_iter().cycle();

    let c = match ast {
        // If the AST is a concatenation of multiple tokens, those tokens need to be parsed more thoroughly
        Ast::Concat(c) => c,
        a => {
            // If the AST is only a single token, the highlighting is simple
            let mut sections = vec![0];
            match a {
                // If the single token is a capture group, it is equivalent to the whole match
                Ast::Group(g) if g.capture_index().is_some() => sections.push(0),
                _ => {}
            }
            return regex_simple_layout(regex, font_id, colors_iter.next().unwrap(), sections);
        }
    };

    // Capture groups are 1-indexed, so occupy index 0 with a placeholder value that will never be read
    let mut capture_group_sections = vec![0];

    // There will be at most one section for each token, but usually less because of
    // chains of consecutive literals being highlighted all together with one section
    let mut sections = Vec::with_capacity(c.asts.len());

    let mut previous_color = Color32::TRANSPARENT;
    let mut literal_start = 0;

    // Iterate over the current and next tokens
    let mut asts_iter = c.asts.iter().peekable();
    while let (Some(ast), peeked) = (asts_iter.next(), asts_iter.peek()) {
        // Get the byte range to be highlighted
        let byte_range = match (ast, peeked) {
            // Skip over consecutive literals as only the offsets of the first and last literals in the chain are needed
            (Ast::Literal(_), Some(Ast::Literal(_))) => continue,
            // At the end of a chain of literals, get the byte range of the whole chain to be highlighted
            (Ast::Literal(l), _) => literal_start..l.span.range().end,
            _ => {
                if let Some(Ast::Literal(l)) = peeked {
                    // Take note of the byte offset of the first literal so that the whole chain can be highlighted later
                    literal_start = l.span.start.offset;
                }

                ast.span().range()
            }
        };

        let mut color = None;

        if let Ast::Group(g) = ast {
            if let Some(i) = g.capture_index() {
                let i = i as usize;

                assert_eq!(
                    capture_group_sections.len(),
                    i,
                    "Regex capture group index is not ordinal"
                );

                // Take note of the section index for this capture group
                capture_group_sections.push(sections.len());

                // Try and retrieve the previous color for this capture group
                color = previous_layout
                    .and_then(|l| l.get_section_from_group(i))
                    .map(|s| s.get_color());
            }
        }

        // Find a color that does not clash with the preceding or following colors
        if color.is_none() {
            if let Some(Ast::Group(g)) = peeked {
                if let Some(i) = g.capture_index() {
                    if let Some(l) = previous_layout {
                        if let Some(s) = l.get_section_from_group(i as usize) {
                            let next_color = s.get_color();
                            color = colors_iter.find(|&c| c != previous_color && c != next_color);
                        }
                    }
                }
            }
        }

        let color = color.unwrap_or_else(|| colors_iter.find(|&c| c != previous_color).unwrap());
        previous_color = color;

        // Push a section to highlight the determined byte range; either a single token or multiple consecutive literals
        sections.push(LayoutSection {
            leading_space: 0.0,
            byte_range,
            format: TextFormat::simple(font_id.clone(), color),
        });
    }

    sections.shrink_to_fit();

    RegexLayout::new(
        LayoutJob {
            text: regex,
            sections,
            ..Default::default()
        },
        capture_group_sections,
    )
}

/// Returns information about how a malformed regular expression string should be rendered
pub fn layout_regex_err(regex: String, style: &Style) -> RegexLayout {
    let font_id = FontSelection::from(TextStyle::Monospace).resolve(style);
    regex_simple_layout(regex, font_id, Color32::RED, vec![0])
}

/// Information about how matched text should be rendered
#[derive(Default)]
pub struct MatchedTextLayout {
    /// The layout job describing how to render the matched text
    pub job: LayoutJob,
    /// A mapping from the indexes of layout sections in the layout job, to the indexes of layout sections in
    /// the layout job of the regular expression text, that correspond to the part of the regular expression text
    /// that matched the corresponding part of the input text, if it exists
    pub layout_section_map: Vec<Option<usize>>,
}

impl MatchedTextLayout {
    pub fn new(job: LayoutJob, layout_section_map: Vec<Option<usize>>) -> Self {
        Self {
            job,
            layout_section_map,
        }
    }
}

/// Matches a regex against some text and returns information about how the matched text should be rendered
pub fn layout_matched_text(
    text: String,
    regex: &Regex,
    style: &Style,
    regex_layout: &RegexLayout,
) -> MatchedTextLayout {
    let font_id = FontSelection::from(TextStyle::Monospace).resolve(style);

    let mut sections = Vec::new();
    let mut layout_section_map = Vec::new();
    let mut previous_match_end = 0;

    // Iterate over all of the capture groups in all of the matches in the given text
    for (m, &s) in regex.captures_iter(&text).flat_map(|c| {
        c.iter()
            .zip(&regex_layout.capture_group_sections) // Get the regex section indexes for each group
            .skip(1) // Skip the first group as it is always the entire match
            .filter_map(|(m, s)| m.zip(Some(s))) // Filter out any groups that didn't participate in the match
            .collect::<Vec<_>>()
    }) {
        // Push a section for the text between the previous and current matches,
        // or on the first iteration, the text between the start of the string and the first match (if any)
        if previous_match_end < m.start() {
            layout_section_map.push(None);
            sections.push(LayoutSection {
                leading_space: 0.0,
                byte_range: previous_match_end..m.start(),
                format: TextFormat {
                    font_id: font_id.clone(),
                    ..Default::default()
                },
            });
        }

        // Push a section for this match
        layout_section_map.push(Some(s));
        sections.push(LayoutSection {
            leading_space: 0.0,
            byte_range: m.range(),
            format: TextFormat::simple(font_id.clone(), regex_layout.job.sections[s].get_color()),
        });

        previous_match_end = m.end();
    }

    // Push a section for the text between the last match and the end of the string, if any
    if previous_match_end < text.len() {
        layout_section_map.push(None);
        sections.push(LayoutSection {
            leading_space: 0.0,
            byte_range: previous_match_end..text.len(),
            format: TextFormat {
                font_id,
                ..Default::default()
            },
        });
    }

    MatchedTextLayout::new(
        LayoutJob {
            text,
            sections,
            ..Default::default()
        },
        layout_section_map,
    )
}

/// Returns information about how plain text should be rendered without any special treatment
pub fn layout_plain_text(text: String, style: &Style) -> MatchedTextLayout {
    MatchedTextLayout::new(
        LayoutJob::single_section(
            text,
            TextFormat {
                font_id: FontSelection::from(TextStyle::Monospace).resolve(style),
                ..Default::default()
            },
        ),
        vec![],
    )
}
