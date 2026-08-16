use crate::espanso::EspansoStatus;
use crate::i18n::{self, Language, TextKey};
use crate::navigation::command_shortcut;
use crate::theme;
use crate::ui_components::{badge, primary_button};
use eframe::egui::{self, Align, Button, Frame, Layout, Margin, RichText, Stroke, Ui};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TopBarAction {
    Save,
    Reload,
    RestartEspanso,
}

pub(crate) fn show(
    ui: &mut Ui,
    status: &EspansoStatus,
    language: Language,
    can_save: bool,
    dirty: bool,
) -> Option<TopBarAction> {
    let compact = crate::ui_components::compact_layout(ui.ctx().content_rect().width());
    let mut action = None;
    egui::Panel::top("top-bar")
        .frame(
            Frame::new()
                .fill(theme::palette(ui).panel)
                .inner_margin(Margin::symmetric(
                    theme::PADDING_LG,
                    if compact {
                        theme::PADDING_COMPACT
                    } else {
                        theme::PADDING_MD
                    },
                ))
                .stroke(Stroke::new(
                    theme::STROKE_STANDARD,
                    theme::palette(ui).border_subtle,
                )),
        )
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("E/")
                        .size(theme::TEXT_DISPLAY)
                        .strong()
                        .color(theme::palette(ui).accent),
                );
                ui.label(
                    RichText::new("Espanso GUI")
                        .size(theme::TEXT_SECTION)
                        .strong(),
                );
                if compact {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        action = actions(ui, status, language, can_save, dirty, false);
                        status_badge(ui, status, language, true);
                    });
                } else {
                    ui.add_space(theme::SPACE_MD);
                    status_badge(ui, status, language, false);
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        action = actions(ui, status, language, can_save, dirty, true);
                    });
                }
            });
        });
    action
}

fn actions(
    ui: &mut Ui,
    status: &EspansoStatus,
    language: Language,
    can_save: bool,
    dirty: bool,
    show_secondary: bool,
) -> Option<TopBarAction> {
    let save_label = if show_secondary {
        format!(
            "{}  {}",
            i18n::text(language, TextKey::Save),
            command_shortcut("S")
        )
    } else if dirty {
        format!("{} •", i18n::text(language, TextKey::Save))
    } else {
        i18n::text(language, TextKey::Save).to_owned()
    };
    let save_response = if can_save {
        ui.add(primary_button(ui, save_label))
    } else {
        ui.add_enabled(false, Button::new(save_label))
    };
    let mut action = save_response.clicked().then_some(TopBarAction::Save);

    if show_secondary
        && ui.button(i18n::text(language, TextKey::Reload)).clicked()
        && action.is_none()
    {
        action = Some(TopBarAction::Reload);
    }
    if show_secondary
        && status.installed
        && ui
            .button(i18n::text(language, TextKey::RestartEspanso))
            .clicked()
        && action.is_none()
    {
        action = Some(TopBarAction::RestartEspanso);
    }
    if dirty && show_secondary {
        ui.label(
            RichText::new(i18n::text(language, TextKey::Unsaved))
                .color(theme::palette(ui).amber)
                .strong(),
        );
    }
    action
}

fn status_badge(ui: &mut Ui, status: &EspansoStatus, language: Language, compact: bool) {
    let (text, color) = if status.installed {
        (
            i18n::text(
                language,
                if compact {
                    TextKey::ConnectedShort
                } else {
                    TextKey::Connected
                },
            ),
            theme::palette(ui).accent,
        )
    } else {
        (
            i18n::text(
                language,
                if compact {
                    TextKey::NotDetectedShort
                } else {
                    TextKey::NotDetected
                },
            ),
            theme::palette(ui).amber,
        )
    };
    badge(ui, text, color);
}
