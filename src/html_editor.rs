use crate::i18n::{self, Language, TextKey};

#[derive(Debug, Clone, Copy)]
pub(crate) enum HtmlCommand {
    Bold,
    Italic,
    Heading,
    Link,
    UnorderedList,
    OrderedList,
    Color,
    Image,
}

pub(crate) fn fragment(language: Language, command: HtmlCommand) -> String {
    match command {
        HtmlCommand::Bold => format!("<strong>{}</strong>", i18n::text(language, TextKey::Bold)),
        HtmlCommand::Italic => format!("<em>{}</em>", i18n::text(language, TextKey::Italic)),
        HtmlCommand::Heading => format!("<h2>{}</h2>", i18n::text(language, TextKey::Heading)),
        HtmlCommand::Link => format!(
            "<a href=\"https://example.com\">{}</a>",
            i18n::text(language, TextKey::Link)
        ),
        HtmlCommand::UnorderedList => format!(
            "<ul><li>{}</li><li>{}</li></ul>",
            i18n::text(language, TextKey::ListItemOne),
            i18n::text(language, TextKey::ListItemTwo)
        ),
        HtmlCommand::OrderedList => format!(
            "<ol><li>{}</li><li>{}</li></ol>",
            i18n::text(language, TextKey::ListItemOne),
            i18n::text(language, TextKey::ListItemTwo)
        ),
        HtmlCommand::Color => format!(
            "<span style=\"color: #197966\">{}</span>",
            i18n::text(language, TextKey::ColoredText)
        ),
        HtmlCommand::Image => format!(
            "<img src=\"$CONFIG/assets/image.png\" alt=\"{}\">",
            i18n::text(language, TextKey::Image)
        ),
    }
}

pub(crate) fn safe_preview(language: Language, html: &str) -> String {
    let lower = html.to_ascii_lowercase();
    let mut output = String::new();
    let mut position = 0;
    while position < html.len() {
        let Some(relative_start) = html[position..].find('<') else {
            output.push_str(&decode_entities(&html[position..]));
            break;
        };
        let start = position + relative_start;
        output.push_str(&decode_entities(&html[position..start]));
        let Some(relative_end) = html[start..].find('>') else {
            output.push_str(&decode_entities(&html[start..]));
            break;
        };
        let end = start + relative_end;
        let tag = lower[start + 1..end].trim();
        if tag.starts_with("script") || tag.starts_with("style") {
            let closing = if tag.starts_with("script") {
                "</script"
            } else {
                "</style"
            };
            let Some(close_start) = lower[end + 1..].find(closing) else {
                break;
            };
            let close_start = end + 1 + close_start;
            let Some(close_end) = html[close_start..].find('>') else {
                break;
            };
            position = close_start + close_end + 1;
            continue;
        }
        if tag.starts_with("img") {
            output.push('[');
            output.push_str(i18n::text(language, TextKey::Image));
            output.push(']');
        } else if tag == "br" || tag == "br/" || is_block_boundary(tag) {
            if !output.ends_with('\n') {
                output.push('\n');
            }
            if tag.starts_with("li") {
                output.push_str("• ");
            }
        }
        position = end + 1;
    }
    output
        .lines()
        .map(str::trim_end)
        .fold(String::new(), |mut preview, line| {
            if !preview.is_empty() {
                preview.push('\n');
            }
            preview.push_str(line);
            preview
        })
        .trim()
        .to_string()
}

fn is_block_boundary(tag: &str) -> bool {
    [
        "p", "/p", "div", "/div", "h1", "/h1", "h2", "/h2", "h3", "/h3", "ul", "/ul", "ol", "/ol",
        "li", "/li",
    ]
    .contains(&tag)
}

fn decode_entities(text: &str) -> String {
    text.replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composer_emits_predictable_portable_fragments() {
        assert_eq!(
            fragment(Language::Japanese, HtmlCommand::Bold),
            "<strong>太字</strong>"
        );
        assert_eq!(
            fragment(Language::Japanese, HtmlCommand::UnorderedList),
            "<ul><li>項目1</li><li>項目2</li></ul>"
        );
        assert_eq!(
            fragment(Language::English, HtmlCommand::Link),
            "<a href=\"https://example.com\">Link</a>"
        );
        assert!(fragment(Language::Japanese, HtmlCommand::Color).contains("color: #197966"));
        assert!(
            fragment(Language::Japanese, HtmlCommand::Image).contains("$CONFIG/assets/image.png")
        );
    }

    #[test]
    fn preview_never_exposes_active_content_or_remote_urls() {
        let html = r#"<h2>Hello</h2><script>steal()</script><style>body{}</style><img src="https://example.com/tracker.png"><p>Safe &amp; sound</p>"#;
        let preview = safe_preview(Language::Japanese, html);
        assert!(preview.contains("Hello"));
        assert!(preview.contains("[画像]"));
        assert!(preview.contains("Safe & sound"));
        assert!(!preview.contains("steal"));
        assert!(!preview.contains("body"));
        assert!(!preview.contains("https://"));
        assert!(safe_preview(Language::English, "<img src=\"x\">").contains("[Image]"));
    }
}
