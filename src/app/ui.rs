use super::colors::GetColorExt;
use super::layout::{curve_between, galley_section_bounds, Orientation};
use super::parsing::*;
use super::state::{AppState, LogicState};
use egui::{
    layers::ShapeIdx, text_edit::TextEditOutput, CentralPanel, Color32, Context, Frame, Layout,
    Response, RichText, ScrollArea, Shape, SidePanel, Stroke, TextEdit, TopBottomPanel, Ui, Vec2,
};

/// Displays the entire ui
pub fn root(state: &mut AppState, ctx: &Context, frame: &mut eframe::Frame) {
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = Vec2::new(16.0, 6.0);
    ctx.set_style(style);

    TopBottomPanel::top("menu").show(ctx, |ui| menu_bar(ui, frame));

    SidePanel::right("debug_info")
        .max_width(ctx.available_rect().width() * 0.5)
        .show(ctx, |ui| regex_info(ui, state));

    CentralPanel::default().show(ctx, |ui| editor(ui, state));
}

/// Displays the menu bar (The thing that is usually toggled by pressing `alt`)
fn menu_bar(ui: &mut Ui, _frame: &mut eframe::Frame) {
    egui::menu::bar(ui, |ui| {
        #[cfg(not(target_arch = "wasm32"))]
        ui.menu_button("File", |ui| {
            if ui.button("Quit").clicked() {
                _frame.close();
            }
        });

        ui.with_layout(
            Layout::right_to_left(egui::Align::Center),
            egui::warn_if_debug_build,
        );
    });
}

/// Displays information about the regular expression
fn regex_info(ui: &mut Ui, state: &AppState) {
    ui.heading("Regex Information");
    ui.separator();

    ScrollArea::vertical().show(ui, |ui| match &state.logic {
        Ok(l) => ui.monospace(format!("{:#?}", l.ast)),
        Err(e) => ui.label(RichText::new(e.to_string()).monospace().color(Color32::RED)),
    });
}

/// Displays the main interactive parts of the UI
fn editor(ui: &mut Ui, state: &mut AppState) {
    ScrollArea::vertical().show(ui, |ui| {
        regex_header(ui);
        let regex_result = regex_editor(ui, state);

        input_header(ui);
        let mut idx = None;
        let input_result = ui
            .allocate_ui_with_layout(
                ui.available_size() - (ui.max_rect().size() * Vec2::Y * 0.5),
                Layout::centered_and_justified(ui.layout().main_dir()),
                |ui| input_editor(ui, state, &mut idx),
            )
            .inner;

        replace_header(ui);
        let replace_result = replace_editor(ui, state);

        result_header(ui);
        ui.allocate_ui_with_layout(
            ui.available_size(),
            Layout::centered_and_justified(ui.layout().main_dir()),
            |ui| {
                result_body(
                    ui,
                    state,
                    &regex_result.response,
                    &input_result.response,
                    &replace_result.response,
                )
            },
        );

        connecting_lines(ui, idx.unwrap(), state, &regex_result, &input_result);
    });
}

/// Displays the header for the regex editor
fn regex_header(ui: &mut Ui) {
    ui.label("Regular Expression");
}

/// Handles the regular expression text and associated state
fn regex_editor(ui: &mut Ui, state: &mut AppState) -> TextEditOutput {
    // If the text gets edited the layouter will be ran again; keep track of this to enable caching state
    let mut regex_changed = false;

    let frame = Frame::canvas(ui.style());
    state
        .logic
        .as_ref()
        .map_or_else(|_| frame.stroke(Stroke::new(1.0, Color32::RED)), |_| frame)
        .show(ui, |ui| {
            TextEdit::singleline(&mut state.widgets.regex_text)
                .desired_width(f32::INFINITY)
                .frame(false)
                .margin(Vec2::new(8.0, 4.0))
                .layouter(&mut |ui, text, wrap_width| {
                    if regex_changed {
                        // Recompute relevant state if the text was edited
                        state.logic = LogicState::new(
                            text,
                            ui.style(),
                            text,
                            &state.widgets.input_text,
                            state.logic.as_ref().ok(),
                        );
                    }
                    regex_changed = true;

                    let mut layout_job = state.logic.as_ref().map_or_else(
                        |e| layout_regex_err(text.into(), ui.style(), e).job,
                        |l| l.regex_layout.job.clone(),
                    );
                    layout_job.wrap.max_width = wrap_width;
                    ui.fonts().layout_job(layout_job)
                })
                .show(ui)
        })
        .inner
}

