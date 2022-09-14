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
    fn from(err: regex_syntax::ast::Error) -> Self {
        Self::Parse(err)
    }
}

impl From<regex::Error> for RegexError {
    fn from(err: regex::Error) -> Self {
        Self::Compile(err)
    }
}

impl Display for RegexError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RegexError::Parse(err) => err.fmt(fmt),
            RegexError::Compile(err) => err.fmt(fmt),
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
        Ast::Repetition(repetiton) => {
            ast_find_capture_groups_recurse(depth + 1, &repetiton.ast, vec)
        }
        Ast::Group(group) => {
            if let Some(index) = group.capture_index() {
                assert_eq!(
                    vec.len() + 1,
                    index as usize,
                    "Regex capture group indexes are not consecutive (Expected: {}, Got: {})",
                    vec.len() + 1,
                    index
                );

                vec.push((depth, group.span.range()));
                ast_find_capture_groups_recurse(depth + 1, &group.ast, vec);
            }
        }
        Ast::Alternation(alternation) => {
            for ast in &alternation.asts {
                ast_find_capture_groups_recurse(depth + 1, ast, vec);
            }
        }
        Ast::Concat(concat) => {
            for ast in &concat.asts {
                ast_find_capture_groups_recurse(depth + 1, ast, vec);
            }
        }
        _ => {}
    }
}
