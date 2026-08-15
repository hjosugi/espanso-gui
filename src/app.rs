use crate::conflict::ResolutionChoice;
use crate::espanso::{self, EspansoAction, EspansoStatus};
use crate::model::{ContentKind, DiagnosticLevel, FormField, Snippet, Variable};
use crate::storage::{self, ConfigFile, ExternalConflict, WorkspaceFile};
use crate::theme;
use eframe::egui::{
    self, Align, Button, Color32, ComboBox, FontFamily, FontId, Frame, Key, Layout, Margin,
    RichText, ScrollArea, Sense, Stroke, TextEdit, Ui,
};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const APP_STORAGE_KEY: &str = "espanso-gui.preferences";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    Library,
    Profiles,
    Globals,
    Diagnostics,
    Settings,
    About,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditorTab {
    Content,
    Variables,
    Options,
    RawYaml,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Preferences {
    config_root: PathBuf,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            config_root: espanso::default_config_root(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MessageKind {
    Success,
    Info,
    Error,
}

#[derive(Debug, Clone)]
struct Message {
    kind: MessageKind,
    text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VariableScope {
    Local,
    Global,
}

#[derive(Debug, Clone)]
struct VariableEditor {
    scope: VariableScope,
    index: Option<usize>,
    variable: Variable,
    insert_in_content: bool,
}

impl VariableEditor {
    fn new(scope: VariableScope, kind: &str) -> Self {
        Self {
            scope,
            index: None,
            variable: Variable::new(kind),
            insert_in_content: scope == VariableScope::Local,
        }
    }
}

#[derive(Debug, Clone)]
struct FormFieldEditor {
    original_name: Option<String>,
    name: String,
    field: FormField,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingDelete {
    Snippet,
    File,
}

#[derive(Debug, Clone, Copy)]
enum ConflictTarget {
    Match(usize),
    Config(usize),
}

#[derive(Debug, Clone)]
struct ConflictDialog {
    target: ConflictTarget,
    conflict: ExternalConflict,
    choices: Vec<ResolutionChoice>,
}

#[derive(Debug, Clone)]
struct PendingRestore {
    relative_path: PathBuf,
    backup_path: PathBuf,
    timestamp: String,
}

pub struct EspansoGuiApp {
    preferences: Preferences,
    status: EspansoStatus,
    files: Vec<WorkspaceFile>,
    config_files: Vec<ConfigFile>,
    selected_file: usize,
    selected_config: usize,
    selected_snippet: usize,
    section: Section,
    editor_tab: EditorTab,
    search: String,
    message: Option<Message>,
    load_error: Option<String>,
    new_file_dialog: bool,
    new_file_name: String,
    new_config_dialog: bool,
    new_config_name: String,
    profile_raw_yaml: bool,
    variable_editor: Option<VariableEditor>,
    form_field_editor: Option<FormFieldEditor>,
    pending_delete: Option<PendingDelete>,
    conflict_dialog: Option<ConflictDialog>,
    pending_restore: Option<PendingRestore>,
    confirm_close: bool,
    markdown_cache: CommonMarkCache,
}

impl EspansoGuiApp {
    pub fn new(creation_context: &eframe::CreationContext<'_>) -> Self {
        theme::install(&creation_context.egui_ctx);
        egui_extras::install_image_loaders(&creation_context.egui_ctx);
        let status = espanso::detect();
        let mut preferences = creation_context
            .storage
            .and_then(|storage| eframe::get_value(storage, APP_STORAGE_KEY))
            .unwrap_or_else(|| Preferences {
                config_root: status.config_root.clone(),
            });
        if preferences.config_root.as_os_str().is_empty() {
            preferences.config_root.clone_from(&status.config_root);
        }
        let (files, config_files, load_error) = if preferences.config_root.join("match").is_dir() {
            let matches = storage::load_workspace(&preferences.config_root);
            let profiles = storage::load_config_profiles(&preferences.config_root);
            let load_error = matches
                .as_ref()
                .err()
                .map(ToString::to_string)
                .or_else(|| profiles.as_ref().err().map(ToString::to_string));
            (
                matches.unwrap_or_default(),
                profiles.unwrap_or_default(),
                load_error,
            )
        } else {
            (Vec::new(), Vec::new(), None)
        };

        Self {
            preferences,
            status,
            files,
            config_files,
            selected_file: 0,
            selected_config: 0,
            selected_snippet: 0,
            section: Section::Library,
            editor_tab: EditorTab::Content,
            search: String::new(),
            message: None,
            load_error,
            new_file_dialog: false,
            new_file_name: "snippets".into(),
            new_config_dialog: false,
            new_config_name: "application".into(),
            profile_raw_yaml: false,
            variable_editor: None,
            form_field_editor: None,
            pending_delete: None,
            conflict_dialog: None,
            pending_restore: None,
            confirm_close: false,
            markdown_cache: CommonMarkCache::default(),
        }
    }

    fn has_dirty_files(&self) -> bool {
        self.files.iter().any(|file| file.dirty) || self.config_files.iter().any(|file| file.dirty)
    }

    fn selected_file(&self) -> Option<&WorkspaceFile> {
        self.files.get(self.selected_file)
    }

    fn selected_file_mut(&mut self) -> Option<&mut WorkspaceFile> {
        self.files.get_mut(self.selected_file)
    }

    fn notify(&mut self, kind: MessageKind, text: impl Into<String>) {
        self.message = Some(Message {
            kind,
            text: text.into(),
        });
    }

    fn reload_workspace(&mut self) {
        if self.has_dirty_files() {
            self.notify(
                MessageKind::Error,
                "未保存の変更があります。保存してから再読み込みしてください",
            );
            return;
        }
        match (
            storage::load_workspace(&self.preferences.config_root),
            storage::load_config_profiles(&self.preferences.config_root),
        ) {
            (Ok(files), Ok(config_files)) => {
                self.files = files;
                self.config_files = config_files;
                self.selected_file = self.selected_file.min(self.files.len().saturating_sub(1));
                self.selected_config = self
                    .selected_config
                    .min(self.config_files.len().saturating_sub(1));
                self.selected_snippet = 0;
                self.load_error = None;
                self.notify(MessageKind::Success, "Espanso設定を再読み込みしました");
            }
            (Err(error), _) | (_, Err(error)) => self.notify(MessageKind::Error, error.to_string()),
        }
    }

    fn save_selected(&mut self) {
        let root = self.preferences.config_root.clone();
        let index = self.selected_file;
        let Some(file) = self.files.get(index) else {
            return;
        };
        if file.is_package {
            self.notify(
                MessageKind::Error,
                "Hubパッケージは更新で上書きされるため直接保存できません。ユーザーファイルへコピーしてください",
            );
            return;
        }
        match storage::analyze_workspace_conflict(&root, file) {
            Ok(Some(conflict)) => {
                self.conflict_dialog = Some(ConflictDialog {
                    choices: vec![ResolutionChoice::Local; conflict.plan.conflicts.len()],
                    target: ConflictTarget::Match(index),
                    conflict,
                });
                self.notify(
                    MessageKind::Info,
                    "外部変更を検出しました。local three-way mergeを確認してください",
                );
                return;
            }
            Ok(None) => {}
            Err(error) => {
                self.notify(MessageKind::Error, error.to_string());
                return;
            }
        }
        let file = &mut self.files[index];
        match storage::save_workspace_file(&root, file) {
            Ok(receipt) => {
                let backup = receipt
                    .backup_path
                    .map(|path| format!(" / バックアップ: {}", path.display()))
                    .unwrap_or_default();
                self.notify(
                    MessageKind::Success,
                    format!("保存しました{backup} / {}", &receipt.hash[..8]),
                );
            }
            Err(error) => self.notify(MessageKind::Error, error.to_string()),
        }
    }

    fn save_selected_config(&mut self) {
        let root = self.preferences.config_root.clone();
        let index = self.selected_config;
        let Some(file) = self.config_files.get(index) else {
            return;
        };
        match storage::analyze_config_conflict(&root, file) {
            Ok(Some(conflict)) => {
                self.conflict_dialog = Some(ConflictDialog {
                    choices: vec![ResolutionChoice::Local; conflict.plan.conflicts.len()],
                    target: ConflictTarget::Config(index),
                    conflict,
                });
                self.notify(
                    MessageKind::Info,
                    "外部変更を検出しました。local three-way mergeを確認してください",
                );
                return;
            }
            Ok(None) => {}
            Err(error) => {
                self.notify(MessageKind::Error, error.to_string());
                return;
            }
        }
        let file = &mut self.config_files[index];
        match storage::save_config_file(&root, file) {
            Ok(receipt) => {
                let backup = receipt
                    .backup_path
                    .map(|path| format!(" / バックアップ: {}", path.display()))
                    .unwrap_or_default();
                self.notify(
                    MessageKind::Success,
                    format!(
                        "設定プロファイルを保存しました{backup} / {}",
                        &receipt.hash[..8]
                    ),
                );
            }
            Err(error) => self.notify(MessageKind::Error, error.to_string()),
        }
    }

    fn save_current(&mut self) {
        if self.section == Section::Profiles {
            self.save_selected_config();
        } else {
            self.save_selected();
        }
    }

    fn mark_selected_file_changed(&mut self) {
        if let Some(file) = self.selected_file_mut()
            && let Err(error) = file.refresh_raw_from_document()
        {
            self.notify(MessageKind::Error, error.to_string());
        }
    }

    fn add_snippet(&mut self) {
        let index = if let Some(file) = self.selected_file_mut() {
            if file.is_package {
                self.notify(MessageKind::Error, "パッケージには追加できません");
                return;
            }
            file.document.matches.push(Snippet::new());
            file.document.matches.len() - 1
        } else {
            return;
        };
        self.selected_snippet = index;
        self.editor_tab = EditorTab::Content;
        self.mark_selected_file_changed();
    }

    fn duplicate_snippet(&mut self) {
        let selected = self.selected_snippet;
        let index = if let Some(file) = self.selected_file_mut() {
            if file.is_package {
                self.notify(
                    MessageKind::Info,
                    "パッケージのコピーはユーザーファイルへ作成してください",
                );
                return;
            }
            let Some(mut snippet) = file.document.matches.get(selected).cloned() else {
                return;
            };
            if let Some(label) = &mut snippet.label {
                label.push_str("（コピー）");
            }
            let triggers = snippet
                .trigger_list()
                .into_iter()
                .map(|trigger| format!("{trigger}-copy"))
                .collect();
            snippet.set_trigger_list(triggers);
            file.document.matches.insert(selected + 1, snippet);
            selected + 1
        } else {
            return;
        };
        self.selected_snippet = index;
        self.mark_selected_file_changed();
    }

    fn copy_package_snippet_to_user_file(&mut self) {
        let Some(snippet) = self
            .selected_file()
            .and_then(|file| file.document.matches.get(self.selected_snippet))
            .cloned()
        else {
            return;
        };
        let target = self
            .files
            .iter()
            .position(|file| file.relative_path.as_path() == Path::new("match/base.yml"))
            .or_else(|| self.files.iter().position(|file| !file.is_package));
        let Some(target) = target else {
            self.notify(
                MessageKind::Error,
                "コピー先のユーザーファイルがありません。先にファイルを追加してください",
            );
            return;
        };
        self.files[target].document.matches.push(snippet);
        self.selected_file = target;
        self.selected_snippet = self.files[target].document.matches.len() - 1;
        self.mark_selected_file_changed();
        self.notify(
            MessageKind::Success,
            "ユーザーファイルへコピーしました（トリガーの重複を確認してください）",
        );
    }

    fn delete_selected_snippet(&mut self) {
        let selected = self.selected_snippet;
        if let Some(file) = self.selected_file_mut() {
            if file.is_package || selected >= file.document.matches.len() {
                return;
            }
            file.document.matches.remove(selected);
            self.selected_snippet = selected.min(file.document.matches.len().saturating_sub(1));
            self.mark_selected_file_changed();
            self.notify(
                MessageKind::Info,
                "スニペットを削除しました（まだ未保存です）",
            );
        }
    }

    fn create_file(&mut self) {
        match storage::create_match_file(&self.preferences.config_root, &self.new_file_name) {
            Ok(file) => {
                self.files.push(file);
                self.files
                    .sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
                self.selected_file = self
                    .files
                    .iter()
                    .position(|file| file.display_name == self.new_file_name.trim())
                    .unwrap_or_default();
                self.selected_snippet = 0;
                self.new_file_dialog = false;
                self.notify(MessageKind::Success, "スニペットファイルを作成しました");
            }
            Err(error) => self.notify(MessageKind::Error, error.to_string()),
        }
    }

    fn create_config_file(&mut self) {
        match storage::create_config_file(&self.preferences.config_root, &self.new_config_name) {
            Ok(file) => {
                self.config_files.push(file);
                self.config_files
                    .sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
                self.selected_config = self
                    .config_files
                    .iter()
                    .position(|file| file.display_name == self.new_config_name.trim())
                    .unwrap_or_default();
                self.new_config_dialog = false;
                self.notify(MessageKind::Success, "設定プロファイルを作成しました");
            }
            Err(error) => self.notify(MessageKind::Error, error.to_string()),
        }
    }

    fn delete_selected_file(&mut self) {
        let Some(file) = self.selected_file() else {
            return;
        };
        if file.is_package {
            self.notify(
                MessageKind::Error,
                "パッケージファイルはこの画面から削除できません",
            );
            return;
        }
        let relative = file.relative_path.clone();
        match storage::move_to_recoverable_trash(&self.preferences.config_root, &relative) {
            Ok(destination) => {
                self.files.remove(self.selected_file);
                self.selected_file = self.selected_file.min(self.files.len().saturating_sub(1));
                self.selected_snippet = 0;
                self.notify(
                    MessageKind::Success,
                    format!("ファイルを退避しました: {}", destination.display()),
                );
            }
            Err(error) => self.notify(MessageKind::Error, error.to_string()),
        }
    }

    fn initialize_config(&mut self) {
        match storage::initialize_root(&self.preferences.config_root) {
            Ok(()) => {
                self.reload_workspace();
                self.notify(MessageKind::Success, "Espanso設定フォルダを初期化しました");
            }
            Err(error) => self.notify(MessageKind::Error, error.to_string()),
        }
    }

    fn choose_config_root(&mut self) {
        if self.has_dirty_files() {
            self.notify(
                MessageKind::Error,
                "設定フォルダを変える前に未保存の変更を保存してください",
            );
            return;
        }
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Espanso設定フォルダを選択")
            .pick_folder()
        {
            self.preferences.config_root = path;
            self.reload_workspace();
        }
    }

    fn run_espanso_action(&mut self, action: EspansoAction) {
        match espanso::action(action) {
            Ok(result) if result.success => {
                self.notify(
                    MessageKind::Success,
                    if result.output.is_empty() {
                        format!("Espanso: {action}")
                    } else {
                        result.output
                    },
                );
                self.status = espanso::detect();
            }
            Ok(result) => self.notify(MessageKind::Error, result.output),
            Err(error) => self.notify(MessageKind::Error, error.to_string()),
        }
    }

    fn keyboard_shortcuts(&mut self, ui: &Ui) {
        let save = ui.input(|input| input.modifiers.command && input.key_pressed(Key::S));
        let new = ui.input(|input| input.modifiers.command && input.key_pressed(Key::N));
        if save {
            self.save_current();
        }
        if new && self.section == Section::Library {
            self.add_snippet();
        }
    }

    fn top_bar(&mut self, ui: &mut Ui) {
        let can_save = if self.section == Section::Profiles {
            !self.config_files.is_empty()
        } else {
            self.selected_file().is_some_and(|file| !file.is_package)
        };
        egui::Panel::top("top-bar")
            .frame(
                Frame::new()
                    .fill(theme::PANEL)
                    .inner_margin(Margin::symmetric(18, 10))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(218, 220, 212))),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("E/").size(23.0).strong().color(theme::ACCENT));
                    ui.label(RichText::new("Espanso GUI").size(18.0).strong());
                    ui.add_space(10.0);
                    status_badge(ui, &self.status);
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .add_enabled(
                                can_save,
                                Button::new(RichText::new("保存  ⌘S").strong())
                                    .fill(theme::ACCENT)
                                    .stroke(Stroke::NONE),
                            )
                            .clicked()
                        {
                            self.save_current();
                        }
                        if ui.button("再読み込み").clicked() {
                            self.reload_workspace();
                        }
                        if self.status.installed && ui.button("Espanso再起動").clicked() {
                            self.run_espanso_action(EspansoAction::Restart);
                        }
                        if self.has_dirty_files() {
                            ui.label(RichText::new("未保存").color(theme::AMBER).strong());
                        }
                    });
                });
            });
    }

    fn navigation(&mut self, ui: &mut Ui) {
        egui::Panel::left("navigation")
            .exact_size(220.0)
            .resizable(false)
            .frame(
                Frame::new()
                    .fill(theme::SIDEBAR)
                    .inner_margin(Margin::same(14)),
            )
            .show(ui, |ui| {
                ui.add_space(4.0);
                ui.label(RichText::new("ワークスペース").small().color(theme::MUTED));
                ui.add_space(4.0);
                nav_button(ui, &mut self.section, Section::Library, "スニペット", "⌘1");
                nav_button(
                    ui,
                    &mut self.section,
                    Section::Profiles,
                    "アプリ別設定",
                    "⌘2",
                );
                nav_button(
                    ui,
                    &mut self.section,
                    Section::Globals,
                    "グローバル変数",
                    "⌘3",
                );
                nav_button(ui, &mut self.section, Section::Diagnostics, "診断", "⌘4");
                ui.add_space(16.0);
                ui.label(RichText::new("Espanso").small().color(theme::MUTED));
                ui.add_space(4.0);

                ScrollArea::vertical().id_salt("file-list").show(ui, |ui| {
                    let mut selected = None;
                    for (index, file) in self.files.iter().enumerate() {
                        let count = file.snippet_count();
                        let dirty = if file.dirty { " •" } else { "" };
                        let package = if file.is_package { "  package" } else { "" };
                        let label =
                            format!("{}{}\n{}件{}", file.display_name, dirty, count, package);
                        if ui
                            .add_sized(
                                [190.0, 48.0],
                                Button::new(label).selected(self.selected_file == index),
                            )
                            .clicked()
                        {
                            selected = Some(index);
                        }
                    }
                    if let Some(index) = selected {
                        self.selected_file = index;
                        self.selected_snippet = 0;
                    }
                });

                ui.add_space(6.0);
                if ui
                    .add_sized([190.0, 34.0], Button::new("＋ ファイルを追加"))
                    .clicked()
                {
                    self.new_file_dialog = true;
                }
                ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
                    nav_button(
                        ui,
                        &mut self.section,
                        Section::About,
                        "このアプリについて",
                        "",
                    );
                    nav_button(
                        ui,
                        &mut self.section,
                        Section::Settings,
                        "設定とバックアップ",
                        "",
                    );
                    ui.label(
                        RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                            .small()
                            .color(theme::MUTED),
                    );
                });
            });
    }

    fn library_view(&mut self, ui: &mut Ui) {
        if self.files.is_empty() {
            self.empty_workspace(ui);
            return;
        }
        self.snippet_list(ui);
        egui::CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(theme::PAPER)
                    .inner_margin(Margin::same(22)),
            )
            .show(ui, |ui| self.snippet_editor(ui));
    }

    fn snippet_list(&mut self, ui: &mut Ui) {
        egui::Panel::left("snippet-list")
            .exact_size(330.0)
            .resizable(true)
            .size_range(280.0..=440.0)
            .frame(
                Frame::new()
                    .fill(theme::PANEL)
                    .inner_margin(Margin::same(14))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(220, 222, 214))),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("スニペット");
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("＋ 新規").clicked() {
                            self.add_snippet();
                        }
                    });
                });
                ui.add(
                    TextEdit::singleline(&mut self.search)
                        .hint_text("検索: トリガー、内容、ラベル")
                        .desired_width(f32::INFINITY),
                );
                ui.add_space(4.0);

                let search = self.search.to_lowercase();
                let Some(file) = self.selected_file() else {
                    return;
                };
                let snippets: Vec<_> = file
                    .document
                    .matches
                    .iter()
                    .enumerate()
                    .filter(|(_, snippet)| {
                        search.is_empty() || snippet.searchable_text().contains(&search)
                    })
                    .map(|(index, snippet)| {
                        (
                            index,
                            snippet.title(),
                            snippet.trigger_list().join("  "),
                            snippet
                                .content()
                                .lines()
                                .next()
                                .unwrap_or_default()
                                .to_string(),
                            snippet.content_kind().label(),
                        )
                    })
                    .collect();
                ScrollArea::vertical()
                    .id_salt("snippets")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for (index, title, triggers, preview, kind) in snippets {
                            let selected = self.selected_snippet == index;
                            let fill = if selected {
                                theme::ACCENT_SOFT
                            } else {
                                theme::PANEL
                            };
                            let response = Frame::new()
                                .fill(fill)
                                .stroke(Stroke::new(
                                    1.0,
                                    if selected {
                                        theme::ACCENT
                                    } else {
                                        Color32::from_rgb(223, 225, 217)
                                    },
                                ))
                                .corner_radius(9)
                                .inner_margin(Margin::same(10))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(title).strong());
                                        ui.with_layout(
                                            Layout::right_to_left(Align::Center),
                                            |ui| {
                                                ui.label(
                                                    RichText::new(kind).small().color(theme::MUTED),
                                                );
                                            },
                                        );
                                    });
                                    if !triggers.is_empty() {
                                        ui.label(
                                            RichText::new(triggers)
                                                .family(FontFamily::Monospace)
                                                .color(theme::ACCENT),
                                        );
                                    }
                                    ui.label(
                                        RichText::new(truncate(&preview, 66))
                                            .small()
                                            .color(theme::MUTED),
                                    );
                                })
                                .response
                                .interact(Sense::click());
                            if response.clicked() {
                                self.selected_snippet = index;
                                self.editor_tab = EditorTab::Content;
                            }
                            ui.add_space(6.0);
                        }
                    });

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("複製").clicked() {
                        self.duplicate_snippet();
                    }
                    if ui
                        .button(RichText::new("削除").color(theme::DANGER))
                        .clicked()
                    {
                        self.pending_delete = Some(PendingDelete::Snippet);
                    }
                });
            });
    }

    fn snippet_editor(&mut self, ui: &mut Ui) {
        let Some(snippet_count) = self.selected_file().map(|file| file.document.matches.len())
        else {
            return;
        };
        if snippet_count == 0 {
            centered_empty_state(
                ui,
                "まだスニペットがありません",
                "最初のトリガーと展開内容を作成しましょう。",
            );
            if ui.button("最初のスニペットを作成").clicked() {
                self.add_snippet();
            }
            return;
        }
        self.selected_snippet = self.selected_snippet.min(snippet_count.saturating_sub(1));
        let Some((is_package, mut snippet)) = self.selected_file().map(|file| {
            (
                file.is_package,
                file.document.matches[self.selected_snippet].clone(),
            )
        }) else {
            return;
        };
        let original = snippet.clone();

        ui.horizontal(|ui| {
            ui.heading(snippet.title());
            if is_package {
                ui.label(RichText::new("読み取り専用パッケージ").color(theme::AMBER));
            }
        });
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            tab_button(ui, &mut self.editor_tab, EditorTab::Content, "内容");
            tab_button(ui, &mut self.editor_tab, EditorTab::Variables, "変数");
            tab_button(
                ui,
                &mut self.editor_tab,
                EditorTab::Options,
                "詳細オプション",
            );
            tab_button(ui, &mut self.editor_tab, EditorTab::RawYaml, "Raw YAML");
        });
        ui.separator();

        if is_package {
            ui.add_space(8.0);
            callout(
                ui,
                theme::AMBER,
                "Espanso Hubのパッケージは更新時に上書きされます。内容は確認できますが、直接編集は無効です。",
            );
            if ui
                .button("このスニペットをユーザーファイルへコピー")
                .clicked()
            {
                self.copy_package_snippet_to_user_file();
                return;
            }
            ui.disable();
        }

        ScrollArea::vertical()
            .id_salt("editor-scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| match self.editor_tab {
                EditorTab::Content => self.content_editor(ui, &mut snippet),
                EditorTab::Variables => self.variables_editor(ui, &mut snippet),
                EditorTab::Options => self.options_editor(ui, &mut snippet),
                EditorTab::RawYaml => self.raw_yaml_editor(ui),
            });

        if !is_package
            && snippet != original
            && self.editor_tab != EditorTab::RawYaml
            && let Some(file) = self.files.get_mut(self.selected_file)
        {
            file.document.matches[self.selected_snippet] = snippet;
            if let Err(error) = file.refresh_raw_from_document() {
                self.notify(MessageKind::Error, error.to_string());
            }
        }
    }

    fn content_editor(&mut self, ui: &mut Ui, snippet: &mut Snippet) {
        two_column_field(
            ui,
            "表示名",
            "Espanso検索バーに表示するラベル",
            |ui| {
                let mut label = snippet.label.clone().unwrap_or_default();
                if ui
                    .add(TextEdit::singleline(&mut label).hint_text("例: 署名（日本語）"))
                    .changed()
                {
                    snippet.label = (!label.trim().is_empty()).then_some(label);
                }
            },
        );
        ui.add_space(10.0);

        let mut regex_mode = snippet.regex.is_some();
        two_column_field(
            ui,
            "トリガー",
            "カンマ区切りで複数指定できます",
            |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut regex_mode, false, "通常");
                    ui.selectable_value(&mut regex_mode, true, "正規表現");
                });
                if regex_mode {
                    let mut regex = snippet.regex.clone().unwrap_or_default();
                    if ui
                        .add(
                            TextEdit::singleline(&mut regex)
                                .hint_text("例: :hello\\((?P<name>.*)\\)"),
                        )
                        .changed()
                    {
                        snippet.regex = Some(regex);
                        snippet.trigger = None;
                        snippet.triggers.clear();
                    }
                } else {
                    let mut triggers = snippet.trigger_list().join(", ");
                    if ui
                        .add(TextEdit::singleline(&mut triggers).hint_text(":sig, :signature"))
                        .changed()
                    {
                        snippet.set_trigger_list(triggers.split(',').map(str::to_string).collect());
                    }
                }
            },
        );
        ui.add_space(14.0);

        ui.horizontal(|ui| {
            ui.label(RichText::new("展開タイプ").strong());
            let mut kind = snippet.content_kind();
            ComboBox::from_id_salt("content-kind")
                .selected_text(kind.label())
                .show_ui(ui, |ui| {
                    for candidate in ContentKind::ALL {
                        ui.selectable_value(&mut kind, candidate, candidate.label());
                    }
                });
            if kind != snippet.content_kind() {
                snippet.set_content_kind(kind);
            }
        });

        ui.add_space(8.0);
        editor_toolbar(ui, snippet);
        ui.add_space(6.0);
        match snippet.content_kind() {
            ContentKind::Image => self.image_editor(ui, snippet),
            ContentKind::Form => self.form_editor(ui, snippet),
            kind => {
                ui.add(
                    TextEdit::multiline(snippet.content_mut())
                        .font(FontId::new(15.0, FontFamily::Monospace))
                        .desired_rows(14)
                        .desired_width(f32::INFINITY)
                        .hint_text(match kind {
                            ContentKind::Html => "<strong>HTML</strong> を入力",
                            ContentKind::Markdown => "**Markdown** を入力",
                            _ => "展開するテキストを入力",
                        }),
                );
                self.content_preview(ui, snippet);
            }
        }
    }

    fn image_editor(&mut self, ui: &mut Ui, snippet: &mut Snippet) {
        ui.horizontal(|ui| {
            ui.add(
                TextEdit::singleline(snippet.content_mut())
                    .hint_text("$CONFIG/assets/image.png または絶対パス")
                    .desired_width(500.0),
            );
            if ui.button("画像を選択").clicked()
                && let Some(path) = rfd::FileDialog::new()
                    .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp"])
                    .pick_file()
            {
                *snippet.content_mut() = path.to_string_lossy().into_owned();
            }
        });
        ui.add_space(10.0);
        let path = snippet.content();
        if !path.is_empty() && !path.contains("$CONFIG") {
            ui.add(
                egui::Image::from_uri(format!("file://{path}")).max_size(egui::vec2(520.0, 320.0)),
            );
        } else {
            callout(
                ui,
                theme::ACCENT,
                "$CONFIGはEspanso設定フォルダへ展開されます。LinuxではPNGが最も互換性の高い形式です。",
            );
        }
    }

    fn form_editor(&mut self, ui: &mut Ui, snippet: &mut Snippet) {
        callout(
            ui,
            theme::ACCENT,
            "本文中に [[name]] のように書くと、Espanso展開時に入力欄が表示されます。",
        );
        ui.add(
            TextEdit::multiline(snippet.content_mut())
                .font(FontId::new(15.0, FontFamily::Monospace))
                .desired_rows(9)
                .desired_width(f32::INFINITY)
                .hint_text("お名前: [[name]]\n種類: [[plan]]"),
        );
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("フォーム項目").strong().size(17.0));
            if ui.button("＋ 項目を追加").clicked() {
                self.form_field_editor = Some(FormFieldEditor {
                    original_name: None,
                    name: "field".into(),
                    field: FormField::default(),
                });
            }
        });
        let fields: Vec<_> = snippet
            .form_fields
            .iter()
            .map(|(name, field)| (name.clone(), field.clone()))
            .collect();
        let mut remove = None;
        for (name, field) in fields {
            Frame::group(ui.style()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("[[{name}]]"))
                            .family(FontFamily::Monospace)
                            .strong(),
                    );
                    ui.label(field_kind_label(&field));
                    if let Some(default) = &field.default {
                        ui.label(
                            RichText::new(format!("既定: {default}"))
                                .small()
                                .color(theme::MUTED),
                        );
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.small_button("削除").clicked() {
                            remove = Some(name.clone());
                        }
                        if ui.small_button("編集").clicked() {
                            self.form_field_editor = Some(FormFieldEditor {
                                original_name: Some(name.clone()),
                                name: name.clone(),
                                field: field.clone(),
                            });
                        }
                    });
                });
            });
        }
        if let Some(name) = remove {
            snippet.form_fields.shift_remove(&name);
        }
    }

    fn content_preview(&mut self, ui: &mut Ui, snippet: &Snippet) {
        ui.add_space(14.0);
        ui.label(RichText::new("ライブプレビュー").strong().size(17.0));
        Frame::new()
            .fill(theme::PANEL)
            .stroke(Stroke::new(1.0, Color32::from_rgb(216, 220, 211)))
            .corner_radius(9)
            .inner_margin(Margin::same(14))
            .show(ui, |ui| match snippet.content_kind() {
                ContentKind::Markdown => {
                    CommonMarkViewer::new().show(ui, &mut self.markdown_cache, snippet.content());
                }
                ContentKind::Html => {
                    ui.label(
                        RichText::new("HTMLは展開先アプリのクリップボード機能で描画されます。ここではソースを表示します。")
                            .small()
                            .color(theme::MUTED),
                    );
                    ui.code(snippet.content());
                }
                _ => {
                    ui.label(snippet.content());
                }
            });
    }

    fn variables_editor(&mut self, ui: &mut Ui, snippet: &mut Snippet) {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("すぐ追加").strong());
            for (kind, label) in [
                ("date", "日付・時刻"),
                ("clipboard", "クリップボード"),
                ("choice", "候補選択"),
                ("form", "フォーム"),
                ("random", "ランダム"),
                ("echo", "固定値"),
                ("shell", "シェル"),
                ("script", "スクリプト"),
            ] {
                if ui.button(format!("＋ {label}")).clicked() {
                    self.variable_editor = Some(VariableEditor::new(VariableScope::Local, kind));
                }
            }
        });
        ui.add_space(10.0);
        if snippet.vars.is_empty() {
            centered_empty_state(
                ui,
                "このスニペットには変数がありません",
                "上のボタンから種類を選ぶだけで追加できます。",
            );
        }

        let mut remove = None;
        let variables = snippet.vars.clone();
        for (index, variable) in variables.iter().enumerate() {
            Frame::group(ui.style()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new(&variable.name).strong().size(16.0));
                        ui.label(
                            RichText::new(format!("{}  {}", variable.kind, variable.token()))
                                .family(FontFamily::Monospace)
                                .color(theme::ACCENT),
                        );
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.small_button("削除").clicked() {
                            remove = Some(index);
                        }
                        if ui.small_button("編集").clicked() {
                            self.variable_editor = Some(VariableEditor {
                                scope: VariableScope::Local,
                                index: Some(index),
                                variable: variable.clone(),
                                insert_in_content: false,
                            });
                        }
                        if ui.small_button("本文に挿入").clicked() {
                            snippet.insert_token(&variable.token());
                        }
                    });
                });
                variable_summary(ui, variable);
            });
            ui.add_space(6.0);
        }
        if let Some(index) = remove {
            snippet.vars.remove(index);
        }

        if snippet
            .vars
            .iter()
            .any(|variable| matches!(variable.kind.as_str(), "shell" | "script"))
        {
            callout(
                ui,
                theme::AMBER,
                "シェル／スクリプト変数はトリガー実行時にローカルコマンドを実行します。自分で内容を確認したコードだけを使用してください。",
            );
        }
    }

    fn options_editor(&mut self, ui: &mut Ui, snippet: &mut Snippet) {
        ui.heading("トリガー条件");
        option_checkbox(
            ui,
            &mut snippet.word,
            "単語単位（word）",
            "単語の区切りに囲まれた場合だけ展開",
        );
        option_checkbox(
            ui,
            &mut snippet.left_word,
            "単語の左端（left_word）",
            "単語の先頭でだけ展開",
        );
        option_checkbox(
            ui,
            &mut snippet.right_word,
            "単語の右端（right_word）",
            "単語の末尾でだけ展開",
        );
        ui.add_space(14.0);
        ui.heading("大文字・小文字");
        option_checkbox(
            ui,
            &mut snippet.propagate_case,
            "入力のケースを引き継ぐ",
            "hello / Hello / HELLO に合わせて展開結果を変換",
        );
        two_column_field(
            ui,
            "大文字スタイル",
            "複数単語の変換方法",
            |ui| {
                let mut style = snippet.uppercase_style.clone().unwrap_or_default();
                ComboBox::from_id_salt("uppercase-style")
                    .selected_text(if style.is_empty() { "標準" } else { &style })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut style, String::new(), "標準");
                        ui.selectable_value(
                            &mut style,
                            "capitalize_words".into(),
                            "各単語を大文字",
                        );
                        ui.selectable_value(&mut style, "capitalize".into(), "先頭のみ大文字");
                    });
                snippet.uppercase_style = (!style.is_empty()).then_some(style);
            },
        );
        ui.add_space(14.0);
        ui.heading("展開方法");
        two_column_field(
            ui,
            "強制モード",
            "問題があるアプリ向けの上書き",
            |ui| {
                let mut mode = snippet.force_mode.clone().unwrap_or_default();
                ComboBox::from_id_salt("force-mode")
                    .selected_text(if mode.is_empty() { "自動" } else { &mode })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut mode, String::new(), "自動");
                        ui.selectable_value(&mut mode, "clipboard".into(), "clipboard");
                        ui.selectable_value(&mut mode, "keys".into(), "keys");
                    });
                snippet.force_mode = (!mode.is_empty()).then_some(mode);
            },
        );
        if snippet.content_kind() == ContentKind::Markdown {
            option_checkbox(
                ui,
                &mut snippet.paragraph,
                "段落として貼り付けない",
                "markdownのparagraphオプション",
            );
        }
        ui.add_space(14.0);
        ui.heading("検索");
        let mut terms = snippet.search_terms.join(", ");
        two_column_field(ui, "検索キーワード", "カンマ区切り", |ui| {
            if ui
                .add(TextEdit::singleline(&mut terms).hint_text("署名, email, work"))
                .changed()
            {
                snippet.search_terms = terms
                    .split(',')
                    .map(str::trim)
                    .filter(|term| !term.is_empty())
                    .map(str::to_string)
                    .collect();
            }
        });
    }

    fn raw_yaml_editor(&mut self, ui: &mut Ui) {
        let apply_clicked = {
            let Some(file) = self.files.get_mut(self.selected_file) else {
                return;
            };
            if file.had_comments {
                callout(
                    ui,
                    theme::AMBER,
                    "構造化エディタで変更するとコメント位置は再整形されます。元ファイルは保存時に自動バックアップされます。Raw YAMLだけの編集ならコメントを維持できます。",
                );
            }
            ui.label(
                RichText::new(file.relative_path.display().to_string())
                    .family(FontFamily::Monospace),
            );
            let changed = ui
                .add(
                    TextEdit::multiline(&mut file.raw_yaml)
                        .font(FontId::new(14.0, FontFamily::Monospace))
                        .desired_rows(28)
                        .desired_width(f32::INFINITY),
                )
                .changed();
            if changed {
                file.dirty = true;
            }
            let mut apply_clicked = false;
            ui.horizontal(|ui| {
                apply_clicked = ui.button("YAMLを検証して適用").clicked();
                ui.label(
                    RichText::new("保存時にも必ず検証します")
                        .small()
                        .color(theme::MUTED),
                );
            });
            apply_clicked
        };
        if apply_clicked {
            let result = self.files[self.selected_file].apply_raw_yaml();
            match result {
                Ok(()) => self.notify(MessageKind::Success, "YAMLは有効です"),
                Err(error) => self.notify(MessageKind::Error, error.to_string()),
            }
        }
    }

    fn profiles_view(&mut self, ui: &mut Ui) {
        egui::Panel::left("profile-list")
            .exact_size(280.0)
            .resizable(true)
            .size_range(240.0..=380.0)
            .frame(
                Frame::new()
                    .fill(theme::PANEL)
                    .inner_margin(Margin::same(14))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(218, 220, 212))),
            )
            .show(ui, |ui| {
                ui.heading("設定プロファイル");
                ui.label(
                    RichText::new("config/*.yml")
                        .family(FontFamily::Monospace)
                        .small()
                        .color(theme::MUTED),
                );
                ui.separator();
                ScrollArea::vertical()
                    .id_salt("profile-file-list")
                    .show(ui, |ui| {
                        for (index, file) in self.config_files.iter().enumerate() {
                            let dirty = if file.dirty { " •" } else { "" };
                            let kind = if file.is_default {
                                "既定"
                            } else if file.profile.has_filter() {
                                "アプリ別"
                            } else {
                                "フィルター未設定"
                            };
                            if ui
                                .add_sized(
                                    [250.0, 48.0],
                                    Button::new(format!("{}{dirty}\n{kind}", file.display_name))
                                        .selected(self.selected_config == index),
                                )
                                .clicked()
                            {
                                self.selected_config = index;
                            }
                        }
                    });
                ui.add_space(6.0);
                if ui
                    .add_sized([250.0, 34.0], Button::new("＋ プロファイルを追加"))
                    .clicked()
                {
                    self.new_config_dialog = true;
                }
            });

        egui::CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(theme::PAPER)
                    .inner_margin(Margin::same(22)),
            )
            .show(ui, |ui| {
                if self.config_files.is_empty() {
                    centered_empty_state(
                        ui,
                        "設定プロファイルがありません",
                        "default またはアプリ別プロファイルを追加できます。",
                    );
                    if ui.button("最初のプロファイルを追加").clicked() {
                        self.new_config_dialog = true;
                    }
                    return;
                }

                let index = self
                    .selected_config
                    .min(self.config_files.len().saturating_sub(1));
                self.selected_config = index;
                let file = &self.config_files[index];
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.heading(&file.display_name);
                        ui.label(
                            RichText::new(file.relative_path.display().to_string())
                                .family(FontFamily::Monospace)
                                .small()
                                .color(theme::MUTED),
                        );
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.selectable_value(&mut self.profile_raw_yaml, true, "Raw YAML");
                        ui.selectable_value(&mut self.profile_raw_yaml, false, "ビジュアル");
                    });
                });
                ui.separator();

                if self.profile_raw_yaml {
                    self.profile_raw_editor(ui, index);
                } else {
                    self.profile_visual_editor(ui, index);
                }
            });
    }

    fn profile_visual_editor(&mut self, ui: &mut Ui, index: usize) {
        let is_default = self.config_files[index].is_default;
        let original = self.config_files[index].profile.clone();
        let mut profile = original.clone();

        ScrollArea::vertical()
            .id_salt("profile-editor-scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if is_default {
                    callout(
                        ui,
                        theme::ACCENT,
                        "default.yml はすべてのアプリの基準です。アプリ別ファイルはここで設定した値を継承します。",
                    );
                } else {
                    callout(
                        ui,
                        theme::ACCENT,
                        "フィルターは正規表現です。WaylandではEspansoのアプリ別設定自体が未対応です。",
                    );
                    ui.add_space(10.0);
                    ui.heading("適用するアプリ");
                    optional_text_field(
                        ui,
                        &mut profile.filter_exec,
                        "実行ファイル（filter_exec）",
                        "例: Code|VSCodium",
                    );
                    optional_text_field(
                        ui,
                        &mut profile.filter_class,
                        "ウィンドウクラス（filter_class）",
                        "Linuxでは最も安定した指定",
                    );
                    optional_text_field(
                        ui,
                        &mut profile.filter_title,
                        "ウィンドウタイトル（filter_title）",
                        "例: YouTube",
                    );
                    two_column_field(ui, "OS（filter_os）", "共有設定のOS限定", |ui| {
                        ComboBox::from_id_salt("profile-filter-os")
                            .selected_text(profile.filter_os.as_deref().unwrap_or("継承"))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut profile.filter_os, None, "継承");
                                for value in ["linux", "macos", "windows"] {
                                    ui.selectable_value(
                                        &mut profile.filter_os,
                                        Some(value.into()),
                                        value,
                                    );
                                }
                            });
                    });
                    if !profile.has_filter() {
                        callout(
                            ui,
                            theme::AMBER,
                            "アプリ別ファイルには filter_exec、filter_class、filter_title、filter_os のいずれかが必要です。",
                        );
                    }
                }

                ui.add_space(16.0);
                ui.heading("動作と注入");
                optional_bool_field(
                    ui,
                    &mut profile.enable,
                    "Espansoを有効化",
                    "未指定なら既定設定を継承",
                );
                two_column_field(ui, "注入方式（backend）", "auto / inject / clipboard", |ui| {
                    ComboBox::from_id_salt("profile-backend")
                        .selected_text(profile.backend.as_deref().unwrap_or("継承"))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut profile.backend, None, "継承");
                            for (value, label) in [
                                ("auto", "自動"),
                                ("inject", "キー注入"),
                                ("clipboard", "クリップボード"),
                            ] {
                                ui.selectable_value(
                                    &mut profile.backend,
                                    Some(value.into()),
                                    label,
                                );
                            }
                        });
                });
                optional_bool_field(
                    ui,
                    &mut profile.apply_patch,
                    "組み込みpatchを適用",
                    "terminal等へのEspanso既定補正",
                );
                optional_text_field(
                    ui,
                    &mut profile.paste_shortcut,
                    "貼り付けshortcut",
                    "例: CTRL+SHIFT+V",
                );

                ui.add_space(16.0);
                ui.heading("遅延（ミリ秒）");
                optional_number_field(ui, &mut profile.inject_delay, "文字注入間隔", "inject_delay");
                optional_number_field(ui, &mut profile.key_delay, "キー注入間隔", "key_delay");
                optional_number_field(
                    ui,
                    &mut profile.pre_paste_delay,
                    "貼り付け前",
                    "pre_paste_delay",
                );
                optional_number_field(
                    ui,
                    &mut profile.paste_shortcut_event_delay,
                    "貼り付けキー間隔",
                    "paste_shortcut_event_delay",
                );
                optional_number_field(
                    ui,
                    &mut profile.post_form_delay,
                    "フォーム後",
                    "post_form_delay",
                );
                optional_number_field(
                    ui,
                    &mut profile.post_search_delay,
                    "検索後",
                    "post_search_delay",
                );

                ui.add_space(16.0);
                ui.heading("フォーム上限");
                optional_number_field(
                    ui,
                    &mut profile.max_form_width,
                    "最大幅（px）",
                    "max_form_width",
                );
                optional_number_field(
                    ui,
                    &mut profile.max_form_height,
                    "最大高（px）",
                    "max_form_height",
                );

                if is_default {
                    ui.add_space(16.0);
                    ui.heading("検索と全体設定");
                    optional_text_field(
                        ui,
                        &mut profile.search_shortcut,
                        "検索shortcut",
                        "例: ALT+SPACE / off",
                    );
                    optional_text_field(
                        ui,
                        &mut profile.search_trigger,
                        "検索trigger",
                        "例: .search / off",
                    );
                    optional_text_field(
                        ui,
                        &mut profile.toggle_key,
                        "有効/無効toggle key",
                        "例: RIGHT_CTRL / OFF",
                    );
                    optional_bool_field(
                        ui,
                        &mut profile.preserve_clipboard,
                        "clipboardを復元",
                        "展開前のclipboard内容を保持",
                    );
                    optional_bool_field(
                        ui,
                        &mut profile.show_icon,
                        "status iconを表示",
                        "macOS menu bar / Windows tray",
                    );
                    optional_bool_field(
                        ui,
                        &mut profile.show_notifications,
                        "通知を表示",
                        "Espansoの通知全体",
                    );
                }
            });

        if profile != original {
            self.config_files[index].profile = profile;
            if let Err(error) = self.config_files[index].refresh_raw_from_profile() {
                self.notify(MessageKind::Error, error.to_string());
            }
        }
    }

    fn profile_raw_editor(&mut self, ui: &mut Ui, index: usize) {
        let apply_clicked = {
            let file = &mut self.config_files[index];
            if file.had_comments {
                callout(
                    ui,
                    theme::AMBER,
                    "Raw YAML編集ではコメントを保持できます。ビジュアル編集の前にも元ファイルを自動バックアップします。",
                );
            }
            let changed = ui
                .add(
                    TextEdit::multiline(&mut file.raw_yaml)
                        .font(FontId::new(14.0, FontFamily::Monospace))
                        .desired_rows(30)
                        .desired_width(f32::INFINITY),
                )
                .changed();
            if changed {
                file.dirty = true;
            }
            ui.button("YAMLを検証して適用").clicked()
        };
        if apply_clicked {
            match self.config_files[index].apply_raw_yaml() {
                Ok(()) => self.notify(MessageKind::Success, "YAMLは有効です"),
                Err(error) => self.notify(MessageKind::Error, error.to_string()),
            }
        }
    }

    fn globals_view(&mut self, ui: &mut Ui) {
        egui::CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(theme::PAPER)
                    .inner_margin(Margin::same(24)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.heading("グローバル変数");
                        ui.label("同じファイルと子ファイルのスニペットから利用できます。");
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("＋ 変数を追加").clicked() {
                            self.variable_editor =
                                Some(VariableEditor::new(VariableScope::Global, "echo"));
                        }
                    });
                });
                ui.separator();
                let Some(file) = self.selected_file() else {
                    centered_empty_state(
                        ui,
                        "ファイルがありません",
                        "設定フォルダを選択してください。",
                    );
                    return;
                };
                let variables = file.document.global_vars.clone();
                let mut remove = None;
                for (index, variable) in variables.iter().enumerate() {
                    Frame::group(ui.style()).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(RichText::new(&variable.name).strong().size(17.0));
                                ui.label(
                                    RichText::new(format!(
                                        "{}  {}",
                                        variable.kind,
                                        variable.token()
                                    ))
                                    .family(FontFamily::Monospace)
                                    .color(theme::ACCENT),
                                );
                            });
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui.small_button("削除").clicked() {
                                    remove = Some(index);
                                }
                                if ui.small_button("編集").clicked() {
                                    self.variable_editor = Some(VariableEditor {
                                        scope: VariableScope::Global,
                                        index: Some(index),
                                        variable: variable.clone(),
                                        insert_in_content: false,
                                    });
                                }
                            });
                        });
                        variable_summary(ui, variable);
                    });
                    ui.add_space(8.0);
                }
                if let Some(index) = remove
                    && let Some(file) = self.selected_file_mut()
                    && !file.is_package
                {
                    file.document.global_vars.remove(index);
                    self.mark_selected_file_changed();
                }
            });
    }

    fn diagnostics_view(&mut self, ui: &mut Ui) {
        egui::CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(theme::PAPER)
                    .inner_margin(Margin::same(24)),
            )
            .show(ui, |ui| {
                ui.heading("設定診断");
                ui.label("保存前にトリガー、変数参照、Espansoの基本構造を確認します。");
                ui.separator();
                let Some(file) = self.selected_file() else {
                    return;
                };
                let diagnostics = file.document.diagnostics();
                if diagnostics.is_empty() {
                    callout(
                        ui,
                        theme::ACCENT,
                        "問題は見つかりませんでした。保存できます。",
                    );
                    return;
                }
                for diagnostic in diagnostics {
                    let color = match diagnostic.level {
                        DiagnosticLevel::Error => theme::DANGER,
                        DiagnosticLevel::Warning => theme::AMBER,
                    };
                    Frame::group(ui.style()).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(match diagnostic.level {
                                    DiagnosticLevel::Error => "エラー",
                                    DiagnosticLevel::Warning => "警告",
                                })
                                .color(color)
                                .strong(),
                            );
                            ui.label(&diagnostic.message);
                            if let Some(index) = diagnostic.snippet_index {
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if ui.small_button("開く").clicked() {
                                        self.selected_snippet = index;
                                        self.section = Section::Library;
                                    }
                                });
                            }
                        });
                    });
                    ui.add_space(6.0);
                }
            });
    }

    fn settings_view(&mut self, ui: &mut Ui) {
        egui::CentralPanel::default()
            .frame(Frame::new().fill(theme::PAPER).inner_margin(Margin::same(24)))
            .show(ui, |ui| {
                ui.heading("設定とバックアップ");
                ui.separator();
                ui.heading("Espanso設定フォルダ");
                ui.horizontal(|ui| {
                    ui.add(
                        TextEdit::singleline(
                            &mut self.preferences.config_root.to_string_lossy().into_owned(),
                        )
                        .desired_width(580.0)
                        .interactive(false),
                    );
                    if ui.button("変更").clicked() {
                        self.choose_config_root();
                    }
                    if ui.button("フォルダを開く").clicked()
                        && let Err(error) = open::that(&self.preferences.config_root)
                    {
                        self.notify(MessageKind::Error, error.to_string());
                    }
                });
                ui.add_space(14.0);
                ui.heading("Espansoサービス");
                ui.label(format!(
                    "インストール: {}  /  バージョン: {}  /  状態: {}",
                    if self.status.installed { "検出済み" } else { "未検出" },
                    self.status.version.as_deref().unwrap_or("—"),
                    self.status.service.as_deref().unwrap_or("—")
                ));
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(self.status.installed, Button::new("開始"))
                        .clicked()
                    {
                        self.run_espanso_action(EspansoAction::Start);
                    }
                    if ui
                        .add_enabled(self.status.installed, Button::new("停止"))
                        .clicked()
                    {
                        self.run_espanso_action(EspansoAction::Stop);
                    }
                    if ui
                        .add_enabled(self.status.installed, Button::new("再起動"))
                        .clicked()
                    {
                        self.run_espanso_action(EspansoAction::Restart);
                    }
                    if ui.button("状態を更新").clicked() {
                        self.status = espanso::detect();
                    }
                });
                ui.add_space(18.0);
                ui.heading("バックアップとデータ移行");
                ui.horizontal_wrapped(|ui| {
                    if ui.button("設定全体をバックアップ").clicked()
                        && let Some(destination) = rfd::FileDialog::new()
                            .set_title("バックアップ先を選択")
                            .pick_folder()
                    {
                        match storage::create_backup_snapshot(
                            &self.preferences.config_root,
                            &destination,
                        ) {
                            Ok(path) => self.notify(
                                MessageKind::Success,
                                format!("バックアップを作成しました: {}", path.display()),
                            ),
                            Err(error) => self.notify(MessageKind::Error, error.to_string()),
                        }
                    }
                    if ui.button("選択ファイルをCSVへ書き出す").clicked()
                        && let Some(file) = self.selected_file()
                        && let Some(destination) = rfd::FileDialog::new()
                            .set_file_name(format!("{}.csv", file.display_name))
                            .add_filter("CSV", &["csv"])
                            .save_file()
                    {
                        match storage::export_csv(file, &destination) {
                            Ok(()) => self.notify(MessageKind::Success, "CSVを書き出しました"),
                            Err(error) => self.notify(MessageKind::Error, error.to_string()),
                        }
                    }
                    if ui.button("CSVから読み込む").clicked()
                        && let Some(source) = rfd::FileDialog::new()
                            .add_filter("CSV", &["csv"])
                            .pick_file()
                        && let Some(file) = self.selected_file_mut()
                    {
                        match storage::import_csv(file, &source) {
                            Ok(count) => self.notify(
                                MessageKind::Success,
                                format!("{count}件のスニペットを読み込みました（未保存）"),
                            ),
                            Err(error) => self.notify(MessageKind::Error, error.to_string()),
                        }
                    }
                });
                callout(
                    ui,
                    theme::ACCENT,
                    "通常の保存でも変更前ファイルを .espanso-gui/backups に自動保存します。ファイル削除は .espanso-gui/trash へ退避します。",
                );
                ui.add_space(18.0);
                ui.heading("選択ファイルの保存履歴");
                if let Some(relative) = self
                    .selected_file()
                    .map(|file| file.relative_path.clone())
                {
                    self.history_list(ui, &relative);
                } else {
                    ui.label("履歴を表示するスニペットファイルを選択してください。");
                }
                ui.add_space(18.0);
                ui.heading("ファイル操作");
                if ui.button(RichText::new("選択中のファイルを削除…").color(theme::DANGER)).clicked() {
                    self.pending_delete = Some(PendingDelete::File);
                }
            });
    }

    fn history_list(&mut self, ui: &mut Ui, relative_path: &Path) {
        match storage::list_history(&self.preferences.config_root, relative_path) {
            Ok(entries) if entries.is_empty() => {
                ui.label(RichText::new("まだ履歴はありません").color(theme::MUTED));
            }
            Ok(entries) => {
                let can_restore = !self.has_dirty_files();
                for entry in entries.into_iter().take(10) {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(&entry.timestamp)
                                .family(FontFamily::Monospace)
                                .small(),
                        );
                        if ui
                            .add_enabled(can_restore, Button::new("この版を復元…"))
                            .clicked()
                        {
                            self.pending_restore = Some(PendingRestore {
                                relative_path: relative_path.to_path_buf(),
                                backup_path: entry.backup_path,
                                timestamp: entry.timestamp,
                            });
                        }
                    });
                }
                if !can_restore {
                    ui.label(
                        RichText::new("未保存の変更があるため、履歴復元は一時的に無効です。")
                            .small()
                            .color(theme::AMBER),
                    );
                }
            }
            Err(error) => {
                ui.label(RichText::new(error.to_string()).color(theme::DANGER));
            }
        }
    }

    fn about_view(&mut self, ui: &mut Ui) {
        egui::CentralPanel::default()
            .frame(Frame::new().fill(theme::PAPER).inner_margin(Margin::same(32)))
            .show(ui, |ui| {
                ui.heading("Espanso GUI");
                ui.label(RichText::new("A polished visual editor for Espanso — written in Rust.").size(18.0));
                ui.add_space(16.0);
                ui.label("EspansoのYAML設定を、スニペット・変数・フォーム・リッチテキスト単位で安全に編集する独立アプリです。");
                ui.add_space(12.0);
                callout(
                    ui,
                    theme::AMBER,
                    "非公式プロジェクトです。EspansoおよびEspanso開発者による承認・提携・サポートはありません。本アプリのIssueは本アプリのリポジトリだけで扱います。",
                );
                ui.add_space(12.0);
                ui.label("ライセンス: MIT");
                ui.label(format!("バージョン: {}", env!("CARGO_PKG_VERSION")));
                ui.label("実装言語: Rust / GUI: eframe + egui");
                ui.add_space(16.0);
                if ui.link("Espanso公式ドキュメントを開く").clicked() {
                    let _ = open::that("https://espanso.org/docs/");
                }
            });
    }

    fn empty_workspace(&mut self, ui: &mut Ui) {
        egui::CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(theme::PAPER)
                    .inner_margin(Margin::same(34)),
            )
            .show(ui, |ui| {
                centered_empty_state(
                    ui,
                    "Espanso設定を接続しましょう",
                    "設定フォルダを自動検出できない場合は手動で選択できます。",
                );
                ui.add_space(12.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new(self.preferences.config_root.display().to_string())
                            .family(FontFamily::Monospace)
                            .color(theme::MUTED),
                    );
                    ui.horizontal(|ui| {
                        if ui.button("設定フォルダを選択").clicked() {
                            self.choose_config_root();
                        }
                        if ui.button("この場所を初期化").clicked() {
                            self.initialize_config();
                        }
                    });
                    if let Some(error) = &self.load_error {
                        ui.label(RichText::new(error).color(theme::DANGER));
                    }
                });
            });
    }

    fn modal_windows(&mut self, ui: &mut Ui) {
        self.new_file_window(ui);
        self.new_config_window(ui);
        self.variable_window(ui);
        self.form_field_window(ui);
        self.conflict_window(ui);
        self.restore_confirmation(ui);
        self.delete_confirmation(ui);
        self.close_confirmation(ui);
    }

    fn new_file_window(&mut self, ui: &mut Ui) {
        if !self.new_file_dialog {
            return;
        }
        let mut open = true;
        let mut create = false;
        egui::Window::new("スニペットファイルを追加")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ui.ctx(), |ui| {
                ui.label("ファイル名");
                ui.add(TextEdit::singleline(&mut self.new_file_name).hint_text("work"));
                ui.label(
                    RichText::new("match/<名前>.yml として作成します")
                        .small()
                        .color(theme::MUTED),
                );
                ui.horizontal(|ui| {
                    if ui.button("キャンセル").clicked() {
                        self.new_file_dialog = false;
                    }
                    if ui.add(Button::new("作成").fill(theme::ACCENT)).clicked() {
                        create = true;
                    }
                });
            });
        if !open {
            self.new_file_dialog = false;
        }
        if create {
            self.create_file();
        }
    }

    fn new_config_window(&mut self, ui: &mut Ui) {
        if !self.new_config_dialog {
            return;
        }
        let mut open = true;
        let mut create = false;
        egui::Window::new("設定プロファイルを追加")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ui.ctx(), |ui| {
                ui.label("ファイル名");
                ui.add(TextEdit::singleline(&mut self.new_config_name).hint_text("telegram"));
                ui.label(
                    RichText::new("config/<名前>.yml として作成します。default は全体設定です。")
                        .small()
                        .color(theme::MUTED),
                );
                ui.horizontal(|ui| {
                    if ui.button("キャンセル").clicked() {
                        self.new_config_dialog = false;
                    }
                    if ui.add(Button::new("作成").fill(theme::ACCENT)).clicked() {
                        create = true;
                    }
                });
            });
        if !open {
            self.new_config_dialog = false;
        }
        if create {
            self.create_config_file();
        }
    }

    fn conflict_window(&mut self, ui: &mut Ui) {
        let Some(mut dialog) = self.conflict_dialog.take() else {
            return;
        };
        let mut open = true;
        let mut apply = false;
        let mut cancel = false;
        egui::Window::new("外部変更をthree-way merge")
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_size([760.0, 560.0])
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ui.ctx(), |ui| {
                callout(
                    ui,
                    theme::AMBER,
                    "読み込み時点（base）、編集中（local）、現在のdiskを比較しました。保存前にdisk最新版も自動バックアップします。",
                );
                ui.add_space(8.0);
                if dialog.conflict.plan.conflicts.is_empty() {
                    callout(
                        ui,
                        theme::ACCENT,
                        "同じfieldを双方が変更した箇所はありません。独立した変更を自動mergeできます。",
                    );
                } else {
                    ui.label(format!(
                        "{} fieldで双方の変更が重なっています。各fieldの採用値を選択してください。",
                        dialog.conflict.plan.conflicts.len()
                    ));
                    ui.separator();
                    ScrollArea::vertical()
                        .id_salt("conflict-fields")
                        .max_height(390.0)
                        .show(ui, |ui| {
                            for (index, conflict) in
                                dialog.conflict.plan.conflicts.iter().enumerate()
                            {
                                Frame::group(ui.style()).show(ui, |ui| {
                                    ui.label(
                                        RichText::new(&conflict.label)
                                            .family(FontFamily::Monospace)
                                            .strong(),
                                    );
                                    ui.label(
                                        RichText::new(format!(
                                            "base: {}",
                                            conflict.base_summary()
                                        ))
                                        .small()
                                        .color(theme::MUTED),
                                    );
                                    ui.horizontal(|ui| {
                                        ui.selectable_value(
                                            &mut dialog.choices[index],
                                            ResolutionChoice::Local,
                                            "localを採用",
                                        );
                                        ui.code(conflict.local_summary());
                                    });
                                    ui.horizontal(|ui| {
                                        ui.selectable_value(
                                            &mut dialog.choices[index],
                                            ResolutionChoice::Disk,
                                            "diskを採用",
                                        );
                                        ui.code(conflict.disk_summary());
                                    });
                                });
                                ui.add_space(8.0);
                            }
                        });
                }
                ui.separator();
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .add(Button::new("mergeして保存").fill(theme::ACCENT))
                        .clicked()
                    {
                        apply = true;
                    }
                    if ui.button("キャンセル").clicked() {
                        cancel = true;
                    }
                });
            });

        if apply {
            let root = self.preferences.config_root.clone();
            let result = match dialog.target {
                ConflictTarget::Match(index) => self.files.get_mut(index).map_or_else(
                    || {
                        Err(storage::StorageError::Message(
                            "対象ファイルがありません".into(),
                        ))
                    },
                    |file| {
                        storage::resolve_workspace_conflict(
                            &root,
                            file,
                            &dialog.conflict,
                            &dialog.choices,
                        )
                    },
                ),
                ConflictTarget::Config(index) => self.config_files.get_mut(index).map_or_else(
                    || {
                        Err(storage::StorageError::Message(
                            "対象ファイルがありません".into(),
                        ))
                    },
                    |file| {
                        storage::resolve_config_conflict(
                            &root,
                            file,
                            &dialog.conflict,
                            &dialog.choices,
                        )
                    },
                ),
            };
            match result {
                Ok(receipt) => self.notify(
                    MessageKind::Success,
                    format!("three-way mergeを保存しました / {}", &receipt.hash[..8]),
                ),
                Err(error) => self.notify(MessageKind::Error, error.to_string()),
            }
        } else if open && !cancel {
            self.conflict_dialog = Some(dialog);
        }
    }

    fn restore_confirmation(&mut self, ui: &mut Ui) {
        let Some(pending) = self.pending_restore.clone() else {
            return;
        };
        let mut open = true;
        let mut restore = false;
        let mut cancel = false;
        egui::Window::new("保存履歴を復元")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ui.ctx(), |ui| {
                ui.label(format!(
                    "{} を {} の内容へ戻します。",
                    pending.relative_path.display(),
                    pending.timestamp
                ));
                callout(
                    ui,
                    theme::AMBER,
                    "現在のdisk版も先に新しい履歴としてbackupするため、復元操作自体を取り消せます。",
                );
                ui.horizontal(|ui| {
                    if ui.button("キャンセル").clicked() {
                        cancel = true;
                    }
                    if ui
                        .add(Button::new("backupして復元").fill(theme::ACCENT))
                        .clicked()
                    {
                        restore = true;
                    }
                });
            });
        if restore {
            match storage::restore_history(
                &self.preferences.config_root,
                &pending.relative_path,
                &pending.backup_path,
            ) {
                Ok(_) => {
                    self.pending_restore = None;
                    self.reload_workspace();
                    self.notify(MessageKind::Success, "保存履歴から復元しました");
                }
                Err(error) => {
                    self.pending_restore = None;
                    self.notify(MessageKind::Error, error.to_string());
                }
            }
        } else if cancel || !open {
            self.pending_restore = None;
        }
    }

    fn variable_window(&mut self, ui: &mut Ui) {
        let Some(mut editor) = self.variable_editor.take() else {
            return;
        };
        let mut open = true;
        let mut save = false;
        let mut cancel = false;
        egui::Window::new(if editor.index.is_some() {
            "変数を編集"
        } else {
            "変数を追加"
        })
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_width(560.0)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("変数名").strong());
                    ui.add(
                        TextEdit::singleline(&mut editor.variable.name).hint_text("my_variable"),
                    );
                });
                ui.vertical(|ui| {
                    ui.label(RichText::new("種類").strong());
                    let old_kind = editor.variable.kind.clone();
                    ComboBox::from_id_salt("variable-kind")
                        .selected_text(variable_kind_label(&editor.variable.kind))
                        .show_ui(ui, |ui| {
                            for kind in [
                                "date",
                                "clipboard",
                                "choice",
                                "random",
                                "echo",
                                "shell",
                                "script",
                                "form",
                                "global",
                            ] {
                                ui.selectable_value(
                                    &mut editor.variable.kind,
                                    kind.into(),
                                    variable_kind_label(kind),
                                );
                            }
                        });
                    if editor.variable.kind != old_kind {
                        let name = editor.variable.name.clone();
                        editor.variable = Variable::new(&editor.variable.kind);
                        editor.variable.name = name;
                    }
                });
            });
            ui.separator();
            variable_parameters(ui, &mut editor.variable);
            ui.separator();
            let mut dependencies = editor.variable.depends_on.join(", ");
            two_column_field(
                ui,
                "依存変数",
                "評価順を固定する場合だけ指定",
                |ui| {
                    if ui
                        .add(TextEdit::singleline(&mut dependencies).hint_text("first, second"))
                        .changed()
                    {
                        editor.variable.depends_on = dependencies
                            .split(',')
                            .map(str::trim)
                            .filter(|value| !value.is_empty())
                            .map(str::to_string)
                            .collect();
                    }
                },
            );
            if editor.scope == VariableScope::Local {
                ui.checkbox(
                    &mut editor.insert_in_content,
                    "保存時に本文へ {{変数名}} を挿入",
                );
            }
            ui.horizontal(|ui| {
                if ui.button("キャンセル").clicked() {
                    cancel = true;
                }
                if ui
                    .add(Button::new("変数を保存").fill(theme::ACCENT))
                    .clicked()
                {
                    save = true;
                }
            });
        });
        if save {
            if editor.variable.name.is_empty()
                || !editor
                    .variable
                    .name
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || character == '_')
            {
                self.notify(
                    MessageKind::Error,
                    "変数名には英数字とアンダースコアだけを使用してください",
                );
                self.variable_editor = Some(editor);
                return;
            }
            self.apply_variable_editor(editor);
        } else if open && !cancel {
            self.variable_editor = Some(editor);
        }
    }

    fn apply_variable_editor(&mut self, editor: VariableEditor) {
        let selected_snippet = self.selected_snippet;
        let Some(file) = self.selected_file_mut() else {
            return;
        };
        if file.is_package {
            return;
        }
        match editor.scope {
            VariableScope::Global => {
                if let Some(index) = editor.index {
                    if index < file.document.global_vars.len() {
                        file.document.global_vars[index] = editor.variable;
                    }
                } else {
                    file.document.global_vars.push(editor.variable);
                }
            }
            VariableScope::Local => {
                let Some(snippet) = file.document.matches.get_mut(selected_snippet) else {
                    return;
                };
                let token = editor.variable.token();
                if let Some(index) = editor.index {
                    if index < snippet.vars.len() {
                        snippet.vars[index] = editor.variable;
                    }
                } else {
                    snippet.vars.push(editor.variable);
                }
                if editor.insert_in_content {
                    snippet.insert_token(&token);
                }
            }
        }
        self.mark_selected_file_changed();
        self.notify(
            MessageKind::Success,
            "変数を保存しました（ファイルは未保存）",
        );
    }

    fn form_field_window(&mut self, ui: &mut Ui) {
        let Some(mut editor) = self.form_field_editor.take() else {
            return;
        };
        let mut open = true;
        let mut save = false;
        let mut cancel = false;
        egui::Window::new("フォーム項目")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(500.0)
            .show(ui.ctx(), |ui| {
                ui.label("項目名（[[name]] のname部分）");
                ui.add(TextEdit::singleline(&mut editor.name));
                let mut kind = form_field_kind(&editor.field);
                ui.label("入力タイプ");
                ComboBox::from_id_salt("form-field-kind")
                    .selected_text(&kind)
                    .show_ui(ui, |ui| {
                        for candidate in ["text", "multiline", "choice", "list"] {
                            ui.selectable_value(&mut kind, candidate.into(), candidate);
                        }
                    });
                set_form_field_kind(&mut editor.field, &kind);
                ui.label("初期値");
                let mut default = editor.field.default.clone().unwrap_or_default();
                if ui.add(TextEdit::singleline(&mut default)).changed() {
                    editor.field.default = (!default.is_empty()).then_some(default);
                }
                if matches!(kind.as_str(), "choice" | "list") {
                    ui.label("選択肢（1行に1つ）");
                    let mut values = editor.field.values.join("\n");
                    if ui
                        .add(TextEdit::multiline(&mut values).desired_rows(6))
                        .changed()
                    {
                        editor.field.values = values.lines().map(str::to_string).collect();
                    }
                }
                ui.horizontal(|ui| {
                    if ui.button("キャンセル").clicked() {
                        cancel = true;
                    }
                    if ui
                        .add(Button::new("項目を保存").fill(theme::ACCENT))
                        .clicked()
                    {
                        save = true;
                    }
                });
            });
        if save {
            let name = editor.name.trim().to_string();
            if name.is_empty()
                || !name
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || character == '_')
            {
                self.notify(
                    MessageKind::Error,
                    "項目名には英数字とアンダースコアだけを使用してください",
                );
                self.form_field_editor = Some(editor);
                return;
            }
            let selected_snippet = self.selected_snippet;
            if let Some(file) = self.selected_file_mut()
                && let Some(snippet) = file.document.matches.get_mut(selected_snippet)
            {
                if let Some(original) = editor.original_name
                    && original != name
                {
                    snippet.form_fields.shift_remove(&original);
                    if let Some(form) = &mut snippet.form {
                        *form = form.replace(&format!("[[{original}]]"), &format!("[[{name}]]"));
                    }
                }
                snippet.form_fields.insert(name.clone(), editor.field);
                if let Some(form) = &mut snippet.form
                    && !form.contains(&format!("[[{name}]]"))
                {
                    if !form.ends_with('\n') && !form.is_empty() {
                        form.push('\n');
                    }
                    form.push_str(&format!("{name}: [[{name}]]"));
                }
                self.mark_selected_file_changed();
            }
        } else if open && !cancel {
            self.form_field_editor = Some(editor);
        }
    }

    fn delete_confirmation(&mut self, ui: &mut Ui) {
        let Some(pending) = self.pending_delete else {
            return;
        };
        let mut keep_open = true;
        egui::Window::new("削除の確認")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ui.ctx(), |ui| {
                ui.label(match pending {
                    PendingDelete::Snippet => {
                        "選択したスニペットを削除しますか？保存前なら再読み込みで戻せます。"
                    }
                    PendingDelete::File => {
                        "選択したファイルを削除しますか？ファイルは復元用フォルダへ移動されます。"
                    }
                });
                ui.horizontal(|ui| {
                    if ui.button("キャンセル").clicked() {
                        keep_open = false;
                    }
                    if ui
                        .add(Button::new("削除する").fill(theme::DANGER))
                        .clicked()
                    {
                        match pending {
                            PendingDelete::Snippet => self.delete_selected_snippet(),
                            PendingDelete::File => self.delete_selected_file(),
                        }
                        keep_open = false;
                    }
                });
            });
        if !keep_open {
            self.pending_delete = None;
        }
    }

    fn close_confirmation(&mut self, ui: &mut Ui) {
        if !self.confirm_close {
            return;
        }
        egui::Window::new("未保存の変更があります")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ui.ctx(), |ui| {
                ui.label("保存していない変更を破棄して終了しますか？");
                ui.horizontal(|ui| {
                    if ui.button("編集に戻る").clicked() {
                        self.confirm_close = false;
                    }
                    if ui
                        .add(Button::new("破棄して終了").fill(theme::DANGER))
                        .clicked()
                    {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        self.confirm_close = false;
                        for file in &mut self.files {
                            file.dirty = false;
                        }
                    }
                });
            });
    }
}

