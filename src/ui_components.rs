use crate::theme;
use eframe::egui::{
    self, Align, Button, Color32, FontFamily, Frame, Id, Layout, Margin, Modal, RichText,
    ScrollArea, Sense, Stroke, TextBuffer, TextEdit, Ui,
};
use std::path::Path;

pub(crate) fn compact_layout(logical_width: f32) -> bool {
    logical_width < theme::COMPACT_LAYOUT_BREAKPOINT
}

pub(crate) fn compact_collection_width(logical_width: f32, preferred_width: f32) -> f32 {
    preferred_width.min(
        (logical_width - theme::COMPACT_DETAIL_MIN_WIDTH).max(theme::COLLECTION_PANEL_MIN_WIDTH),
    )
}

pub(crate) fn responsive_modal_width(logical_width: f32, preferred_width: f32) -> f32 {
    responsive_modal_extent(logical_width, preferred_width, theme::MODAL_MIN_WIDTH)
}

fn responsive_modal_extent(viewport: f32, preferred: f32, minimum: f32) -> f32 {
    let available = (viewport - theme::MODAL_VIEWPORT_GUTTER).max(0.0);
    preferred.min(available).max(minimum.min(available))
}

pub(crate) fn responsive_modal_size(viewport: egui::Vec2, preferred: egui::Vec2) -> egui::Vec2 {
    egui::vec2(
        responsive_modal_extent(viewport.x, preferred.x, theme::MODAL_MIN_WIDTH),
        responsive_modal_extent(viewport.y, preferred.y, theme::MODAL_MIN_HEIGHT),
    )
}

pub(crate) fn centered_content_rect(available: egui::Rect) -> egui::Rect {
    let width = available.width().min(theme::CONTENT_MAX_WIDTH);
    let left = available.center().x - width / 2.0;
    egui::Rect::from_min_max(
        egui::pos2(left, available.top()),
        egui::pos2(left + width, available.bottom()),
    )
}

pub(crate) fn centered_content_panel(
    ui: &mut Ui,
    scroll_id: &'static str,
    content: impl FnOnce(&mut Ui),
) {
    egui::CentralPanel::default()
        .frame(
            Frame::new()
                .fill(theme::palette(ui).paper)
                .inner_margin(Margin::same(theme::CONTENT_PADDING)),
        )
        .show(ui, |ui| {
            ScrollArea::vertical()
                .id_salt(scroll_id)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let rect = centered_content_rect(ui.available_rect_before_wrap());
                    ui.scope_builder(
                        egui::UiBuilder::new()
                            .max_rect(rect)
                            .layout(Layout::top_down(Align::LEFT)),
                        content,
                    );
                });
        });
}

pub(crate) fn show_modal(
    ui: &Ui,
    id: &'static str,
    title: &str,
    content: impl FnOnce(&mut Ui),
) -> bool {
    let modal_height = (ui.ctx().content_rect().height() - theme::MODAL_VIEWPORT_GUTTER).max(0.0);
    let response = Modal::new(Id::new(id)).show(ui.ctx(), |ui| {
        ui.push_id((id, "dialog"), |ui| {
            ui.set_max_height(modal_height);
            section_heading(ui, title);
            ui.separator();
            let content_height = ui.available_height();
            ScrollArea::vertical()
                .id_salt((id, "content"))
                .max_height(content_height)
                .auto_shrink([false, true])
                .show(ui, content);
        })
        .response
        .id
    });
    ui.ctx().accesskit_node_builder(response.inner, |node| {
        node.set_role(egui::accesskit::Role::Dialog);
        node.set_label(title);
        node.set_modal();
    });
    response.should_close()
}

pub(crate) fn set_responsive_modal_width(ui: &mut Ui, preferred_width: f32) {
    ui.set_width(responsive_modal_width(
        ui.ctx().content_rect().width(),
        preferred_width,
    ));
}

pub(crate) fn set_responsive_modal_size(ui: &mut Ui, preferred_width: f32, preferred_height: f32) {
    let size = responsive_modal_size(
        ui.ctx().content_rect().size(),
        egui::vec2(preferred_width, preferred_height),
    );
    ui.set_width(size.x);
    ui.set_min_height(size.y);
}

