use super::parsing::*;
use super::state::{AppState, LogicState};
use eframe::{epaint::CubicBezierShape, Frame};
use egui::{
    text_edit::TextEditOutput, CentralPanel, Color32, Context, Grid, Layout, Response, RichText,
    ScrollArea, SidePanel, Stroke, TextEdit, TopBottomPanel, Ui, Vec2, Visuals,
};

/// Displays the entire ui
pub fn root(state: &mut AppState, ctx: &Context, frame: &mut Frame) {
    TopBottomPanel::top("menu").show(ctx, |ui| menu_bar(ui, frame));

    SidePanel::right("debug_info")
        .max_width(ctx.available_rect().width() * 0.5)
        .show(ctx, |ui| debug_info(ui, state));

    CentralPanel::default().show(ctx, |ui| {
        Grid::new("grid").num_columns(2).show(ui, |ui| {
            let regex_result = regex_input(ui, state);
            ui.end_row();

            let input_result = text_input(ui, state);
            ui.end_row();

            let replace_response = replace_input(ui, state);
            ui.end_row();

            result_text(
                ui,
                state,
                &regex_result.response,
                &input_result.response,
                &replace_response,
            );
            ui.end_row();

            connecting_lines(ui, state, &regex_result, &input_result);
            ui.end_row();
        });
    });
}

/// Renders the menu bar (The thing that is usually toggled by pressing `alt`)
fn menu_bar(ui: &mut Ui, frame: &mut Frame) {
    egui::menu::bar(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("Quit").clicked() {
                frame.quit();
            }
        });

        ui.with_layout(Layout::right_to_left(), egui::warn_if_debug_build);
    });
}

/// Pretty-prints the debug output of the regex AST
fn debug_info(ui: &mut Ui, state: &AppState) {
    ui.heading("Regex Debug Info");
    ui.separator();

    if let Ok(l) = &state.logic {
        ScrollArea::vertical().show(ui, |ui| ui.monospace(format!("{:#?}", l.ast)));
    }
}

/// Handles the regular expression input and associated state
fn regex_input(ui: &mut Ui, state: &mut AppState) -> TextEditOutput {
    ui.label("Regex input:");

    // Gets the style elements associated with the outline of textboxes
    fn textbox_stroke_style(
        visuals: &mut Visuals,
    ) -> (&mut Color32, &mut Color32, &mut Color32, &mut f32) {
        (
            &mut visuals.selection.stroke.color,
            &mut visuals.widgets.hovered.bg_stroke.color,
            &mut visuals.widgets.inactive.bg_stroke.color,
            &mut visuals.widgets.inactive.bg_stroke.width,
        )
    }

    // Displays the textbox and does the associated state management
    fn show(ui: &mut Ui, state: &mut AppState) -> TextEditOutput {
        // If the text gets edited the layouter will be ran again; keep track of this to allow caching state
        let mut regex_changed = false;
        TextEdit::multiline(&mut state.widgets.regex_input)
            .layouter(&mut |ui, text, wrap_width| {
                if regex_changed {
                    // Recompute relevant state if the text was edited
                    state.logic =
                        LogicState::new(text, ui.style(), text, &state.widgets.text_input);
                }
                regex_changed = true;

                let mut layout_job = state.logic.as_ref().map_or_else(
                    |_| layout_regex_err(text.into(), ui.style()).job,
                    |l| l.regex_layout.job.clone(),
                );
                layout_job.wrap.max_width = wrap_width;
                ui.fonts().layout_job(layout_job)
            })
            .show(ui)
    }

    if let Err(e) = state.logic.as_ref().map_err(ToString::to_string) {
        // If the regex is malformed, adjust the style to give the textbox a red border
        let (stroke_a, stroke_b, stroke_c, stroke_width) = textbox_stroke_style(ui.visuals_mut());
        let old_stroke_a = std::mem::replace(stroke_a, Color32::RED);
        let old_stroke_b = std::mem::replace(stroke_b, Color32::RED);
        let old_stroke_c = std::mem::replace(stroke_c, Color32::RED);
        let old_stroke_width = std::mem::replace(stroke_width, 0.75);

        let result = show(ui, state);

        // Restore the prior style to avoid messing up other ui elements
        let (stroke_a, stroke_b, stroke_c, stroke_width) = textbox_stroke_style(ui.visuals_mut());
        *stroke_a = old_stroke_a;
        *stroke_b = old_stroke_b;
        *stroke_c = old_stroke_c;
        *stroke_width = old_stroke_width;

        // Display the error text
        egui::Frame::popup(ui.style()).show(ui, |ui| {
            ui.label(RichText::new(e).monospace().color(Color32::RED))
        });

        result
    } else {
        // If the regex is well-formed nothing special needs to be done
        show(ui, state)
    }
}

