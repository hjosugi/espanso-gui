use crate::i18n::{self, Language, TextKey};
use crate::model::ConfigProfile;
use crate::theme;
use crate::ui_components::{
    callout, labelled_two_column_field, section_heading, singleline_text_edit,
};
use eframe::egui::{self, ComboBox, ScrollArea, Ui};

pub(crate) fn visual_editor(
    ui: &mut Ui,
    language: Language,
    is_default: bool,
    profile: &mut ConfigProfile,
) {
    ScrollArea::vertical()
        .id_salt("profile-editor-scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            if is_default {
                callout(
                    ui,
                    theme::palette(ui).accent,
                    i18n::text(language, TextKey::DefaultProfileNotice),
                );
            } else {
                app_filter_controls(ui, language, profile);
            }

            ui.add_space(theme::SPACE_LG);
            section_heading(ui, i18n::text(language, TextKey::BehaviorAndInjection));
            optional_bool_field(
                ui,
                language,
                &mut profile.enable,
                i18n::text(language, TextKey::EnableEspanso),
                i18n::text(language, TextKey::InheritDefaultsDescription),
            );
            labelled_two_column_field(
                ui,
                i18n::text(language, TextKey::InjectionBackend),
                i18n::text(language, TextKey::InjectionBackendDescription),
                |ui, label_id| {
                    let response = ComboBox::from_id_salt("profile-backend")
                        .selected_text(
                            profile
                                .backend
                                .as_deref()
                                .unwrap_or(i18n::text(language, TextKey::Inherit)),
                        )
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut profile.backend,
                                None,
                                i18n::text(language, TextKey::Inherit),
                            );
                            for (value, label) in [
                                ("auto", i18n::text(language, TextKey::Automatic)),
                                ("inject", i18n::text(language, TextKey::KeyInjection)),
                                ("clipboard", i18n::text(language, TextKey::Clipboard)),
                            ] {
                                ui.selectable_value(
                                    &mut profile.backend,
                                    Some(value.into()),
                                    label,
                                );
                            }
                        });
                    response.response.labelled_by(label_id);
                },
            );
            optional_bool_field(
                ui,
                language,
                &mut profile.apply_patch,
                i18n::text(language, TextKey::ApplyBuiltInPatch),
                i18n::text(language, TextKey::ApplyBuiltInPatchDescription),
            );
            optional_text_field(
                ui,
                language,
                &mut profile.paste_shortcut,
                i18n::text(language, TextKey::PasteShortcut),
                i18n::text(language, TextKey::PasteShortcutHint),
            );

            ui.add_space(theme::SPACE_LG);
            section_heading(ui, i18n::text(language, TextKey::DelaysMilliseconds));
            for (value, key, yaml_key) in [
                (
                    &mut profile.inject_delay,
                    TextKey::CharacterInjectionDelay,
                    "inject_delay",
                ),
                (
                    &mut profile.key_delay,
                    TextKey::KeyInjectionDelay,
                    "key_delay",
                ),
                (
                    &mut profile.pre_paste_delay,
                    TextKey::BeforePaste,
                    "pre_paste_delay",
                ),
                (
                    &mut profile.paste_shortcut_event_delay,
                    TextKey::PasteKeyInterval,
                    "paste_shortcut_event_delay",
                ),
                (
                    &mut profile.post_form_delay,
                    TextKey::AfterForm,
                    "post_form_delay",
                ),
                (
                    &mut profile.post_search_delay,
                    TextKey::AfterSearch,
                    "post_search_delay",
                ),
            ] {
                optional_number_field(ui, language, value, i18n::text(language, key), yaml_key);
            }

            ui.add_space(theme::SPACE_LG);
            section_heading(ui, i18n::text(language, TextKey::FormLimits));
            optional_number_field(
                ui,
                language,
                &mut profile.max_form_width,
                i18n::text(language, TextKey::MaximumWidthPx),
                "max_form_width",
            );
            optional_number_field(
                ui,
                language,
                &mut profile.max_form_height,
                i18n::text(language, TextKey::MaximumHeightPx),
                "max_form_height",
            );

            if is_default {
                default_profile_controls(ui, language, profile);
            }
        });
}

fn app_filter_controls(ui: &mut Ui, language: Language, profile: &mut ConfigProfile) {
    callout(
        ui,
        theme::palette(ui).accent,
        i18n::text(language, TextKey::ProfileFilterNotice),
    );
    ui.add_space(theme::SPACE_MD);
    section_heading(ui, i18n::text(language, TextKey::TargetApplications));
    optional_text_field(
        ui,
        language,
        &mut profile.filter_exec,
        i18n::text(language, TextKey::ExecutableFilter),
        i18n::text(language, TextKey::ExecutableFilterHint),
    );
    optional_text_field(
        ui,
        language,
        &mut profile.filter_class,
        i18n::text(language, TextKey::WindowClassFilter),
        i18n::text(language, TextKey::WindowClassFilterDescription),
    );
    optional_text_field(
        ui,
        language,
        &mut profile.filter_title,
        i18n::text(language, TextKey::WindowTitleFilter),
        i18n::text(language, TextKey::WindowTitleFilterHint),
    );
    labelled_two_column_field(
        ui,
        i18n::text(language, TextKey::OperatingSystemFilter),
        i18n::text(language, TextKey::OperatingSystemFilterDescription),
        |ui, label_id| {
            let response = ComboBox::from_id_salt("profile-filter-os")
                .selected_text(
                    profile
                        .filter_os
                        .as_deref()
                        .unwrap_or(i18n::text(language, TextKey::Inherit)),
                )
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut profile.filter_os,
                        None,
                        i18n::text(language, TextKey::Inherit),
                    );
                    for value in ["linux", "macos", "windows"] {
                        ui.selectable_value(&mut profile.filter_os, Some(value.into()), value);
                    }
                });
            response.response.labelled_by(label_id);
        },
    );
    if !profile.has_filter() {
        callout(
            ui,
            theme::palette(ui).amber,
            i18n::text(language, TextKey::ProfileFilterRequired),
        );
    }
}

