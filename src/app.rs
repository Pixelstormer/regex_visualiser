mod colors;
mod parsing;

use self::parsing::*;
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
    group_colors: Vec<Color32>,
    #[serde(skip)]
    group_section_indexes: Vec<Option<usize>>,
    #[serde(skip)]
    section_index_map: Vec<Option<usize>>,
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
            group_colors: Default::default(),
            group_section_indexes: Default::default(),
            section_index_map: Default::default(),
            replace_input: "$0".into(),
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

                let err = self.regex_output.as_ref().err().map(ToString::to_string);
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

                let mut regex_result = TextEdit::multiline(&mut self.regex_input)
                    .layouter(&mut |ui, text, wrap_width| {
                        if text != self.regex_layout.text {
                            self.regex_output = compile_regex(text);
                            self.regex_layout = layout_regex(
                                ui.style(),
                                text.to_owned(),
                                &self.regex_output,
                                &mut self.group_colors,
                                &mut self.group_section_indexes,
                            );
                        }
                        let mut layout_job = self.regex_layout.clone();
                        layout_job.wrap.max_width = wrap_width;
                        ui.fonts().layout_job(layout_job)
                    })
                    .show(ui);

                if let Some(e) = &err {
                    let visuals = ui.visuals_mut();
                    let widgets = &mut visuals.widgets;

                    visuals.selection.stroke.color = old_colors[0];
                    widgets.hovered.bg_stroke.color = old_colors[1];
                    widgets.inactive.bg_stroke.color = old_colors[2];
                    widgets.inactive.bg_stroke.width = 0.0;

                    regex_result.response = regex_result
                        .response
                        .on_hover_text(RichText::new(e).monospace().color(Color32::RED));
                }

                ui.end_row();
                ui.label("Text input:");
                let input_result = TextEdit::multiline(&mut self.text_input)
                    .layouter(&mut |ui, text, wrap_width| {
                        if regex_result.response.changed() || text != self.text_layout.text {
                            self.text_layout = input_layouter(
                                ui.style(),
                                text.to_owned(),
                                &self.regex_output,
                                &self.group_colors,
                                &self.group_section_indexes,
                                &mut self.section_index_map,
                            );
                        }
                        let mut layout_job = self.text_layout.clone();
                        layout_job.wrap.max_width = wrap_width;
                        ui.fonts().layout_job(layout_job)
                    })
                    .show(ui);
                ui.end_row();

                ui.label("Replace with:");
                let replace_response = ui.add(
                    TextEdit::multiline(&mut self.replace_input)
                        .hint_text(RichText::new("<Empty String>").monospace()),
                );
                ui.end_row();

                ui.label("Result:");
                if input_result.response.changed()
                    || regex_result.response.changed()
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

                if self.regex_output.is_ok() {
                    if let Some(regex_row) = regex_result.galley.rows.first() {
                        let regex_y = regex_row.rect.bottom();
                        let mut regex_bounds: Vec<(f32, f32)> =
                            Vec::with_capacity(regex_result.galley.job.sections.len());
                        for glyph in &regex_row.glyphs {
                            let (min, max) = (glyph.pos.x, glyph.pos.x + glyph.size.x);
                            match regex_bounds.get_mut(glyph.section_index as usize) {
                                Some(bounds) => *bounds = (bounds.0.min(min), bounds.1.max(max)),
                                None => regex_bounds.push((min, max)),
                            }
                        }

                        if let Some(input_row) = input_result.galley.rows.first() {
                            let input_y = input_row.rect.top();
                            let mut input_bounds: Vec<(f32, f32)> =
                                Vec::with_capacity(input_result.galley.job.sections.len());
                            for glyph in &input_row.glyphs {
                                let (min, max) = (glyph.pos.x, glyph.pos.x + glyph.size.x);
                                match input_bounds.get_mut(glyph.section_index as usize) {
                                    Some(bounds) => {
                                        *bounds = (bounds.0.min(min), bounds.1.max(max))
                                    }
                                    None => input_bounds.push((min, max)),
                                }
                            }

                            for (regex_section, input_section) in self
                                .section_index_map
                                .iter()
                                .enumerate()
                                .filter_map(|(r, i)| i.zip(Some(r)))
                            {
                                let (from_min, from_max) = regex_bounds[regex_section];
                                let from = regex_result.text_draw_pos
                                    + Vec2::new((from_max + from_min) * 0.5, regex_y);

                                let (to_min, to_max) = input_bounds[input_section];
                                let to = input_result.text_draw_pos
                                    + Vec2::new((to_max + to_min) * 0.5, input_y);

                                let control_scale = ((to.y - from.y) / 2.0).max(30.0);
                                let from_control = from + Vec2::Y * control_scale;
                                let to_control = to - Vec2::Y * control_scale;

                                let bezier = CubicBezierShape::from_points_stroke(
                                    [from, from_control, to_control, to],
                                    false,
                                    Color32::TRANSPARENT,
                                    Stroke::new(
                                        2.5,
                                        regex_result.galley.job.sections[regex_section]
                                            .format
                                            .color,
                                    ),
                                );

                                ui.painter().add(bezier);
                            }
                        }
                    }
                }
            });
        });
    }
}
