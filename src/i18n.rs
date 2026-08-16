use crate::espanso::EspansoAction;
use crate::model::{ContentKind, DiagnosticKind, FormFieldKind};
use crate::storage::{StorageError, StorageIssue};
use crate::theme::Appearance;
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
struct Translation {
    japanese: &'static str,
    english: &'static str,
}

macro_rules! define_catalog {
    ($($key:ident => ($japanese:expr, $english:expr)),+ $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum TextKey {
            $($key),+
        }

        impl TextKey {
            #[cfg(test)]
            const ALL: &'static [Self] = &[$(Self::$key),+];

            fn translation(self) -> Translation {
                match self {
                    $(Self::$key => Translation {
                        japanese: $japanese,
                        english: $english,
                    }),+
                }
            }
        }
    };
}

define_catalog! {
    Workspace => ("ワークスペース", "Workspace"),
    Snippets => ("スニペット", "Snippets"),
    Profiles => ("アプリ別設定", "App profiles"),
    Globals => ("グローバル変数", "Global variables"),
    Diagnostics => ("診断", "Diagnostics"),
    SettingsNav => ("設定", "Settings"),
    Settings => ("設定とバックアップ", "Settings & backups"),
    About => ("このアプリについて", "About this app"),
    Save => ("保存", "Save"),
    Reload => ("再読み込み", "Reload"),
    RestartEspanso => ("Espanso再起動", "Restart Espanso"),
    Unsaved => ("未保存", "Unsaved"),
    AddFile => ("＋ ファイルを追加", "+ Add file"),
    MatchFiles => ("スニペットファイル", "Match files"),
    NewSnippet => ("＋ 新規", "+ New"),
    NewSnippetLabel => ("新しいスニペット", "New snippet"),
    NewSnippetContent => (
        "ここに展開するテキストを入力",
        "Enter replacement text here"
    ),
    UntitledSnippet => ("名称未設定", "Untitled snippet"),
    Search => ("スニペットを検索", "Search snippets"),
    SearchHint => (
        "トリガー・本文",
        "Trigger or text"
    ),
    NoSearchResults => ("一致するスニペットがありません", "No matching snippets"),
    NoSearchResultsDescription => (
        "検索語を変えるか、検索をクリアしてください。",
        "Try another term or clear the search."
    ),
    ClearSearch => ("検索をクリア", "Clear search"),
    SortBy => ("並べ替え", "Sort by"),
    SortFileOrder => ("YAMLの順序", "YAML order"),
    SortName => ("名前", "Name"),
    SortTrigger => ("トリガー", "Trigger"),
    FilterByTag => ("タグで絞り込み", "Filter by tag"),
    AllTags => ("すべてのタグ", "All tags"),
    Connected => ("Espanso 接続済み", "Espanso connected"),
    NotDetected => ("Espanso 未検出", "Espanso not detected"),
    ConnectedShort => ("接続済み", "Connected"),
    NotDetectedShort => ("未検出", "Not detected"),
    Accessibility => ("表示・アクセシビリティ", "Display & accessibility"),
    Language => ("表示言語", "Language"),
    Appearance => ("外観", "Appearance"),
    AppearanceDescription => (
        "OS設定に合わせるか、明るい／暗い表示を固定",
        "Follow the operating system or choose a fixed light or dark appearance"
    ),
    SystemAppearance => ("システム設定", "System"),
    LightAppearance => ("ライト", "Light"),
    DarkAppearance => ("ダーク", "Dark"),
    UiScale => ("UI拡大率", "UI scale"),
    KeyboardShortcuts => ("キーボード操作", "Keyboard shortcuts"),
    ShortcutHelp => (
        "⌘/Ctrl+1〜5: 画面移動　⌘/Ctrl+F: 検索　⌘/Ctrl+S: 保存　⌘/Ctrl+N: 新規　Esc: ダイアログを閉じる",
        "Cmd/Ctrl+1–5: navigate  Cmd/Ctrl+F: search  Cmd/Ctrl+S: save  Cmd/Ctrl+N: new  Esc: close dialogs"
    ),
    Duplicate => ("複製", "Duplicate"),
    Delete => ("削除", "Delete"),
    Edit => ("編集", "Edit"),
    Open => ("開く", "Open"),
    OpenFailed => ("開けませんでした", "Could not open"),
    Cancel => ("キャンセル", "Cancel"),
    Create => ("作成", "Create"),
    Content => ("内容", "Content"),
    Variables => ("変数", "Variables"),
    AdvancedOptions => ("詳細オプション", "Advanced options"),
    RawYaml => ("Raw YAML", "Raw YAML"),
    NoSnippetsTitle => ("まだスニペットがありません", "No snippets yet"),
    NoSnippetsDescription => (
        "最初のトリガーと展開内容を作成しましょう。",
        "Create your first trigger and replacement."
    ),
    CreateFirstSnippet => ("最初のスニペットを作成", "Create first snippet"),
    ReadOnlyPackage => ("読み取り専用パッケージ", "Read-only package"),
    Package => ("パッケージ", "package"),
    ProfileListTitle => ("設定プロファイル", "Configuration profiles"),
    AddProfile => ("＋ プロファイルを追加", "+ Add profile"),
    DefaultProfile => ("既定", "Default"),
    AppProfile => ("アプリ別", "App-specific"),
    MissingFilter => ("フィルター未設定", "Filter missing"),
    Visual => ("ビジュアル", "Visual"),
    NoProfilesTitle => ("設定プロファイルがありません", "No configuration profiles"),
    NoProfilesDescription => (
        "default またはアプリ別プロファイルを追加できます。",
        "Add the default profile or an app-specific profile."
    ),
    AddFirstProfile => ("最初のプロファイルを追加", "Add first profile"),
    GlobalsDescription => (
        "同じファイルと子ファイルのスニペットから利用できます。",
        "Available to snippets in this file and its child files."
    ),
    AddVariable => ("＋ 変数を追加", "+ Add variable"),
    NoFileTitle => ("ファイルがありません", "No file selected"),
    SelectConfigFolderDescription => (
        "設定フォルダを選択してください。",
        "Select a configuration folder."
    ),
    DiagnosticsTitle => ("設定診断", "Configuration diagnostics"),
    DiagnosticsDescription => (
        "保存前にトリガー、変数参照、Espansoの基本構造を確認します。",
        "Check triggers, variable references, and the basic Espanso structure before saving."
    ),
    NoProblems => (
        "問題は見つかりませんでした。保存できます。",
        "No problems found. This file is ready to save."
    ),
    Error => ("エラー", "Error"),
    Warning => ("警告", "Warning"),
    ConfigFolder => ("Espanso設定フォルダ", "Espanso configuration folder"),
    Change => ("変更", "Change"),
    OpenFolder => ("フォルダを開く", "Open folder"),
    EspansoService => ("Espansoサービス", "Espanso service"),
    Installed => ("インストール", "Installed"),
    Detected => ("検出済み", "Detected"),
    Undetected => ("未検出", "Not detected"),
    Version => ("バージョン", "Version"),
    Status => ("状態", "Status"),
    Start => ("開始", "Start"),
    Stop => ("停止", "Stop"),
    Restart => ("再起動", "Restart"),
    RefreshStatus => ("状態を更新", "Refresh status"),
    BackupsAndMigration => ("バックアップとデータ移行", "Backups & data migration"),
    BackupAll => ("設定全体をバックアップ", "Back up all settings"),
    BackupDestination => ("バックアップ先を選択", "Choose backup destination"),
    BackupSafety => (
        "通常の保存でも変更前ファイルを .espanso-gui/backups に自動保存します。ファイル削除は .espanso-gui/trash へ退避します。",
        "Normal saves automatically retain the previous file in .espanso-gui/backups. Deleted files are moved to .espanso-gui/trash."
    ),
    ExportCsv => ("選択ファイルをCSVへ書き出す", "Export selected file to CSV"),
    ImportCsv => ("CSVから読み込む", "Import from CSV"),
    HistoryTitle => ("選択ファイルの保存履歴", "Selected file history"),
    SelectHistoryFile => (
        "履歴を表示するスニペットファイルを選択してください。",
        "Select a snippet file to view its history."
    ),
    FileOperations => ("ファイル操作", "File operations"),
    DeleteSelectedFile => ("選択中のファイルを削除…", "Delete selected file…"),
    NoHistory => ("まだ履歴はありません", "No history yet"),
    RestoreVersion => ("この版を復元…", "Restore this version…"),
    HistoryDisabled => (
        "未保存の変更があるため、履歴復元は一時的に無効です。",
        "History restore is disabled while there are unsaved changes."
    ),
    AboutDescription => (
        "EspansoのYAML設定を、スニペット・変数・フォーム・リッチテキスト単位で安全に編集する独立アプリです。",
        "An independent app for safely editing Espanso YAML as snippets, variables, forms, and rich text."
    ),
    ProductTagline => (
        "Rustで作られた、洗練されたEspansoビジュアルエディタ。",
        "A polished visual editor for Espanso — written in Rust."
    ),
    UnofficialNotice => (
        "非公式プロジェクトです。EspansoおよびEspanso開発者による承認・提携・サポートはありません。本アプリのIssueは本アプリのリポジトリだけで扱います。",
        "This is an unofficial project and is not endorsed, affiliated with, or supported by Espanso or its developers. Report this app's issues only in this app's repository."
    ),
    License => ("ライセンス", "License"),
    Implementation => ("実装言語: Rust / GUI: eframe + egui", "Built with Rust / GUI: eframe + egui"),
    OpenEspansoDocs => ("Espanso公式ドキュメントを開く", "Open Espanso documentation"),
    ConnectTitle => ("Espanso設定を接続しましょう", "Connect your Espanso configuration"),
    ConnectDescription => (
        "設定フォルダを自動検出できない場合は手動で選択できます。",
        "If the configuration folder cannot be detected, select it manually."
    ),
    EspansoInstallRequired => (
        "Espanso本体が見つかりません。先にEspansoをインストールして起動してください。",
        "Espanso was not found. Install and start Espanso before connecting its configuration."
    ),
    OpenEspansoSetup => ("Espansoの導入ガイドを開く", "Open the Espanso setup guide"),
    ConfigLocation => ("接続する設定フォルダ", "Configuration folder to connect"),
    ChooseConfigFolder => ("設定フォルダを選択", "Choose configuration folder"),
    InitializeHere => ("この場所を初期化", "Initialize here"),
    InitializeHelp => (
        "不足しているmatchフォルダと空のbase.ymlだけを作成し、既存ファイルは上書きしません。",
        "Creates only a missing match folder and empty base.yml; existing files are not overwritten."
    ),
    NewMatchFileTitle => ("スニペットファイルを追加", "Add snippet file"),
    FileName => ("ファイル名", "File name"),
    NewMatchFileDescription => (
        "match/<名前>.yml として作成します",
        "Creates match/<name>.yml"
    ),
    NewProfileTitle => ("設定プロファイルを追加", "Add configuration profile"),
    NewProfileDescription => (
        "config/<名前>.yml として作成します。default は全体設定です。",
        "Creates config/<name>.yml. Use default for global settings."
    ),
    ConflictTitle => ("外部変更を3方向マージ", "Merge external changes"),
    ConflictIntroduction => (
        "読み込み時点、編集中、現在のディスク上の内容を比較しました。保存前にディスク上の最新版も自動バックアップします。",
        "The loaded base, your local edits, and the current file on disk were compared. The latest disk version will also be backed up before saving."
    ),
    NoOverlappingChanges => (
        "同じ項目を双方が変更した箇所はありません。独立した変更を自動マージできます。",
        "There are no overlapping field changes. The independent changes can be merged automatically."
    ),
    UseLocal => ("編集中の値を採用", "Use local"),
    UseDisk => ("ディスク上の値を採用", "Use disk"),
    BaseValue => ("基準値", "Base value"),
    DeletedValue => ("（削除）", "(deleted)"),
    UnavailableValue => ("（表示できません）", "(unavailable)"),
    MergeAndSave => ("マージして保存", "Merge and save"),
    RestoreHistoryTitle => ("保存履歴を復元", "Restore saved history"),
    RestoreWarning => (
        "現在のディスク版も先に新しい履歴としてバックアップするため、復元操作自体を取り消せます。",
        "The current disk version is saved as a new backup first, so this restore can itself be undone."
    ),
    BackupAndRestore => ("バックアップして復元", "Back up and restore"),
    RestoreComplete => ("保存履歴から復元しました", "Restored from saved history."),
    EditVariableTitle => ("変数を編集", "Edit variable"),
    AddVariableTitle => ("変数を追加", "Add variable"),
    VariableName => ("変数名", "Variable name"),
    Kind => ("種類", "Type"),
    Dependencies => ("依存変数", "Dependencies"),
    DependenciesDescription => (
        "評価順を固定する場合だけ指定",
        "Set only when evaluation order must be fixed"
    ),
    InsertVariableToken => (
        "保存時に本文へ {{変数名}} を挿入",
        "Insert {{variable_name}} into the content when saving"
    ),
    SaveVariable => ("変数を保存", "Save variable"),
    InvalidVariableName => (
        "変数名には英数字とアンダースコアだけを使用してください",
        "Use only letters, numbers, and underscores in the variable name."
    ),
    VariableSaved => (
        "変数を保存しました（ファイルは未保存）",
        "Variable saved; the file still has unsaved changes."
    ),
    FormFieldTitle => ("フォーム項目", "Form field"),
    FieldName => ("項目名（[[name]] のname部分）", "Field name (the name in [[name]])"),
    InputType => ("入力タイプ", "Input type"),
    InitialValue => ("初期値", "Initial value"),
    ChoicesPerLine => ("選択肢（1行に1つ）", "Choices (one per line)"),
    ChoicesHint => ("選択肢を1行に1つ", "One choice per line"),
    SaveField => ("項目を保存", "Save field"),
    InvalidFieldName => (
        "項目名には英数字とアンダースコアだけを使用してください",
        "Use only letters, numbers, and underscores in the field name."
    ),
    DeleteConfirmationTitle => ("削除の確認", "Confirm deletion"),
    DeleteSnippetQuestion => (
        "選択したスニペットを削除しますか？保存前なら再読み込みで戻せます。",
        "Delete the selected snippet? You can reload to recover it before saving."
    ),
    DeleteFileQuestion => (
        "選択したファイルを削除しますか？ファイルは復元用フォルダへ移動されます。",
        "Delete the selected file? It will be moved to the recovery folder."
    ),
    ConfirmDelete => ("削除する", "Delete"),
    UnsavedChangesTitle => ("未保存の変更があります", "Unsaved changes"),
    DiscardChangesQuestion => (
        "保存していない変更を破棄して終了しますか？",
        "Discard the unsaved changes and exit?"
    ),
    ReturnToEditor => ("編集に戻る", "Return to editor"),
    DiscardAndExit => ("破棄して終了", "Discard and exit"),
    DateFormat => ("表示形式", "Display format"),
    StrftimeFormat => ("strftime形式", "strftime format"),
    ChooseFormat => ("形式を選択", "Choose a format"),
    DateOffset => ("日時の移動", "Date and time offset"),
    DateOffsetDescription => ("秒単位。明日は86400", "In seconds; tomorrow is 86400"),
    Yesterday => ("昨日", "Yesterday"),
    Today => ("今日", "Today"),
    Tomorrow => ("明日", "Tomorrow"),
    NextWeek => ("1週間後", "Next week"),
    SecondsSuffix => (" 秒", " seconds"),
    Locale => ("ロケール", "Locale"),
    LocaleDescription => ("BCP 47。空欄ならOS設定", "BCP 47; leave blank for the OS setting"),
    LocaleHint => ("ja-JP", "en-US"),
    Timezone => ("タイムゾーン", "Time zone"),
    TimezoneDescription => ("IANA名。空欄ならローカル", "IANA name; leave blank for local time"),
    TimezoneHint => ("Asia/Tokyo", "America/New_York"),
    ClipboardDescription => (
        "展開時点のクリップボード内容を挿入します。追加設定はありません。",
        "Inserts the clipboard contents at expansion time. No additional settings are required."
    ),
    FixedValue => ("固定値", "Fixed value"),
    FixedValueDescription => ("複数のスニペットで再利用する値", "A value reused by multiple snippets"),
    Candidates => ("候補", "Candidates"),
    RandomDescription => ("1行に1つ。ランダムに1件を選択", "One per line; one is chosen at random"),
    Choices => ("選択肢", "Choices"),
    ChoiceDescription => (
        "1行に1つ。展開時に選択画面を表示",
        "One per line; show a chooser during expansion"
    ),
    ShellWarning => (
        "このコマンドはEspansoのトリガー実行時にローカル環境で実行されます。",
        "This command runs locally when Espanso expands the trigger."
    ),
    Command => ("コマンド", "Command"),
    CommandDescription => ("短時間で終了する処理を推奨", "Prefer a short-running command"),
    Shell => ("シェル", "Shell"),
    DefaultOs => ("OS既定", "OS default"),
    DefaultOsDescription => ("空欄ならOS既定", "Leave blank for the OS default"),
    TrimOutput => ("出力前後の空白と改行を除去", "Trim surrounding whitespace and newlines"),
    DebugOutput => ("Espansoログへデバッグ情報を出力", "Write debug information to the Espanso log"),
    ScriptWarning => (
        "1行目に実行コマンド、2行目以降に引数を入力します。変数はESPANSO_<名前>環境変数でも参照できます。",
        "Enter the executable on the first line and its arguments on subsequent lines. Variables are also available as ESPANSO_<name> environment variables."
    ),
    CommandAndArguments => ("コマンドと引数", "Command and arguments"),
    OnePerLine => ("1行に1要素", "One item per line"),
    FormLayout => ("フォーム配置", "Form layout"),
    FormLayoutDescription => ("[[field]] で入力欄を配置", "Place inputs with [[field]]"),
    FormFields => ("フォーム項目", "Form fields"),
    FormFieldNameShort => ("項目名", "Field name"),
    AddField => ("＋ 項目", "+ Add field"),
    GlobalVariableDescription => (
        "同名のグローバル変数をローカル評価順へ明示的に含めます。追加パラメータはありません。",
        "Explicitly includes the global variable with the same name in local evaluation order. No additional settings are required."
    ),
    UnknownVariableType => (
        "未知の変数タイプです。既存パラメータはRaw YAMLで保持されます。",
        "Unknown variable type. Existing parameters are preserved in Raw YAML."
    ),
    SaveBeforeReload => (
        "未保存の変更があります。保存してから再読み込みしてください",
        "Save the unsaved changes before reloading."
    ),
    WorkspaceReloaded => ("Espanso設定を再読み込みしました", "Reloaded the Espanso configuration."),
    EspansoCommandFailed => ("Espansoコマンドに失敗しました", "Espanso command failed"),
    PackageCannotSave => (
        "Hubパッケージは更新で上書きされるため直接保存できません。ユーザーファイルへコピーしてください",
        "Hub packages may be overwritten by updates and cannot be saved directly. Copy the snippet to a user file."
    ),
    ExternalChangeDetected => (
        "外部変更を検出しました。3方向マージの内容を確認してください",
        "External changes detected. Review the local three-way merge."
    ),
    PackageCannotAdd => ("パッケージには追加できません", "Snippets cannot be added to a package."),
    CopyPackageToUserFile => (
        "パッケージのコピーはユーザーファイルへ作成してください",
        "Create the package copy in a user file."
    ),
    CopySuffix => ("（コピー）", " (copy)"),
    MissingCopyTarget => (
        "コピー先のユーザーファイルがありません。先にファイルを追加してください",
        "No user file is available as the copy destination. Add a file first."
    ),
    CopiedToUserFile => (
        "ユーザーファイルへコピーしました（トリガーの重複を確認してください）",
        "Copied to a user file. Check for duplicate triggers."
    ),
    SnippetDeleted => (
        "スニペットを削除しました（まだ未保存です）",
        "Snippet deleted; the file still has unsaved changes."
    ),
    MatchFileCreated => ("スニペットファイルを作成しました", "Snippet file created."),
    ProfileCreated => ("設定プロファイルを作成しました", "Configuration profile created."),
    PackageCannotDelete => (
        "パッケージファイルはこの画面から削除できません",
        "Package files cannot be deleted from this screen."
    ),
    ConfigInitialized => ("Espanso設定フォルダを初期化しました", "Initialized the Espanso configuration folder."),
    SaveBeforeChangingFolder => (
        "設定フォルダを変える前に未保存の変更を保存してください",
        "Save the unsaved changes before changing the configuration folder."
    ),
    ChooseEspansoFolder => ("Espanso設定フォルダを選択", "Choose the Espanso configuration folder"),
    PackageEditorWarning => (
        "Espanso Hubのパッケージは更新時に上書きされます。内容は確認できますが、直接編集は無効です。",
        "Espanso Hub packages may be overwritten by updates. You can inspect their contents, but direct editing is disabled."
    ),
    CopyThisSnippet => (
        "このスニペットをユーザーファイルへコピー",
        "Copy this snippet to a user file"
    ),
    ValidateAndApplyYaml => ("YAMLを検証して適用", "Validate and apply YAML"),
    YamlValid => ("YAMLは有効です", "YAML is valid."),
    ValidateAgainOnSave => ("保存時にも必ず検証します", "YAML is always validated again when saving."),
    CsvExported => ("CSVを書き出しました", "CSV exported."),
    DisplayName => ("表示名", "Display name"),
    DisplayNameDescription => (
        "Espanso検索バーに表示するラベル",
        "Label in the Espanso search bar"
    ),
    DisplayNameHint => ("例: 署名（日本語）", "Example: Email signature"),
    Trigger => ("トリガー", "Trigger"),
    TriggerDescription => ("カンマ区切りで複数指定できます", "Separate multiple triggers with commas"),
    RegexTriggerDescription => (
        "切り替えると現在のトリガーをエスケープし、同じ文字列に一致する正規表現として引き継ぎます",
        "Switching escapes the current trigger so the regular expression keeps matching the same literal text"
    ),
    NormalTrigger => ("通常", "Normal"),
    RegularExpression => ("正規表現", "Regular expression"),
    RegexTriggerHint => (
        "例: :hello\\((?P<name>.*)\\)",
        "Example: :hello\\((?P<name>.*)\\)"
    ),
    ExpansionType => ("展開タイプ", "Expansion type"),
    HtmlEditing => ("HTML編集", "HTML editing"),
    Composer => ("編集", "Composer"),
    Source => ("ソース", "Source"),
    SafeHtmlNotice => (
        "プレビューは動的コンテンツを実行・取得しません",
        "The preview does not run or fetch active content"
    ),
    HtmlContentHint => ("<strong>HTML</strong> を入力", "Enter <strong>HTML</strong>"),
    MarkdownContentHint => ("**Markdown** を入力", "Enter **Markdown**"),
    PlainContentHint => ("展開するテキストを入力", "Enter replacement text"),
    ImagePath => ("画像パス", "Image path"),
    ImagePathHint => (
        "$CONFIG/assets/image.png または絶対パス",
        "$CONFIG/assets/image.png or an absolute path"
    ),
    ChooseImage => ("画像を選択", "Choose image"),
    ImagePreview => ("画像プレビュー", "Image preview"),
    ImageCompatibility => (
        "$CONFIGはEspanso設定フォルダへ展開されます。LinuxではPNGが最も互換性の高い形式です。",
        "$CONFIG expands to the Espanso configuration folder. PNG has the broadest compatibility on Linux."
    ),
    FormEditorDescription => (
        "本文中に [[name]] のように書くと、Espanso展開時に入力欄が表示されます。",
        "Place [[name]] in the content to show an input when Espanso expands the match."
    ),
    FormContent => ("フォーム本文", "Form content"),
    FormContentHint => ("お名前: [[name]]\n種類: [[plan]]", "Name: [[name]]\nPlan: [[plan]]"),
    AddFormField => ("＋ 項目を追加", "+ Add field"),
    DefaultValue => ("既定", "Default"),
    LivePreview => ("ライブプレビュー", "Live preview"),
    SafeTextPreview => (
        "安全なテキストプレビュー（スクリプト、スタイル、外部コンテンツは実行・取得しません）",
        "Safe text preview; scripts, styles, and remote content are not run or fetched"
    ),
    NoPreviewContent => ("プレビューできる本文がありません", "No previewable content"),
    GeneratedSource => ("生成されたソース", "Generated source"),
    QuickAdd => ("すぐ追加", "Quick add"),
    NoVariablesTitle => ("このスニペットには変数がありません", "This snippet has no variables"),
    NoVariablesDescription => (
        "上のボタンから種類を選ぶだけで追加できます。",
        "Choose a type above to add one."
    ),
    InsertIntoContent => ("本文に挿入", "Insert into content"),
    ScriptVariableWarning => (
        "シェル／スクリプト変数はトリガー実行時にローカルコマンドを実行します。自分で内容を確認したコードだけを使用してください。",
        "Shell and script variables run local commands when the trigger expands. Use only code you have reviewed."
    ),
    TriggerConditions => ("トリガー条件", "Trigger conditions"),
    WholeWord => ("単語単位（word）", "Whole word (word)"),
    WholeWordDescription => (
        "単語の区切りに囲まれた場合だけ展開",
        "Expand only when surrounded by word boundaries"
    ),
    LeftWord => ("単語の左端（left_word）", "Left word boundary (left_word)"),
    LeftWordDescription => ("単語の先頭でだけ展開", "Expand only at the start of a word"),
    RightWord => ("単語の右端（right_word）", "Right word boundary (right_word)"),
    RightWordDescription => ("単語の末尾でだけ展開", "Expand only at the end of a word"),
    LetterCase => ("大文字・小文字", "Letter case"),
    PropagateCase => ("入力のケースを引き継ぐ", "Propagate input case"),
    PropagateCaseDescription => (
        "hello / Hello / HELLO に合わせて展開結果を変換",
        "Transform the replacement to match hello / Hello / HELLO"
    ),
    UppercaseStyle => ("大文字スタイル", "Uppercase style"),
    UppercaseStyleDescription => ("複数単語の変換方法", "How to transform multiple words"),
    Standard => ("標準", "Standard"),
    CapitalizeWords => ("各単語を大文字", "Capitalize each word"),
    CapitalizeFirst => ("先頭のみ大文字", "Capitalize first word"),
    ExpansionMethod => ("展開方法", "Expansion method"),
    ForceMode => ("強制モード", "Force mode"),
    ForceModeDescription => ("問題があるアプリ向けの上書き", "Override for problematic applications"),
    Automatic => ("自動", "Automatic"),
    Enabled => ("有効", "Enabled"),
    NoMarkdownParagraph => ("段落として貼り付けない", "Do not paste as a paragraph"),
    NoMarkdownParagraphDescription => (
        "EspansoのMarkdown段落オプション",
        "Espanso's Markdown paragraph option"
    ),
    SearchKeywords => ("検索キーワード／タグ", "Search keywords / tags"),
    CommaSeparated => (
        "カンマ区切り。検索とタグ絞り込みに使用",
        "Comma-separated; used for search and tag filtering"
    ),
    SearchKeywordsHint => ("署名, email, work", "signature, email, work"),
    RawYamlFormattingWarning => (
        "構造化エディタで変更するとコメント位置は再整形されます。元ファイルは保存時に自動バックアップされます。Raw YAMLだけの編集ならコメントを維持できます。",
        "Structured edits may reformat comment positions. The original file is backed up automatically when saving; Raw YAML-only edits preserve comments."
    ),
    Bold => ("太字", "Bold"),
    Italic => ("斜体", "Italic"),
    Heading => ("見出し", "Heading"),
    Link => ("リンク", "Link"),
    Code => ("コード", "Code"),
    BulletedList => ("箇条書き", "Bulleted list"),
    NumberedList => ("番号リスト", "Numbered list"),
    Color => ("色", "Color"),
    Image => ("画像", "Image"),
    CursorPosition => ("カーソル位置", "Cursor position"),
    ListItemOne => ("項目1", "Item 1"),
    ListItemTwo => ("項目2", "Item 2"),
    ColoredText => ("色付きテキスト", "Colored text"),
    CurrentClipboard => ("現在のクリップボード", "Current clipboard"),
    AdvancedVariable => ("高度な変数", "Advanced variable"),
    ExampleValue => ("値", "Value"),
    ExampleCandidateOne => ("候補1", "Candidate 1"),
    ExampleCandidateTwo => ("候補2", "Candidate 2"),
    ExampleFormLayout => ("名前: [[name]]", "Name: [[name]]"),
    DefaultProfileNotice => (
        "default.yml はすべてのアプリの基準です。アプリ別ファイルはここで設定した値を継承します。",
        "default.yml is the baseline for every application. App-specific files inherit values configured here."
    ),
    ProfileFilterNotice => (
        "フィルターは正規表現です。WaylandではEspansoのアプリ別設定自体が未対応です。",
        "Filters are regular expressions. Espanso app-specific configuration is not supported on Wayland."
    ),
    TargetApplications => ("適用するアプリ", "Target applications"),
    ExecutableFilter => ("実行ファイル（filter_exec）", "Executable (filter_exec)"),
    ExecutableFilterHint => ("例: Code|VSCodium", "Example: Code|VSCodium"),
    WindowClassFilter => ("ウィンドウクラス（filter_class）", "Window class (filter_class)"),
    WindowClassFilterDescription => (
        "Linuxでは最も安定した指定",
        "Usually the most reliable filter on Linux"
    ),
    WindowTitleFilter => ("ウィンドウタイトル（filter_title）", "Window title (filter_title)"),
    WindowTitleFilterHint => ("例: YouTube", "Example: YouTube"),
    OperatingSystemFilter => ("OS（filter_os）", "Operating system (filter_os)"),
    OperatingSystemFilterDescription => ("共有設定のOS限定", "Limit a shared profile by OS"),
    Inherit => ("継承", "Inherit"),
    Override => ("上書き", "Override"),
    Disabled => ("無効", "Disabled"),
    ProfileFilterRequired => (
        "アプリ別ファイルには filter_exec、filter_class、filter_title、filter_os のいずれかが必要です。",
        "An app-specific file requires at least one of filter_exec, filter_class, filter_title, or filter_os."
    ),
    BehaviorAndInjection => ("動作と注入", "Behavior & injection"),
    EnableEspanso => ("Espansoを有効化", "Enable Espanso"),
    InheritDefaultsDescription => ("未指定なら既定設定を継承", "Inherit the default when unspecified"),
    InjectionBackend => ("注入方式（backend）", "Injection backend (backend)"),
    InjectionBackendDescription => ("auto / inject / clipboard", "auto / inject / clipboard"),
    KeyInjection => ("キー注入", "Key injection"),
    Clipboard => ("クリップボード", "Clipboard"),
    ApplyBuiltInPatch => ("組み込み補正を適用", "Apply built-in patches"),
    ApplyBuiltInPatchDescription => (
        "ターミナルなどに対するEspanso既定の互換性補正",
        "Espanso's default compatibility fixes for terminals and similar apps"
    ),
    PasteShortcut => ("貼り付けショートカット", "Paste shortcut"),
    PasteShortcutHint => ("例: CTRL+SHIFT+V", "Example: CTRL+SHIFT+V"),
    DelaysMilliseconds => ("遅延（ミリ秒）", "Delays (milliseconds)"),
    CharacterInjectionDelay => ("文字注入間隔", "Character injection delay"),
    KeyInjectionDelay => ("キー注入間隔", "Key injection delay"),
    BeforePaste => ("貼り付け前", "Before paste"),
    PasteKeyInterval => ("貼り付けキー間隔", "Paste-key interval"),
    AfterForm => ("フォーム後", "After form"),
    AfterSearch => ("検索後", "After search"),
    FormLimits => ("フォーム上限", "Form limits"),
    MaximumWidthPx => ("最大幅（px）", "Maximum width (px)"),
    MaximumHeightPx => ("最大高（px）", "Maximum height (px)"),
    SearchAndGlobalSettings => ("検索と全体設定", "Search & global settings"),
    SearchShortcut => ("検索ショートカット", "Search shortcut"),
    SearchShortcutHint => ("例: ALT+SPACE / off", "Example: ALT+SPACE / off"),
    SearchTrigger => ("検索トリガー", "Search trigger"),
    SearchTriggerHint => ("例: .search / off", "Example: .search / off"),
    ToggleKey => ("有効／無効の切り替えキー", "Enable/disable toggle key"),
    ToggleKeyHint => ("例: RIGHT_CTRL / OFF", "Example: RIGHT_CTRL / OFF"),
    RestoreClipboard => ("クリップボードを復元", "Restore clipboard"),
    RestoreClipboardDescription => (
        "展開前のクリップボード内容を保持",
        "Keep the clipboard contents from before expansion"
    ),
    ShowStatusIcon => ("ステータスアイコンを表示", "Show status icon"),
    ShowStatusIconDescription => (
        "macOSのメニューバーまたはWindowsの通知領域",
        "macOS menu bar or Windows notification area"
    ),
    ShowNotifications => ("通知を表示", "Show notifications"),
    ShowNotificationsDescription => ("Espansoの通知全体", "All Espanso notifications"),
    ProfileRawYamlNotice => (
        "Raw YAML編集ではコメントを保持できます。ビジュアル編集の前にも元ファイルを自動バックアップします。",
        "Raw YAML editing preserves comments. The original file is also backed up before visual edits."
    ),
    LanguageDescription => ("日本語 / English", "Japanese / English"),
}

