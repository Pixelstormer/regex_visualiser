use crate::app::{
    state::AppState,
    text::{layout_plain_text, layout_regex_err},
};
use egui::{
    text_edit::TextEditOutput, Button, Color32, ComboBox, Context, Frame, Grid, SidePanel, Stroke,
    TextEdit, TextFormat, TextStyle, Ui,
};

/// Adds a container that displays an inspector that provides detailed breakdowns of the regex and its matches
pub fn inspector(ctx: &Context, state: &mut AppState) {
    SidePanel::right("inspector")
        .max_width(ctx.available_rect().width() - 64.0)
        .show(ctx, |ui| inspector_ui(ui, state));
}

/// Displays an inspector that provides detailed breakdowns of the regex and its matches
pub fn inspector_ui(ui: &mut Ui, state: &mut AppState) {
    ui.heading("Inspector");
    ui.separator();

    regular_expression(ui, state);
    ui.add_space(16.0);
    matches(ui, state);
}

fn regular_expression(ui: &mut Ui, state: &AppState) -> TextEditOutput {
    ui.label("Regular Expression");

    let mut frame = Frame::canvas(ui.style());
    if state.logic.is_err() {
        frame = frame.stroke(Stroke::new(1.0, Color32::RED));
    }

    frame
        .show(ui, |ui| {
            // Convert from a String to a &str to make the textedit immutable
            TextEdit::singleline(&mut state.widgets.regex_text.as_str())
                .desired_width(f32::INFINITY)
                .layouter(&mut |ui, text, wrap_width| {
                    let mut layout_job = state.logic.as_ref().map_or_else(
                        |err| layout_regex_err(text.into(), ui.style(), err).job,
                        |state| state.regex_layout.job.clone(),
                    );
                    layout_job.wrap.max_width = wrap_width;
                    ui.fonts().layout_job(layout_job)
                })
                .show(ui)
        })
        .inner
}

fn matches(ui: &mut Ui, state: &mut AppState) -> TextEditOutput {
    Grid::new("inspector").num_columns(5).show(ui, |ui| {
        whole_matches(ui, state);
        ui.label("Named groups");
        ui.end_row();

        capture_groups(ui, state);
        ui.end_row();
    });

    let logic = state.logic.as_mut().ok();

    Frame::canvas(ui.style())
        .show(ui, |ui| {
            TextEdit::singleline(
                &mut logic
                    .as_ref()
                    .and_then(|logic| logic.selector.current_str())
                    .unwrap_or_default(),
            )
            .desired_width(f32::INFINITY)
            .layouter(&mut |ui, text, wrap_width| {
                let mut layout_job = logic
                    .as_ref()
                    .and_then(|logic| Some(logic).zip(logic.selector.current_range()))
                    .map(|(logic, range)| {
                        let mut formatting = logic.input_layout.formatting.substring(range.clone());
                        let font_id = TextStyle::Monospace.resolve(ui.style());
                        formatting
                            .replace_format('\n', TextFormat::simple(font_id, Color32::DARK_GRAY));
                        formatting.replace(b'\n', "\\n");
                        formatting.convert_to_layout_job()
                    })
                    .unwrap_or_else(|| layout_plain_text(text.to_owned(), ui.style()));

                layout_job.wrap.max_width = wrap_width;
                ui.fonts().layout_job(layout_job)
            })
            .show(ui)
        })
        .inner
}

fn whole_matches(ui: &mut Ui, state: &mut AppState) {
    ui.label("Whole Matches");

    let mut matches = state
        .logic
        .as_mut()
        .map(|logic| &mut logic.selector.matches);

    let enabled = matches
        .as_ref()
        .map_or(false, |matches| !matches.is_empty());

    if ui.add_enabled(enabled, Button::new("<")).clicked() {
        matches.as_mut().unwrap().dec();
    }

    if enabled {
        let matches = matches.as_mut().unwrap();
        ui.label(format!("{}/{}", matches.index() + 1, matches.len()));
    } else {
        ui.label("-/-");
    }

    if ui.add_enabled(enabled, Button::new(">")).clicked() {
        matches.unwrap().inc();
    }
}

fn capture_groups(ui: &mut Ui, state: &mut AppState) {
    ui.label("Capture Groups");

    let mut groups = state
        .logic
        .as_mut()
        .ok()
        .and_then(|logic| logic.selector.matches.get_current_mut());

    let enabled = groups.as_ref().map_or(false, |matches| !matches.is_empty());

    if ui.add_enabled(enabled, Button::new("<")).clicked() {
        groups.as_mut().unwrap().dec();
    }

    if enabled {
        let groups = groups.as_mut().unwrap();
        ui.label(format!("{}/{}", groups.index() + 1, groups.len()));
    } else {
        ui.label("-/-");
    }

    if ui.add_enabled(enabled, Button::new(">")).clicked() {
        groups.as_mut().unwrap().inc();
    }

    ComboBox::from_id_source("combobox")
        .selected_text(
            groups
                .as_ref()
                .and_then(|groups| groups.get_current())
                .and_then(|(_, name)| name.as_deref())
                .unwrap_or_default(),
        )
        .show_ui(ui, |ui| {
            if let Some(groups) = groups {
                let mut new_index = groups.index();
                for (index, name) in groups
                    .iter()
                    .enumerate()
                    .filter_map(|(index, (_, name))| Some(index).zip(name.as_ref()))
                {
                    ui.selectable_value(&mut new_index, index, name);
                }
                groups.try_set_index(new_index);
            }
        });
}
