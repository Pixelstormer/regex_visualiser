use super::parsing::*;
use egui::Style;
use lazy_static::lazy_static;
use regex::Regex;
use regex_syntax::ast::Ast;

/// State for the application as a whole
pub struct AppState {
    pub widgets: WidgetState,
    pub logic: LogicResult,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            widgets: Default::default(),
            logic: Ok(Default::default()),
        }
    }
}

/// State for egui widgets
pub struct WidgetState {
    pub regex_text: String,
    pub input_text: String,
    pub replace_text: String,
    pub result_text: String,
}

impl Default for WidgetState {
    fn default() -> Self {
        Self {
            regex_text: Default::default(),
            input_text: Default::default(),
            replace_text: "$0".into(),
            result_text: Default::default(),
        }
    }
}

pub type LogicResult = Result<LogicState, RegexError>;

/// State for application logic
pub struct LogicState {
    pub ast: Ast,
    pub regex: Regex,
    pub regex_layout: RegexLayout,
    pub input_layout: MatchedTextLayout,
}

impl Default for LogicState {
    fn default() -> Self {
        lazy_static! {
            static ref EMPTY_REGEX: (Ast, Regex) = compile_regex("").unwrap();
        };
        Self {
            ast: EMPTY_REGEX.0.clone(),
            regex: EMPTY_REGEX.1.clone(),
            regex_layout: Default::default(),
            input_layout: Default::default(),
        }
    }
}

impl LogicState {
    /// Compiles the given regular expression pattern and lays out the given text accordingly
    pub fn new(
        pattern: &str,
        style: &Style,
        regex_text: impl ToString,
        input_text: impl ToString,
        previous_state: Option<&Self>,
    ) -> LogicResult {
        compile_regex(pattern).map(|(ast, regex)| {
            let regex_layout = layout_regex(
                regex_text.to_string(),
                &ast,
                style,
                previous_state.map(|s| &s.regex_layout),
            );
            let input_layout =
                layout_matched_text(input_text.to_string(), &regex, style, &regex_layout);
            Self {
                ast,
                regex,
                regex_layout,
                input_layout,
            }
        })
    }
}