pub fn text(language: Language, key: TextKey) -> &'static str {
    let translation = key.translation();
    match language {
        Language::Japanese => translation.japanese,
        Language::English => translation.english,
    }
}

pub fn snippet_count(language: Language, count: usize) -> String {
    match language {
        Language::Japanese => format!("{count}件"),
        Language::English if count == 1 => "1 snippet".to_owned(),
        Language::English => format!("{count} snippets"),
    }
}

pub fn appearance_label(language: Language, appearance: Appearance) -> &'static str {
    text(
        language,
        match appearance {
            Appearance::System => TextKey::SystemAppearance,
            Appearance::Light => TextKey::LightAppearance,
            Appearance::Dark => TextKey::DarkAppearance,
        },
    )
}

pub fn search_result_count(language: Language, count: usize) -> String {
    match language {
        Language::Japanese => format!("全ファイルから{count}件"),
        Language::English if count == 1 => "1 result across all files".to_owned(),
        Language::English => format!("{count} results across all files"),
    }
}

pub fn espanso_action_completed(language: Language, action: EspansoAction) -> &'static str {
    match (language, action) {
        (Language::Japanese, EspansoAction::Start) => "Espansoを開始しました。",
        (Language::Japanese, EspansoAction::Stop) => "Espansoを停止しました。",
        (Language::Japanese, EspansoAction::Restart) => "Espansoを再起動しました。",
        (Language::English, EspansoAction::Start) => "Started Espanso.",
        (Language::English, EspansoAction::Stop) => "Stopped Espanso.",
        (Language::English, EspansoAction::Restart) => "Restarted Espanso.",
    }
}

