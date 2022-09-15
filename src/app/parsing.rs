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

/// Finds all capture groups in the given AST and returns the depth and span of each one
pub fn ast_find_capture_groups(ast: &Ast) -> (Vec<usize>, Vec<Range<usize>>) {
    let mut stack = vec![(0, ast)];
    let mut depths = Vec::new();
    let mut ranges = Vec::new();
    while let Some((depth, ast)) = stack.pop() {
        match ast {
            Ast::Repetition(repetition) => stack.push((depth + 1, &repetition.ast)),
            Ast::Group(group) => {
                if let Some(index) = group.capture_index() {
                    assert_eq!(
                        depths.len() + 1,
                        index as usize,
                        "Regex capture group indexes are not consecutive (Expected: {}, Got: {})",
                        depths.len() + 1,
                        index
                    );

                    depths.push(depth);
                    ranges.push(group.span.range());
                    stack.push((depth + 1, &group.ast))
                }
            }
            Ast::Alternation(alt) => stack.extend(alt.asts.iter().map(|ast| (depth + 1, ast))),
            Ast::Concat(concat) => stack.extend(concat.asts.iter().map(|ast| (depth + 1, ast))),
            _ => {}
        }
    }
    (depths, ranges)
}
