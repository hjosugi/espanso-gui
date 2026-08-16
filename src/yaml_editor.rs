use crate::theme;
use crate::ui_components::multiline_text_edit;
use crate::yaml_syntax;
use eframe::egui::{self, FontId, TextStyle, Ui};

#[derive(Default)]
struct YamlHighlighter;

impl egui::cache::ComputerMut<(&FontId, theme::Palette, &str), egui::text::LayoutJob>
    for YamlHighlighter
{
    fn compute(
        &mut self,
        (font, palette, source): (&FontId, theme::Palette, &str),
    ) -> egui::text::LayoutJob {
        highlight_yaml(font.clone(), palette, source)
    }
}

type YamlHighlightCache<'a> = egui::cache::FrameCache<egui::text::LayoutJob, YamlHighlighter>;

pub(crate) fn editor(ui: &mut Ui, source: &mut String, desired_rows: usize) -> egui::Response {
    let mut layouter = |ui: &Ui, text: &dyn egui::TextBuffer, wrap_width: f32| {
        let font = TextStyle::Monospace.resolve(ui.style());
        let palette = theme::palette(ui);
        let mut job = ui.ctx().memory_mut(|memory| {
            memory
                .caches
                .cache::<YamlHighlightCache<'_>>()
                .get((&font, palette, text.as_str()))
                .clone()
        });
        job.wrap.max_width = wrap_width;
        ui.fonts_mut(|fonts| fonts.layout_job(job))
    };

    ui.add(
        multiline_text_edit(source)
            .font(TextStyle::Monospace)
            .desired_rows(desired_rows)
            .desired_width(f32::INFINITY)
            .layouter(&mut layouter),
    )
}

fn highlight_yaml(font: FontId, palette: theme::Palette, source: &str) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    let plain = egui::TextFormat::simple(font.clone(), palette.ink);
    let key = egui::TextFormat::simple(font.clone(), palette.accent);
    let quoted = egui::TextFormat::simple(font.clone(), palette.amber);
    let comment = egui::TextFormat::simple(font, palette.muted);

    let mut block_scalar_parent_indent = None;
    for line in source.split_inclusive('\n') {
        let indentation = line.len() - line.trim_start_matches([' ', '\t']).len();
        if let Some(parent_indent) = block_scalar_parent_indent {
            if line.trim().is_empty() || indentation > parent_indent {
                job.append(line, 0.0, plain.clone());
                continue;
            }
            block_scalar_parent_indent = None;
        }

        let comment_start = yaml_syntax::comment_start(line).unwrap_or(line.len());
        let code = &line[..comment_start];
        if let Some((key_start, colon)) = yaml_key_range(code) {
            job.append(&code[..key_start], 0.0, plain.clone());
            job.append(&code[key_start..colon], 0.0, key.clone());
            job.append(&code[colon..=colon], 0.0, plain.clone());
            let value = &code[colon + 1..];
            append_quoted_scalars(&mut job, value, &plain, &quoted);
            if value.trim_start().starts_with(['|', '>']) {
                block_scalar_parent_indent = Some(indentation);
            }
        } else {
            append_quoted_scalars(&mut job, code, &plain, &quoted);
        }
        if comment_start < line.len() {
            job.append(&line[comment_start..], 0.0, comment.clone());
        }
    }
    job
}

fn yaml_key_range(code: &str) -> Option<(usize, usize)> {
    let indentation = code.len() - code.trim_start().len();
    let mut key_start = indentation;
    if code[key_start..].starts_with("- ") {
        key_start += 2;
    }
    let candidate = &code[key_start..];
    let mut quote = None;
    let mut escaped = false;
    for (relative, character) in candidate.char_indices() {
        match quote {
            Some('"') if escaped => escaped = false,
            Some('"') if character == '\\' => escaped = true,
            Some(current) if character == current => quote = None,
            Some(_) => {}
            None if matches!(character, '\'' | '"') => quote = Some(character),
            None if character == ':' => {
                let colon = key_start + relative;
                let follows_separator = code[colon + 1..]
                    .chars()
                    .next()
                    .is_none_or(char::is_whitespace);
                let key = code[key_start..colon].trim();
                return (!key.is_empty() && follows_separator).then_some((key_start, colon));
            }
            None => {}
        }
    }
    None
}

