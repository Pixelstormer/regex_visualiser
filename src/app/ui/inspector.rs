use crate::app::{
    state::AppState,
    text::{layout_plain_text, layout_regex_err},
};
use egui::{
    Color32, ComboBox, Context, Frame, Grid, SidePanel, Stroke, TextEdit, TextFormat, TextStyle, Ui,
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
    matches(ui, state);
}

fn regular_expression(ui: &mut Ui, state: &AppState) {
    ui.label("Regular Expression");

    let mut frame = Frame::canvas(ui.style());
    if state.logic.is_err() {
        frame = frame.stroke(Stroke::new(1.0, Color32::RED));
    }

    frame.show(ui, |ui| {
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
            .show(ui);
    });
}

fn matches(ui: &mut Ui, state: &mut AppState) {
    let logic = if let Ok(logic) = &mut state.logic {
        logic
    } else {
        return;
    };

    let selector = &mut logic.selector;

    Grid::new("inspector").show(ui, |ui| {
        let matches = &mut selector.matches;

        ui.label("Whole Matches");
        if ui.button("<").clicked() {
            matches.dec();
        }

        if matches.is_empty() {
            ui.label("-/-");
        } else {
            ui.label(format!("{}/{}", matches.index() + 1, matches.len()));
        }

        if ui.button(">").clicked() {
            matches.inc();
        }

        ui.end_row();

        let mut groups = matches.get_current_mut();

        ui.label("Capture Groups");
        if ui.button("<").clicked() {
            if let Some(ref mut groups) = groups {
                groups.dec();
            }
        }

        if let Some(groups) = groups.as_ref().filter(|groups| !groups.is_empty()) {
            ui.label(format!("{}/{}", groups.index() + 1, groups.len()));
        } else {
            ui.label("-/-");
        }

        if ui.button(">").clicked() {
            if let Some(ref mut groups) = groups {
                groups.inc();
            }
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

        ui.end_row();
    });

    Frame::canvas(ui.style()).show(ui, |ui| {
        TextEdit::singleline(&mut selector.current_str().unwrap_or_default())
            .desired_width(f32::INFINITY)
            .layouter(&mut |ui, text, wrap_width| {
                let mut layout_job = selector
                    .current_range()
                    .map(|range| {
                        let mut job = logic.input_layout.job.substring(range.clone());
                        let font_id = TextStyle::Monospace.resolve(ui.style());
                        job.replace_format('\n', TextFormat::simple(font_id, Color32::DARK_GRAY));
                        job.replace(b'\n', "\\n");
                        job.convert_to_layout_job()
                    })
                    .unwrap_or_else(|| layout_plain_text(text.to_owned(), ui.style()));

                layout_job.wrap.max_width = wrap_width;
                ui.fonts().layout_job(layout_job)
            })
            .show(ui);
    });
}
