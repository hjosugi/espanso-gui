use eframe::egui::{self, Color32, FontData, FontDefinitions, FontFamily, Stroke, Visuals};
use fontdb::{Database, Family, Query, Stretch, Style, Weight};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Appearance {
    #[default]
    System,
    Light,
    Dark,
}

impl Appearance {
    pub const ALL: [Self; 3] = [Self::System, Self::Light, Self::Dark];
}

pub fn apply_appearance(ctx: &egui::Context, appearance: Appearance) {
    ctx.set_theme(match appearance {
        Appearance::System => egui::ThemePreference::System,
        Appearance::Light => egui::ThemePreference::Light,
        Appearance::Dark => egui::ThemePreference::Dark,
    });
}

// Semantic color tokens. UI code should use these names instead of embedding RGB values so every
// state stays consistent and the contrast test below covers the complete application palette.
pub const INK: Color32 = Color32::from_rgb(37, 43, 48);
pub const MUTED: Color32 = Color32::from_rgb(94, 101, 101);
pub const PAPER: Color32 = Color32::from_rgb(247, 244, 237);
pub const PANEL: Color32 = Color32::from_rgb(255, 253, 248);
pub const SIDEBAR: Color32 = Color32::from_rgb(232, 239, 234);
pub const ACCENT: Color32 = Color32::from_rgb(25, 121, 102);
pub const ACCENT_SOFT: Color32 = Color32::from_rgb(211, 235, 226);
pub const AMBER: Color32 = Color32::from_rgb(160, 88, 20);
pub const DANGER: Color32 = Color32::from_rgb(177, 65, 58);
pub const INFO: Color32 = Color32::from_rgb(66, 103, 146);
pub const ON_ACCENT: Color32 = Color32::WHITE;
pub const SURFACE_MUTED: Color32 = Color32::from_rgb(241, 240, 234);
pub const SURFACE_RECESSED: Color32 = Color32::from_rgb(226, 229, 222);
pub const BORDER: Color32 = Color32::from_rgb(120, 129, 123);
pub const BORDER_SUBTLE: Color32 = Color32::from_rgb(220, 222, 214);
const WIDGET_INACTIVE: Color32 = Color32::from_rgb(238, 239, 233);
const WIDGET_HOVERED: Color32 = Color32::from_rgb(211, 235, 226);
const WIDGET_ACTIVE: Color32 = Color32::from_rgb(193, 226, 214);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Palette {
    pub ink: Color32,
    pub muted: Color32,
    pub paper: Color32,
    pub panel: Color32,
    pub sidebar: Color32,
    pub accent: Color32,
    pub accent_soft: Color32,
    pub amber: Color32,
    pub danger: Color32,
    pub info: Color32,
    pub on_action: Color32,
    pub surface_muted: Color32,
    pub surface_recessed: Color32,
    pub border: Color32,
    pub border_subtle: Color32,
    pub widget_inactive: Color32,
    pub widget_hovered: Color32,
    pub widget_active: Color32,
}

pub const LIGHT_PALETTE: Palette = Palette {
    ink: INK,
    muted: MUTED,
    paper: PAPER,
    panel: PANEL,
    sidebar: SIDEBAR,
    accent: ACCENT,
    accent_soft: ACCENT_SOFT,
    amber: AMBER,
    danger: DANGER,
    info: INFO,
    on_action: ON_ACCENT,
    surface_muted: SURFACE_MUTED,
    surface_recessed: SURFACE_RECESSED,
    border: BORDER,
    border_subtle: BORDER_SUBTLE,
    widget_inactive: WIDGET_INACTIVE,
    widget_hovered: WIDGET_HOVERED,
    widget_active: WIDGET_ACTIVE,
};

