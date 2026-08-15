use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    Japanese,
    English,
}

impl Language {
    pub const ALL: [Self; 2] = [Self::Japanese, Self::English];

    pub fn native_name(self) -> &'static str {
        match self {
            Self::Japanese => "日本語",
            Self::English => "English",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TextKey {
    Workspace,
    Snippets,
    Profiles,
    Globals,
    Diagnostics,
    Settings,
    About,
    Save,
    Reload,
    RestartEspanso,
    Unsaved,
    AddFile,
    NewSnippet,
    Search,
    SearchHint,
    Connected,
    NotDetected,
    Accessibility,
    Language,
    UiScale,
    KeyboardShortcuts,
    ShortcutHelp,
}

impl TextKey {
    #[cfg(test)]
    const ALL: [Self; 22] = [
        Self::Workspace,
        Self::Snippets,
        Self::Profiles,
        Self::Globals,
        Self::Diagnostics,
        Self::Settings,
        Self::About,
        Self::Save,
        Self::Reload,
        Self::RestartEspanso,
        Self::Unsaved,
        Self::AddFile,
        Self::NewSnippet,
        Self::Search,
        Self::SearchHint,
        Self::Connected,
        Self::NotDetected,
        Self::Accessibility,
        Self::Language,
        Self::UiScale,
        Self::KeyboardShortcuts,
        Self::ShortcutHelp,
    ];
}

pub fn text(language: Language, key: TextKey) -> &'static str {
    match (language, key) {
        (Language::Japanese, TextKey::Workspace) => "ワークスペース",
        (Language::English, TextKey::Workspace) => "Workspace",
        (Language::Japanese, TextKey::Snippets) => "スニペット",
        (Language::English, TextKey::Snippets) => "Snippets",
        (Language::Japanese, TextKey::Profiles) => "アプリ別設定",
        (Language::English, TextKey::Profiles) => "App profiles",
        (Language::Japanese, TextKey::Globals) => "グローバル変数",
        (Language::English, TextKey::Globals) => "Global variables",
        (Language::Japanese, TextKey::Diagnostics) => "診断",
        (Language::English, TextKey::Diagnostics) => "Diagnostics",
        (Language::Japanese, TextKey::Settings) => "設定とバックアップ",
        (Language::English, TextKey::Settings) => "Settings & backups",
        (Language::Japanese, TextKey::About) => "このアプリについて",
        (Language::English, TextKey::About) => "About this app",
        (Language::Japanese, TextKey::Save) => "保存",
        (Language::English, TextKey::Save) => "Save",
        (Language::Japanese, TextKey::Reload) => "再読み込み",
        (Language::English, TextKey::Reload) => "Reload",
        (Language::Japanese, TextKey::RestartEspanso) => "Espanso再起動",
        (Language::English, TextKey::RestartEspanso) => "Restart Espanso",
        (Language::Japanese, TextKey::Unsaved) => "未保存",
        (Language::English, TextKey::Unsaved) => "Unsaved",
        (Language::Japanese, TextKey::AddFile) => "＋ ファイルを追加",
        (Language::English, TextKey::AddFile) => "+ Add file",
        (Language::Japanese, TextKey::NewSnippet) => "＋ 新規",
        (Language::English, TextKey::NewSnippet) => "+ New",
        (Language::Japanese, TextKey::Search) => "スニペットを検索",
        (Language::English, TextKey::Search) => "Search snippets",
        (Language::Japanese, TextKey::SearchHint) => "トリガー、内容、ラベル",
        (Language::English, TextKey::SearchHint) => "Trigger, content, or label",
        (Language::Japanese, TextKey::Connected) => "Espanso 接続済み",
        (Language::English, TextKey::Connected) => "Espanso connected",
        (Language::Japanese, TextKey::NotDetected) => "Espanso 未検出",
        (Language::English, TextKey::NotDetected) => "Espanso not detected",
        (Language::Japanese, TextKey::Accessibility) => "表示・アクセシビリティ",
        (Language::English, TextKey::Accessibility) => "Display & accessibility",
        (Language::Japanese, TextKey::Language) => "表示言語",
        (Language::English, TextKey::Language) => "Language",
        (Language::Japanese, TextKey::UiScale) => "UI拡大率",
        (Language::English, TextKey::UiScale) => "UI scale",
        (Language::Japanese, TextKey::KeyboardShortcuts) => "キーボード操作",
        (Language::English, TextKey::KeyboardShortcuts) => "Keyboard shortcuts",
        (Language::Japanese, TextKey::ShortcutHelp) => {
            "⌘/Ctrl+1〜5: 画面移動　⌘/Ctrl+F: 検索　⌘/Ctrl+S: 保存　⌘/Ctrl+N: 新規　Esc: dialogを閉じる"
        }
        (Language::English, TextKey::ShortcutHelp) => {
            "Cmd/Ctrl+1–5: navigate  Cmd/Ctrl+F: search  Cmd/Ctrl+S: save  Cmd/Ctrl+N: new  Esc: close dialogs"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_catalogs_cover_every_key() {
        for language in Language::ALL {
            for key in TextKey::ALL {
                assert!(!text(language, key).trim().is_empty());
            }
        }
    }
}
