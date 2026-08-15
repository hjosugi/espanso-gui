use crate::model::{ConfigProfile, MatchFile};
use serde::Serialize;

pub fn patch_match_file(
    source: &str,
    previous: &MatchFile,
    next: &MatchFile,
) -> Result<String, serde_yaml_ng::Error> {
    if previous == next {
        return Ok(source.to_string());
    }

    // Unknown top-level values are only editable in Raw YAML mode. Falling back here avoids
    // silently discarding a future structured field that has not received a lossless patcher.
    if previous.extra != next.extra {
        return next.to_yaml();
    }

    let patched = patch_sequence(
        source,
        "global_vars",
        &previous.global_vars,
        &next.global_vars,
    )?;
    patch_sequence(&patched, "matches", &previous.matches, &next.matches)
}

pub fn patch_config_profile(
    source: &str,
    previous: &ConfigProfile,
    next: &ConfigProfile,
) -> Result<String, serde_yaml_ng::Error> {
    if previous == next {
        return Ok(source.to_string());
    }
    if previous.extra != next.extra {
        return next.to_yaml();
    }

    const KEYS: [&str; 22] = [
        "filter_title",
        "filter_exec",
        "filter_class",
        "filter_os",
        "enable",
        "backend",
        "apply_patch",
        "inject_delay",
        "key_delay",
        "pre_paste_delay",
        "paste_shortcut_event_delay",
        "post_form_delay",
        "post_search_delay",
        "paste_shortcut",
        "max_form_width",
        "max_form_height",
        "search_shortcut",
        "search_trigger",
        "preserve_clipboard",
        "show_icon",
        "show_notifications",
        "toggle_key",
    ];
    let previous_value = serde_yaml_ng::to_value(previous)?;
    let next_value = serde_yaml_ng::to_value(next)?;
    let mut output = if source.trim() == "{}" {
        String::new()
    } else {
        source.to_string()
    };
    for key in KEYS {
        let old = mapping_value(&previous_value, key);
        let new = mapping_value(&next_value, key);
        if old != new {
            output = patch_top_level_value(&output, key, new)?;
        }
    }
    Ok(output)
}

fn mapping_value<'a>(
    value: &'a serde_yaml_ng::Value,
    key: &str,
) -> Option<&'a serde_yaml_ng::Value> {
    value
        .as_mapping()
        .and_then(|mapping| mapping.get(serde_yaml_ng::Value::String(key.into())))
}

fn patch_top_level_value(
    source: &str,
    key: &str,
    value: Option<&serde_yaml_ng::Value>,
) -> Result<String, serde_yaml_ng::Error> {
    let line_ending = if source.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let lines: Vec<&str> = source.split_inclusive('\n').collect();
    let existing = lines.iter().position(|line| is_top_level_key(line, key));
    let replacement = if let Some(value) = value {
        let mut mapping = serde_yaml_ng::Mapping::new();
        mapping.insert(serde_yaml_ng::Value::String(key.into()), value.clone());
        let serialized = serde_yaml_ng::to_string(&mapping)?;
        let first_line = serialized.lines().next().unwrap_or_default();
        let comment = existing
            .and_then(|index| {
                strip_line_ending(lines[index])
                    .find('#')
                    .map(|at| (index, at))
            })
            .map(|(index, at)| strip_line_ending(lines[index])[at..].trim_end());
        format!(
            "{first_line}{}{line_ending}",
            comment.map(|value| format!(" {value}")).unwrap_or_default()
        )
    } else {
        String::new()
    };

    if let Some(index) = existing {
        let mut output = String::new();
        output.push_str(&lines[..index].concat());
        output.push_str(&replacement);
        output.push_str(&lines[index + 1..].concat());
        Ok(output)
    } else if value.is_some() {
        let mut output = source.to_string();
        if !output.is_empty() && !output.ends_with('\n') {
            output.push_str(line_ending);
        }
        output.push_str(&replacement);
        Ok(output)
    } else {
        Ok(source.to_string())
    }
}