pub(crate) fn badge(ui: &mut Ui, text: &str, color: Color32) {
    Frame::new()
        .fill(color.gamma_multiply(theme::TINT_BADGE))
        .corner_radius(theme::RADIUS_BADGE)
        .inner_margin(Margin::symmetric(theme::PADDING_MD, theme::PADDING_SM))
        .show(ui, |ui| {
            ui.label(
                RichText::new(text)
                    .small()
                    .color(theme::palette(ui).ink)
                    .strong(),
            );
        });
}

fn semantic_heading(ui: &mut Ui, text: &str, size: f32, level: usize) -> egui::Response {
    let response = ui.heading(RichText::new(text).size(size).strong());
    ui.ctx().accesskit_node_builder(response.id, |node| {
        node.set_role(egui::accesskit::Role::Heading);
        node.set_level(level);
    });
    response
}

pub(crate) fn display_heading(ui: &mut Ui, text: &str) -> egui::Response {
    semantic_heading(ui, text, theme::TEXT_DISPLAY, 1)
}

pub(crate) fn section_heading(ui: &mut Ui, text: &str) -> egui::Response {
    semantic_heading(ui, text, theme::TEXT_SECTION, 2)
}

fn filled_action_button(ui: &Ui, label: impl Into<String>, fill: Color32) -> Button<'static> {
    Button::new(
        RichText::new(label.into())
            .color(theme::palette(ui).on_action)
            .strong(),
    )
    .fill(fill)
    .stroke(Stroke::NONE)
}

pub(crate) fn primary_button(ui: &Ui, label: impl Into<String>) -> Button<'static> {
    filled_action_button(ui, label, theme::palette(ui).accent)
}

pub(crate) fn danger_button(ui: &Ui, label: impl Into<String>) -> Button<'static> {
    filled_action_button(ui, label, theme::palette(ui).danger)
}

pub(crate) fn singleline_text_edit<'a>(text: &'a mut dyn TextBuffer) -> TextEdit<'a> {
    TextEdit::singleline(text)
        .min_size(egui::vec2(theme::CONTROL_MIN_WIDTH, theme::CONTROL_HEIGHT))
        .margin(Margin::symmetric(theme::PADDING_MD, theme::PADDING_SM))
}

pub(crate) fn multiline_text_edit<'a>(text: &'a mut dyn TextBuffer) -> TextEdit<'a> {
    TextEdit::multiline(text)
        .min_size(egui::vec2(theme::CONTROL_MIN_WIDTH, theme::CONTROL_HEIGHT))
        .margin(Margin::symmetric(theme::PADDING_MD, theme::PADDING_SM))
}

pub(crate) fn wrapped_path_label(ui: &mut Ui, path: &Path) -> egui::Response {
    ui.add(
        egui::Label::new(
            RichText::new(path.to_string_lossy())
                .family(FontFamily::Monospace)
                .color(theme::palette(ui).muted),
        )
        .wrap()
        .selectable(true),
    )
}

pub(crate) fn context_row_button(
    ui: &mut Ui,
    visible_label: &str,
    context: &str,
) -> egui::Response {
    // Contextual row actions still need the same readable type and touch target as every other
    // action. `Ui::small_button` bypasses the shared button padding and made these controls the
    // last visibly undersized elements in otherwise regular rows.
    let response = ui.add(
        Button::new(visible_label)
            .min_size(egui::vec2(theme::CONTROL_MIN_WIDTH, theme::CONTROL_HEIGHT)),
    );
    label_button_with_context(response, visible_label, context)
}

pub(crate) fn context_button_enabled(
    ui: &mut Ui,
    enabled: bool,
    visible_label: &str,
    context: &str,
) -> egui::Response {
    let response = ui.add_enabled(enabled, Button::new(visible_label));
    label_button_with_context(response, visible_label, context)
}

fn label_button_with_context(
    response: egui::Response,
    visible_label: &str,
    context: &str,
) -> egui::Response {
    let enabled = response.enabled();
    let accessible_label = format!("{visible_label}: {context}");
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, accessible_label.clone())
    });
    response
}

