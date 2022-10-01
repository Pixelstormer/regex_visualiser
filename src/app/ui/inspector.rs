use crate::app::{state::AppState, text::layout_regex_err};
use egui::{Color32, Context, Frame, SidePanel, Stroke, TextEdit, Ui};

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
    let matches = if let Ok(logic) = &mut state.logic {
        &mut logic.matches
    } else {
        return;
    };

    ui.label("Matches");

    ui.horizontal(|ui| {
        if ui.button("<").clicked() {
            matches.dec();
        }

        ui.label(format!(
            "{}/{}",
            (matches.selected + 1).min(matches.matches.len()),
            matches.matches.len()
        ));

        if ui.button(">").clicked() {
            matches.inc();
        }
    });

    Frame::canvas(ui.style()).show(ui, |ui| {
        TextEdit::singleline(&mut matches.current().unwrap_or("")).show(ui);
    });
}
