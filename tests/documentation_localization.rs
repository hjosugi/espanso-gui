use std::fs;
use std::path::{Path, PathBuf};

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn markdown_files(directory: &Path) -> Vec<PathBuf> {
    let mut files = fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("could not read {}: {error}", directory.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "md"))
        .collect::<Vec<_>>();
    files.sort();
    files
}

fn markdown_tree(directory: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("could not read {}: {error}", directory.display()))
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if path.is_dir() {
            files.extend(markdown_tree(&path));
        } else if path.extension().is_some_and(|extension| extension == "md") {
            files.push(path);
        }
    }
    files.sort();
    files
}

fn contains_japanese(text: &str) -> bool {
    text.chars().any(|character| {
        matches!(
            character,
            '\u{3040}'..='\u{30ff}' | '\u{3400}'..='\u{4dbf}' | '\u{4e00}'..='\u{9fff}'
        )
    })
}

fn assert_language_pair(english: &Path, japanese: &Path) {
    assert!(
        english.is_file(),
        "missing English document: {}",
        english.display()
    );
    assert!(
        japanese.is_file(),
        "missing Japanese document: {}",
        japanese.display()
    );

    let english_text = fs::read_to_string(english).unwrap();
    let japanese_text = fs::read_to_string(japanese).unwrap();
    assert!(
        english_text.contains("[日本語]"),
        "English document has no Japanese switch: {}",
        english.display()
    );
    assert!(
        japanese_text.contains("[English]"),
        "Japanese document has no English switch: {}",
        japanese.display()
    );
    assert!(
        contains_japanese(&japanese_text),
        "Japanese document has no Japanese prose: {}",
        japanese.display()
    );
}

#[test]
fn every_public_document_has_an_english_and_japanese_version() {
    let root = repository_root();
    for (english, japanese) in [
        ("README.md", "README.ja.md"),
        ("CHANGELOG.md", "CHANGELOG.ja.md"),
        ("CONTRIBUTING.md", "CONTRIBUTING.ja.md"),
        ("SECURITY.md", "SECURITY.ja.md"),
    ] {
        assert_language_pair(&root.join(english), &root.join(japanese));
    }

    for (english_directory, japanese_directory) in [
        (root.join("docs"), root.join("docs/ja")),
        (root.join("docs/issues"), root.join("docs/ja/issues")),
        (root.join("docs/releases"), root.join("docs/ja/releases")),
    ] {
        for english in markdown_files(&english_directory) {
            let japanese = japanese_directory.join(english.file_name().unwrap());
            assert_language_pair(&english, &japanese);
        }
    }
}

#[test]
fn issue_spec_front_matter_remains_valid() {
    let root = repository_root();
    for directory in [root.join("docs/issues"), root.join("docs/ja/issues")] {
        for path in markdown_files(&directory) {
            let text = fs::read_to_string(&path).unwrap();
            let normalized = text.replace("\r\n", "\n");
            assert!(
                normalized.starts_with("---\ntitle:"),
                "broken issue front matter in {}",
                path.display()
            );
            assert!(
                normalized.contains("\nlabels:") && normalized.contains("\n---\n"),
                "incomplete issue front matter in {}",
                path.display()
            );
        }
    }
}

#[test]
fn repository_contribution_surfaces_and_app_metadata_are_bilingual() {
    let root = repository_root();
    for relative in [
        ".github/pull_request_template.md",
        ".github/ISSUE_TEMPLATE/bug_report.yml",
        ".github/ISSUE_TEMPLATE/feature_request.yml",
        ".github/ISSUE_TEMPLATE/config.yml",
        ".github/labels.yml",
    ] {
        let text = fs::read_to_string(root.join(relative)).unwrap();
        if relative.ends_with(".yml") {
            serde_yaml_ng::from_str::<serde_yaml_ng::Value>(&text)
                .unwrap_or_else(|error| panic!("invalid YAML in {relative}: {error}"));
        }
        assert!(
            contains_japanese(&text),
            "missing Japanese copy in {relative}"
        );
    }

    let metadata =
        fs::read_to_string(root.join("packaging/dev.hjosugi.espanso-gui.metainfo.xml")).unwrap();
    assert!(metadata.contains("xml:lang=\"ja\""));
    assert!(contains_japanese(&metadata));
}

#[test]
fn local_documentation_links_resolve() {
    let root = repository_root();
    let mut documents = markdown_tree(&root.join("docs"));
    documents.extend(
        [
            "README.md",
            "README.ja.md",
            "CHANGELOG.md",
            "CHANGELOG.ja.md",
            "CONTRIBUTING.md",
            "CONTRIBUTING.ja.md",
            "SECURITY.md",
            "SECURITY.ja.md",
        ]
        .map(|relative| root.join(relative)),
    );

    for document in documents {
        let text = fs::read_to_string(&document).unwrap();
        for remainder in text.split("](").skip(1) {
            let Some(raw_target) = remainder.split(')').next() else {
                continue;
            };
            let target = raw_target.trim_matches(['<', '>']);
            if target.is_empty()
                || target.starts_with('#')
                || target.starts_with("https://")
                || target.starts_with("http://")
                || target.starts_with("mailto:")
            {
                continue;
            }
            let path_without_fragment = target.split('#').next().unwrap();
            let resolved = document.parent().unwrap().join(path_without_fragment);
            assert!(
                resolved.exists(),
                "broken local link {target:?} in {}",
                document.display()
            );
        }
    }
}