pub(crate) fn context_selectable_value<Value: PartialEq>(
    ui: &mut Ui,
    current_value: &mut Value,
    selected_value: Value,
    visible_label: &str,
    context: &str,
) -> egui::Response {
    let selected = *current_value == selected_value;
    let response = ui.add(
        Button::new(selected_option_label(visible_label, selected))
            .selected(selected)
            .frame(true)
            .min_size(egui::vec2(theme::CONTROL_MIN_WIDTH, theme::CONTROL_HEIGHT)),
    );
    if response.clicked() {
        *current_value = selected_value;
    }
    let enabled = response.enabled();
    let accessible_label = format!("{visible_label}: {context}");
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::Button,
            enabled,
            selected,
            accessible_label.clone(),
        )
    });
    response
}

pub(crate) fn unambiguous_selectable_value<Value: PartialEq>(
    ui: &mut Ui,
    current_value: &mut Value,
    selected_value: Value,
    label: &str,
) -> egui::Response {
    let selected = *current_value == selected_value;
    let response = ui.add(
        Button::new(selected_option_label(label, selected))
            .selected(selected)
            .frame(true)
            .min_size(egui::vec2(theme::CONTROL_MIN_WIDTH, theme::CONTROL_HEIGHT)),
    );
    if response.clicked() {
        *current_value = selected_value;
    }
    let enabled = response.enabled();
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Button, enabled, selected, label)
    });
    response
}

fn selected_option_label(label: &str, selected: bool) -> String {
    if selected {
        format!("{label}  ✓")
    } else {
        label.to_owned()
    }
}

pub(crate) fn selection_list_row(
    ui: &mut Ui,
    label: impl Into<String>,
    selected: bool,
) -> egui::Response {
    let accessible_label = label.into();
    let mut label = RichText::new(accessible_label.clone());
    if selected {
        label = label.color(theme::palette(ui).on_action).strong();
    }
    let trailing = if selected {
        RichText::new("✓")
            .color(theme::palette(ui).on_action)
            .strong()
    } else {
        RichText::new("")
    };
    let response = ui.add_sized(
        [ui.available_width(), theme::LIST_ROW_HEIGHT],
        Button::new(label).right_text(trailing).selected(selected),
    );
    let enabled = response.enabled();
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::Button,
            enabled,
            selected,
            accessible_label.clone(),
        )
    });
    response
}

pub(crate) fn modal_actions(ui: &mut Ui, content: impl FnOnce(&mut Ui)) {
    ui.with_layout(Layout::right_to_left(Align::Center), content);
}

pub(crate) fn responsive_detail_actions(
    ui: &mut Ui,
    details: impl FnOnce(&mut Ui),
    actions: impl FnOnce(&mut Ui),
) {
    if ui.available_width() < theme::FIELD_STACK_BREAKPOINT {
        ui.vertical(|ui| {
            ui.horizontal_wrapped(details);
            ui.add_space(theme::SPACE_SM);
            ui.horizontal_wrapped(actions);
        });
    } else {
        ui.horizontal(|ui| {
            details(ui);
            ui.with_layout(
                Layout::left_to_right(Align::Center).with_main_align(Align::Max),
                actions,
            );
        });
    }
}

pub(crate) fn labelled_two_column_field(
    ui: &mut Ui,
    label: &str,
    description: &str,
    content: impl FnOnce(&mut Ui, Id),
) {
    if ui.available_width() < theme::FIELD_STACK_BREAKPOINT {
        ui.vertical(|ui| {
            let label_id = ui.label(RichText::new(label).strong()).id;
            content(ui, label_id);
            if !description.is_empty() {
                ui.label(
                    RichText::new(description)
                        .small()
                        .color(theme::palette(ui).muted),
                );
            }
        });
    } else {
        ui.horizontal(|ui| {
            ui.set_min_height(theme::FIELD_ROW_MIN_HEIGHT);
            let label_id = ui
                .vertical(|ui| {
                    ui.set_width(theme::FIELD_LABEL_WIDTH);
                    let label = ui.label(RichText::new(label).strong());
                    if !description.is_empty() {
                        ui.label(
                            RichText::new(description)
                                .small()
                                .color(theme::palette(ui).muted),
                        );
                    }
                    label.id
                })
                .inner;
            ui.vertical(|ui| content(ui, label_id));
        });
    }
}

