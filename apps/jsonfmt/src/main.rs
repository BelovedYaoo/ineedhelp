#![windows_subsystem = "windows"]

mod app;
mod context_menu;
mod edit;
mod ui;

use app::JsonFmtApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "JSON 格式化",
        options,
        Box::new(|_cc| Ok(Box::new(JsonFmtApp::default()))),
    )
}