pub fn espanso_command_error(language: Language, detail: &str) -> String {
    match language {
        Language::Japanese => format!("Espansoコマンドを実行できませんでした: {detail}"),
        Language::English => format!("Could not run the Espanso command: {detail}"),
    }
}

pub fn open_failed_text(language: Language, source: &str) -> String {
    format!("{}: {source}", text(language, TextKey::OpenFailed))
}

pub fn conflict_count_text(language: Language, count: usize) -> String {
    match language {
        Language::Japanese => {
            format!("{count}項目で双方の変更が重なっています。各項目の採用値を選択してください。")
        }
        Language::English if count == 1 => {
            "Both sides changed 1 field. Choose which value to keep.".to_owned()
        }
        Language::English => {
            format!("Both sides changed {count} fields. Choose which values to keep.")
        }
    }
}

pub fn restore_target_text(language: Language, path: &str, timestamp: &str) -> String {
    match language {
        Language::Japanese => format!("{path} を {timestamp} の内容へ戻します。"),
        Language::English => format!("Restore {path} to the version from {timestamp}."),
    }
}

pub fn merge_saved_text(language: Language, hash: &str) -> String {
    match language {
        Language::Japanese => format!("3方向マージを保存しました / {hash}"),
        Language::English => format!("Three-way merge saved / {hash}"),
    }
}