pub const DARK_PALETTE: Palette = Palette {
    ink: Color32::from_rgb(242, 240, 233),
    muted: Color32::from_rgb(184, 192, 188),
    paper: Color32::from_rgb(23, 27, 26),
    panel: Color32::from_rgb(32, 37, 35),
    sidebar: Color32::from_rgb(25, 36, 31),
    accent: Color32::from_rgb(111, 209, 183),
    accent_soft: Color32::from_rgb(36, 72, 63),
    amber: Color32::from_rgb(243, 181, 98),
    danger: Color32::from_rgb(255, 138, 128),
    info: Color32::from_rgb(140, 185, 238),
    on_action: Color32::from_rgb(18, 22, 21),
    surface_muted: Color32::from_rgb(37, 42, 40),
    surface_recessed: Color32::from_rgb(17, 21, 19),
    border: Color32::from_rgb(130, 145, 139),
    border_subtle: Color32::from_rgb(61, 71, 67),
    widget_inactive: Color32::from_rgb(42, 48, 45),
    widget_hovered: Color32::from_rgb(40, 74, 65),
    widget_active: Color32::from_rgb(50, 98, 86),
};

pub fn palette(ui: &egui::Ui) -> Palette {
    if ui.visuals().dark_mode {
        DARK_PALETTE
    } else {
        LIGHT_PALETTE
    }
}

// Typography and layout tokens. In particular, never fall back to egui's 11-point Small style;
// secondary text is still normal reading content in this application.
pub const TEXT_DISPLAY: f32 = 32.0;
pub const TEXT_SECTION: f32 = 24.0;
pub const TEXT_BODY: f32 = 20.0;
pub const TEXT_SMALL: f32 = 18.0;
pub const SPACE_XS: f32 = 4.0;
pub const SPACE_SM: f32 = 12.0;
pub const SPACE_MD: f32 = 16.0;
pub const SPACE_LG: f32 = 24.0;
pub const PADDING_COMPACT: i8 = 8;
pub const PADDING_SM: i8 = 12;
pub const PADDING_MD: i8 = 16;
pub const PADDING_LG: i8 = 24;
pub const PADDING_XL: i8 = 32;
pub const PADDING_2XL: i8 = 40;
pub const EMPTY_STATE_TOP_SPACE: f32 = PADDING_2XL as f32 + SPACE_SM;
pub const COMPACT_LAYOUT_BREAKPOINT: f32 = 1180.0;
pub const FIELD_STACK_BREAKPOINT: f32 = 660.0;
pub const CONTENT_MAX_WIDTH: f32 = 1040.0;
pub const CONTENT_PADDING: i8 = PADDING_XL;
pub const CONTROL_MIN_WIDTH: f32 = 40.0;
pub const CONTROL_HEIGHT: f32 = 48.0;
pub const CONTROL_ICON_SIZE: f32 = 24.0;
pub const CONTROL_ICON_INNER_SIZE: f32 = 12.0;
pub const CONTROL_ICON_GAP: f32 = 12.0;
pub const SLIDER_WIDTH: f32 = 180.0;
pub const SLIDER_RAIL_HEIGHT: f32 = 12.0;
pub const COMBO_WIDTH: f32 = 180.0;
pub const COMBO_POPUP_HEIGHT: f32 = 320.0;
pub const LIST_ROW_HEIGHT: f32 = 68.0;
pub const SCROLL_BAR_WIDTH: f32 = 12.0;
pub const SCROLL_HANDLE_MIN_LENGTH: f32 = 48.0;
pub const SCROLL_BAR_MARGIN: f32 = 8.0;
pub const TEXT_EDIT_DEFAULT_WIDTH: f32 = 320.0;
pub const FIELD_ROW_MIN_HEIGHT: f32 = 68.0;
pub const FIELD_LABEL_WIDTH: f32 = 340.0;
pub const FIELD_CONTROL_MAX_WIDTH: f32 = 380.0;
pub const FIELD_NAME_WIDTH: f32 = 160.0;
// Keep enough horizontal room for the longest label/shortcut pair with the
// different system-font metrics used by Linux, macOS, and Windows.
pub const NAVIGATION_WIDTH: f32 = 344.0;
// `CONTROL_HEIGHT` is a minimum: a platform font can make a button taller.
// Reserve a full cross-platform footer block so the bounded file list cannot
// push the version, Settings, or About surfaces below the viewport.
pub const NAVIGATION_FOOTER_HEIGHT: f32 = 200.0;
pub const COLLECTION_PANEL_MIN_WIDTH: f32 = 200.0;
pub const COMPACT_DETAIL_MIN_WIDTH: f32 = 320.0;
pub const COMPACT_SECTION_SELECTOR_WIDTH: f32 = 252.0;
pub const COMPACT_FILE_SELECTOR_WIDTH: f32 = 248.0;
pub const SNIPPET_LIST_COMPACT_WIDTH: f32 = 280.0;
pub const SNIPPET_LIST_WIDTH: f32 = 380.0;
pub const PROFILE_LIST_COMPACT_WIDTH: f32 = 240.0;
pub const PROFILE_LIST_WIDTH: f32 = 320.0;
pub const IMAGE_PREVIEW_MAX_SIZE: [f32; 2] = [520.0, 320.0];
pub const MODAL_VIEWPORT_GUTTER: f32 = 32.0;
pub const MODAL_MIN_WIDTH: f32 = 240.0;
pub const MODAL_MIN_HEIGHT: f32 = 180.0;
pub const MODAL_WIDTH_SM: f32 = 360.0;
pub const MODAL_WIDTH_MD: f32 = 440.0;
pub const MODAL_WIDTH_LG: f32 = 500.0;
pub const MODAL_WIDTH_XL: f32 = 560.0;
pub const MODAL_WIDTH_WIDE: f32 = 760.0;
pub const MODAL_HEIGHT_TALL: f32 = 560.0;
pub const CONFLICT_LIST_MAX_HEIGHT: f32 = 390.0;
pub const DEFAULT_WINDOW_SIZE: [f32; 2] = [1440.0, 900.0];
pub const MINIMUM_WINDOW_SIZE: [f32; 2] = [1080.0, 720.0];
pub const STROKE_STANDARD: f32 = 1.0;
pub const STROKE_LABEL_EMPHASIS: f32 = 1.2;
pub const STROKE_SELECTION: f32 = 1.5;
pub const STROKE_FOCUS: f32 = 2.0;
pub const SELECTION_INDICATOR_INSET: f32 = 2.0;
pub const SELECTION_INDICATOR_WIDTH: f32 = 4.0;
pub const TINT_BADGE: f32 = 0.12;
pub const TINT_CALLOUT: f32 = 0.10;
pub const TINT_CALLOUT_BORDER: f32 = 0.45;
pub const DISABLED_OPACITY: f32 = 0.85;
pub const UI_SCALE_MIN: f32 = 0.8;
pub const UI_SCALE_MAX: f32 = 2.0;
pub const UI_SCALE_STEP: f64 = 0.01;
pub const RADIUS_CONTROL: u8 = 7;
pub const RADIUS_CALLOUT: u8 = 8;
pub const RADIUS_CARD: u8 = 9;
pub const RADIUS_BADGE: u8 = 10;
pub const RADIUS_WINDOW: u8 = 12;