impl eframe::App for EspansoGuiApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.keyboard_shortcuts(ui);
        let close_requested = ui.ctx().input(|input| input.viewport().close_requested());
        if close_requested && self.has_dirty_files() && !self.confirm_close {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.confirm_close = true;
        }

        egui::CentralPanel::default()
            .frame(Frame::new().fill(theme::PAPER))
            .show(ui, |ui| {
                self.top_bar(ui);
                self.navigation(ui);
                if let Some(message) = &self.message {
                    message_bar(ui, message);
                }
                match self.section {
                    Section::Library => self.library_view(ui),
                    Section::Profiles => self.profiles_view(ui),
                    Section::Globals => self.globals_view(ui),
                    Section::Diagnostics => self.diagnostics_view(ui),
                    Section::Settings => self.settings_view(ui),
                    Section::About => self.about_view(ui),
                }
                self.modal_windows(ui);
            });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, APP_STORAGE_KEY, &self.preferences);
    }
}

fn status_badge(ui: &mut Ui, status: &EspansoStatus) {
    let (text, color) = if status.installed {
        ("Espanso 接続済み", theme::ACCENT)
    } else {
        ("Espanso 未検出", theme::AMBER)
    };
    Frame::new()
        .fill(color.gamma_multiply(0.12))
        .corner_radius(10)
        .inner_margin(Margin::symmetric(9, 4))
        .show(ui, |ui| {
            ui.label(
                RichText::new(format!("●  {text}"))
                    .small()
                    .color(color)
                    .strong(),
            );
        });
}

