use crate::espanso::{EspansoAction, EspansoStatus};
use crate::i18n::{self, Language, TextKey};
use crate::preferences::{Preferences, format_ui_scale, parse_ui_scale};
use crate::theme;
use crate::ui_components::{
    callout, compact_layout, display_heading, labelled_two_column_field, section_heading,
    wrapped_path_label,
};
use eframe::egui::{self, Button, ComboBox, RichText, Ui};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SettingsAction {
    ChooseConfigRoot,
    OpenConfigRoot,
    Espanso(EspansoAction),
    RefreshStatus,
    BackupAll,
    ExportCsv,
    ImportCsv,
    DeleteSelectedFile,
}

pub(crate) fn appearance_and_accessibility(ui: &mut Ui, preferences: &mut Preferences) {
    let language = preferences.language;
    display_heading(ui, i18n::text(language, TextKey::Settings));
    ui.separator();
    if !compact_layout(ui.ctx().content_rect().width()) {
        section_heading(ui, i18n::text(language, TextKey::Accessibility));
    }
    labelled_two_column_field(
        ui,
        i18n::text(language, TextKey::Language),
        i18n::text(language, TextKey::LanguageDescription),
        |ui, label_id| {
            let response = ComboBox::from_id_salt("ui-language")
                .selected_text(preferences.language.native_name())
                .show_ui(ui, |ui| {
                    for candidate in Language::ALL {
                        ui.selectable_value(
                            &mut preferences.language,
                            candidate,
                            candidate.native_name(),
                        );
                    }
                });
            response.response.labelled_by(label_id);
        },
    );
    labelled_two_column_field(
        ui,
        i18n::text(preferences.language, TextKey::UiScale),
        "80%–200%",
        |ui, label_id| {
            let mut changed = false;
            ui.horizontal(|ui| {
                changed |= ui
                    .add(
                        egui::Slider::new(
                            &mut preferences.ui_scale,
                            theme::UI_SCALE_MIN..=theme::UI_SCALE_MAX,
                        )
                        .show_value(false)
                        .step_by(theme::UI_SCALE_STEP),
                    )
                    .labelled_by(label_id)
                    .changed();
                changed |= ui
                    .add(
                        egui::DragValue::new(&mut preferences.ui_scale)
                            .range(theme::UI_SCALE_MIN..=theme::UI_SCALE_MAX)
                            .speed(theme::UI_SCALE_STEP)
                            .custom_formatter(format_ui_scale)
                            .custom_parser(parse_ui_scale),
                    )
                    .labelled_by(label_id)
                    .changed();
            });
            if changed {
                ui.ctx().set_zoom_factor(preferences.ui_scale);
            }
        },
    );
    labelled_two_column_field(
        ui,
        i18n::text(preferences.language, TextKey::Appearance),
        i18n::text(preferences.language, TextKey::AppearanceDescription),
        |ui, label_id| {
            let language = preferences.language;
            let previous = preferences.appearance;
            let response = ComboBox::from_id_salt("ui-appearance")
                .selected_text(i18n::appearance_label(language, preferences.appearance))
                .show_ui(ui, |ui| {
                    for appearance in theme::Appearance::ALL {
                        ui.selectable_value(
                            &mut preferences.appearance,
                            appearance,
                            i18n::appearance_label(language, appearance),
                        );
                    }
                });
            response.response.labelled_by(label_id);
            if preferences.appearance != previous {
                theme::apply_appearance(ui.ctx(), preferences.appearance);
            }
        },
    );
    ui.label(RichText::new(i18n::text(preferences.language, TextKey::KeyboardShortcuts)).strong());
    callout(
        ui,
        theme::palette(ui).accent,
        i18n::text(preferences.language, TextKey::ShortcutHelp),
    );
}

pub(crate) fn config_folder(
    ui: &mut Ui,
    language: Language,
    config_root: &Path,
) -> Option<SettingsAction> {
    ui.separator();
    section_heading(ui, i18n::text(language, TextKey::ConfigFolder));
    let mut action = None;
    ui.horizontal_wrapped(|ui| {
        wrapped_path_label(ui, config_root);
        if ui.button(i18n::text(language, TextKey::Change)).clicked() {
            action = Some(SettingsAction::ChooseConfigRoot);
        }
        if ui
            .button(i18n::text(language, TextKey::OpenFolder))
            .clicked()
        {
            action = Some(SettingsAction::OpenConfigRoot);
        }
    });
    action
}