pub fn install(ctx: &egui::Context) {
    install_system_font(ctx);
    configure_theme(ctx, egui::Theme::Light, LIGHT_PALETTE);
    configure_theme(ctx, egui::Theme::Dark, DARK_PALETTE);
}

fn configure_theme(ctx: &egui::Context, theme: egui::Theme, palette: Palette) {
    ctx.style_mut_of(theme, |style| {
        let mut visuals = match theme {
            egui::Theme::Dark => Visuals::dark(),
            egui::Theme::Light => Visuals::light(),
        };
        // Leave this unset so selected widgets can use `selection.stroke` as their foreground.
        // Ordinary widget states below still resolve to the semantic ink token.
        visuals.override_text_color = None;
        visuals.weak_text_color = Some(palette.muted);
        visuals.panel_fill = palette.paper;
        visuals.window_fill = palette.panel;
        visuals.faint_bg_color = palette.surface_muted;
        visuals.extreme_bg_color = palette.surface_recessed;
        visuals.code_bg_color = palette.surface_muted;
        visuals.hyperlink_color = palette.accent;
        visuals.warn_fg_color = palette.amber;
        visuals.error_fg_color = palette.danger;
        visuals.disabled_alpha = DISABLED_OPACITY;
        // Selection must remain unmistakable without relying on a subtle tint alone. The
        // foreground/background pair is contrast-tested for both themes below.
        visuals.selection.bg_fill = palette.accent;
        visuals.selection.stroke = Stroke::new(STROKE_SELECTION, palette.on_action);
        visuals.widgets.noninteractive.bg_fill = palette.panel;
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(STROKE_STANDARD, palette.ink);
        visuals.widgets.inactive.bg_fill = palette.widget_inactive;
        visuals.widgets.inactive.weak_bg_fill = palette.widget_inactive;
        visuals.widgets.inactive.bg_stroke = Stroke::new(STROKE_STANDARD, palette.border);
        visuals.widgets.inactive.fg_stroke = Stroke::new(STROKE_STANDARD, palette.ink);
        visuals.widgets.hovered.bg_fill = palette.widget_hovered;
        visuals.widgets.hovered.weak_bg_fill = palette.widget_hovered;
        visuals.widgets.hovered.bg_stroke = Stroke::new(STROKE_STANDARD, palette.accent);
        visuals.widgets.hovered.fg_stroke = Stroke::new(STROKE_LABEL_EMPHASIS, palette.ink);
        visuals.widgets.active.bg_fill = palette.widget_active;
        visuals.widgets.active.weak_bg_fill = palette.widget_active;
        visuals.widgets.active.bg_stroke = Stroke::new(STROKE_FOCUS, palette.accent);
        visuals.widgets.active.fg_stroke = Stroke::new(STROKE_LABEL_EMPHASIS, palette.ink);
        visuals.widgets.open.bg_fill = palette.accent_soft;
        visuals.widgets.open.bg_stroke = Stroke::new(STROKE_STANDARD, palette.accent);
        visuals.window_stroke = Stroke::new(STROKE_STANDARD, palette.border);
        visuals.window_corner_radius = egui::CornerRadius::same(RADIUS_WINDOW);
        style.visuals = visuals;
        style.spacing.item_spacing = egui::vec2(SPACE_MD, SPACE_SM);
        style.spacing.window_margin = egui::Margin::same(PADDING_LG);
        style.spacing.menu_margin = egui::Margin::same(PADDING_SM);
        style.spacing.button_padding = egui::vec2(PADDING_MD as f32, PADDING_SM as f32);
        style.spacing.indent = PADDING_LG as f32;
        style.spacing.interact_size = egui::vec2(CONTROL_MIN_WIDTH, CONTROL_HEIGHT);
        style.spacing.slider_width = SLIDER_WIDTH;
        style.spacing.slider_rail_height = SLIDER_RAIL_HEIGHT;
        style.spacing.combo_width = COMBO_WIDTH;
        style.spacing.combo_height = COMBO_POPUP_HEIGHT;
        style.spacing.extra_text_line_spacing = SPACE_XS;
        style.spacing.icon_width = CONTROL_ICON_SIZE;
        style.spacing.icon_width_inner = CONTROL_ICON_INNER_SIZE;
        style.spacing.icon_spacing = CONTROL_ICON_GAP;
        style.spacing.scroll = egui::style::ScrollStyle::solid();
        style.spacing.scroll.bar_width = SCROLL_BAR_WIDTH;
        style.spacing.scroll.handle_min_length = SCROLL_HANDLE_MIN_LENGTH;
        style.spacing.scroll.bar_inner_margin = SCROLL_BAR_MARGIN;
        style.spacing.scroll.foreground_color = true;
        style.spacing.text_edit_width = TEXT_EDIT_DEFAULT_WIDTH;
        style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(RADIUS_CONTROL);
        style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(RADIUS_CONTROL);
        style.visuals.widgets.active.corner_radius = egui::CornerRadius::same(RADIUS_CONTROL);
        style.visuals.widgets.open.corner_radius = egui::CornerRadius::same(RADIUS_CONTROL);
        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::new(TEXT_SECTION, FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(TEXT_BODY, FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(TEXT_BODY, FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Monospace,
            egui::FontId::new(TEXT_BODY, FontFamily::Monospace),
        );
        style.text_styles.insert(
            egui::TextStyle::Small,
            egui::FontId::new(TEXT_SMALL, FontFamily::Proportional),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn relative_luminance(color: Color32) -> f32 {
        fn channel(value: u8) -> f32 {
            let value = f32::from(value) / 255.0;
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * channel(color.r()) + 0.7152 * channel(color.g()) + 0.0722 * channel(color.b())
    }

    fn contrast_ratio(left: Color32, right: Color32) -> f32 {
        let (lighter, darker) = {
            let left = relative_luminance(left);
            let right = relative_luminance(right);
            if left >= right {
                (left, right)
            } else {
                (right, left)
            }
        };
        (lighter + 0.05) / (darker + 0.05)
    }

    fn composite_over(foreground: Color32, background: Color32) -> Color32 {
        egui::Rgba::from(background)
            .blend(egui::Rgba::from(foreground))
            .into()
    }

    fn production_source(source: &str) -> &str {
        source.split("\n#[cfg(test)]").next().unwrap_or(source)
    }

    fn argument_starts_with_number(argument: &str) -> bool {
        let argument = argument.trim();
        if argument
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_digit())
        {
            return true;
        }
        argument
            .strip_prefix('[')
            .and_then(|argument| argument.strip_suffix(']'))
            .is_some_and(arguments_contain_number)
    }

    fn arguments_contain_number(arguments: &str) -> bool {
        let mut start = 0;
        let mut depth = 0;
        for (index, character) in arguments.char_indices() {
            match character {
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => depth -= 1,
                ',' if depth == 0 => {
                    if argument_starts_with_number(&arguments[start..index]) {
                        return true;
                    }
                    start = index + character.len_utf8();
                }
                _ => {}
            }
        }
        argument_starts_with_number(&arguments[start..])
    }

    fn assert_call_uses_tokens(source: &str, call: &str) {
        for remainder in source.split(call).skip(1) {
            let mut depth = 0;
            let mut end = None;
            for (index, character) in remainder.char_indices() {
                match character {
                    '(' | '[' | '{' => depth += 1,
                    ')' if depth == 0 => {
                        end = Some(index);
                        break;
                    }
                    ')' | ']' | '}' => depth -= 1,
                    _ => {}
                }
            }
            let arguments = &remainder[..end.expect("complete UI call")];
            assert!(
                !arguments_contain_number(arguments),
                "literal design value found in {call}{arguments})"
            );
        }
    }

    #[test]
    fn text_palette_meets_wcag_aa_contrast_on_primary_surfaces() {
        for palette in [LIGHT_PALETTE, DARK_PALETTE] {
            for foreground in [
                palette.ink,
                palette.muted,
                palette.accent,
                palette.amber,
                palette.danger,
                palette.info,
            ] {
                for background in [palette.paper, palette.panel, palette.sidebar] {
                    assert!(
                        contrast_ratio(foreground, background) >= 4.5,
                        "{foreground:?} does not meet WCAG AA on {background:?}"
                    );
                }
            }
            assert!(contrast_ratio(palette.on_action, palette.accent) >= 4.5);
            assert!(contrast_ratio(palette.on_action, palette.danger) >= 4.5);
            assert!(contrast_ratio(palette.ink, palette.accent_soft) >= 4.5);
            for background in [
                palette.surface_muted,
                palette.surface_recessed,
                palette.widget_inactive,
            ] {
                assert!(
                    contrast_ratio(palette.muted, background) >= 4.5,
                    "supporting and placeholder text does not meet WCAG AA on {background:?}"
                );
            }
            let disabled_text = composite_over(
                palette.ink.gamma_multiply(DISABLED_OPACITY),
                palette.widget_inactive,
            );
            let disabled_contrast = contrast_ratio(disabled_text, palette.widget_inactive);
            assert!(
                disabled_contrast >= 4.5,
                "disabled text contrast is only {disabled_contrast:.2}:1 on the inactive surface"
            );
            for surface in [
                palette.paper,
                palette.panel,
                palette.sidebar,
                palette.accent_soft,
                palette.widget_inactive,
                palette.widget_hovered,
                palette.widget_active,
            ] {
                assert!(
                    contrast_ratio(palette.accent, surface) >= 3.0,
                    "focus outline does not meet 3:1 on {surface:?}"
                );
            }
            for background in [palette.paper, palette.panel, palette.sidebar] {
                assert!(contrast_ratio(palette.border, background) >= 3.0);
                for semantic_color in [palette.accent, palette.amber, palette.danger, palette.info]
                {
                    for tint in [TINT_BADGE, TINT_CALLOUT] {
                        let tinted_surface =
                            composite_over(semantic_color.gamma_multiply(tint), background);
                        assert!(
                            contrast_ratio(palette.ink, tinted_surface) >= 4.5,
                            "callout text does not meet WCAG AA on {tinted_surface:?}"
                        );
                    }
                }
            }
            for control_surface in [palette.surface_recessed, palette.widget_inactive] {
                assert!(
                    contrast_ratio(palette.border, control_surface) >= 3.0,
                    "control border does not meet 3:1 on {control_surface:?}"
                );
            }
        }
    }

    #[test]
    fn secondary_text_never_falls_back_to_tiny_default_sizes() {
        let context = egui::Context::default();
        install(&context);
        let mut configured_sizes = [TEXT_DISPLAY, TEXT_SECTION, TEXT_BODY, TEXT_SMALL]
            .map(f32::to_bits)
            .to_vec();
        configured_sizes.sort_unstable();
        configured_sizes.dedup();
        assert_eq!(configured_sizes.len(), 4);

        for theme in [egui::Theme::Light, egui::Theme::Dark] {
            let style = context.style_of(theme);
            for (text_style, minimum) in [
                (egui::TextStyle::Body, 20.0),
                (egui::TextStyle::Button, 20.0),
                (egui::TextStyle::Monospace, 20.0),
                (egui::TextStyle::Small, 18.0),
            ] {
                assert!(style.text_styles[&text_style].size >= minimum);
            }
            assert_eq!(
                style.text_styles[&egui::TextStyle::Heading].size,
                TEXT_SECTION
            );

            let mut sizes = style
                .text_styles
                .values()
                .map(|font| font.size.to_bits())
                .collect::<Vec<_>>();
            sizes.sort_unstable();
            sizes.dedup();
            assert!(sizes.len() <= 4, "found more than four text sizes");
        }
    }

    #[test]
    fn controls_and_insets_keep_comfortable_shared_dimensions() {
        let context = egui::Context::default();
        install(&context);

        for theme in [egui::Theme::Light, egui::Theme::Dark] {
            let style = context.style_of(theme);
            assert!(style.spacing.button_padding.x >= 16.0);
            assert!(style.spacing.button_padding.y >= 12.0);
            assert!(style.spacing.item_spacing.x >= 16.0);
            assert!(style.spacing.item_spacing.y >= 12.0);
            assert!(style.spacing.interact_size.x >= 40.0);
            assert!(style.spacing.interact_size.y >= 48.0);
            assert!(style.spacing.window_margin.left >= 24);
            assert!(style.spacing.menu_margin.left >= 12);
            assert!(style.spacing.extra_text_line_spacing >= 4.0);
            assert!(style.spacing.icon_width >= 24.0);
            assert!(style.spacing.icon_spacing >= 12.0);
            assert!(style.spacing.slider_width >= 180.0);
            assert!(style.spacing.slider_rail_height >= 12.0);
            assert!(style.spacing.combo_width >= 180.0);
            assert!(style.spacing.combo_height >= 320.0);
            assert!(!style.spacing.scroll.floating);
            assert!(style.spacing.scroll.foreground_color);
            assert!(style.spacing.scroll.bar_width >= 12.0);
            assert!(style.spacing.scroll.handle_min_length >= 48.0);
            assert!(style.spacing.scroll.bar_inner_margin >= 8.0);
            assert_eq!(style.spacing.button_padding.x % 4.0, 0.0);
            assert_eq!(style.spacing.button_padding.y % 4.0, 0.0);
        }
        const {
            assert!(CONTENT_PADDING >= 32);
            assert!(PADDING_LG >= 24);
            assert!(LIST_ROW_HEIGHT >= 68.0);
            assert!(FIELD_ROW_MIN_HEIGHT >= 68.0);
        }
        for value in [
            PADDING_COMPACT,
            PADDING_SM,
            PADDING_MD,
            PADDING_LG,
            PADDING_XL,
            PADDING_2XL,
        ] {
            assert_eq!(value % 4, 0, "padding token {value} is off the shared grid");
        }

        for source in [
            include_str!("app.rs"),
            include_str!("navigation.rs"),
            include_str!("profile_editor.rs"),
            include_str!("settings_editor.rs"),
            include_str!("snippet_editor.rs"),
            include_str!("top_bar.rs"),
            include_str!("ui_components.rs"),
            include_str!("variable_editor.rs"),
            include_str!("yaml_editor.rs"),
        ] {
            assert!(
                !production_source(source).contains(".small_button("),
                "compact buttons bypass the shared readable control treatment"
            );
        }
    }

    #[test]
    fn semantic_palette_follows_the_selected_theme() {
        let context = egui::Context::default();
        install(&context);

        for (appearance, expected) in [
            (Appearance::Light, LIGHT_PALETTE),
            (Appearance::Dark, DARK_PALETTE),
        ] {
            apply_appearance(&context, appearance);
            let theme = match appearance {
                Appearance::System => unreachable!("system appearance is not in this fixture"),
                Appearance::Light => egui::Theme::Light,
                Appearance::Dark => egui::Theme::Dark,
            };
            let style = context.style_of(theme);
            assert_eq!(style.visuals.selection.bg_fill, expected.accent);
            assert_eq!(style.visuals.selection.stroke.color, expected.on_action);
            let mut actual = None;
            let mut output = context.run_ui(egui::RawInput::default(), |ui| {
                actual = Some(palette(ui));
            });
            output.textures_delta.clear();
            assert_eq!(actual, Some(expected));
        }
    }

    #[test]
    fn ui_code_uses_design_tokens_instead_of_visual_literals() {
        for source in [
            include_str!("app.rs"),
            include_str!("html_editor.rs"),
            include_str!("navigation.rs"),
            include_str!("preferences.rs"),
            include_str!("profile_editor.rs"),
            include_str!("settings_editor.rs"),
            include_str!("snippet_editor.rs"),
            include_str!("snippet_library.rs"),
            include_str!("top_bar.rs"),
            include_str!("ui_components.rs"),
            include_str!("variable_editor.rs"),
            include_str!("yaml_editor.rs"),
        ] {
            let source = production_source(source);
            for call in [
                ".size(",
                "FontId::new(",
                "FontId::monospace(",
                "add_space(",
                "Margin::same(",
                "Margin::symmetric(",
                ".exact_size(",
                ".max_height(",
                ".min_scrolled_height(",
                ".max_size(",
                ".set_min_height(",
                ".set_min_size(",
                ".set_width(",
                ".desired_width(",
                ".add_sized(",
                "set_responsive_modal_width(",
                "set_responsive_modal_size(",
                "Stroke::new(",
                "egui::vec2(",
                "egui::pos2(",
                ".gamma_multiply(",
            ] {
                assert_call_uses_tokens(source, call);
            }
        }
    }

    #[test]
    fn text_fields_use_the_shared_comfortable_insets() {
        for source in [
            include_str!("app.rs"),
            include_str!("html_editor.rs"),
            include_str!("navigation.rs"),
            include_str!("profile_editor.rs"),
            include_str!("settings_editor.rs"),
            include_str!("snippet_editor.rs"),
            include_str!("top_bar.rs"),
            include_str!("variable_editor.rs"),
            include_str!("yaml_editor.rs"),
        ] {
            assert!(
                !production_source(source).contains("TextEdit::singleline"),
                "single-line fields must use ui_components::singleline_text_edit"
            );
            assert!(
                !production_source(source).contains("TextEdit::multiline"),
                "multi-line fields must use ui_components::multiline_text_edit"
            );
        }
    }
}