fn patch_sequence<T>(
    source: &str,
    key: &str,
    previous: &[T],
    next: &[T],
) -> Result<String, serde_yaml_ng::Error>
where
    T: Serialize + PartialEq,
{
    if previous == next {
        return Ok(source.to_string());
    }

    let line_ending = if source.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let lines: Vec<&str> = source.split_inclusive('\n').collect();
    let Some(header_index) = lines.iter().position(|line| is_top_level_key(line, key)) else {
        let mut output = source.to_string();
        if !output.is_empty() && !output.ends_with('\n') {
            output.push_str(line_ending);
        }
        output.push_str(key);
        output.push(':');
        if next.is_empty() {
            output.push_str(" []");
            output.push_str(line_ending);
        } else {
            output.push_str(line_ending);
            for item in next {
                output.push_str(&serialize_item(item, 2, line_ending)?);
            }
        }
        return Ok(output);
    };

    let section_end = lines
        .iter()
        .enumerate()
        .skip(header_index + 1)
        .find_map(|(index, line)| is_top_level_value(line).then_some(index))
        .unwrap_or(lines.len());
    let item_starts = find_item_starts(&lines, header_index + 1, section_end);
    let item_indent = item_starts
        .first()
        .map(|index| indentation(lines[*index]))
        .unwrap_or(2);
    let prefix_end = item_starts.first().copied().unwrap_or(section_end);
    let prefix = lines[header_index + 1..prefix_end].concat();
    let chunks = item_chunks(&lines, &item_starts, section_end, item_indent);
    let matches = lcs_matches(previous, next);
    let mut used_old = vec![false; previous.len()];

    let mut body = prefix;
    for (next_index, item) in next.iter().enumerate() {
        if let Some(old_index) = matches[next_index] {
            used_old[old_index] = true;
            if let Some(chunk) = chunks.get(old_index) {
                body.push_str(&chunk.core);
                body.push_str(&chunk.suffix);
                continue;
            }
        }

        body.push_str(&serialize_item(item, item_indent, line_ending)?);
        if let Some(old_index) = (0..previous.len()).find(|index| {
            !used_old[*index]
                && matches.iter().all(|matched| *matched != Some(*index))
                && *index == next_index
        }) {
            used_old[old_index] = true;
            if let Some(chunk) = chunks.get(old_index) {
                body.push_str(&chunk.suffix);
            }
        }
    }

    // Separator comments belonging to a removed item are retained at the end of the sequence.
    for (index, chunk) in chunks.iter().enumerate() {
        if !used_old[index] && !chunk.suffix.is_empty() {
            body.push_str(&chunk.suffix);
        }
    }

    let original_header = strip_line_ending(lines[header_index]);
    let comment = original_header
        .find('#')
        .map(|index| original_header[index..].trim_end());
    let block_header = original_header
        .split_once(':')
        .is_some_and(|(_, value)| value.trim().is_empty() || value.trim_start().starts_with('#'));
    let header = if next.is_empty() {
        format!(
            "{key}: []{}{line_ending}",
            comment.map(|value| format!(" {value}")).unwrap_or_default()
        )
    } else if block_header {
        lines[header_index].to_string()
    } else {
        format!(
            "{key}:{}{line_ending}",
            comment.map(|value| format!(" {value}")).unwrap_or_default()
        )
    };

    let mut output = String::new();
    output.push_str(&lines[..header_index].concat());
    output.push_str(&header);
    output.push_str(&body);
    output.push_str(&lines[section_end..].concat());
    Ok(output)
}

#[derive(Debug)]
struct ItemChunk {
    core: String,
    suffix: String,
}

fn item_chunks(
    lines: &[&str],
    starts: &[usize],
    section_end: usize,
    item_indent: usize,
) -> Vec<ItemChunk> {
    starts
        .iter()
        .enumerate()
        .map(|(index, start)| {
            let end = starts.get(index + 1).copied().unwrap_or(section_end);
            let mut core_end = end;
            while core_end > *start + 1 && is_separator_line(lines[core_end - 1], item_indent) {
                core_end -= 1;
            }
            ItemChunk {
                core: lines[*start..core_end].concat(),
                suffix: lines[core_end..end].concat(),
            }
        })
        .collect()
}

fn find_item_starts(lines: &[&str], section_start: usize, section_end: usize) -> Vec<usize> {
    let indent = lines[section_start..section_end]
        .iter()
        .filter_map(|line| {
            let trimmed = strip_line_ending(line).trim_start();
            is_sequence_marker(trimmed).then_some(indentation(line))
        })
        .min();
    let Some(indent) = indent else {
        return Vec::new();
    };
    (section_start..section_end)
        .filter(|index| {
            indentation(lines[*index]) == indent
                && is_sequence_marker(strip_line_ending(lines[*index]).trim_start())
        })
        .collect()
}

fn lcs_matches<T: PartialEq>(previous: &[T], next: &[T]) -> Vec<Option<usize>> {
    let mut lengths = vec![vec![0_usize; next.len() + 1]; previous.len() + 1];
    for old in (0..previous.len()).rev() {
        for new in (0..next.len()).rev() {
            lengths[old][new] = if previous[old] == next[new] {
                lengths[old + 1][new + 1] + 1
            } else {
                lengths[old + 1][new].max(lengths[old][new + 1])
            };
        }
    }

    let mut matches = vec![None; next.len()];
    let (mut old, mut new) = (0, 0);
    while old < previous.len() && new < next.len() {
        if previous[old] == next[new] {
            matches[new] = Some(old);
            old += 1;
            new += 1;
        } else if lengths[old + 1][new] >= lengths[old][new + 1] {
            old += 1;
        } else {
            new += 1;
        }
    }
    matches
}

