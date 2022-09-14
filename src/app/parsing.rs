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
    let mut stack = vec![(1, ast)];
    let mut result = Vec::new();
    while let Some((depth, ast)) = stack.pop() {
        match ast {
            Ast::Repetition(repetiton) => stack.push((depth + 1, &repetiton.ast)),
            Ast::Group(group) => {
                if let Some(index) = group.capture_index() {
                    assert_eq!(
                        result.len() + 1,
                        index as usize,
                        "Regex capture group indexes are not consecutive (Expected: {}, Got: {})",
                        result.len() + 1,
                        index
                    );

                    result.push((depth, group.span.range()));
                    stack.push((depth + 1, &group.ast))
                }
            }
            Ast::Alternation(alt) => stack.extend(alt.asts.iter().map(|ast| (depth + 1, ast))),
            Ast::Concat(concat) => stack.extend(concat.asts.iter().map(|ast| (depth + 1, ast))),
            _ => {}
        }
    }
    result
}