fn append_quoted_scalars(
    job: &mut egui::text::LayoutJob,
    value: &str,
    plain: &egui::TextFormat,
    quoted: &egui::TextFormat,
) {
    let mut segment_start = 0;
    let mut quote_start = None;
    let mut quote = '\0';
    let mut escaped = false;
    for (index, character) in value.char_indices() {
        if let Some(start) = quote_start {
            if quote == '"' && escaped {
                escaped = false;
            } else if quote == '"' && character == '\\' {
                escaped = true;
            } else if character == quote {
                let end = index + character.len_utf8();
                job.append(&value[segment_start..start], 0.0, plain.clone());
                job.append(&value[start..end], 0.0, quoted.clone());
                segment_start = end;
                quote_start = None;
            }
        } else if matches!(character, '\'' | '"') {
            quote_start = Some(index);
            quote = character;
        }
    }
    if let Some(start) = quote_start {
        job.append(&value[segment_start..start], 0.0, plain.clone());
        job.append(&value[start..], 0.0, quoted.clone());
    } else {
        job.append(&value[segment_start..], 0.0, plain.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlighting_preserves_every_yaml_byte_and_marks_keys_strings_and_comments() {
        let source =
            "matches:\n  - trigger: \";sig#literal\" # visible comment\n    replace: 'Hello'\n";
        for palette in [theme::LIGHT_PALETTE, theme::DARK_PALETTE] {
            let job = highlight_yaml(FontId::monospace(theme::TEXT_BODY), palette, source);
            assert_eq!(job.text, source);

            let colored = job
                .sections
                .iter()
                .map(|section| {
                    (
                        &job.text[section.byte_range.start.0..section.byte_range.end.0],
                        section.format.color,
                    )
                })
                .collect::<Vec<_>>();
            assert!(
                colored.contains(&("matches", palette.accent)),
                "{colored:#?}"
            );
            assert!(
                colored.contains(&("trigger", palette.accent)),
                "{colored:#?}"
            );
            assert!(
                colored.contains(&("\";sig#literal\"", palette.amber)),
                "{colored:#?}"
            );
            assert!(
                colored.contains(&("# visible comment\n", palette.muted)),
                "{colored:#?}"
            );
        }
    }

    #[test]
    fn urls_and_hashes_inside_scalars_are_not_mistaken_for_yaml_syntax() {
        assert_eq!(
            yaml_syntax::comment_start("url: https://example.com/#docs\n"),
            None
        );
        assert_eq!(
            yaml_syntax::comment_start("value: \"text # stays quoted\" # comment"),
            Some(29)
        );
        assert_eq!(yaml_key_range("  - https://example.com"), None);
    }

    #[test]
    fn block_scalar_content_is_not_mistaken_for_keys_or_comments() {
        let source =
            "replace: |\n  literal: # this is replacement text\n  next line\nlabel: done\n";
        let palette = theme::LIGHT_PALETTE;
        let job = highlight_yaml(FontId::monospace(theme::TEXT_BODY), palette, source);
        assert_eq!(job.text, source);

        let literal_offset = source.find("  literal:").unwrap();
        let literal_section = job
            .sections
            .iter()
            .find(|section| {
                section.byte_range.start.0 <= literal_offset
                    && literal_offset < section.byte_range.end.0
            })
            .expect("block scalar section");
        assert_eq!(literal_section.format.color, palette.ink);

        let label_offset = source.find("label:").unwrap();
        let label_section = job
            .sections
            .iter()
            .find(|section| {
                section.byte_range.start.0 <= label_offset
                    && label_offset < section.byte_range.end.0
            })
            .expect("following YAML key section");
        assert_eq!(label_section.format.color, palette.accent);
    }
}
