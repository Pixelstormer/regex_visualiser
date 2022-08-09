use super::colors;
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
    pub fn new(job: LayoutJob, capture_group_sections: Vec<usize>) -> Self {
        Self {
            job,
            capture_group_sections,
        }
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
pub fn layout_regex(regex: String, ast: &Ast, style: &Style) -> RegexLayout {
    let font_id = FontSelection::from(TextStyle::Monospace).resolve(style);
    let mut colors_iter = colors::FOREGROUND_COLORS.into_iter().cycle();

    let c = match ast {
        Ast::Concat(c) => c,
        a => {
            let mut sections = Vec::new();
            match a {
                Ast::Group(g) if g.capture_index().is_some() => sections.push(0),
                _ => {}
            }
            return regex_simple_layout(regex, font_id, colors_iter.next().unwrap(), sections);
        }
    };

    let mut asts_iter = c.asts.iter().peekable();
    let mut sections = Vec::with_capacity(c.asts.len());
    let mut capture_group_sections = Vec::new();

    let mut literal_start = 0;
    while let (Some(ast), peeked) = (asts_iter.next(), asts_iter.peek()) {
        let range = match (ast, peeked) {
            (Ast::Literal(_), Some(Ast::Literal(_))) => continue,
            (Ast::Literal(l), _) => Some(literal_start..l.span.range().end),
            (Ast::Group(g), _) if g.capture_index().is_some() => {
                capture_group_sections.push(sections.len());
                None
            }
            _ => None,
        };

        if let (None, Some(Ast::Literal(l))) = (&range, peeked) {
            literal_start = l.span.start.offset;
        }

        sections.push(LayoutSection {
            leading_space: 0.0,
            byte_range: range.unwrap_or_else(|| ast.span().range()),
            format: TextFormat::simple(font_id.clone(), colors_iter.next().unwrap()),
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
    regex_simple_layout(regex, font_id, Color32::RED, vec![])
}

/// Information about how matched text should be rendered
#[derive(Default)]
pub struct MatchedTextLayout {
    /// The layout job describing how to render the matched text
    pub job: LayoutJob,
    /// A mapping from the indexes of layout sections in the layout job, to the indexes of layout sections in
    /// the layout job of the regular expression text, that correspond to the part of the regular expression text
    /// that matched the corresponding part of the input text
    pub capture_group_sections: Vec<Option<usize>>,
}

impl MatchedTextLayout {
    pub fn new(job: LayoutJob, capture_group_sections: Vec<Option<usize>>) -> Self {
        Self {
            job,
            capture_group_sections,
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
    let mut capture_group_sections = Vec::new();
    let mut previous_match_end = 0;

    // Iterate over all of the capture groups in all of the matches in the given text
    for (m, &s) in regex.captures_iter(&text).flat_map(|c| {
        c.iter()
            .skip(1) // Skip the first group as it is always the entire match
            .zip(&regex_layout.capture_group_sections) // Get the regex section indexes for each group
            .filter_map(|(m, s)| m.zip(Some(s))) // Filter out any groups that didn't participate in the match
            .collect::<Vec<_>>()
    }) {
        // Push a section for the text between the previous and current matches,
        // or on the first iteration, the text between the start of the string and the first match (if any)
        if previous_match_end < m.start() {
            capture_group_sections.push(None);
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
        capture_group_sections.push(Some(s));
        sections.push(LayoutSection {
            leading_space: 0.0,
            byte_range: m.range(),
            format: TextFormat::simple(font_id.clone(), regex_layout.job.sections[s].format.color),
        });

        previous_match_end = m.end();
    }

    // Push a section for the text between the last match and the end of the string, if any
    if previous_match_end < text.len() {
        capture_group_sections.push(None);
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
        capture_group_sections,
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
