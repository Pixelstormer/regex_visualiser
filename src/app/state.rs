use super::parsing::{compile_regex, RegexError};
use super::text::{layout_matched_text, regex_parse_ast, MatchedTextLayout, RegexLayout};
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

#[derive(Default, Eq, PartialEq, Copy, Clone)]
pub enum TabBarState {
    #[default]
    Collapsed,
    SyntaxGuide,
    Information,
}

impl TabBarState {
    pub fn toggle(&mut self, variant: Self) {
        if *self == variant {
            *self = Self::Collapsed;
        } else {
            *self = variant;
        }
    }
}

/// State for egui widgets
pub struct WidgetState {
    pub regex_text: String,
    pub input_text: String,
    pub replace_text: String,
    pub result_text: String,
    pub tab_bar_state: TabBarState,
    #[cfg(not(target_arch = "wasm32"))]
    pub about_visible: bool,
}

impl Default for WidgetState {
    fn default() -> Self {
        Self {
            regex_text: Default::default(),
            input_text: Default::default(),
            replace_text: "$0".into(),
            result_text: Default::default(),
            tab_bar_state: Default::default(),
            #[cfg(not(target_arch = "wasm32"))]
            about_visible: Default::default(),
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
            let regex_layout = regex_parse_ast(
                regex_text.to_string(),
                &ast,
                style,
                previous_state.map(|s| &s.regex_layout),
            );

            let input_layout = layout_matched_text(
                input_text.to_string(),
                &regex,
                style,
                &regex_layout.capture_group_colors,
            );

            Self {
                ast,
                regex,
                regex_layout,
                input_layout,
            }
        })
    }
}