pub fn workspace_saved_text(language: Language, backup_path: Option<&str>, hash: &str) -> String {
    match (language, backup_path) {
        (Language::Japanese, Some(path)) => {
            format!("保存しました / バックアップ: {path} / {hash}")
        }
        (Language::Japanese, None) => format!("保存しました / {hash}"),
        (Language::English, Some(path)) => format!("Saved / backup: {path} / {hash}"),
        (Language::English, None) => format!("Saved / {hash}"),
    }
}

pub fn profile_saved_text(language: Language, backup_path: Option<&str>, hash: &str) -> String {
    match (language, backup_path) {
        (Language::Japanese, Some(path)) => {
            format!("設定プロファイルを保存しました / バックアップ: {path} / {hash}")
        }
        (Language::Japanese, None) => format!("設定プロファイルを保存しました / {hash}"),
        (Language::English, Some(path)) => {
            format!("Configuration profile saved / backup: {path} / {hash}")
        }
        (Language::English, None) => format!("Configuration profile saved / {hash}"),
    }
}

pub fn file_moved_text(language: Language, path: &str) -> String {
    match language {
        Language::Japanese => format!("ファイルを退避しました: {path}"),
        Language::English => format!("File moved to recovery: {path}"),
    }
}

pub fn backup_created_text(language: Language, path: &str) -> String {
    match language {
        Language::Japanese => format!("バックアップを作成しました: {path}"),
        Language::English => format!("Backup created: {path}"),
    }
}

