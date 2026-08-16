/// Returns the byte offset of a YAML comment marker.
///
/// A `#` starts a comment only outside quoted scalars and when it begins the line or follows
/// whitespace. This is intentionally a small lexical helper rather than a YAML parser; callers use
/// it only to preserve or color source text that has already been parsed separately.
pub(crate) fn comment_start(line: &str) -> Option<usize> {
    let mut quote = None;
    let mut escaped = false;
    let mut previous = None;
    for (index, character) in line.char_indices() {
        match quote {
            Some('"') if escaped => escaped = false,
            Some('"') if character == '\\' => escaped = true,
            Some(current) if character == current => quote = None,
            Some(_) => {}
            None if matches!(character, '\'' | '"') => quote = Some(character),
            None if character == '#' && previous.is_none_or(char::is_whitespace) => {
                return Some(index);
            }
            None => {}
        }
        previous = Some(character);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distinguishes_plain_hashes_quoted_hashes_and_comments() {
        assert_eq!(comment_start("# comment"), Some(0));
        assert_eq!(comment_start("value # comment"), Some(6));
        assert_eq!(comment_start("https://example.com/#docs"), None);
        assert_eq!(comment_start("\"# quoted\" # comment"), Some(11));
        assert_eq!(comment_start("'# quoted' # comment"), Some(11));
        assert_eq!(
            comment_start("\"escaped \\\"# quoted\" # comment"),
            Some(21)
        );
    }
}