pub(crate) fn snippet_card(
    ui: &mut Ui,
    selected: bool,
    title: &str,
    triggers: &str,
    preview: &str,
    kind: &str,
) -> egui::Response {
    let palette = theme::palette(ui);
    let fill = if selected {
        palette.accent_soft
    } else {
        palette.panel
    };
    // The regular accent and muted colors are readable on primary panels, but are not guaranteed
    // to meet normal-text contrast on the tinted selection surface. Selected-card text therefore
    // uses the contrast-tested ink token while the edge and frame continue to carry accent state.
    let (trigger_color, secondary_color) = snippet_card_text_colors(palette, selected);
    let frame_response = Frame::new()
        .fill(fill)
        .stroke(Stroke::new(
            theme::STROKE_STANDARD,
            if selected {
                palette.accent
            } else {
                palette.border
            },
        ))
        .corner_radius(theme::RADIUS_CARD)
        .inner_margin(Margin::same(theme::PADDING_MD))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.add(
                egui::Label::new(RichText::new(title).strong())
                    .wrap()
                    .halign(Align::LEFT),
            );
            ui.horizontal_wrapped(|ui| {
                if !triggers.is_empty() {
                    let mut trigger = RichText::new(triggers)
                        .family(FontFamily::Monospace)
                        .color(trigger_color);
                    if selected {
                        trigger = trigger.strong();
                    }
                    ui.label(trigger);
                }
                ui.label(RichText::new(kind).small().color(secondary_color));
            });
            ui.label(
                RichText::new(truncate(preview, 66))
                    .small()
                    .color(secondary_color),
            );
        })
        .response;
    // Use a dedicated widget id rather than upgrading the frame response in place. Reusing the
    // frame id exposes a Focus action to AccessKit, but egui's focus graph still remembers the
    // original non-interactive frame and drops assistive-technology focus requests.
    let response = ui.interact(
        frame_response.rect,
        frame_response.id.with("interaction"),
        Sense::click(),
    );

    if selected {
        let indicator = egui::Rect::from_min_max(
            egui::pos2(
                response.rect.left() + theme::SELECTION_INDICATOR_INSET,
                response.rect.top() + theme::SPACE_SM,
            ),
            egui::pos2(
                response.rect.left()
                    + theme::SELECTION_INDICATOR_INSET
                    + theme::SELECTION_INDICATOR_WIDTH,
                response.rect.bottom() - theme::SPACE_SM,
            ),
        );
        ui.painter()
            .rect_filled(indicator, theme::RADIUS_CONTROL, palette.accent);
    }

    let accessible_label = [title, triggers, kind, &truncate(preview, 120)]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(". ");
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::Button,
            ui.is_enabled(),
            selected,
            accessible_label.clone(),
        )
    });
    if response.has_focus() {
        ui.painter().rect_stroke(
            response.rect,
            theme::RADIUS_CARD,
            Stroke::new(theme::STROKE_FOCUS, palette.accent),
            egui::StrokeKind::Inside,
        );
    }
    response
}

fn snippet_card_text_colors(palette: theme::Palette, selected: bool) -> (Color32, Color32) {
    if selected {
        (palette.ink, palette.ink)
    } else {
        (palette.accent, palette.muted)
    }
}

pub(crate) fn callout(ui: &mut Ui, color: Color32, text: &str) {
    Frame::new()
        .fill(color.gamma_multiply(theme::TINT_CALLOUT))
        .stroke(Stroke::new(
            theme::STROKE_STANDARD,
            color.gamma_multiply(theme::TINT_CALLOUT_BORDER),
        ))
        .corner_radius(theme::RADIUS_CALLOUT)
        .inner_margin(Margin::same(theme::PADDING_MD))
        .show(ui, |ui| {
            ui.label(RichText::new(text).color(theme::palette(ui).ink));
        });
}

pub(crate) fn centered_empty_state(ui: &mut Ui, title: &str, description: &str) {
    centered_empty_state_content(ui, title, description, |_| {});
}

