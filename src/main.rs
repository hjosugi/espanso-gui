mod app;
mod espanso;
mod model;
mod storage;
mod theme;

use app::EspansoGuiApp;
use eframe::egui;

fn main() -> eframe::Result {
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../icons/icon.png"))
        .expect("the embedded application icon must be a valid PNG");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Espanso GUI")
            .with_icon(icon)
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([1040.0, 680.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Espanso GUI",
        options,
        Box::new(|creation_context| Ok(Box::new(EspansoGuiApp::new(creation_context)))),
    )
}
