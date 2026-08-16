use crate::i18n::{self, Language, TextKey};
use crate::storage::WorkspaceFile;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SnippetSort {
    #[default]
    FileOrder,
    Name,
    Trigger,
}

impl SnippetSort {
    pub(crate) const ALL: [Self; 3] = [Self::FileOrder, Self::Name, Self::Trigger];

    pub(crate) fn text_key(self) -> TextKey {
        match self {
            Self::FileOrder => TextKey::SortFileOrder,
            Self::Name => TextKey::SortName,
            Self::Trigger => TextKey::SortTrigger,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SnippetListEntry {
    pub(crate) file_index: usize,
    pub(crate) snippet_index: usize,
    pub(crate) title: String,
    pub(crate) triggers: String,
    pub(crate) preview: String,
    pub(crate) context: String,
}

pub(crate) fn entries(
    files: &[WorkspaceFile],
    selected_file: usize,
    query: &str,
    language: Language,
    sort: SnippetSort,
) -> Vec<SnippetListEntry> {
    let query = query.trim().to_lowercase();
    let searching = !query.is_empty();
    let mut entries = files
        .iter()
        .enumerate()
        .filter(|(file_index, _)| searching || *file_index == selected_file)
        .flat_map(|(file_index, file)| {
            let query = &query;
            file.document
                .matches
                .iter()
                .enumerate()
                .filter(move |(_, snippet)| !searching || snippet.searchable_text().contains(query))
                .map(move |(snippet_index, snippet)| {
                    let kind = i18n::content_kind_label(language, snippet.content_kind());
                    SnippetListEntry {
                        file_index,
                        snippet_index,
                        title: snippet.title().unwrap_or_else(|| {
                            i18n::text(language, TextKey::UntitledSnippet).into()
                        }),
                        triggers: snippet.trigger_list().join("  "),
                        preview: snippet
                            .content()
                            .lines()
                            .next()
                            .unwrap_or_default()
                            .to_owned(),
                        context: if searching {
                            format!("{kind} · {}", file.relative_path.display())
                        } else {
                            kind.to_owned()
                        },
                    }
                })
        })
        .collect::<Vec<_>>();
    match sort {
        SnippetSort::FileOrder => {}
        SnippetSort::Name => entries.sort_by(|left, right| {
            left.title
                .to_lowercase()
                .cmp(&right.title.to_lowercase())
                .then_with(|| left.file_index.cmp(&right.file_index))
                .then_with(|| left.snippet_index.cmp(&right.snippet_index))
        }),
        SnippetSort::Trigger => entries.sort_by(|left, right| {
            left.triggers
                .to_lowercase()
                .cmp(&right.triggers.to_lowercase())
                .then_with(|| left.file_index.cmp(&right.file_index))
                .then_with(|| left.snippet_index.cmp(&right.snippet_index))
        }),
    }
    entries
}

pub(crate) fn search_terms(files: &[WorkspaceFile]) -> Vec<(String, usize)> {
    let mut terms = BTreeMap::<String, (String, usize)>::new();
    for snippet in files.iter().flat_map(|file| &file.document.matches) {
        for term in &snippet.search_terms {
            let term = term.trim();
            if term.is_empty() {
                continue;
            }
            let entry = terms
                .entry(term.to_lowercase())
                .or_insert_with(|| (term.to_owned(), 0));
            entry.1 += 1;
        }
    }
    terms.into_values().collect()
}
