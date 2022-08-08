use super::colors;
use eframe::{epaint::CubicBezierShape, CreationContext, Frame};
use egui::{
    text::{LayoutJob, LayoutSection},
    CentralPanel, Color32, Context, FontData, FontDefinitions, FontFamily, FontId, FontSelection,
    Grid, Layout, Pos2, RichText, ScrollArea, SidePanel, Stroke, Style, TextEdit, TextFormat,
    TextStyle, TopBottomPanel, Vec2,
};
use regex::{Error as CompileError, Regex};
use regex_syntax::ast::{parse::Parser, Ast, Error as AstError, Span};
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

/// Parses and compiles a regular expression, returning the parsed AST and compiled regex.
pub fn compile_regex(pattern: &str) -> anyhow::Result<(Ast, Regex)> {
    Ok((Parser::new().parse(pattern)?, Regex::new(pattern)?))
}

/// Hold information about how a regular expression should be rendered
pub struct RegexLayout {
    /// The layout job describing how to render the regular expression text
    job: LayoutJob,
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

/// Parses the AST of a regular expression and returns information about how it should be rendered
pub fn layout_regex(regex: String, ast: &Ast, style: &Style) -> RegexLayout {
    let font_id = FontSelection::from(TextStyle::Monospace).resolve(style);
    let mut colors_iter = colors::FOREGROUND_COLORS.into_iter().cycle();

    let c = match ast {
        Ast::Concat(c) => c,
        Ast::Group(g) if g.capture_index().is_some() => {
            return RegexLayout::new(
                LayoutJob::single_section(
                    regex,
                    TextFormat::simple(font_id, colors_iter.next().unwrap()),
                ),
                vec![0],
            )
        }
        _ => {
            return RegexLayout::new(
                LayoutJob::single_section(
                    regex,
                    TextFormat::simple(font_id, colors_iter.next().unwrap()),
                ),
                vec![],
            )
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

pub fn input_layouter(
    style: &Style,
    text: String,
    regex: &RegexOutput,
    capture_colors: &[Color32],
    group_section_indexes: &[Option<usize>],
    section_index_map: &mut Vec<Option<usize>>,
) -> LayoutJob {
    let font_id = FontSelection::from(TextStyle::Monospace).resolve(style);

    match regex {
        Ok((_, r)) => {
            let mut new_section_info = Vec::new();
            let mut sections = Vec::new();
            let mut previous_match_end = 0;

            for captures in r.captures_iter(&text) {
                for (m, (color, section)) in captures
                    .iter()
                    .skip(1)
                    .zip(capture_colors)
                    .zip(group_section_indexes)
                    .filter_map(|((m, c), s)| m.zip(Some((c, s))))
                {
                    if previous_match_end < m.start() {
                        new_section_info.push(None);
                        sections.push(LayoutSection {
                            leading_space: 0.0,
                            byte_range: previous_match_end..m.start(),
                            format: TextFormat {
                                font_id: font_id.clone(),
                                ..Default::default()
                            },
                        });
                    }

                    new_section_info.push(*section);
                    sections.push(LayoutSection {
                        leading_space: 0.0,
                        byte_range: m.range(),
                        format: TextFormat::simple(font_id.clone(), *color),
                    });

                    previous_match_end = m.end();
                }
            }

            if previous_match_end < text.len() {
                new_section_info.push(None);
                sections.push(LayoutSection {
                    leading_space: 0.0,
                    byte_range: previous_match_end..text.len(),
                    format: TextFormat {
                        font_id,
                        ..Default::default()
                    },
                });
            }

            *section_index_map = new_section_info;

            LayoutJob {
                text,
                sections,
                ..Default::default()
            }
        }
        Err(_) => LayoutJob::single_section(
            text,
            TextFormat {
                font_id,
                ..Default::default()
            },
        ),
    }
}