pub(crate) fn centered_empty_state_action(
    ui: &mut Ui,
    title: &str,
    description: &str,
    action_label: &str,
) -> bool {
    let mut clicked = false;
    centered_empty_state_content(ui, title, description, |ui| {
        ui.add_space(theme::SPACE_MD);
        clicked = ui.add(primary_button(ui, action_label)).clicked();
    });
    clicked
}

fn centered_empty_state_content(
    ui: &mut Ui,
    title: &str,
    description: &str,
    actions: impl FnOnce(&mut Ui),
) {
    ui.add_space(empty_state_top_spacing(ui.ctx().content_rect().width()));
    ui.vertical_centered(|ui| {
        ui.label(RichText::new(title).size(theme::TEXT_DISPLAY).strong());
        ui.label(RichText::new(description).color(theme::palette(ui).muted));
        actions(ui);
    });
}

fn empty_state_top_spacing(logical_width: f32) -> f32 {
    if compact_layout(logical_width) {
        theme::SPACE_SM
    } else {
        theme::EMPTY_STATE_TOP_SPACE
    }
}

pub(crate) fn live_message_bar(
    ui: &mut Ui,
    text: &str,
    color: Color32,
    live: egui::accesskit::Live,
) {
    egui::Panel::bottom("message-bar")
        .frame(
            Frame::new()
                .fill(color.gamma_multiply(theme::TINT_BADGE))
                .inner_margin(Margin::symmetric(theme::PADDING_LG, theme::PADDING_SM)),
        )
        .show(ui, |ui| {
            let response = ui.label(RichText::new(text).color(theme::palette(ui).ink).strong());
            ui.ctx()
                .accesskit_node_builder(response.id, |node| node.set_live(live));
        });
}

