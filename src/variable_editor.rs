use crate::i18n::{self, Language, TextKey};
use crate::model::{FormField, FormFieldKind, Variable};
use crate::theme;
use crate::ui_components::{
    callout, context_row_button, labelled_two_column_field, multiline_text_edit,
    singleline_text_edit,
};
use eframe::egui::{self, ComboBox, FontId, Frame, RichText, Ui};

pub(crate) fn variable_parameters(ui: &mut Ui, language: Language, variable: &mut Variable) {
    match variable.kind.as_str() {
        "date" => {
            let mut format = variable.param_str("format");
            labelled_two_column_field(
                ui,
                i18n::text(language, TextKey::DateFormat),
                i18n::text(language, TextKey::StrftimeFormat),
                |ui, label_id| {
                    let response = ComboBox::from_id_salt("date-format-presets")
                        .selected_text(if format.is_empty() {
                            i18n::text(language, TextKey::ChooseFormat)
                        } else {
                            &format
                        })
                        .show_ui(ui, |ui| {
                            for (value, label) in i18n::date_format_presets(language) {
                                ui.selectable_value(
                                    &mut format,
                                    value.into(),
                                    format!("{label}  ({value})"),
                                );
                            }
                        });
                    response.response.labelled_by(label_id);
                    ui.add(singleline_text_edit(&mut format).hint_text("%Y-%m-%d"))
                        .labelled_by(label_id);
                },
            );
            variable.set_param("format", format);
            let mut offset = variable.param_i64("offset");
            labelled_two_column_field(
                ui,
                i18n::text(language, TextKey::DateOffset),
                i18n::text(language, TextKey::DateOffsetDescription),
                |ui, label_id| {
                    ui.horizontal_wrapped(|ui| {
                        if ui
                            .button(i18n::text(language, TextKey::Yesterday))
                            .clicked()
                        {
                            offset = -86_400;
                        }
                        if ui.button(i18n::text(language, TextKey::Today)).clicked() {
                            offset = 0;
                        }
                        if ui.button(i18n::text(language, TextKey::Tomorrow)).clicked() {
                            offset = 86_400;
                        }
                        if ui.button(i18n::text(language, TextKey::NextWeek)).clicked() {
                            offset = 604_800;
                        }
                    });
                    ui.add(
                        egui::DragValue::new(&mut offset)
                            .speed(60)
                            .suffix(i18n::text(language, TextKey::SecondsSuffix)),
                    )
                    .labelled_by(label_id);
                },
            );
            variable.set_i64("offset", offset, true);
            let mut locale = variable.param_str("locale");
            labelled_two_column_field(
                ui,
                i18n::text(language, TextKey::Locale),
                i18n::text(language, TextKey::LocaleDescription),
                |ui, label_id| {
                    ui.add(
                        singleline_text_edit(&mut locale)
                            .hint_text(i18n::text(language, TextKey::LocaleHint)),
                    )
                    .labelled_by(label_id);
                },
            );
            variable.set_param_optional("locale", &locale);
            let mut timezone = variable.param_str("tz");
            labelled_two_column_field(
                ui,
                i18n::text(language, TextKey::Timezone),
                i18n::text(language, TextKey::TimezoneDescription),
                |ui, label_id| {
                    ui.add(
                        singleline_text_edit(&mut timezone)
                            .hint_text(i18n::text(language, TextKey::TimezoneHint)),
                    )
                    .labelled_by(label_id);
                },
            );
            variable.set_param_optional("tz", &timezone);
        }
        "clipboard" => {
            callout(
                ui,
                theme::palette(ui).accent,
                i18n::text(language, TextKey::ClipboardDescription),
            );
        }
        "echo" => {
            let mut value = variable.param_str("echo");
            labelled_two_column_field(
                ui,
                i18n::text(language, TextKey::FixedValue),
                i18n::text(language, TextKey::FixedValueDescription),
                |ui, label_id| {
                    ui.add(multiline_text_edit(&mut value).desired_rows(4))
                        .labelled_by(label_id);
                },
            );
            variable.set_param("echo", value);
        }
        "random" => {
            let mut values = variable.param_strings("choices").join("\n");
            labelled_two_column_field(
                ui,
                i18n::text(language, TextKey::Candidates),
                i18n::text(language, TextKey::RandomDescription),
                |ui, label_id| {
                    ui.add(multiline_text_edit(&mut values).desired_rows(7))
                        .labelled_by(label_id);
                },
            );
            variable.set_string_list(
                "choices",
                &values.lines().map(str::to_string).collect::<Vec<_>>(),
            );
        }
        "choice" => {
            let mut values = variable.param_strings("values").join("\n");
            labelled_two_column_field(
                ui,
                i18n::text(language, TextKey::Choices),
                i18n::text(language, TextKey::ChoiceDescription),
                |ui, label_id| {
                    ui.add(multiline_text_edit(&mut values).desired_rows(7))
                        .labelled_by(label_id);
                },
            );
            variable.set_string_list(
                "values",
                &values.lines().map(str::to_string).collect::<Vec<_>>(),
            );
        }
        "shell" => {
            callout(
                ui,
                theme::palette(ui).amber,
                i18n::text(language, TextKey::ShellWarning),
            );
            let mut command = variable.param_str("cmd");
            labelled_two_column_field(
                ui,
                i18n::text(language, TextKey::Command),
                i18n::text(language, TextKey::CommandDescription),
                |ui, label_id| {
                    ui.add(
                        multiline_text_edit(&mut command)
                            .font(FontId::monospace(theme::TEXT_BODY))
                            .desired_rows(5),
                    )
                    .labelled_by(label_id);
                },
            );
            variable.set_param("cmd", command);
            let mut shell = variable.param_str("shell");
            labelled_two_column_field(
                ui,
                i18n::text(language, TextKey::Shell),
                i18n::text(language, TextKey::DefaultOsDescription),
                |ui, label_id| {
                    let response = ComboBox::from_id_salt("shell-kind")
                        .selected_text(if shell.is_empty() {
                            i18n::text(language, TextKey::DefaultOs)
                        } else {
                            &shell
                        })
                        .show_ui(ui, |ui| {
                            for value in
                                ["", "sh", "bash", "powershell", "pwsh", "cmd", "wsl", "nu"]
                            {
                                ui.selectable_value(
                                    &mut shell,
                                    value.into(),
                                    if value.is_empty() {
                                        i18n::text(language, TextKey::DefaultOs)
                                    } else {
                                        value
                                    },
                                );
                            }
                        });
                    response.response.labelled_by(label_id);
                },
            );
            variable.set_param_optional("shell", &shell);
            let mut trim = variable.param_bool("trim", true);
            ui.checkbox(&mut trim, i18n::text(language, TextKey::TrimOutput));
            variable.set_bool("trim", trim, true);
            let mut debug = variable.param_bool("debug", false);
            ui.checkbox(&mut debug, i18n::text(language, TextKey::DebugOutput));
            variable.set_bool("debug", debug, false);
        }
        "script" => {
            callout(
                ui,
                theme::palette(ui).amber,
                i18n::text(language, TextKey::ScriptWarning),
            );
            let mut args = variable.param_strings("args").join("\n");
            labelled_two_column_field(
                ui,
                i18n::text(language, TextKey::CommandAndArguments),
                i18n::text(language, TextKey::OnePerLine),
                |ui, label_id| {
                    ui.add(
                        multiline_text_edit(&mut args)
                            .font(FontId::monospace(theme::TEXT_BODY))
                            .desired_rows(7),
                    )
                    .labelled_by(label_id);
                },
            );
            variable.set_string_list(
                "args",
                &args.lines().map(str::to_string).collect::<Vec<_>>(),
            );
            let mut trim = variable.param_bool("trim", true);
            ui.checkbox(&mut trim, i18n::text(language, TextKey::TrimOutput));
            variable.set_bool("trim", trim, true);
        }
        "form" => {
            let mut layout = variable.param_str("layout");
            labelled_two_column_field(
                ui,
                i18n::text(language, TextKey::FormLayout),
                i18n::text(language, TextKey::FormLayoutDescription),
                |ui, label_id| {
                    ui.add(multiline_text_edit(&mut layout).desired_rows(8))
                        .labelled_by(label_id);
                },
            );
            variable.set_param("layout", layout);
            let mut fields = variable.form_fields();
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(i18n::text(language, TextKey::FormFields)).strong());
                if ui.button(i18n::text(language, TextKey::AddField)).clicked() {
                    let mut suffix = 1;
                    let mut name = "field".to_string();
                    while fields.contains_key(&name) {
                        suffix += 1;
                        name = format!("field{suffix}");
                    }
                    fields.insert(name, FormField::default());
                }
            });
            let mut remove = None;
            let mut rename = None;
            for (name, original_field) in fields.clone() {
                let mut next_name = name.clone();
                let mut field = original_field;
                Frame::group(ui.style()).show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.vertical(|ui| {
                            let name_label =
                                ui.label(i18n::text(language, TextKey::FormFieldNameShort));
                            ui.add(
                                singleline_text_edit(&mut next_name)
                                    .desired_width(theme::FIELD_NAME_WIDTH),
                            )
                            .labelled_by(name_label.id);
                        });
                        let original_kind = field.kind();
                        let mut kind = original_kind.clone();
                        ui.vertical(|ui| {
                            let kind_label = ui.label(i18n::text(language, TextKey::InputType));
                            let response = ComboBox::from_id_salt(("variable-form-field", &name))
                                .selected_text(i18n::form_field_kind_label(language, &kind))
                                .show_ui(ui, |ui| {
                                    for candidate in FormFieldKind::ALL {
                                        let label =
                                            i18n::form_field_kind_label(language, &candidate);
                                        ui.selectable_value(&mut kind, candidate, label);
                                    }
                                });
                            response.response.labelled_by(kind_label.id);
                        });
                        if kind != original_kind {
                            field.set_kind(&kind);
                        }
                        if context_row_button(ui, i18n::text(language, TextKey::Delete), &name)
                            .clicked()
                        {
                            remove = Some(name.clone());
                        }
                    });
                    let mut default = field.default.clone().unwrap_or_default();
                    let default_label = ui.label(i18n::text(language, TextKey::DefaultValue));
                    if ui
                        .add(
                            singleline_text_edit(&mut default)
                                .hint_text(i18n::text(language, TextKey::InitialValue)),
                        )
                        .labelled_by(default_label.id)
                        .changed()
                    {
                        field.default = (!default.is_empty()).then_some(default);
                    }
                    if matches!(field.kind(), FormFieldKind::Choice | FormFieldKind::List) {
                        let mut values = field.values.join("\n");
                        let choices_label = ui.label(i18n::text(language, TextKey::ChoicesPerLine));
                        if ui
                            .add(
                                multiline_text_edit(&mut values)
                                    .desired_rows(3)
                                    .hint_text(i18n::text(language, TextKey::ChoicesHint)),
                            )
                            .labelled_by(choices_label.id)
                            .changed()
                        {
                            field.values = values.lines().map(str::to_string).collect();
                        }
                    }
                });
                fields.insert(name.clone(), field);
                if next_name != name
                    && !next_name.is_empty()
                    && next_name
                        .chars()
                        .all(|character| character.is_ascii_alphanumeric() || character == '_')
                    && !fields.contains_key(&next_name)
                {
                    rename = Some((name, next_name));
                }
            }
            if let Some(name) = remove {
                fields.shift_remove(&name);
            }
            if let Some((old, new)) = rename
                && let Some(field) = fields.shift_remove(&old)
            {
                fields.insert(new, field);
            }
            variable.set_form_fields(&fields);
        }
        "global" => {
            callout(
                ui,
                theme::palette(ui).accent,
                i18n::text(language, TextKey::GlobalVariableDescription),
            );
        }
        _ => {
            callout(
                ui,
                theme::palette(ui).amber,
                i18n::text(language, TextKey::UnknownVariableType),
            );
        }
    }
}

pub(crate) fn variable_summary(ui: &mut Ui, language: Language, variable: &Variable) {
    let summary = match variable.kind.as_str() {
        "date" => i18n::date_summary_text(
            language,
            &variable.param_str("format"),
            variable.param_i64("offset"),
        ),
        "clipboard" => i18n::text(language, TextKey::CurrentClipboard).into(),
        "echo" => truncate(&variable.param_str("echo"), 80),
        "random" => i18n::random_summary_text(language, variable.param_strings("choices").len()),
        "choice" => i18n::choice_summary_text(language, variable.param_strings("values").len()),
        "shell" => truncate(&variable.param_str("cmd"), 80),
        "script" => variable.param_strings("args").join(" "),
        "form" => truncate(&variable.param_str("layout"), 80),
        _ => i18n::text(language, TextKey::AdvancedVariable).into(),
    };
    ui.label(
        RichText::new(summary)
            .small()
            .color(theme::palette(ui).muted),
    );
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
    use super::truncate;

    #[test]
    fn summaries_truncate_by_characters_not_bytes() {
        assert_eq!(truncate("日本語テキスト", 3), "日本語…");
        assert_eq!(truncate("short", 10), "short");
    }
}