/// Displays the header for the input editor
fn input_header(ui: &mut Ui) {
    ui.label("Input Text");
}

/// Handles the input text and associated state
fn input_editor(ui: &mut Ui, state: &mut AppState, idx: &mut Option<ShapeIdx>) -> TextEditOutput {
    // If the text gets edited the layouter will be ran again; keep track of this to enable caching state
    let mut input_changed = false;
    Frame::canvas(ui.style())
        .show(ui, |ui| {
            TextEdit::multiline(&mut state.widgets.input_text)
                .desired_width(f32::INFINITY)
                .frame(false)
                .layouter(&mut |ui, text, wrap_width| {
                    *idx = Some(ui.painter().add(Shape::Noop));

                    if input_changed {
                        if let Ok(logic) = &mut state.logic {
                            // Re-layout the text if it or the regex were changed
                            logic.input_layout = layout_matched_text(
                                text.to_owned(),
                                &logic.regex,
                                ui.style(),
                                &logic.regex_layout,
                            );
                        }
                    }
                    input_changed = true;

                    let mut layout_job = state.logic.as_ref().map_or_else(
                        |_| layout_plain_text(text.to_owned(), ui.style()),
                        |l| l.input_layout.job.clone(),
                    );
                    layout_job.wrap.max_width = wrap_width;
                    ui.fonts().layout_job(layout_job)
                })
                .show(ui)
        })
        .inner
}

/// Displays the header for the replace editor
fn replace_header(ui: &mut Ui) {
    ui.label("Replace With");
}

/// Handles the replace text and associated state
fn replace_editor(ui: &mut Ui, state: &mut AppState) -> TextEditOutput {
    Frame::canvas(ui.style())
        .show(ui, |ui| {
            TextEdit::singleline(&mut state.widgets.replace_text)
                .desired_width(f32::INFINITY)
                .margin(Vec2::new(8.0, 4.0))
                .hint_text(RichText::new("<Empty String>").monospace())
                .show(ui)
        })
        .inner
}

/// Displays the header for the result body
fn result_header(ui: &mut Ui) {
    ui.label("Result Text");
}

/// Displays the result text from using the regex and replace text to alter the input text
fn result_body(
    ui: &mut Ui,
    state: &mut AppState,
    regex_response: &Response,
    input_response: &Response,
    replace_response: &Response,
) {
    // Re-run the regex replacement if any of the inputs changed
    if regex_response.changed() || input_response.changed() || replace_response.changed() {
        if let Ok(logic) = &state.logic {
            state.widgets.result_text = logic
                .regex
                .replace_all(&state.widgets.input_text, &state.widgets.replace_text)
                .into_owned();
        }
    }

    Frame::canvas(ui.style()).show(ui, |ui| {
        TextEdit::multiline(&mut state.widgets.result_text.as_str())
            .desired_width(f32::INFINITY)
            .show(ui)
    });
}

/// Renders connecting lines between corresponding parts of the input text and regular expression text
fn connecting_lines(
    ui: &mut Ui,
    idx: ShapeIdx,
    state: &AppState,
    regex_result: &TextEditOutput,
    input_result: &TextEditOutput,
) {
    // `layout_section_map` determines which sections should have lines drawn between them
    let layout_section_map = match &state.logic {
        Ok(l) => &l.input_layout.layout_section_map,
        Err(_) => return,
    };

    let regex_bounds = galley_section_bounds(&regex_result.galley);
    let input_bounds = galley_section_bounds(&input_result.galley);

    // The rects returned by `galley_section_bounds` are relative to galley position, but painted shapes need absolute coordinates
    let regex_offset = regex_result.text_draw_pos.to_vec2();
    let input_offset = input_result.text_draw_pos.to_vec2();

    let shapes = layout_section_map
        .iter()
        .enumerate()
        .filter_map(|(input_section, &regex_section)| {
            Some(
                curve_between(
                    // The regex text is rendered above the input text so the lines should terminate at the bottom of the regex text
                    regex_bounds.get(regex_section?)?.center_bottom() + regex_offset,
                    // The input text is rendered below the regex text so the lines should terminate at the top of the input text
                    input_bounds.get(input_section)?.center_top() + input_offset,
                    (
                        2.5,
                        // Draw the line with the same color as the sections it's being drawn between
                        input_result.galley.job.sections[input_section].get_color(),
                    ),
                    Orientation::Vertical,
                )
                .into(),
            )
        })
        .collect::<Vec<_>>();

    ui.painter().set(idx, shapes);
}