pub(crate) fn espanso_service(
    ui: &mut Ui,
    language: Language,
    status: &EspansoStatus,
) -> Option<SettingsAction> {
    ui.add_space(theme::SPACE_LG);
    section_heading(ui, i18n::text(language, TextKey::EspansoService));
    ui.label(format!(
        "{}: {}  /  {}: {}  /  {}: {}",
        i18n::text(language, TextKey::Installed),
        if status.installed {
            i18n::text(language, TextKey::Detected)
        } else {
            i18n::text(language, TextKey::Undetected)
        },
        i18n::text(language, TextKey::Version),
        status.version.as_deref().unwrap_or("—"),
        i18n::text(language, TextKey::Status),
        status.service.as_deref().unwrap_or("—")
    ));
    let mut action = None;
    ui.horizontal_wrapped(|ui| {
        for (key, command) in [
            (TextKey::Start, EspansoAction::Start),
            (TextKey::Stop, EspansoAction::Stop),
            (TextKey::Restart, EspansoAction::Restart),
        ] {
            if ui
                .add_enabled(status.installed, Button::new(i18n::text(language, key)))
                .clicked()
            {
                action = Some(SettingsAction::Espanso(command));
            }
        }
        if ui
            .button(i18n::text(language, TextKey::RefreshStatus))
            .clicked()
        {
            action = Some(SettingsAction::RefreshStatus);
        }
    });
    action
}

pub(crate) fn backup_and_migration(
    ui: &mut Ui,
    language: Language,
    can_backup: bool,
    can_export: bool,
    can_import: bool,
) -> Option<SettingsAction> {
    ui.add_space(theme::SPACE_LG);
    section_heading(ui, i18n::text(language, TextKey::BackupsAndMigration));
    let mut action = None;
    ui.horizontal_wrapped(|ui| {
        for (enabled, key, candidate) in [
            (can_backup, TextKey::BackupAll, SettingsAction::BackupAll),
            (can_export, TextKey::ExportCsv, SettingsAction::ExportCsv),
            (can_import, TextKey::ImportCsv, SettingsAction::ImportCsv),
        ] {
            if ui
                .add_enabled(enabled, Button::new(i18n::text(language, key)))
                .clicked()
            {
                action = Some(candidate);
            }
        }
    });
    callout(
        ui,
        theme::palette(ui).accent,
        i18n::text(language, TextKey::BackupSafety),
    );
    action
}

pub(crate) fn delete_file_action(
    ui: &mut Ui,
    language: Language,
    can_delete: bool,
) -> Option<SettingsAction> {
    ui.add_space(theme::SPACE_LG);
    section_heading(ui, i18n::text(language, TextKey::FileOperations));
    let label = i18n::text(language, TextKey::DeleteSelectedFile);
    let button = if can_delete {
        Button::new(RichText::new(label).color(theme::palette(ui).danger))
    } else {
        Button::new(label)
    };
    ui.add_enabled(can_delete, button)
        .clicked()
        .then_some(SettingsAction::DeleteSelectedFile)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn controls(render: impl FnMut(&mut Ui)) -> Vec<(String, bool)> {
        let context = egui::Context::default();
        context.enable_accesskit();
        theme::install(&context);
        let mut output = context.run_ui(Default::default(), render);
        output.textures_delta.clear();
        output
            .platform_output
            .accesskit_update
            .expect("accessibility should be enabled")
            .nodes
            .into_iter()
            .filter_map(|(_, node)| {
                node.label()
                    .map(|label| (label.to_owned(), node.is_disabled()))
            })
            .collect()
    }

    #[test]
    fn unavailable_storage_actions_are_disabled_instead_of_silently_ignored() {
        let controls = controls(|ui| {
            let _ = backup_and_migration(ui, Language::English, false, false, false);
            let _ = delete_file_action(ui, Language::English, false);
        });

        for key in [
            TextKey::BackupAll,
            TextKey::ExportCsv,
            TextKey::ImportCsv,
            TextKey::DeleteSelectedFile,
        ] {
            let label = i18n::text(Language::English, key);
            assert_eq!(
                controls
                    .iter()
                    .find(|(name, _)| name == label)
                    .map(|(_, disabled)| *disabled),
                Some(true),
                "{label:?} should explain its unavailable state as disabled"
            );
        }
    }

    #[test]
    fn service_commands_follow_detection_but_refresh_remains_available() {
        let controls = controls(|ui| {
            let _ = espanso_service(ui, Language::English, &EspansoStatus::default());
        });

        for key in [TextKey::Start, TextKey::Stop, TextKey::Restart] {
            let label = i18n::text(Language::English, key);
            assert_eq!(
                controls
                    .iter()
                    .find(|(name, _)| name == label)
                    .map(|(_, disabled)| *disabled),
                Some(true)
            );
        }
        let refresh = i18n::text(Language::English, TextKey::RefreshStatus);
        assert_eq!(
            controls
                .iter()
                .find(|(name, _)| name == refresh)
                .map(|(_, disabled)| *disabled),
            Some(false)
        );
    }
}
