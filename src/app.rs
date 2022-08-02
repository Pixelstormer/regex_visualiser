use eframe::{CreationContext, Frame};
use egui::{
    text::{LayoutJob, LayoutSection},
    CentralPanel, Color32, Context, FontData, FontDefinitions, FontFamily, FontSelection, Grid,
    Layout, RichText, ScrollArea, SidePanel, Style, TextEdit, TextFormat, TextStyle,
    TopBottomPanel,
};
use regex::{Error as CompileError, Regex};
use regex_syntax::ast::{Ast, Error as AstError, Position, Span};
use std::fmt::{Display, Formatter};

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

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Application {
    #[serde(skip)]
    text_input: String,
    #[serde(skip)]
    text_layout: LayoutJob,
    #[serde(skip)]
    regex_input: String,
    #[serde(skip)]
    regex_output: RegexOutput,
    #[serde(skip)]
    regex_layout: LayoutJob,
    #[serde(skip)]
    replace_input: String,
    #[serde(skip)]
    replace_output: String,
}

impl Default for Application {
    fn default() -> Self {
        Self {
            text_input: Default::default(),
            text_layout: Default::default(),
            regex_input: Default::default(),
            regex_output: compile_regex(""),
            regex_layout: Default::default(),
            replace_input: Default::default(),
            replace_output: Default::default(),
        }
    }
}

impl Application {
    /// Called once before the first frame.
    pub fn new(cc: &CreationContext<'_>) -> Self {
        let mut fonts = FontDefinitions::default();

        fonts.font_data.insert(
            "Atkinson-Hyperlegible-Regular".into(),
            FontData::from_static(include_bytes!(
                "../assets/fonts/Atkinson-Hyperlegible-Regular-102.ttf"
            )),
        );

        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "Atkinson-Hyperlegible-Regular".to_owned());

        for data in fonts.font_data.values_mut() {
            data.tweak.scale *= 1.15;
        }

        cc.egui_ctx.set_fonts(fonts);

        // Load previous app state (if any).
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for Application {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        TopBottomPanel::top("menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });

                ui.with_layout(Layout::right_to_left(), egui::warn_if_debug_build);
            });
        });

        SidePanel::right("debug_info")
            .max_width(ctx.available_rect().width() * 0.5)
            .show(ctx, |ui| {
                ui.heading("Regex Debug Info");
                ui.separator();

                if let Ok((ast, _)) = &self.regex_output {
                    ScrollArea::vertical().show(ui, |ui| ui.monospace(format!("{:#?}", ast)));
                }
            });

        CentralPanel::default().show(ctx, |ui| {
            Grid::new("grid").num_columns(2).show(ui, |ui| {
                ui.label("Regex input:");

                let err = self.regex_output.as_ref().err().cloned();
                let mut old_colors = [Color32::TRANSPARENT; 3];

                if err.is_some() {
                    let visuals = ui.visuals_mut();
                    let widgets = &mut visuals.widgets;

                    old_colors = [
                        std::mem::replace(&mut visuals.selection.stroke.color, Color32::RED),
                        std::mem::replace(&mut widgets.hovered.bg_stroke.color, Color32::RED),
                        std::mem::replace(&mut widgets.inactive.bg_stroke.color, Color32::RED),
                    ];
                    widgets.inactive.bg_stroke.width = 1.0;
                }

                let mut regex_response =
                    ui.add(TextEdit::multiline(&mut self.regex_input).layouter(
                        &mut |ui, text, wrap_width| {
                            if text != self.regex_layout.text {
                                self.regex_output = compile_regex(text);
                                self.regex_layout =
                                    regex_layouter(ui.style(), text.to_owned(), &self.regex_output);
                            }
                            let mut layout_job = self.regex_layout.clone();
                            layout_job.wrap.max_width = wrap_width;
                            ui.fonts().layout_job(layout_job)
                        },
                    ));

                if let Some(e) = err {
                    let visuals = ui.visuals_mut();
                    let widgets = &mut visuals.widgets;

                    visuals.selection.stroke.color = old_colors[0];
                    widgets.hovered.bg_stroke.color = old_colors[1];
                    widgets.inactive.bg_stroke.color = old_colors[2];
                    widgets.inactive.bg_stroke.width = 0.0;

                    regex_response = regex_response.on_hover_text(
                        RichText::new(e.to_string()).monospace().color(Color32::RED),
                    );
                }

                ui.end_row();
                ui.label("Text input:");
                let input_response = ui.add(TextEdit::multiline(&mut self.text_input).layouter(
                    &mut |ui, text, wrap_width| {
                        if regex_response.changed() || text != self.text_layout.text {
                            self.text_layout =
                                input_layouter(ui.style(), text.to_owned(), &self.regex_output);
                        }
                        let mut layout_job = self.text_layout.clone();
                        layout_job.wrap.max_width = wrap_width;
                        ui.fonts().layout_job(layout_job)
                    },
                ));
                ui.end_row();

                ui.label("Replace with:");
                let replace_response = ui.text_edit_multiline(&mut self.replace_input);
                ui.end_row();

                ui.label("Result:");
                if input_response.changed()
                    || regex_response.changed()
                    || replace_response.changed()
                {
                    if let Ok((_, regex)) = &self.regex_output {
                        self.replace_output = regex
                            .replace_all(&self.text_input, &self.replace_input)
                            .into_owned();
                    }
                }
                ui.text_edit_multiline(&mut self.replace_output);
                ui.end_row();
            });
        });
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