fn nav_button(ui: &mut Ui, current: &mut Section, value: Section, label: &str, shortcut: &str) {
    let selected = *current == value;
    let response = ui.add_sized(
        [190.0, 38.0],
        Button::new(format!("{label}                         {shortcut}"))
            .selected(selected)
            .frame(selected),
    );
    if response.clicked() {
        *current = value;
    }
}

fn tab_button(ui: &mut Ui, current: &mut EditorTab, value: EditorTab, label: &str) {
    if ui.selectable_label(*current == value, label).clicked() {
        *current = value;
    }
}

fn two_column_field(ui: &mut Ui, label: &str, description: &str, content: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.set_min_height(50.0);
        ui.vertical(|ui| {
            ui.set_width(190.0);
            ui.label(RichText::new(label).strong());
            ui.label(RichText::new(description).small().color(theme::MUTED));
        });
        ui.vertical(content);
    });
}

fn optional_text_field(ui: &mut Ui, value: &mut Option<String>, label: &str, description: &str) {
    two_column_field(ui, label, description, |ui| {
        let mut overridden = value.is_some();
        ui.horizontal(|ui| {
            if ui.checkbox(&mut overridden, "上書き").changed() {
                if overridden {
                    *value = Some(String::new());
                } else {
                    *value = None;
                }
            }
            ui.add_enabled_ui(overridden, |ui| {
                if let Some(value) = value {
                    ui.add(TextEdit::singleline(value).desired_width(320.0));
                }
            });
        });
    });
}

