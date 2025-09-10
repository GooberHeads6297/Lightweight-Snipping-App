mod gui;
mod image_ops;

fn main() {
    let native_options = eframe::NativeOptions::default();
eframe::run_native(
    "Rust Snipping Tool",
    native_options,
    Box::new(|_cc| Ok(Box::new(gui::SnippingApp::default()))),
).unwrap();
}