pub fn csv_imported_text(language: Language, count: usize) -> String {
    match language {
        Language::Japanese => format!("{count}件のスニペットを読み込みました（未保存）"),
        Language::English if count == 1 => {
            "Imported 1 snippet; the file still has unsaved changes.".to_owned()
        }
        Language::English => {
            format!("Imported {count} snippets; the file still has unsaved changes.")
        }
    }
}

pub fn variable_kind_label(language: Language, kind: &str) -> &'static str {
    match (language, kind) {
        (Language::Japanese, "date") => "日付・時刻",
        (Language::English, "date") => "Date & time",
        (Language::Japanese, "clipboard") => "クリップボード",
        (Language::English, "clipboard") => "Clipboard",
        (Language::Japanese, "choice") => "候補選択",
        (Language::English, "choice") => "Choice",
        (Language::Japanese, "random") => "ランダム",
        (Language::English, "random") => "Random",
        (Language::Japanese, "echo") => "固定値",
        (Language::English, "echo") => "Fixed value",
        (Language::Japanese, "shell") => "シェルコマンド",
        (Language::English, "shell") => "Shell command",
        (Language::Japanese, "script") => "スクリプト",
        (Language::English, "script") => "Script",
        (Language::Japanese, "form") => "フォーム",
        (Language::English, "form") => "Form",
        (Language::Japanese, "global") => "グローバル参照",
        (Language::English, "global") => "Global reference",
        (Language::Japanese, _) => "カスタム",
        (Language::English, _) => "Custom",
    }
}