fn optional_number_field(ui: &mut Ui, value: &mut Option<u64>, label: &str, description: &str) {
    two_column_field(ui, label, description, |ui| {
        let mut overridden = value.is_some();
        ui.horizontal(|ui| {
            if ui.checkbox(&mut overridden, "上書き").changed() {
                if overridden {
                    *value = Some(0);
                } else {
                    *value = None;
                }
            }
            ui.add_enabled_ui(overridden, |ui| {
                if let Some(value) = value {
                    ui.add(egui::DragValue::new(value).range(0..=60_000));
                }
            });
        });
    });
}

fn optional_bool_field(ui: &mut Ui, value: &mut Option<bool>, label: &str, description: &str) {
    two_column_field(ui, label, description, |ui| {
        ComboBox::from_id_salt(("optional-bool", label))
            .selected_text(match value {
                None => "継承",
                Some(true) => "有効",
                Some(false) => "無効",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(value, None, "継承");
                ui.selectable_value(value, Some(true), "有効");
                ui.selectable_value(value, Some(false), "無効");
            });
    });
}

fn option_checkbox(ui: &mut Ui, value: &mut Option<bool>, label: &str, description: &str) {
    let mut enabled = value.unwrap_or(false);
    two_column_field(ui, label, description, |ui| {
        if ui.checkbox(&mut enabled, "有効").changed() {
            *value = enabled.then_some(true);
        }
    });
}