fn serialize_item<T: Serialize>(
    item: &T,
    indent: usize,
    line_ending: &str,
) -> Result<String, serde_yaml_ng::Error> {
    let serialized = serde_yaml_ng::to_string(&[item])?;
    let padding = " ".repeat(indent);
    let mut output = String::new();
    for line in serialized.lines() {
        output.push_str(&padding);
        output.push_str(line);
        output.push_str(line_ending);
    }
    Ok(output)
}

fn is_top_level_key(line: &str, key: &str) -> bool {
    if indentation(line) != 0 {
        return false;
    }
    strip_line_ending(line)
        .strip_prefix(key)
        .is_some_and(|rest| rest.starts_with(':'))
}

fn is_top_level_value(line: &str) -> bool {
    let line = strip_line_ending(line);
    !line.trim().is_empty()
        && !line.trim_start().starts_with('#')
        && indentation(line) == 0
        && line != "---"
}

fn is_sequence_marker(trimmed: &str) -> bool {
    trimmed == "-" || trimmed.starts_with("- ")
}

fn is_separator_line(line: &str, item_indent: usize) -> bool {
    let line = strip_line_ending(line);
    line.trim().is_empty()
        || (line.trim_start().starts_with('#') && indentation(line) <= item_indent)
}

fn indentation(line: &str) -> usize {
    line.bytes().take_while(|byte| *byte == b' ').count()
}

fn strip_line_ending(line: &str) -> &str {
    line.strip_suffix("\n")
        .unwrap_or(line)
        .strip_suffix("\r")
        .unwrap_or_else(|| line.strip_suffix("\n").unwrap_or(line))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editing_one_match_preserves_unrelated_yaml_byte_for_byte() {
        let source = r#"# file header
custom: &shared "quoted: value"
matches:
  # first match comment
  - trigger: ":one" # inline comment
    replace: |
      first line
      # block scalar text
    future: *shared

  # second match comment
  - trigger: ':two'
    replace: old
# file footer
"#;
        let previous = MatchFile::from_yaml(source).unwrap();
        let mut next = previous.clone();
        next.matches[1].replace = Some("new".into());

        let output = patch_match_file(source, &previous, &next).unwrap();
        let unchanged = r#"  # first match comment
  - trigger: ":one" # inline comment
    replace: |
      first line
      # block scalar text
    future: *shared

"#;
        assert!(output.contains(unchanged));
        assert!(output.starts_with("# file header\ncustom: &shared \"quoted: value\"\n"));
        assert!(output.ends_with("# file footer\n"));
        assert!(output.contains("replace: new"));
        assert_eq!(MatchFile::from_yaml(&output).unwrap(), next);
    }

    #[test]
    fn adding_to_an_empty_flow_sequence_keeps_the_header_comment() {
        let source = "matches: [] # keep this\ncustom: value\n";
        let previous = MatchFile::from_yaml(source).unwrap();
        let mut next = previous.clone();
        next.matches.push(crate::model::Snippet::new());

        let output = patch_match_file(source, &previous, &next).unwrap();
        assert!(output.starts_with("matches: # keep this\n  - trigger: :new\n"));
        assert!(output.ends_with("custom: value\n"));
        assert_eq!(MatchFile::from_yaml(&output).unwrap(), next);
    }

    #[test]
    fn inserting_a_match_keeps_existing_items_exact() {
        let source =
            "matches:\n  - trigger: :one\n    replace: one\n  - trigger: :two\n    replace: two\n";
        let previous = MatchFile::from_yaml(source).unwrap();
        let mut next = previous.clone();
        next.matches.insert(0, crate::model::Snippet::new());

        let output = patch_match_file(source, &previous, &next).unwrap();
        assert!(output.contains("  - trigger: :one\n    replace: one\n"));
        assert!(output.contains("  - trigger: :two\n    replace: two\n"));
        assert_eq!(MatchFile::from_yaml(&output).unwrap(), next);
    }

    #[test]
    fn config_profile_patch_changes_only_the_selected_option_line() {
        let source = "# profile\nfilter_exec: 'Code|VSCodium' # regex\ncustom: &value keep\nbackend: auto\nfuture: *value\n";
        let previous = ConfigProfile::from_yaml(source).unwrap();
        let mut next = previous.clone();
        next.backend = Some("clipboard".into());

        let output = patch_config_profile(source, &previous, &next).unwrap();
        assert!(output.starts_with("# profile\nfilter_exec: 'Code|VSCodium' # regex\n"));
        assert!(output.contains("custom: &value keep\n"));
        assert!(output.contains("backend: clipboard\n"));
        assert!(output.ends_with("future: *value\n"));
        assert_eq!(ConfigProfile::from_yaml(&output).unwrap(), next);
    }
}
