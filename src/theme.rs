use eframe::egui::{self, Color32, FontData, FontDefinitions, FontFamily, Stroke, Visuals};
use fontdb::{Database, Family, Query, Stretch, Style, Weight};
use std::sync::Arc;

pub const INK: Color32 = Color32::from_rgb(37, 43, 48);
pub const MUTED: Color32 = Color32::from_rgb(105, 112, 112);
pub const PAPER: Color32 = Color32::from_rgb(247, 244, 237);
pub const PANEL: Color32 = Color32::from_rgb(255, 253, 248);
pub const SIDEBAR: Color32 = Color32::from_rgb(232, 239, 234);
pub const ACCENT: Color32 = Color32::from_rgb(25, 121, 102);
pub const ACCENT_SOFT: Color32 = Color32::from_rgb(211, 235, 226);
pub const AMBER: Color32 = Color32::from_rgb(195, 120, 35);
pub const DANGER: Color32 = Color32::from_rgb(177, 65, 58);

pub fn install(ctx: &egui::Context) {
    install_system_font(ctx);
    let mut visuals = Visuals::light();
    visuals.override_text_color = Some(INK);
    visuals.panel_fill = PAPER;
    visuals.window_fill = PANEL;
    visuals.faint_bg_color = Color32::from_rgb(241, 240, 234);
    visuals.extreme_bg_color = Color32::from_rgb(226, 229, 222);
    visuals.selection.bg_fill = ACCENT;
    visuals.selection.stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.widgets.noninteractive.bg_fill = PANEL;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, INK);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(238, 239, 233);
    visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(238, 239, 233);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, INK);
    visuals.widgets.hovered.bg_fill = ACCENT_SOFT;
    visuals.widgets.hovered.weak_bg_fill = ACCENT_SOFT;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.2, INK);
    visuals.widgets.active.bg_fill = Color32::from_rgb(193, 226, 214);
    visuals.widgets.active.weak_bg_fill = Color32::from_rgb(193, 226, 214);
    visuals.widgets.active.fg_stroke = Stroke::new(1.2, INK);
    visuals.widgets.open.bg_fill = ACCENT_SOFT;
    visuals.window_stroke = Stroke::new(1.0, Color32::from_rgb(203, 207, 198));
    visuals.window_corner_radius = egui::CornerRadius::same(12);
    ctx.set_visuals(visuals);

    ctx.style_mut_of(egui::Theme::Light, |style| {
        style.spacing.item_spacing = egui::vec2(10.0, 8.0);
        style.spacing.button_padding = egui::vec2(12.0, 7.0);
        style.spacing.interact_size.y = 34.0;
        style.spacing.text_edit_width = 260.0;
        style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(7);
        style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(7);
        style.visuals.widgets.active.corner_radius = egui::CornerRadius::same(7);
        style.visuals.widgets.open.corner_radius = egui::CornerRadius::same(7);
        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::new(24.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(15.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(14.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Monospace,
            egui::FontId::new(14.0, FontFamily::Monospace),
        );
        style.url_in_tooltip = true;
    });
}

fn install_system_font(ctx: &egui::Context) {
    let mut database = Database::new();
    database.load_system_fonts();
    let candidates = [
        "Noto Sans CJK JP",
        "Noto Sans JP",
        "Yu Gothic UI",
        "Yu Gothic",
        "Hiragino Sans",
        "Meiryo",
        "IPA Gothic",
        "DejaVu Sans",
    ];
    let query = Query {
        families: &candidates.map(Family::Name),
        weight: Weight::NORMAL,
        stretch: Stretch::Normal,
        style: Style::Normal,
    };
    let Some(id) = database.query(&query) else {
        return;
    };
    let Some(data) = database.with_face_data(id, |data, _| data.to_vec()) else {
        return;
    };
    let mut fonts = FontDefinitions::default();
    fonts
        .font_data
        .insert("system-ui".into(), Arc::new(FontData::from_owned(data)));
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "system-ui".into());
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .push("system-ui".into());
    ctx.set_fonts(fonts);
}
