mod app;
mod conflict;
mod espanso;
mod html_editor;
mod i18n;
mod lossless_yaml;
mod model;
mod navigation;
mod preferences;
mod profile_editor;
mod settings_editor;
mod snippet_editor;
mod snippet_library;
mod storage;
mod theme;
mod top_bar;
mod ui_components;
mod variable_editor;
mod yaml_editor;
mod yaml_syntax;

use app::EspansoGuiApp;
use eframe::egui;

fn main() -> eframe::Result {
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../icons/icon.png"))
        .expect("the embedded application icon must be a valid PNG");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Espanso GUI")
            .with_icon(icon)
            .with_inner_size(theme::DEFAULT_WINDOW_SIZE)
            .with_min_inner_size(theme::MINIMUM_WINDOW_SIZE),
        ..Default::default()
    };
    eframe::run_native(
        "Espanso GUI",
        options,
        Box::new(|creation_context| Ok(Box::new(EspansoGuiApp::new(creation_context)))),
    )
}
