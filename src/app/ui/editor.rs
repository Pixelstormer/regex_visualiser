use crate::app::shape::{curve_between, Orientation};
use crate::app::state::{AppState, LogicState};
use crate::app::text::{glyph_bounds, layout_matched_text, layout_plain_text, layout_regex_err};
use egui::{
    layers::ShapeIdx, text_edit::TextEditOutput, Align, CentralPanel, Color32, Context, Frame,
    Layout, Response, RichText, ScrollArea, Shape, Stroke, TextEdit, Ui, Vec2,
};

/// Adds a container that displays the main interactive parts of the UI
pub fn editor(ctx: &Context, state: &mut AppState) {
    CentralPanel::default().show(ctx, |ui| editor_ui(ui, state));
}

/// Displays the main interactive parts of the UI
pub fn editor_ui(ui: &mut Ui, state: &mut AppState) {
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

        connecting_lines(ui, state, idx.unwrap(), &regex_result, &input_result);
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

    let mut frame = Frame::canvas(ui.style());
    if state.logic.is_err() {
        frame = frame.stroke(Stroke::new(1.0, Color32::RED));
    }

    frame
        .show(ui, |ui| {
            ui.shrink_height_to_current();
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(3.0);

                let c = state.logic.is_err().then_some("⊗").unwrap_or_default();
                let response = ui.label(RichText::new(c).color(Color32::RED).size(21.0));
                if let Err(e) = &state.logic {
                    response.on_hover_text(
                        RichText::new(e.to_string()).color(Color32::RED).monospace(),
                    );
                }

                let result = TextEdit::singleline(&mut state.widgets.regex_text)
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
                    .show(ui);
                result
            })
        })
        .inner
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
                                &logic.regex_layout.capture_group_colors,
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
    state: &AppState,
    idx: ShapeIdx,
    regex_result: &TextEditOutput,
    input_result: &TextEditOutput,
) {
    let logic = match &state.logic {
        Ok(logic) => logic,
        Err(_) => return,
    };

    let regex_ranges = &logic.regex_layout.capture_group_chars;

    // Each capture group in the regex is highlighted with the color at the corresponding index in `capture_group_colors`,
    // including the implicit capture group corresponding to the whole match that always occupies index 0
    // (Which is represented in `capture_group_colors` with `Color32::TRANSPARENT`),
    // so if `capture_group_colors` has 0 or 1 elements, that means the regex does not contain any real capture groups,
    // meaning there isn't anything to draw connecting lines between
    let regex_colors = match logic.regex_layout.capture_group_colors.as_slice() {
        [_, tail @ ..] if !tail.is_empty() => tail,
        _ => return,
    };

    assert_eq!(
        regex_ranges.len(),
        regex_colors.len(),
        "Different number of char ranges and colors for regex capture groups (Ranges: {}, Colors: {})",
        regex_ranges.len(),
        regex_colors.len(),
    );

    let regex_rows = &regex_result.galley.rows;
    let input_rows = &input_result.galley.rows;

    // The rects returned by `galley_section_bounds` are relative to galley position, but painted shapes need absolute coordinates
    let regex_offset = regex_result.text_draw_pos.to_vec2();
    let input_offset = input_result.text_draw_pos.to_vec2();

    let shapes = logic
        .input_layout
        .capture_group_chars
        .iter()
        .flat_map(|ranges| {
            assert_eq!(
                regex_ranges.len(),
                ranges.len(),
                "Different number of char ranges for regex and input text (Regex: {}, Input: {})",
                regex_ranges.len(),
                ranges.len(),
            );

            ranges
                .iter()
                .zip(regex_ranges)
                .zip(regex_colors)
                .filter_map(|((input_range, (depth, regex_range)), &color)| {
                    Some(
                        curve_between(
                            glyph_bounds(regex_rows, regex_range)?.center_bottom() + regex_offset,
                            glyph_bounds(input_rows, input_range.as_ref()?)?.center_top()
                                + input_offset,
                            ((*depth as f32 + 1.0) * 2.0, color),
                            Orientation::Vertical,
                        )
                        .into(),
                    )
                })
        })
        .collect::<Vec<_>>();

    ui.painter().set(idx, shapes);
}