/// Handles the text input and associated state
fn text_input(ui: &mut Ui, state: &mut AppState) -> TextEditOutput {
    ui.label("Text input:");

    // If the text gets edited the layouter will be ran again; keep track of this to allow caching state
    let mut input_changed = false;
    TextEdit::multiline(&mut state.widgets.text_input)
        .layouter(&mut |ui, text, wrap_width| {
            if input_changed {
                if let Ok(logic) = &mut state.logic {
                    // Re-layout the text if it or the regex were changed
                    logic.text_layout = layout_matched_text(
                        text.to_owned(),
                        &logic.regex,
                        ui.style(),
                        &logic.regex_layout,
                    );
                }
            }
            input_changed = true;

            let mut layout_job = state.logic.as_ref().map_or_else(
                |_| layout_plain_text(text.to_owned(), ui.style()).job,
                |l| l.text_layout.job.clone(),
            );
            layout_job.wrap.max_width = wrap_width;
            ui.fonts().layout_job(layout_job)
        })
        .show(ui)
}

/// Handles the replace text input and associated state
fn replace_input(ui: &mut Ui, state: &mut AppState) -> Response {
    ui.label("Replace with:");
    ui.add(
        TextEdit::multiline(&mut state.widgets.replace_input)
            .hint_text(RichText::new("<Empty String>").monospace()),
    )
}

/// Displays the result text from using the regex and replace text to alter the input text
fn result_text(
    ui: &mut Ui,
    state: &mut AppState,
    regex_response: &Response,
    input_response: &Response,
    replace_response: &Response,
) {
    ui.label("Result:");

    // Re-run the regex replacement if any of the inputs changed
    if regex_response.changed() || input_response.changed() || replace_response.changed() {
        if let Ok(logic) = &state.logic {
            state.widgets.replace_output = logic
                .regex
                .replace_all(&state.widgets.text_input, &state.widgets.replace_input)
                .into_owned();
        }
    }
    ui.text_edit_multiline(&mut state.widgets.replace_output.as_str());
}

/// Renders connecting lines between corresponding parts of the input text and regular expression text
fn connecting_lines(
    ui: &mut Ui,
    state: &AppState,
    regex_result: &TextEditOutput,
    input_result: &TextEditOutput,
) {
    let capture_group_sections = match &state.logic {
        Ok(l) => &l.text_layout.capture_group_sections,
        Err(_) => return,
    };

    // Only handle a single line of text (for now)
    let regex_row = match regex_result.galley.rows.first() {
        Some(r) => r,
        None => return,
    };

    // The regex text is rendered above the input text so the lines should terminate at the bottom of the regex text
    let regex_y = regex_row.rect.bottom();

    // Calculate the min and max x coordinates of all of the glyphs in each section
    let mut regex_bounds: Vec<(f32, f32)> =
        Vec::with_capacity(regex_result.galley.job.sections.len());
    for glyph in &regex_row.glyphs {
        let (min, max) = (glyph.pos.x, glyph.pos.x + glyph.size.x);
        match regex_bounds.get_mut(glyph.section_index as usize) {
            Some(bounds) => *bounds = (bounds.0.min(min), bounds.1.max(max)),
            None => regex_bounds.push((min, max)),
        }
    }

    let input_row = match input_result.galley.rows.first() {
        Some(r) => r,
        None => return,
    };

    // The input text is rendered below the regex text so the lines should terminate at the top of the input text
    let input_y = input_row.rect.top();

    // Calculate the min and max x coordinates of all of the glyphs in each section
    let mut input_bounds: Vec<(f32, f32)> =
        Vec::with_capacity(input_result.galley.job.sections.len());
    for glyph in &input_row.glyphs {
        let (min, max) = (glyph.pos.x, glyph.pos.x + glyph.size.x);
        match input_bounds.get_mut(glyph.section_index as usize) {
            Some(bounds) => *bounds = (bounds.0.min(min), bounds.1.max(max)),
            None => input_bounds.push((min, max)),
        }
    }

    // `capture_group_sections` determines which sections should have lines drawn between them
    for (regex_section, input_section) in capture_group_sections
        .iter()
        .enumerate()
        .filter_map(|(i, r)| r.zip(Some(i)))
    {
        // The x coordinates of each end of the line are the midpoints of the corresponding sections.
        let (from_min, from_max) = regex_bounds[regex_section];
        let from = regex_result.text_draw_pos + Vec2::new((from_max + from_min) * 0.5, regex_y);

        let (to_min, to_max) = input_bounds[input_section];
        let to = input_result.text_draw_pos + Vec2::new((to_max + to_min) * 0.5, input_y);

        // Use cubic bezier lines for a nice looking curve
        let control_scale = ((to.y - from.y) / 2.0).max(30.0);
        let from_control = from + Vec2::Y * control_scale;
        let to_control = to - Vec2::Y * control_scale;

        let bezier = CubicBezierShape::from_points_stroke(
            [from, from_control, to_control, to],
            false,
            Color32::TRANSPARENT,
            Stroke::new(
                2.5,
                regex_result.galley.job.sections[regex_section].format.color,
            ),
        );

        ui.painter().add(bezier);
    }
}
