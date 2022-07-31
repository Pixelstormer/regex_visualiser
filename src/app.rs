use egui::{
    text::{LayoutJob, LayoutSection},
    Color32,
};
use regex::{Error as CompileError, Regex};
use regex_syntax::ast::{Ast, Error as AstError, Span};

#[derive(Debug)]
pub enum RegexError {
    Ast(AstError),
    Compiled(CompileError),
}

impl std::fmt::Display for RegexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
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
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(), egui::warn_if_debug_build);
            });
        });

        egui::SidePanel::right("right").show(ctx, |ui| {
            ui.heading("Regex Info");
            ui.separator();
            match &self.regex_output {
                Ok((ast, _)) => ui.add(egui::Label::new(
                    egui::RichText::new(format!("{:?}", ast))
                        .monospace()
                        .color(egui::Color32::GRAY),
                )),
                Err(e) => ui.add(
                    egui::Label::new(
                        egui::RichText::new(format!("{:?}", e))
                            .monospace()
                            .color(egui::Color32::RED),
                    )
                    .wrap(false),
                ),
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Grid::new("grid").num_columns(2).show(ui, |ui| {
                ui.label("Text input:");
                let input_response = ui.text_edit_multiline(&mut self.text_input);
                if input_response.changed() {
                    //
                }
                ui.end_row();

                ui.label("Regex input:");
                let regex_response =
                    ui.add(egui::TextEdit::multiline(&mut self.regex_input).layouter(
                        &mut |ui, text, wrap_width| {
                            if text != self.regex_layout.text {
                                self.regex_output = compile_regex(text);
                                self.regex_layout = regex_layouter(
                                    ui.style(),
                                    text.to_string(),
                                    &self.regex_output,
                                );
                            }
                            let mut layout_job = self.regex_layout.clone();
                            layout_job.wrap.max_width = wrap_width;
                            ui.fonts().layout_job(layout_job)
                        },
                    ));
                ui.end_row();

                ui.label("Replace with:");
                let replace_response = ui.text_edit_multiline(&mut self.replace_input);
                if input_response.changed()
                    || regex_response.changed()
                    || replace_response.changed()
                {
                    if let Ok((_, regex)) = &self.regex_output {
                        self.replace_output = regex
                            .replace_all(&self.text_input, &self.replace_input)
                            .to_string();
                    }
                }
                ui.end_row();

                ui.label("Result:");
                ui.text_edit_multiline(&mut self.replace_output.as_ref());
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

pub fn regex_layouter(style: &egui::Style, text: String, regex: &RegexOutput) -> LayoutJob {
    let font_id = egui::FontSelection::from(egui::TextStyle::Monospace).resolve(style);

    match regex {
        Ok((ast, _)) => {
            if let Ast::Concat(c) = ast {
                let mut layout_job = LayoutJob {
                    text,
                    sections: Vec::with_capacity(c.asts.len()),
                    ..Default::default()
                };

                let mut asts_iter = c.asts.iter().peekable();
                let mut colors_iter = highlight_colors::get_looping();

                let mut section = |byte_range: std::ops::Range<usize>| LayoutSection {
                    leading_space: 0.0,
                    byte_range,
                    format: egui::TextFormat {
                        background: colors_iter.next().unwrap(),
                        font_id: font_id.clone(),
                        ..Default::default()
                    },
                };

                let mut literal_start = None;
                while let Some(ast) = asts_iter.next() {
                    if let Ast::Literal(_) = ast {
                        let start_offset = *literal_start.get_or_insert(ast.span().start.offset);
                        match asts_iter.peek() {
                            Some(Ast::Literal(_)) => {}
                            _ => {
                                let end_offset = ast.span().end.offset;
                                literal_start = None;
                                layout_job.sections.push(section(start_offset..end_offset));
                            }
                        }
                    } else {
                        let Span { start, end } = ast.span();
                        layout_job.sections.push(section(start.offset..end.offset));
                    }
                }

                layout_job
            } else {
                LayoutJob::single_section(
                    text,
                    egui::TextFormat {
                        background: highlight_colors::RED,
                        font_id,
                        ..Default::default()
                    },
                )
            }
        }
        Err(RegexError::Ast(e)) => {
            let mut layout_job = LayoutJob {
                text,
                sections: Vec::with_capacity(3),
                ..Default::default()
            };

            let span = e.span();

            layout_job.sections.push(LayoutSection {
                leading_space: 0.0,
                byte_range: 0..span.start.offset,
                format: egui::TextFormat {
                    color: Color32::RED,
                    font_id: font_id.clone(),
                    ..Default::default()
                },
            });

            layout_job.sections.push(LayoutSection {
                leading_space: 0.0,
                byte_range: span.start.offset..span.end.offset,
                format: egui::TextFormat {
                    background: highlight_colors::RED,
                    font_id: font_id.clone(),
                    ..Default::default()
                },
            });

            layout_job.sections.push(LayoutSection {
                leading_space: 0.0,
                byte_range: span.end.offset..layout_job.text.len(),
                format: egui::TextFormat {
                    color: Color32::RED,
                    font_id,
                    ..Default::default()
                },
            });

            layout_job
        }
        Err(RegexError::Compiled(_)) => LayoutJob::single_section(
            text,
            egui::TextFormat {
                color: Color32::RED,
                font_id,
                ..Default::default()
            },
        ),
    }
}

pub mod highlight_colors {
    use egui::Color32;
    pub const RED: Color32 = Color32::from_rgba_premultiplied(188, 0, 0, 128);
    pub const YELLOW: Color32 = Color32::from_rgba_premultiplied(188, 188, 0, 128);
    pub const GREEN: Color32 = Color32::from_rgba_premultiplied(0, 188, 0, 128);
    pub const CYAN: Color32 = Color32::from_rgba_premultiplied(0, 188, 188, 128);
    pub const BLUE: Color32 = Color32::from_rgba_premultiplied(0, 0, 188, 128);
    pub const MAGENTA: Color32 = Color32::from_rgba_premultiplied(188, 0, 188, 128);

    pub fn get_looping() -> impl Iterator<Item = Color32> {
        [RED, YELLOW, GREEN, CYAN, BLUE, MAGENTA]
            .into_iter()
            .cycle()
    }
}