fn truncate(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let prefix: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{prefix}…")
    } else {
        prefix
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render_snippet_card(ctx: &egui::Context, input: egui::RawInput) -> egui::Response {
        let mut response = None;
        let mut output = ctx.run_ui(input, |ui| {
            response = Some(snippet_card(
                ui,
                true,
                "Accessible title",
                ":trigger",
                "Readable preview",
                "Text",
            ));
        });
        output.textures_delta.clear();
        response.expect("snippet card response")
    }

    #[test]
    fn long_form_content_is_centered_without_overflowing_narrow_views() {
        let wide = centered_content_rect(egui::Rect::from_min_max(
            egui::pos2(20.0, 10.0),
            egui::pos2(1_420.0, 810.0),
        ));
        assert_eq!(wide.width(), theme::CONTENT_MAX_WIDTH);
        assert_eq!(wide.center().x, 720.0);

        let narrow = centered_content_rect(egui::Rect::from_min_max(
            egui::pos2(20.0, 10.0),
            egui::pos2(620.0, 410.0),
        ));
        assert_eq!(narrow.width(), 600.0);
        assert_eq!(narrow.left(), 20.0);
    }

    #[test]
    fn compact_empty_states_keep_the_first_action_near_the_initial_fold() {
        assert_eq!(
            empty_state_top_spacing(theme::COMPACT_LAYOUT_BREAKPOINT - theme::SPACE_XS),
            theme::SPACE_SM
        );
        assert_eq!(
            empty_state_top_spacing(theme::COMPACT_LAYOUT_BREAKPOINT),
            theme::EMPTY_STATE_TOP_SPACE
        );
    }

    #[test]
    fn long_configuration_paths_wrap_inside_the_available_width() {
        let context = egui::Context::default();
        theme::install(&context);
        let mut rect = egui::Rect::NOTHING;
        let path = Path::new(
            "/a/very/long/configuration/location/that/must/not/force/the/editor/outside/the/window",
        );
        let mut output = context.run_ui(Default::default(), |ui| {
            ui.set_width(theme::MODAL_MIN_WIDTH);
            rect = wrapped_path_label(ui, path).rect;
        });
        output.textures_delta.clear();

        assert!(rect.width() <= theme::MODAL_MIN_WIDTH);
        assert!(rect.height() > theme::TEXT_BODY);
    }

    #[test]
    fn maximum_zoom_keeps_the_minimum_window_in_responsive_modes() {
        let logical_viewport = egui::vec2(
            theme::MINIMUM_WINDOW_SIZE[0] / 2.0,
            theme::MINIMUM_WINDOW_SIZE[1] / 2.0,
        );
        assert!(compact_layout(logical_viewport.x));
        assert!(logical_viewport.x < theme::FIELD_STACK_BREAKPOINT);

        let modal_size = responsive_modal_size(
            logical_viewport,
            egui::vec2(theme::MODAL_WIDTH_WIDE, theme::MODAL_HEIGHT_TALL),
        );
        assert_eq!(
            modal_size,
            logical_viewport - egui::Vec2::splat(theme::MODAL_VIEWPORT_GUTTER)
        );

        for preferred_width in [
            theme::SNIPPET_LIST_COMPACT_WIDTH,
            theme::PROFILE_LIST_COMPACT_WIDTH,
        ] {
            let collection_width = compact_collection_width(logical_viewport.x, preferred_width);
            assert!(collection_width >= theme::COLLECTION_PANEL_MIN_WIDTH);
            assert!(
                logical_viewport.x - collection_width >= theme::COMPACT_DETAIL_MIN_WIDTH,
                "compact collection panel left too little editor width"
            );
        }
    }

    #[test]
    fn snippet_cards_fill_one_consistent_list_column() {
        let context = egui::Context::default();
        theme::install(&context);
        let mut widths = Vec::new();
        let mut output = context.run_ui(egui::RawInput::default(), |ui| {
            ui.set_width(320.0);
            widths.push(snippet_card(ui, true, "A", ":a", "x", "Text").rect.width());
            widths.push(
                snippet_card(
                    ui,
                    false,
                    "A much longer title",
                    ":a-longer-trigger",
                    "A preview with substantially more text",
                    "Text",
                )
                .rect
                .width(),
            );
        });
        output.textures_delta.clear();

        assert_eq!(widths.len(), 2);
        assert!((widths[0] - widths[1]).abs() < f32::EPSILON, "{widths:?}");
        assert!(widths[0] >= 320.0, "{widths:?}");
    }

    #[test]
    fn selected_snippet_cards_use_the_contrast_tested_text_color() {
        for palette in [theme::LIGHT_PALETTE, theme::DARK_PALETTE] {
            assert_eq!(
                snippet_card_text_colors(palette, true),
                (palette.ink, palette.ink)
            );
            assert_eq!(
                snippet_card_text_colors(palette, false),
                (palette.accent, palette.muted)
            );
        }
    }

    #[test]
    fn selected_options_have_a_non_color_marker_without_changing_the_accessible_name() {
        assert_eq!(
            selected_option_label("Regular expression", true),
            "Regular expression  ✓"
        );
        assert_eq!(
            selected_option_label("Regular expression", false),
            "Regular expression"
        );

        let context = egui::Context::default();
        context.enable_accesskit();
        theme::install(&context);
        let mut value = true;
        let mut output = context.run_ui(Default::default(), |ui| {
            let _ = unambiguous_selectable_value(ui, &mut value, true, "Regular expression");
        });
        output.textures_delta.clear();
        let update = output
            .platform_output
            .accesskit_update
            .expect("accessibility should be enabled");
        let selected = update
            .nodes
            .iter()
            .map(|(_, node)| node)
            .find(|node| node.label() == Some("Regular expression"))
            .expect("selected option");

        assert_eq!(selected.toggled(), Some(egui::accesskit::Toggled::True));
    }

    #[test]
    fn selected_list_rows_keep_their_plain_accessible_name() {
        let context = egui::Context::default();
        context.enable_accesskit();
        theme::install(&context);
        let mut output = context.run_ui(Default::default(), |ui| {
            ui.set_width(theme::SNIPPET_LIST_WIDTH);
            let _ = selection_list_row(ui, "base\n4 snippets", true);
        });
        output.textures_delta.clear();
        let update = output
            .platform_output
            .accesskit_update
            .expect("accessibility should be enabled");
        let selected = update
            .nodes
            .iter()
            .map(|(_, node)| node)
            .find(|node| node.label() == Some("base\n4 snippets"))
            .expect("selected list row");

        assert_eq!(selected.toggled(), Some(egui::accesskit::Toggled::True));
    }

    #[test]
    fn modal_minimums_never_force_content_beyond_a_tiny_viewport() {
        let viewport = egui::vec2(200.0, 120.0);
        let modal_size = responsive_modal_size(
            viewport,
            egui::vec2(theme::MODAL_WIDTH_SM, theme::MODAL_MIN_HEIGHT),
        );

        assert!(modal_size.x <= viewport.x);
        assert!(modal_size.y <= viewport.y);
        assert_eq!(modal_size.x, 200.0 - theme::MODAL_VIEWPORT_GUTTER);
        assert_eq!(modal_size.y, 120.0 - theme::MODAL_VIEWPORT_GUTTER);
    }

    #[test]
    fn snippet_cards_accept_accesskit_focus_requests() {
        let context = egui::Context::default();
        context.enable_accesskit();
        let response = render_snippet_card(&context, egui::RawInput::default());

        let mut input = egui::RawInput::default();
        input.events.push(egui::Event::AccessKitActionRequest(
            egui::accesskit::ActionRequest {
                action: egui::accesskit::Action::Focus,
                target_tree: egui::accesskit::TreeId::ROOT,
                target_node: response.id.accesskit_id(),
                data: None,
            },
        ));
        let response = render_snippet_card(&context, input);

        assert!(context.memory(|memory| memory.has_focus(response.id)));
    }

    #[test]
    fn title_hierarchy_keeps_heading_semantics_and_levels() {
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut output = context.run_ui(Default::default(), |ui| {
            display_heading(ui, "Page title");
            section_heading(ui, "Section title");
        });
        output.textures_delta.clear();
        let update = output
            .platform_output
            .accesskit_update
            .expect("accessibility should be enabled");
        let nodes = update
            .nodes
            .iter()
            .map(|(id, node)| (*id, node))
            .collect::<std::collections::HashMap<_, _>>();
        for (text, level) in [("Page title", 1), ("Section title", 2)] {
            let heading = update
                .nodes
                .iter()
                .map(|(_, node)| node)
                .find(|node| {
                    node.role() == egui::accesskit::Role::Heading && node.level() == Some(level)
                })
                .unwrap_or_else(|| panic!("missing level-{level} heading"));

            assert!(
                heading.children().iter().any(|child| {
                    nodes
                        .get(child)
                        .is_some_and(|node| node.value().or_else(|| node.label()) == Some(text))
                }),
                "heading level {level} does not own {text:?}"
            );
        }
    }

    #[test]
    fn contextual_buttons_keep_short_visual_text_and_expose_their_target() {
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut output = context.run_ui(Default::default(), |ui| {
            let _ = context_row_button(ui, "Edit", "today");
            let _ = context_button_enabled(ui, false, "Restore", "2026-08-16T12:00:00");
            let mut choice = 0;
            let _ = context_selectable_value(ui, &mut choice, 0, "Use local", "matches[0]");
        });
        output.textures_delta.clear();
        let update = output
            .platform_output
            .accesskit_update
            .expect("accessibility should be enabled");

        let button = update
            .nodes
            .iter()
            .map(|(_, node)| node)
            .find(|node| node.label() == Some("Edit: today"))
            .expect("contextual action button");
        assert_eq!(button.role(), egui::accesskit::Role::Button);

        let restore = update
            .nodes
            .iter()
            .map(|(_, node)| node)
            .find(|node| node.label() == Some("Restore: 2026-08-16T12:00:00"))
            .expect("contextual restore button");
        assert!(restore.is_disabled());

        let choice = update
            .nodes
            .iter()
            .map(|(_, node)| node)
            .find(|node| node.label() == Some("Use local: matches[0]"))
            .expect("contextual conflict choice");
        assert_eq!(choice.toggled(), Some(egui::accesskit::Toggled::True));
    }

    #[test]
    fn contextual_row_actions_keep_full_size_without_clipping_long_labels() {
        let context = egui::Context::default();
        theme::install(&context);
        let mut rect = egui::Rect::NOTHING;
        let mut output = context.run_ui(Default::default(), |ui| {
            rect = context_row_button(ui, "選択した項目を削除", "対象").rect;
        });
        output.textures_delta.clear();

        assert!(rect.height() >= theme::CONTROL_HEIGHT, "{rect:?}");
        assert!(rect.width() > theme::CONTROL_MIN_WIDTH, "{rect:?}");
    }
}