fn default_profile_controls(ui: &mut Ui, language: Language, profile: &mut ConfigProfile) {
    ui.add_space(theme::SPACE_LG);
    section_heading(ui, i18n::text(language, TextKey::SearchAndGlobalSettings));
    optional_text_field(
        ui,
        language,
        &mut profile.search_shortcut,
        i18n::text(language, TextKey::SearchShortcut),
        i18n::text(language, TextKey::SearchShortcutHint),
    );
    optional_text_field(
        ui,
        language,
        &mut profile.search_trigger,
        i18n::text(language, TextKey::SearchTrigger),
        i18n::text(language, TextKey::SearchTriggerHint),
    );
    optional_text_field(
        ui,
        language,
        &mut profile.toggle_key,
        i18n::text(language, TextKey::ToggleKey),
        i18n::text(language, TextKey::ToggleKeyHint),
    );
    optional_bool_field(
        ui,
        language,
        &mut profile.preserve_clipboard,
        i18n::text(language, TextKey::RestoreClipboard),
        i18n::text(language, TextKey::RestoreClipboardDescription),
    );
    optional_bool_field(
        ui,
        language,
        &mut profile.show_icon,
        i18n::text(language, TextKey::ShowStatusIcon),
        i18n::text(language, TextKey::ShowStatusIconDescription),
    );
    optional_bool_field(
        ui,
        language,
        &mut profile.show_notifications,
        i18n::text(language, TextKey::ShowNotifications),
        i18n::text(language, TextKey::ShowNotificationsDescription),
    );
}

fn optional_text_field(
    ui: &mut Ui,
    language: Language,
    value: &mut Option<String>,
    label: &str,
    description: &str,
) {
    labelled_two_column_field(ui, label, description, |ui, label_id| {
        let mut overridden = value.is_some();
        ui.horizontal(|ui| {
            if override_checkbox(ui, language, &mut overridden, label).changed() {
                if overridden {
                    *value = Some(String::new());
                } else {
                    *value = None;
                }
            }
            ui.add_enabled_ui(overridden, |ui| {
                if let Some(value) = value {
                    ui.add(
                        singleline_text_edit(value).desired_width(
                            ui.available_width().min(theme::FIELD_CONTROL_MAX_WIDTH),
                        ),
                    )
                    .labelled_by(label_id);
                }
            });
        });
    });
}

fn optional_number_field(
    ui: &mut Ui,
    language: Language,
    value: &mut Option<u64>,
    label: &str,
    description: &str,
) {
    labelled_two_column_field(ui, label, description, |ui, label_id| {
        let mut overridden = value.is_some();
        ui.horizontal(|ui| {
            if override_checkbox(ui, language, &mut overridden, label).changed() {
                if overridden {
                    *value = Some(0);
                } else {
                    *value = None;
                }
            }
            ui.add_enabled_ui(overridden, |ui| {
                if let Some(value) = value {
                    ui.add(egui::DragValue::new(value).range(0..=60_000))
                        .labelled_by(label_id);
                }
            });
        });
    });
}

fn override_checkbox(
    ui: &mut Ui,
    language: Language,
    overridden: &mut bool,
    field_label: &str,
) -> egui::Response {
    let visible_label = i18n::text(language, TextKey::Override);
    let response = ui.checkbox(overridden, visible_label);
    let accessible_label = format!("{field_label}: {visible_label}");
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::Checkbox,
            ui.is_enabled(),
            *overridden,
            accessible_label.clone(),
        )
    });
    response
}

fn optional_bool_field(
    ui: &mut Ui,
    language: Language,
    value: &mut Option<bool>,
    label: &str,
    description: &str,
) {
    labelled_two_column_field(ui, label, description, |ui, label_id| {
        let response = ComboBox::from_id_salt(("optional-bool", label))
            .selected_text(match value {
                None => i18n::text(language, TextKey::Inherit),
                Some(true) => i18n::text(language, TextKey::Enabled),
                Some(false) => i18n::text(language, TextKey::Disabled),
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(value, None, i18n::text(language, TextKey::Inherit));
                ui.selectable_value(value, Some(true), i18n::text(language, TextKey::Enabled));
                ui.selectable_value(value, Some(false), i18n::text(language, TextKey::Disabled));
            });
        response.response.labelled_by(label_id);
    });
}
