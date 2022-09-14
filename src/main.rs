#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Hide console window on Windows in release

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Regex Visualiser",
        native_options,
        Box::new(|creation_context| Box::new(regex_visualiser::Application::new(creation_context))),
    );
}

// When compiling to wasm:
#[cfg(target_arch = "wasm32")]
fn main() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();
    eframe::start_web(
        "the_canvas_id", // This id is duplicated in `index.html` as a hardcoded value
        web_options,
        Box::new(|creation_context| Box::new(regex_visualiser::Application::new(creation_context))),
    )
    .expect("Failed to start eframe");
}