pub fn form_field_kind_label(language: Language, kind: &FormFieldKind) -> String {
    match (language, kind) {
        (Language::Japanese, FormFieldKind::Choice) => "選択ボタン".into(),
        (Language::English, FormFieldKind::Choice) => "Choice buttons".into(),
        (Language::Japanese, FormFieldKind::List) => "リスト".into(),
        (Language::English, FormFieldKind::List) => "List".into(),
        (Language::Japanese, FormFieldKind::Multiline) => "複数行テキスト".into(),
        (Language::English, FormFieldKind::Multiline) => "Multiline text".into(),
        (Language::Japanese, FormFieldKind::Text) => "テキスト".into(),
        (Language::English, FormFieldKind::Text) => "Text".into(),
        (Language::Japanese, FormFieldKind::Unknown(kind)) => {
            format!("未対応の種類: {kind}")
        }
        (Language::English, FormFieldKind::Unknown(kind)) => {
            format!("Unsupported type: {kind}")
        }
    }
}

pub fn content_kind_label(language: Language, kind: ContentKind) -> &'static str {
    match (language, kind) {
        (Language::Japanese, ContentKind::Plain) => "テキスト",
        (Language::English, ContentKind::Plain) => "Text",
        (_, ContentKind::Markdown) => "Markdown",
        (_, ContentKind::Html) => "HTML",
        (Language::Japanese, ContentKind::Image) => "画像",
        (Language::English, ContentKind::Image) => "Image",
        (Language::Japanese, ContentKind::Form) => "フォーム",
        (Language::English, ContentKind::Form) => "Form",
    }
}

pub fn default_value_text(language: Language, value: &str) -> String {
    format!("{}: {value}", text(language, TextKey::DefaultValue))
}

pub fn date_summary_text(language: Language, format: &str, offset: i64) -> String {
    match language {
        Language::Japanese => format!("形式 {format} / オフセット {offset}秒"),
        Language::English => format!("Format {format} / offset {offset} seconds"),
    }
}

pub fn random_summary_text(language: Language, count: usize) -> String {
    match language {
        Language::Japanese => format!("{count}件からランダム選択"),
        Language::English if count == 1 => "Randomly choose from 1 candidate".to_owned(),
        Language::English => format!("Randomly choose from {count} candidates"),
    }
}

pub fn choice_summary_text(language: Language, count: usize) -> String {
    match language {
        Language::Japanese => format!("{count}件の選択肢"),
        Language::English if count == 1 => "1 choice".to_owned(),
        Language::English => format!("{count} choices"),
    }
}

