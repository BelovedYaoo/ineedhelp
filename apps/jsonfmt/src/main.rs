#![windows_subsystem = "windows"]

mod app;
mod context_menu;
mod edit;
mod ui;

use app::JsonFmtApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    // 加载窗口图标
    let icon_data = include_bytes!("../jsonfmt.png");
    let icon_image = image::load_from_memory(icon_data)
        .expect("Failed to load icon")
        .to_rgba8();
    let (icon_width, icon_height) = icon_image.dimensions();
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 400.0])
            .with_icon(egui::IconData {
                rgba: icon_image.into_raw(),
                width: icon_width,
                height: icon_height,
            }),
        ..Default::default()
    };

    eframe::run_native(
        "JSON 格式化",
        options,
        Box::new(|_cc| Ok(Box::new(JsonFmtApp::default()))),
    )
}