fn editor_toolbar(ui: &mut Ui, snippet: &mut Snippet) {
    ui.horizontal_wrapped(|ui| {
        if matches!(snippet.content_kind(), ContentKind::Markdown) {
            if ui.small_button("太字").clicked() {
                snippet.insert_token("**太字**");
            }
            if ui.small_button("斜体").clicked() {
                snippet.insert_token("*斜体*");
            }
            if ui.small_button("リンク").clicked() {
                snippet.insert_token("[リンク](https://example.com)");
            }
            if ui.small_button("コード").clicked() {
                snippet.insert_token("`code`");
            }
            if ui.small_button("箇条書き").clicked() {
                snippet.insert_token("\n- 項目1\n- 項目2");
            }
        } else if matches!(snippet.content_kind(), ContentKind::Html) {
            if ui.small_button("太字").clicked() {
                snippet.insert_token("<strong>太字</strong>");
            }
            if ui.small_button("リンク").clicked() {
                snippet.insert_token("<a href=\"https://example.com\">リンク</a>");
            }
            if ui.small_button("画像").clicked() {
                snippet.insert_token("<img src=\"$CONFIG/assets/image.png\" alt=\"\">");
            }
        }
        if !matches!(snippet.content_kind(), ContentKind::Image)
            && ui.small_button("カーソル位置").clicked()
        {
            snippet.insert_token("$|$");
        }
        for variable in &snippet.vars.clone() {
            if ui.small_button(variable.token()).clicked() {
                snippet.insert_token(&variable.token());
            }
        }
    });
}

