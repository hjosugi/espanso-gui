use crate::html_editor::{self, HtmlCommand};
use crate::i18n::{self, Language, TextKey};
use crate::model::{ContentKind, Snippet};
use crate::ui_components::unambiguous_selectable_value;
use eframe::egui::{self, Ui};

pub(crate) fn trigger_mode_selector(
    ui: &mut Ui,
    language: Language,
    snippet: &mut Snippet,
) -> (bool, egui::Response) {
    let mut regex_mode = snippet.regex.is_some();
    let regex_response = ui
        .horizontal_wrapped(|ui| {
            unambiguous_selectable_value(
                ui,
                &mut regex_mode,
                false,
                i18n::text(language, TextKey::NormalTrigger),
            );
            unambiguous_selectable_value(
                ui,
                &mut regex_mode,
                true,
                i18n::text(language, TextKey::RegularExpression),
            )
        })
        .inner;
    if regex_mode != snippet.regex.is_some() {
        snippet.set_regex_trigger_mode(regex_mode);
    }
    (regex_mode, regex_response)
}

pub(crate) fn editor_toolbar(ui: &mut Ui, language: Language, snippet: &mut Snippet) {
    ui.horizontal_wrapped(|ui| {
        if matches!(snippet.content_kind(), ContentKind::Markdown) {
            if ui.button(i18n::text(language, TextKey::Bold)).clicked() {
                snippet.insert_token(&format!("**{}**", i18n::text(language, TextKey::Bold)));
            }
            if ui.button(i18n::text(language, TextKey::Italic)).clicked() {
                snippet.insert_token(&format!("*{}*", i18n::text(language, TextKey::Italic)));
            }
            if ui.button(i18n::text(language, TextKey::Link)).clicked() {
                snippet.insert_token(&format!(
                    "[{}](https://example.com)",
                    i18n::text(language, TextKey::Link)
                ));
            }
            if ui.button(i18n::text(language, TextKey::Code)).clicked() {
                snippet.insert_token("`code`");
            }
            if ui
                .button(i18n::text(language, TextKey::BulletedList))
                .clicked()
            {
                snippet.insert_token(&format!(
                    "\n- {}\n- {}",
                    i18n::text(language, TextKey::ListItemOne),
                    i18n::text(language, TextKey::ListItemTwo)
                ));
            }
        } else if matches!(snippet.content_kind(), ContentKind::Html) {
            for (key, command) in [
                (TextKey::Bold, HtmlCommand::Bold),
                (TextKey::Italic, HtmlCommand::Italic),
                (TextKey::Heading, HtmlCommand::Heading),
                (TextKey::Link, HtmlCommand::Link),
                (TextKey::BulletedList, HtmlCommand::UnorderedList),
                (TextKey::NumberedList, HtmlCommand::OrderedList),
                (TextKey::Color, HtmlCommand::Color),
                (TextKey::Image, HtmlCommand::Image),
            ] {
                if ui.button(i18n::text(language, key)).clicked() {
                    snippet.insert_token(&html_editor::fragment(language, command));
                }
            }
        }
        if !matches!(snippet.content_kind(), ContentKind::Image)
            && ui
                .button(i18n::text(language, TextKey::CursorPosition))
                .clicked()
        {
            snippet.insert_token("$|$");
        }
        for variable in &snippet.vars.clone() {
            if ui.button(variable.token()).clicked() {
                snippet.insert_token(&variable.token());
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme;

    #[test]
    fn regular_expression_mode_can_be_selected_with_pointer_input() {
        let context = egui::Context::default();
        theme::install(&context);
        let mut snippet = Snippet::new();
        let mut regex_response = None;

        let mut output = context.run_ui(egui::RawInput::default(), |ui| {
            regex_response = Some(trigger_mode_selector(ui, Language::Japanese, &mut snippet).1);
        });
        output.textures_delta.clear();
        let regex_response = regex_response.expect("regex selector");
        assert!(regex_response.rect.height() >= theme::CONTROL_HEIGHT);
        let position = regex_response.rect.center();

        for pressed in [true, false] {
            let mut input = egui::RawInput::default();
            input.events.push(egui::Event::PointerMoved(position));
            input.events.push(egui::Event::PointerButton {
                pos: position,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            });
            let mut output = context.run_ui(input, |ui| {
                trigger_mode_selector(ui, Language::Japanese, &mut snippet);
            });
            output.textures_delta.clear();
        }

        assert_eq!(snippet.regex.as_deref(), Some(":new"));
        assert!(snippet.trigger.is_none());
        assert!(snippet.triggers.is_empty());
    }

    #[test]
    fn trigger_mode_exposes_the_selected_state_in_both_languages() {
        for language in Language::ALL {
            let context = egui::Context::default();
            context.enable_accesskit();
            theme::install(&context);
            let mut snippet = Snippet {
                regex: Some(":.+".into()),
                ..Snippet::default()
            };
            let mut output = context.run_ui(egui::RawInput::default(), |ui| {
                trigger_mode_selector(ui, language, &mut snippet);
            });
            output.textures_delta.clear();
            let update = output
                .platform_output
                .accesskit_update
                .expect("accessibility should be enabled");
            let expected = i18n::text(language, TextKey::RegularExpression);
            let selected = update
                .nodes
                .iter()
                .map(|(_, node)| node)
                .find(|node| node.label().or_else(|| node.value()) == Some(expected))
                .unwrap_or_else(|| panic!("missing regex mode in {language:?}"));

            assert_eq!(selected.toggled(), Some(egui::accesskit::Toggled::True));
        }
    }
}
