#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod player;
mod ui;
mod video;

use std::path::PathBuf;

use app::TrimontinaApp;

fn main() -> eframe::Result {
    let initial_media: Option<PathBuf> = std::env::args_os().nth(1).map(PathBuf::from);

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1180.0, 760.0])
            .with_min_inner_size([720.0, 480.0])
            .with_title("Trimontina"),
        // libmpv's render API is built on this context's function pointers.
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    eframe::run_native(
        "Trimontina",
        options,
        Box::new(move |cc| {
            Ok(Box::new(TrimontinaApp::new(cc, initial_media)?) as Box<dyn eframe::App>)
        }),
    )
}