pub fn input_layouter(style: &Style, text: String, regex: &RegexOutput) -> LayoutJob {
    let font_id = FontSelection::from(TextStyle::Monospace).resolve(style);

    match regex {
        Ok((_, r)) => {
            let mut sections = Vec::new();
            let mut previous_match_end = 0;

            for (m, color) in r
                .find_iter(&text)
                .zip(colors::FOREGROUND_COLORS.into_iter().cycle())
            {
                if previous_match_end < m.start() {
                    sections.push(LayoutSection {
                        leading_space: 0.0,
                        byte_range: previous_match_end..m.start(),
                        format: TextFormat {
                            font_id: font_id.clone(),
                            ..Default::default()
                        },
                    });
                }

                sections.push(LayoutSection {
                    leading_space: 0.0,
                    byte_range: m.range(),
                    format: TextFormat::simple(font_id.clone(), color),
                });

                previous_match_end = m.end();
            }

            if previous_match_end < text.len() {
                sections.push(LayoutSection {
                    leading_space: 0.0,
                    byte_range: previous_match_end..text.len(),
                    format: TextFormat {
                        font_id,
                        ..Default::default()
                    },
                });
            }

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

pub fn regex_layouter(style: &Style, text: String, regex: &RegexOutput) -> LayoutJob {
    let font_id = FontSelection::from(TextStyle::Monospace).resolve(style);

    match regex {
        Ok((ast, _)) => {
            if let Ast::Concat(c) = ast {
                let mut sections = Vec::with_capacity(c.asts.len());
                let mut asts_iter = c.asts.iter().peekable();
                let mut colors_iter = colors::FOREGROUND_COLORS.into_iter().cycle();

                let mut push_section = |span: &Span| {
                    sections.push(LayoutSection {
                        leading_space: 0.0,
                        byte_range: span.start.offset..span.end.offset,
                        format: TextFormat::simple(font_id.clone(), colors_iter.next().unwrap()),
                    })
                };

                let mut offset = 0;
                while let (Some(ast), peeked) = (asts_iter.next(), asts_iter.peek()) {
                    match (ast, peeked) {
                        (Ast::Literal(_), Some(Ast::Literal(_))) => {}
                        (Ast::Literal(l), _) => push_section(&l.span.with_start(Position {
                            offset,
                            line: 0,
                            column: 0,
                        })),
                        (_, Some(Ast::Literal(l))) => {
                            offset = l.span.start.offset;
                            push_section(ast.span());
                        }
                        _ => push_section(ast.span()),
                    }
                }

                sections.shrink_to_fit();

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

pub mod colors {
    use egui::Color32;
    pub const FG_YELLOW: Color32 = Color32::from_rgb(255, 215, 0);
    pub const FG_PINK: Color32 = Color32::from_rgb(218, 112, 214);
    pub const FG_BLUE: Color32 = Color32::from_rgb(23, 159, 255);

    pub const FOREGROUND_COLORS: [Color32; 3] = [FG_BLUE, FG_YELLOW, FG_PINK];
}