pub fn date_format_presets(language: Language) -> [(&'static str, &'static str); 5] {
    match language {
        Language::Japanese => [
            ("%Y-%m-%d", "2026-08-15"),
            ("%Y年%m月%d日", "2026年08月15日"),
            ("%Y/%m/%d", "2026/08/15"),
            ("%H:%M", "14:30"),
            ("%Y-%m-%d %H:%M", "2026-08-15 14:30"),
        ],
        Language::English => [
            ("%Y-%m-%d", "2026-08-15"),
            ("%B %d, %Y", "August 15, 2026"),
            ("%Y/%m/%d", "2026/08/15"),
            ("%H:%M", "14:30"),
            ("%Y-%m-%d %H:%M", "2026-08-15 14:30"),
        ],
    }
}

pub fn storage_error_text(language: Language, error: &StorageError) -> String {
    match error {
        StorageError::Io(source) => match language {
            Language::Japanese => format!("ファイル操作に失敗しました: {source}"),
            Language::English => format!("File operation failed: {source}"),
        },
        StorageError::Yaml(source) => match language {
            Language::Japanese => format!("YAMLが正しくありません: {source}"),
            Language::English => format!("Invalid YAML: {source}"),
        },
        StorageError::Csv(source) => match language {
            Language::Japanese => format!("CSVを処理できません: {source}"),
            Language::English => format!("CSV processing failed: {source}"),
        },
        StorageError::InvalidYamlFile { path, source } => match language {
            Language::Japanese => {
                format!("{} のYAMLが正しくありません: {source}", path.display())
            }
            Language::English => format!("Invalid YAML in {}: {source}", path.display()),
        },
        StorageError::Issue(issue) => storage_issue_text(language, issue),
    }
}

fn storage_issue_text(language: Language, issue: &StorageIssue) -> String {
    match (language, issue) {
        (Language::Japanese, StorageIssue::ConfigPathResolution) => {
            "設定ファイルのパスを解決できません".into()
        }
        (Language::English, StorageIssue::ConfigPathResolution) => {
            "Could not resolve the configuration file path.".into()
        }
        (Language::Japanese, StorageIssue::AlreadyExists(path)) => {
            format!("{} はすでに存在します", path.display())
        }
        (Language::English, StorageIssue::AlreadyExists(path)) => {
            format!("{} already exists.", path.display())
        }
        (Language::Japanese, StorageIssue::ExternalChange) => {
            "ファイルが他のアプリで変更されました。再読み込みしてから保存してください".into()
        }
        (Language::English, StorageIssue::ExternalChange) => {
            "The file was changed by another application. Reload before saving.".into()
        }
        (Language::Japanese, StorageIssue::DestinationDeleted) => {
            "保存先が削除されています。再読み込みしてください".into()
        }
        (Language::English, StorageIssue::DestinationDeleted) => {
            "The save destination was deleted. Reload the workspace.".into()
        }
        (Language::Japanese, StorageIssue::DeleteTargetMissing) => {
            "削除するファイルが見つかりません".into()
        }
        (Language::English, StorageIssue::DeleteTargetMissing) => {
            "The file to delete was not found.".into()
        }
        (Language::Japanese, StorageIssue::MissingTargetFile) => "対象ファイルがありません".into(),
        (Language::English, StorageIssue::MissingTargetFile) => {
            "The target file is unavailable.".into()
        }
        (Language::Japanese, StorageIssue::RestoreOutsideBackup) => {
            "アプリのバックアップ以外からは復元できません".into()
        }
        (Language::English, StorageIssue::RestoreOutsideBackup) => {
            "Only backups created by this application can be restored.".into()
        }
        (Language::Japanese, StorageIssue::RestoreTooLarge) => {
            "復元する設定ファイルが大きすぎます".into()
        }
        (Language::English, StorageIssue::RestoreTooLarge) => {
            "The configuration file to restore is too large.".into()
        }
        (Language::Japanese, StorageIssue::BackupNotUtf8) => {
            "バックアップがUTF-8ではありません".into()
        }
        (Language::English, StorageIssue::BackupNotUtf8) => "The backup is not valid UTF-8.".into(),
        (Language::Japanese, StorageIssue::ConflictChangedAgain) => {
            "競合確認後にファイルが再度変更されました。もう一度比較してください".into()
        }
        (Language::English, StorageIssue::ConflictChangedAgain) => {
            "The file changed again after conflict review. Compare it again.".into()
        }
        (Language::Japanese, StorageIssue::BackupPathResolution) => {
            "バックアップパスを解決できません".into()
        }
        (Language::English, StorageIssue::BackupPathResolution) => {
            "Could not resolve the backup path.".into()
        }
        (Language::Japanese, StorageIssue::BackupInsideConfig) => {
            "バックアップ先はEspanso設定フォルダの外を選択してください".into()
        }
        (Language::English, StorageIssue::BackupInsideConfig) => {
            "Choose a backup destination outside the Espanso configuration folder.".into()
        }
        (Language::Japanese, StorageIssue::InvalidFileName) => {
            "ファイル名には英数字、ハイフン、アンダースコアを使用してください".into()
        }
        (Language::English, StorageIssue::InvalidFileName) => {
            "Use only letters, numbers, hyphens, and underscores in file names.".into()
        }
        (Language::Japanese, StorageIssue::RelativePathRequired) => {
            "相対パスを指定してください".into()
        }
        (Language::English, StorageIssue::RelativePathRequired) => {
            "Specify a relative path.".into()
        }
        (Language::Japanese, StorageIssue::OutsideConfigPath) => {
            "設定フォルダ外のパスは使用できません".into()
        }
        (Language::English, StorageIssue::OutsideConfigPath) => {
            "Paths outside the configuration folder are not allowed.".into()
        }
        (Language::Japanese, StorageIssue::MatchRootRequired) => {
            "スニペットはmatchフォルダ内に保存してください".into()
        }
        (Language::English, StorageIssue::MatchRootRequired) => {
            "Store snippets in the match folder.".into()
        }
        (Language::Japanese, StorageIssue::ConfigRootRequired) => {
            "設定プロファイルはconfigフォルダ内に保存してください".into()
        }
        (Language::English, StorageIssue::ConfigRootRequired) => {
            "Store configuration profiles in the config folder.".into()
        }
        (Language::Japanese, StorageIssue::ManagedRootRequired) => {
            "matchまたはconfigフォルダ内の設定だけを操作できます".into()
        }
        (Language::English, StorageIssue::ManagedRootRequired) => {
            "Only files under the match or config folders can be managed.".into()
        }
        (Language::Japanese, StorageIssue::YamlExtensionRequired) => {
            "拡張子は.ymlまたは.yamlにしてください".into()
        }
        (Language::English, StorageIssue::YamlExtensionRequired) => {
            "Use a .yml or .yaml extension.".into()
        }
        (Language::Japanese, StorageIssue::OutsideSaveRoot) => {
            "設定フォルダ外には保存できません".into()
        }
        (Language::English, StorageIssue::OutsideSaveRoot) => {
            "Files cannot be saved outside the configuration folder.".into()
        }
        (Language::Japanese, StorageIssue::ConfigRootMissing(path)) => {
            format!("Espanso設定フォルダが見つかりません: {}", path.display())
        }
        (Language::English, StorageIssue::ConfigRootMissing(path)) => {
            format!("Espanso configuration folder not found: {}", path.display())
        }
        (Language::Japanese, StorageIssue::UniqueBackupDestination) => {
            "バックアップ履歴の一意な保存先を作成できません".into()
        }
        (Language::English, StorageIssue::UniqueBackupDestination) => {
            "Could not create a unique backup-history destination.".into()
        }
    }
}

