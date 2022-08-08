use super::colors;
use eframe::{epaint::CubicBezierShape, CreationContext, Frame};
use egui::{
    text::{LayoutJob, LayoutSection},
    CentralPanel, Color32, Context, FontData, FontDefinitions, FontFamily, FontSelection, Grid,
    Layout, Pos2, RichText, ScrollArea, SidePanel, Stroke, Style, TextEdit, TextFormat, TextStyle,
    TopBottomPanel, Vec2,
};
use regex::{Error as CompileError, Regex};
use regex_syntax::ast::{Ast, Error as AstError, Span};
use std::{
    fmt::{Display, Formatter},
    ops::Range,
};

#[derive(Clone, Debug)]
pub enum RegexError {
    Ast(AstError),
    Compiled(CompileError),
}

impl Display for RegexError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RegexError::Ast(e) => e.fmt(f),
            RegexError::Compiled(e) => e.fmt(f),
        }
    }
}

pub type RegexOutput = Result<(Ast, Regex), RegexError>;

pub trait GetRangeExt {
    fn range(&self) -> Range<usize>;
}

impl GetRangeExt for Span {
    fn range(&self) -> Range<usize> {
        self.start.offset..self.end.offset
    }
}

pub fn compile_regex(pattern: &str) -> RegexOutput {
    let ast = match regex_syntax::ast::parse::Parser::new().parse(pattern) {
        Ok(ast) => ast,
        Err(e) => return Err(RegexError::Ast(e)),
    };

    let compiled = match Regex::new(pattern) {
        Ok(regex) => regex,
        Err(e) => return Err(RegexError::Compiled(e)),
    };

    Ok((ast, compiled))
}

pub fn regex_layouter(
    style: &Style,
    text: String,
    regex: &RegexOutput,
    group_info: &mut Vec<Color32>,
    group_section_indexes: &mut Vec<Option<usize>>,
) -> LayoutJob {
    let font_id = FontSelection::from(TextStyle::Monospace).resolve(style);

    match regex {
        Ok((ast, _)) => {
            if let Ast::Concat(c) = ast {
                let mut sections = Vec::with_capacity(c.asts.len());
                let mut asts_iter = c.asts.iter().peekable();
                let mut colors_iter = colors::FOREGROUND_COLORS.into_iter().cycle().peekable();
                group_info.clear();
                group_section_indexes.clear();

                let mut literal_start = 0;
                while let (Some(ast), peeked) = (asts_iter.next(), asts_iter.peek()) {
                    let range = match (ast, peeked) {
                        (Ast::Literal(_), Some(Ast::Literal(_))) => continue,
                        (Ast::Literal(l), _) => Some(literal_start..l.span.range().end),
                        (Ast::Group(g), _) => {
                            if g.capture_index().is_some() {
                                group_info.push(*colors_iter.peek().unwrap());
                                group_section_indexes.push(Some(sections.len()));
                            }
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
                group_info.shrink_to_fit();
                group_section_indexes.shrink_to_fit();

                LayoutJob {
                    text,
                    sections,
                    ..Default::default()
                }
            } else {
                LayoutJob::single_section(
                    text,
                    TextFormat::simple(font_id, colors::FOREGROUND_COLORS[0]),
                )
            }
        }
        Err(_) => LayoutJob::single_section(text, TextFormat::simple(font_id, Color32::RED)),
    }
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