fn variable_parameters(ui: &mut Ui, variable: &mut Variable) {
    match variable.kind.as_str() {
        "date" => {
            let mut format = variable.param_str("format");
            two_column_field(ui, "表示形式", "strftime形式", |ui| {
                ComboBox::from_id_salt("date-format-presets")
                    .selected_text(if format.is_empty() {
                        "形式を選択"
                    } else {
                        &format
                    })
                    .show_ui(ui, |ui| {
                        for (value, label) in [
                            ("%Y-%m-%d", "2026-08-15"),
                            ("%Y年%m月%d日", "2026年08月15日"),
                            ("%Y/%m/%d", "2026/08/15"),
                            ("%H:%M", "14:30"),
                            ("%Y-%m-%d %H:%M", "日時"),
                        ] {
                            ui.selectable_value(
                                &mut format,
                                value.into(),
                                format!("{label}  ({value})"),
                            );
                        }
                    });
                ui.add(TextEdit::singleline(&mut format).hint_text("%Y-%m-%d"));
            });
            variable.set_param("format", format);
            let mut offset = variable.param_i64("offset");
            two_column_field(ui, "日時の移動", "秒単位。明日は86400", |ui| {
                ui.horizontal(|ui| {
                    if ui.small_button("昨日").clicked() {
                        offset = -86_400;
                    }
                    if ui.small_button("今日").clicked() {
                        offset = 0;
                    }
                    if ui.small_button("明日").clicked() {
                        offset = 86_400;
                    }
                    if ui.small_button("1週間後").clicked() {
                        offset = 604_800;
                    }
                });
                ui.add(egui::DragValue::new(&mut offset).speed(60).suffix(" 秒"));
            });
            variable.set_i64("offset", offset, true);
            let mut locale = variable.param_str("locale");
            two_column_field(ui, "ロケール", "BCP 47。空欄ならOS設定", |ui| {
                ui.add(TextEdit::singleline(&mut locale).hint_text("ja-JP"));
            });
            variable.set_param_optional("locale", &locale);
            let mut timezone = variable.param_str("tz");
            two_column_field(
                ui,
                "タイムゾーン",
                "IANA名。空欄ならローカル",
                |ui| {
                    ui.add(TextEdit::singleline(&mut timezone).hint_text("Asia/Tokyo"));
                },
            );
            variable.set_param_optional("tz", &timezone);
        }
        "clipboard" => {
            callout(
                ui,
                theme::ACCENT,
                "展開時点のクリップボード内容を挿入します。追加設定はありません。",
            );
        }
        "echo" => {
            let mut value = variable.param_str("echo");
            two_column_field(
                ui,
                "固定値",
                "複数のスニペットで再利用する値",
                |ui| {
                    ui.add(TextEdit::multiline(&mut value).desired_rows(4));
                },
            );
            variable.set_param("echo", value);
        }
        "random" => {
            let mut values = variable.param_strings("choices").join("\n");
            two_column_field(
                ui,
                "候補",
                "1行に1つ。ランダムに1件を選択",
                |ui| {
                    ui.add(TextEdit::multiline(&mut values).desired_rows(7));
                },
            );
            variable.set_string_list(
                "choices",
                &values.lines().map(str::to_string).collect::<Vec<_>>(),
            );
        }
        "choice" => {
            let mut values = variable.param_strings("values").join("\n");
            two_column_field(
                ui,
                "選択肢",
                "1行に1つ。展開時に選択画面を表示",
                |ui| {
                    ui.add(TextEdit::multiline(&mut values).desired_rows(7));
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
                theme::AMBER,
                "このコマンドはEspansoのトリガー実行時にローカル環境で実行されます。",
            );
            let mut command = variable.param_str("cmd");
            two_column_field(
                ui,
                "コマンド",
                "短時間で終了する処理を推奨",
                |ui| {
                    ui.add(
                        TextEdit::multiline(&mut command)
                            .font(FontId::monospace(14.0))
                            .desired_rows(5),
                    );
                },
            );
            variable.set_param("cmd", command);
            let mut shell = variable.param_str("shell");
            two_column_field(ui, "シェル", "空欄ならOS既定", |ui| {
                ComboBox::from_id_salt("shell-kind")
                    .selected_text(if shell.is_empty() { "OS既定" } else { &shell })
                    .show_ui(ui, |ui| {
                        for value in ["", "sh", "bash", "powershell", "pwsh", "cmd", "wsl", "nu"] {
                            ui.selectable_value(
                                &mut shell,
                                value.into(),
                                if value.is_empty() { "OS既定" } else { value },
                            );
                        }
                    });
            });
            variable.set_param_optional("shell", &shell);
            let mut trim = variable.param_bool("trim", true);
            ui.checkbox(&mut trim, "出力前後の空白と改行を除去");
            variable.set_bool("trim", trim, true);
            let mut debug = variable.param_bool("debug", false);
            ui.checkbox(&mut debug, "Espansoログへデバッグ情報を出力");
            variable.set_bool("debug", debug, false);
        }
        "script" => {
            callout(
                ui,
                theme::AMBER,
                "1行目に実行コマンド、2行目以降に引数を入力します。変数はESPANSO_<名前>環境変数でも参照できます。",
            );
            let mut args = variable.param_strings("args").join("\n");
            two_column_field(ui, "コマンドと引数", "1行に1要素", |ui| {
                ui.add(
                    TextEdit::multiline(&mut args)
                        .font(FontId::monospace(14.0))
                        .desired_rows(7),
                );
            });
            variable.set_string_list(
                "args",
                &args.lines().map(str::to_string).collect::<Vec<_>>(),
            );
            let mut trim = variable.param_bool("trim", true);
            ui.checkbox(&mut trim, "出力前後の空白と改行を除去");
            variable.set_bool("trim", trim, true);
        }
        "form" => {
            let mut layout = variable.param_str("layout");
            two_column_field(
                ui,
                "フォーム配置",
                "[[field]] で入力欄を配置",
                |ui| {
                    ui.add(TextEdit::multiline(&mut layout).desired_rows(8));
                },
            );
            variable.set_param("layout", layout);
            let mut fields = variable.form_fields();
            ui.horizontal(|ui| {
                ui.label(RichText::new("フォーム項目").strong());
                if ui.small_button("＋ 項目").clicked() {
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
                    ui.horizontal(|ui| {
                        ui.add(TextEdit::singleline(&mut next_name).desired_width(130.0));
                        let mut kind = form_field_kind(&field);
                        ComboBox::from_id_salt(("variable-form-field", &name))
                            .selected_text(&kind)
                            .show_ui(ui, |ui| {
                                for candidate in ["text", "multiline", "choice", "list"] {
                                    ui.selectable_value(&mut kind, candidate.into(), candidate);
                                }
                            });
                        set_form_field_kind(&mut field, &kind);
                        if ui.small_button("削除").clicked() {
                            remove = Some(name.clone());
                        }
                    });
                    let mut default = field.default.clone().unwrap_or_default();
                    if ui
                        .add(TextEdit::singleline(&mut default).hint_text("初期値"))
                        .changed()
                    {
                        field.default = (!default.is_empty()).then_some(default);
                    }
                    if matches!(form_field_kind(&field).as_str(), "choice" | "list") {
                        let mut values = field.values.join("\n");
                        if ui
                            .add(
                                TextEdit::multiline(&mut values)
                                    .desired_rows(3)
                                    .hint_text("選択肢を1行に1つ"),
                            )
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
                theme::ACCENT,
                "同名のグローバル変数をローカル評価順へ明示的に含めます。追加パラメータはありません。",
            );
        }
        _ => {
            callout(
                ui,
                theme::AMBER,
                "未知の変数タイプです。既存パラメータはRaw YAMLで保持されます。",
            );
        }
    }
}

fn variable_summary(ui: &mut Ui, variable: &Variable) {
    let summary = match variable.kind.as_str() {
        "date" => format!(
            "形式 {} / offset {}秒",
            variable.param_str("format"),
            variable.param_i64("offset")
        ),
        "clipboard" => "現在のクリップボード".into(),
        "echo" => truncate(&variable.param_str("echo"), 80),
        "random" => format!(
            "{}件からランダム選択",
            variable.param_strings("choices").len()
        ),
        "choice" => format!("{}件の選択肢", variable.param_strings("values").len()),
        "shell" => truncate(&variable.param_str("cmd"), 80),
        "script" => variable.param_strings("args").join(" "),
        "form" => truncate(&variable.param_str("layout"), 80),
        _ => "高度な変数".into(),
    };
    ui.label(RichText::new(summary).small().color(theme::MUTED));
}

fn variable_kind_label(kind: &str) -> &'static str {
    match kind {
        "date" => "日付・時刻",
        "clipboard" => "クリップボード",
        "choice" => "候補選択",
        "random" => "ランダム",
        "echo" => "固定値",
        "shell" => "シェルコマンド",
        "script" => "スクリプト",
        "form" => "フォーム",
        "global" => "グローバル参照",
        _ => "カスタム",
    }
}

fn form_field_kind(field: &FormField) -> String {
    match field.r#type.as_deref() {
        Some("choice") => "choice".into(),
        Some("list") => "list".into(),
        _ if field.multiline == Some(true) => "multiline".into(),
        _ => "text".into(),
    }
}

fn field_kind_label(field: &FormField) -> &'static str {
    match form_field_kind(field).as_str() {
        "choice" => "選択ボタン",
        "list" => "リスト",
        "multiline" => "複数行テキスト",
        _ => "テキスト",
    }
}

fn set_form_field_kind(field: &mut FormField, kind: &str) {
    match kind {
        "choice" | "list" => {
            field.r#type = Some(kind.into());
            field.multiline = None;
        }
        "multiline" => {
            field.r#type = None;
            field.multiline = Some(true);
            field.values.clear();
        }
        _ => {
            field.r#type = None;
            field.multiline = None;
            field.values.clear();
        }
    }
}

fn callout(ui: &mut Ui, color: Color32, text: &str) {
    Frame::new()
        .fill(color.gamma_multiply(0.10))
        .stroke(Stroke::new(1.0, color.gamma_multiply(0.45)))
        .corner_radius(8)
        .inner_margin(Margin::same(10))
        .show(ui, |ui| {
            ui.label(RichText::new(text).color(theme::INK));
        });
}

fn centered_empty_state(ui: &mut Ui, title: &str, description: &str) {
    ui.add_space(40.0);
    ui.vertical_centered(|ui| {
        ui.label(RichText::new(title).size(22.0).strong());
        ui.label(RichText::new(description).color(theme::MUTED));
    });
}

fn message_bar(ui: &mut Ui, message: &Message) {
    let color = match message.kind {
        MessageKind::Success => theme::ACCENT,
        MessageKind::Info => Color32::from_rgb(66, 103, 146),
        MessageKind::Error => theme::DANGER,
    };
    egui::Panel::bottom("message-bar")
        .frame(
            Frame::new()
                .fill(color.gamma_multiply(0.12))
                .inner_margin(Margin::symmetric(18, 8)),
        )
        .show(ui, |ui| {
            ui.label(RichText::new(&message.text).color(color).strong());
        });
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
    use super::*;

    #[test]
    fn truncates_by_characters_not_bytes() {
        assert_eq!(truncate("日本語テキスト", 3), "日本語…");
        assert_eq!(truncate("short", 10), "short");
    }

    #[test]
    fn form_field_kinds_map_to_espanso_options() {
        let mut field = FormField::default();
        set_form_field_kind(&mut field, "multiline");
        assert_eq!(field.multiline, Some(true));
        set_form_field_kind(&mut field, "choice");
        assert_eq!(field.r#type.as_deref(), Some("choice"));
        assert_eq!(field.multiline, None);
    }

    #[test]
    fn variable_labels_are_friendly() {
        assert_eq!(variable_kind_label("date"), "日付・時刻");
        assert_eq!(variable_kind_label("custom"), "カスタム");
    }
}