pub fn diagnostic_text(language: Language, kind: &DiagnosticKind) -> String {
    match (language, kind) {
        (Language::Japanese, DiagnosticKind::MissingTrigger) => {
            "トリガーまたは正規表現が必要です".to_owned()
        }
        (Language::English, DiagnosticKind::MissingTrigger) => {
            "A trigger or regular expression is required.".to_owned()
        }
        (
            Language::Japanese,
            DiagnosticKind::DuplicateTrigger {
                trigger,
                previous_snippet,
            },
        ) => format!("トリガー「{trigger}」は{previous_snippet}番目のスニペットでも使われています"),
        (
            Language::English,
            DiagnosticKind::DuplicateTrigger {
                trigger,
                previous_snippet,
            },
        ) => format!("Trigger “{trigger}” is also used by snippet {previous_snippet}."),
        (Language::Japanese, DiagnosticKind::EmptyContent) => "展開内容が空です".to_owned(),
        (Language::English, DiagnosticKind::EmptyContent) => {
            "The replacement content is empty.".to_owned()
        }
        (Language::Japanese, DiagnosticKind::UndefinedVariable { reference }) => {
            format!("変数「{reference}」が定義されていません")
        }
        (Language::English, DiagnosticKind::UndefinedVariable { reference }) => {
            format!("Variable “{reference}” is not defined.")
        }
        (Language::Japanese, DiagnosticKind::InvalidVariableName { name }) => {
            format!("変数名「{name}」には英数字とアンダースコアだけを使用できます")
        }
        (Language::English, DiagnosticKind::InvalidVariableName { name }) => {
            format!("Variable name “{name}” may contain only letters, numbers, and underscores.")
        }
        (Language::Japanese, DiagnosticKind::MissingVariableKind { name }) => {
            format!("変数「{name}」の種類が未設定です")
        }
        (Language::English, DiagnosticKind::MissingVariableKind { name }) => {
            format!("Variable “{name}” has no type.")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_catalogs_cover_every_key() {
        for language in Language::ALL {
            for &key in TextKey::ALL {
                assert!(!text(language, key).trim().is_empty());
            }
        }
    }

    #[test]
    fn japanese_copy_does_not_regress_to_untranslated_general_ui_terms() {
        for (key, forbidden) in [
            (TextKey::ConflictTitle, "three-way"),
            (TextKey::ConflictIntroduction, "disk"),
            (TextKey::NoOverlappingChanges, "field"),
            (TextKey::SafeHtmlNotice, "preview"),
            (TextKey::SafeTextPreview, "remote content"),
            (TextKey::GeneratedSource, "source"),
            (TextKey::PasteShortcut, "shortcut"),
            (TextKey::RestoreClipboard, "clipboard"),
            (TextKey::ShowStatusIcon, "status icon"),
        ] {
            assert!(
                !text(Language::Japanese, key).contains(forbidden),
                "{key:?} contains untranslated UI term {forbidden:?}"
            );
        }
    }

    #[test]
    fn snippet_counts_follow_the_selected_language() {
        assert_eq!(snippet_count(Language::Japanese, 2), "2件");
        assert_eq!(snippet_count(Language::English, 1), "1 snippet");
        assert_eq!(snippet_count(Language::English, 2), "2 snippets");
        assert_eq!(
            search_result_count(Language::Japanese, 2),
            "全ファイルから2件"
        );
        assert_eq!(
            search_result_count(Language::English, 1),
            "1 result across all files"
        );
        assert_eq!(
            appearance_label(Language::Japanese, Appearance::Dark),
            "ダーク"
        );
        assert_eq!(
            appearance_label(Language::English, Appearance::System),
            "System"
        );
    }

    #[test]
    fn diagnostics_are_rendered_in_the_selected_language() {
        let diagnostic = DiagnosticKind::DuplicateTrigger {
            trigger: ":hello".to_owned(),
            previous_snippet: 2,
        };
        assert!(diagnostic_text(Language::Japanese, &diagnostic).contains("2番目"));
        assert!(diagnostic_text(Language::English, &diagnostic).contains("snippet 2"));
    }

    #[test]
    fn dynamic_dialog_text_and_kind_labels_follow_the_selected_language() {
        assert!(conflict_count_text(Language::Japanese, 2).contains("2項目"));
        assert!(conflict_count_text(Language::English, 2).contains("2 fields"));
        assert_eq!(
            restore_target_text(Language::English, "match/base.yml", "2026-08-16"),
            "Restore match/base.yml to the version from 2026-08-16."
        );
        assert_eq!(
            variable_kind_label(Language::English, "date"),
            "Date & time"
        );
        assert_eq!(
            form_field_kind_label(Language::Japanese, &FormFieldKind::List),
            "リスト"
        );
        assert_eq!(
            form_field_kind_label(
                Language::English,
                &FormFieldKind::Unknown("future_widget".into())
            ),
            "Unsupported type: future_widget"
        );
        assert_eq!(
            content_kind_label(Language::English, ContentKind::Image),
            "Image"
        );
        assert_eq!(
            default_value_text(Language::English, "Acme"),
            "Default: Acme"
        );
        assert_eq!(
            workspace_saved_text(Language::English, Some("backup.yml"), "0123abcd"),
            "Saved / backup: backup.yml / 0123abcd"
        );
        assert!(csv_imported_text(Language::English, 2).contains("2 snippets"));
        assert_eq!(choice_summary_text(Language::English, 1), "1 choice");
        assert_eq!(
            espanso_action_completed(Language::Japanese, EspansoAction::Restart),
            "Espansoを再起動しました。"
        );
        assert_eq!(
            espanso_command_error(Language::English, "not found"),
            "Could not run the Espanso command: not found"
        );
        assert_eq!(
            espanso_command_error(Language::Japanese, "not found"),
            "Espansoコマンドを実行できませんでした: not found"
        );
        assert_eq!(
            open_failed_text(Language::English, "permission denied"),
            "Could not open: permission denied"
        );
        assert_eq!(
            date_format_presets(Language::English)[1],
            ("%B %d, %Y", "August 15, 2026")
        );
        assert_eq!(
            storage_error_text(
                Language::English,
                &StorageError::Issue(StorageIssue::InvalidFileName)
            ),
            "Use only letters, numbers, hyphens, and underscores in file names."
        );
        assert_eq!(
            storage_error_text(
                Language::Japanese,
                &StorageError::Issue(StorageIssue::DestinationDeleted)
            ),
            "保存先が削除されています。再読み込みしてください"
        );
        assert_eq!(
            storage_error_text(
                Language::Japanese,
                &StorageError::Issue(StorageIssue::BackupInsideConfig)
            ),
            "バックアップ先はEspanso設定フォルダの外を選択してください"
        );
    }
}
