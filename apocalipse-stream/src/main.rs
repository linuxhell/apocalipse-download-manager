#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod obs;
mod overlay;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Apocalipse Stream")
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([980.0, 650.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Apocalipse Stream",
        options,
        Box::new(|cc| Ok(Box::new(app::ApocalipseApp::new(cc)))),
    )
}
