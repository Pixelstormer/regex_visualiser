use super::text::GetRangeExt;
use regex::Regex;
use regex_syntax::ast::{parse::Parser, Ast};
use std::{
    fmt::{Display, Formatter},
    ops::Range,
};

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

pub fn ast_find_capture_groups(ast: &Ast) -> Vec<(usize, Range<usize>)> {
    let mut vec = Vec::new();
    ast_find_capture_groups_recurse(0, ast, &mut vec);
    vec
}

pub fn ast_find_capture_groups_recurse(
    depth: usize,
    ast: &Ast,
    vec: &mut Vec<(usize, Range<usize>)>,
) {
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
