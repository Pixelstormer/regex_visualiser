use regex::Regex;
use regex_syntax::{ast::Ast, hir::Hir};

#[derive(Debug)]
pub enum RegexError {
    Ast(regex_syntax::ast::Error),
    Hir(regex_syntax::hir::Error),
    Compiled(regex::Error),
}

impl std::fmt::Display for RegexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegexError::Ast(e) => e.fmt(f),
            RegexError::Hir(e) => e.fmt(f),
            RegexError::Compiled(e) => e.fmt(f),
        }
    }
}

pub type RegexOutput = Result<(Ast, Hir, Regex), RegexError>;

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
                Ok((ast, hir, _)) => ui.add(egui::Label::new(
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
                let regex_response = ui.text_edit_multiline(&mut self.regex_input);
                if regex_response.changed() {
                    self.regex_output = compile_regex(&self.regex_input);
                }
                ui.end_row();

                ui.label("Replace with:");
                let replace_response = ui.text_edit_multiline(&mut self.replace_input);
                if input_response.changed()
                    || regex_response.changed()
                    || replace_response.changed()
                {
                    if let Ok((_, _, regex)) = &self.regex_output {
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

    let hir = match regex_syntax::hir::translate::Translator::new().translate(pattern, &ast) {
        Ok(hir) => hir,
        Err(e) => return Err(RegexError::Hir(e)),
    };

    let compiled = match Regex::new(pattern) {
        Ok(regex) => regex,
        Err(e) => return Err(RegexError::Compiled(e)),
    };

    Ok((ast, hir, compiled))
}
