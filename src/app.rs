use crate::conflict::ResolutionChoice;
use crate::espanso::{self, EspansoAction, EspansoStatus};
use crate::html_editor;
use crate::i18n::{self, Language, TextKey};
use crate::model::{ContentKind, DiagnosticLevel, FormField, FormFieldKind, Snippet, Variable};
use crate::navigation::{self, NavigationAction, Section};
use crate::preferences::Preferences;
use crate::profile_editor;
use crate::settings_editor;
use crate::snippet_editor::{editor_toolbar, trigger_mode_selector};
use crate::snippet_library::{self, SnippetSort};
use crate::storage::{self, ConfigFile, ExternalConflict, WorkspaceFile};
use crate::theme;
use crate::top_bar::{self, TopBarAction};
use crate::ui_components::{
    callout, centered_content_panel, centered_empty_state, centered_empty_state_action,
    compact_collection_width, compact_layout, context_button_enabled, context_row_button,
    context_selectable_value, danger_button, display_heading, labelled_two_column_field,
    live_message_bar, modal_actions, multiline_text_edit, primary_button,
    responsive_detail_actions, section_heading, selection_list_row, set_responsive_modal_size,
    set_responsive_modal_width, show_modal, singleline_text_edit, snippet_card,
    unambiguous_selectable_value, wrapped_path_label,
};
use crate::variable_editor::{variable_parameters, variable_summary};
use crate::yaml_editor;
use eframe::egui::{
    self, Align, Button, ComboBox, FontFamily, FontId, Frame, Key, Layout, Margin, RichText,
    ScrollArea, Stroke, Ui,
};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use std::path::{Path, PathBuf};

const APP_STORAGE_KEY: &str = "espanso-gui.preferences";
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditorTab {
    Content,
    Variables,
    Options,
    RawYaml,
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
    fn new(scope: VariableScope, kind: &str, language: Language) -> Self {
        Self {
            scope,
            index: None,
            variable: localized_new_variable(language, kind),
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
    load_error: Option<storage::StorageError>,
    new_file_dialog: bool,
    new_file_name: String,
    new_config_dialog: bool,
    new_config_name: String,
    profile_raw_yaml: bool,
    html_source_mode: bool,
    focus_search: bool,
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
                ..Preferences::default()
            });
        if preferences.config_root.as_os_str().is_empty() {
            preferences.config_root.clone_from(&status.config_root);
        }
        preferences.ui_scale = preferences
            .ui_scale
            .clamp(theme::UI_SCALE_MIN, theme::UI_SCALE_MAX);
        theme::apply_appearance(&creation_context.egui_ctx, preferences.appearance);
        creation_context
            .egui_ctx
            .set_zoom_factor(preferences.ui_scale);
        let (files, config_files, load_error) = if preferences.config_root.join("match").is_dir() {
            let mut load_error = None;
            let files = storage::load_workspace(&preferences.config_root).unwrap_or_else(|error| {
                load_error = Some(error);
                Vec::new()
            });
            let config_files = storage::load_config_profiles(&preferences.config_root)
                .unwrap_or_else(|error| {
                    if load_error.is_none() {
                        load_error = Some(error);
                    }
                    Vec::new()
                });
            (files, config_files, load_error)
        } else {
            (Vec::new(), Vec::new(), None)
        };

        Self::from_loaded(preferences, status, files, config_files, load_error)
    }

    fn from_loaded(
        preferences: Preferences,
        status: EspansoStatus,
        files: Vec<WorkspaceFile>,
        config_files: Vec<ConfigFile>,
        load_error: Option<storage::StorageError>,
    ) -> Self {
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
            html_source_mode: false,
            focus_search: false,
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

    fn notify_storage_error(&mut self, error: storage::StorageError) {
        self.notify(
            MessageKind::Error,
            i18n::storage_error_text(self.preferences.language, &error),
        );
    }

    fn reload_workspace(&mut self) {
        let language = self.preferences.language;
        if self.has_dirty_files() {
            self.notify(
                MessageKind::Error,
                i18n::text(language, TextKey::SaveBeforeReload),
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
                self.notify(
                    MessageKind::Success,
                    i18n::text(language, TextKey::WorkspaceReloaded),
                );
            }
            (Err(error), _) | (_, Err(error)) => self.notify_storage_error(error),
        }
    }

    fn save_selected(&mut self) {
        let language = self.preferences.language;
        let root = self.preferences.config_root.clone();
        let index = self.selected_file;
        let Some(file) = self.files.get(index) else {
            return;
        };
        if file.is_package {
            self.notify(
                MessageKind::Error,
                i18n::text(language, TextKey::PackageCannotSave),
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
                    i18n::text(language, TextKey::ExternalChangeDetected),
                );
                return;
            }
            Ok(None) => {}
            Err(error) => {
                self.notify_storage_error(error);
                return;
            }
        }
        let file = &mut self.files[index];
        match storage::save_workspace_file(&root, file) {
            Ok(receipt) => {
                let backup_path = receipt.backup_path.map(|path| path.display().to_string());
                self.notify(
                    MessageKind::Success,
                    i18n::workspace_saved_text(
                        language,
                        backup_path.as_deref(),
                        &receipt.hash[..8],
                    ),
                );
            }
            Err(error) => self.notify_storage_error(error),
        }
    }

    fn save_selected_config(&mut self) {
        let language = self.preferences.language;
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
                    i18n::text(language, TextKey::ExternalChangeDetected),
                );
                return;
            }
            Ok(None) => {}
            Err(error) => {
                self.notify_storage_error(error);
                return;
            }
        }
        let file = &mut self.config_files[index];
        match storage::save_config_file(&root, file) {
            Ok(receipt) => {
                let backup_path = receipt.backup_path.map(|path| path.display().to_string());
                self.notify(
                    MessageKind::Success,
                    i18n::profile_saved_text(language, backup_path.as_deref(), &receipt.hash[..8]),
                );
            }
            Err(error) => self.notify_storage_error(error),
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
            self.notify_storage_error(error);
        }
    }

    fn add_snippet(&mut self) {
        let language = self.preferences.language;
        let index = if let Some(file) = self.selected_file_mut() {
            if file.is_package {
                self.notify(
                    MessageKind::Error,
                    i18n::text(language, TextKey::PackageCannotAdd),
                );
                return;
            }
            file.document.matches.push(localized_new_snippet(language));
            file.document.matches.len() - 1
        } else {
            return;
        };
        self.selected_snippet = index;
        self.editor_tab = EditorTab::Content;
        self.mark_selected_file_changed();
    }

    fn duplicate_snippet(&mut self) {
        let language = self.preferences.language;
        let selected = self.selected_snippet;
        let index = if let Some(file) = self.selected_file_mut() {
            if file.is_package {
                self.notify(
                    MessageKind::Info,
                    i18n::text(language, TextKey::CopyPackageToUserFile),
                );
                return;
            }
            let Some(mut snippet) = file.document.matches.get(selected).cloned() else {
                return;
            };
            if let Some(label) = &mut snippet.label {
                label.push_str(i18n::text(language, TextKey::CopySuffix));
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
        let language = self.preferences.language;
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
                i18n::text(language, TextKey::MissingCopyTarget),
            );
            return;
        };
        self.files[target].document.matches.push(snippet);
        self.selected_file = target;
        self.selected_snippet = self.files[target].document.matches.len() - 1;
        self.mark_selected_file_changed();
        self.notify(
            MessageKind::Success,
            i18n::text(language, TextKey::CopiedToUserFile),
        );
    }

    fn delete_selected_snippet(&mut self) {
        let language = self.preferences.language;
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
                i18n::text(language, TextKey::SnippetDeleted),
            );
        }
    }

    fn create_file(&mut self) {
        let language = self.preferences.language;
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
                self.notify(
                    MessageKind::Success,
                    i18n::text(language, TextKey::MatchFileCreated),
                );
            }
            Err(error) => self.notify_storage_error(error),
        }
    }

    fn create_config_file(&mut self) {
        let language = self.preferences.language;
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
                self.notify(
                    MessageKind::Success,
                    i18n::text(language, TextKey::ProfileCreated),
                );
            }
            Err(error) => self.notify_storage_error(error),
        }
    }

    fn delete_selected_file(&mut self) {
        let language = self.preferences.language;
        let Some(file) = self.selected_file() else {
            return;
        };
        if file.is_package {
            self.notify(
                MessageKind::Error,
                i18n::text(language, TextKey::PackageCannotDelete),
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
                    i18n::file_moved_text(language, &destination.display().to_string()),
                );
            }
            Err(error) => self.notify_storage_error(error),
        }
    }

    fn initialize_config(&mut self) {
        let language = self.preferences.language;
        match storage::initialize_root(&self.preferences.config_root) {
            Ok(()) => {
                self.reload_workspace();
                self.notify(
                    MessageKind::Success,
                    i18n::text(language, TextKey::ConfigInitialized),
                );
            }
            Err(error) => self.notify_storage_error(error),
        }
    }

    fn choose_config_root(&mut self) {
        let language = self.preferences.language;
        if self.has_dirty_files() {
            self.notify(
                MessageKind::Error,
                i18n::text(language, TextKey::SaveBeforeChangingFolder),
            );
            return;
        }
        if let Some(path) = rfd::FileDialog::new()
            .set_title(i18n::text(language, TextKey::ChooseEspansoFolder))
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
                        i18n::espanso_action_completed(self.preferences.language, action).into()
                    } else {
                        result.output
                    },
                );
                self.status = espanso::detect();
            }
            Ok(result) => self.notify(
                MessageKind::Error,
                if result.output.is_empty() {
                    i18n::text(self.preferences.language, TextKey::EspansoCommandFailed).into()
                } else {
                    result.output
                },
            ),
            Err(error) => self.notify(
                MessageKind::Error,
                i18n::espanso_command_error(self.preferences.language, &error.to_string()),
            ),
        }
    }

    fn open_espanso_documentation(&mut self) {
        if let Err(error) = open::that("https://espanso.org/docs/") {
            self.notify(
                MessageKind::Error,
                i18n::open_failed_text(self.preferences.language, &error.to_string()),
            );
        }
    }

    fn has_open_modal(&self) -> bool {
        self.new_file_dialog
            || self.new_config_dialog
            || self.variable_editor.is_some()
            || self.form_field_editor.is_some()
            || self.pending_delete.is_some()
            || self.conflict_dialog.is_some()
            || self.pending_restore.is_some()
            || self.confirm_close
    }

    fn keyboard_shortcuts(&mut self, ui: &Ui) {
        // Modal dialogs own keyboard input while they are visible. In particular, this prevents
        // global shortcuts from mutating the obscured editor and lets the top modal consume Escape.
        if self.has_open_modal() {
            return;
        }
        let (save, new, search, destination) = ui.input(|input| {
            let command = input.modifiers.command;
            let destination = if command && input.key_pressed(Key::Num1) {
                Some(Section::Library)
            } else if command && input.key_pressed(Key::Num2) {
                Some(Section::Profiles)
            } else if command && input.key_pressed(Key::Num3) {
                Some(Section::Globals)
            } else if command && input.key_pressed(Key::Num4) {
                Some(Section::Diagnostics)
            } else if command && input.key_pressed(Key::Num5) {
                Some(Section::Settings)
            } else {
                None
            };
            (
                command && input.key_pressed(Key::S),
                command && input.key_pressed(Key::N),
                command && input.key_pressed(Key::F),
                destination,
            )
        });
        if save {
            self.save_current();
        }
        if new && self.section == Section::Library {
            self.add_snippet();
        }
        if search {
            self.section = Section::Library;
            self.focus_search = true;
        }
        if let Some(destination) = destination {
            self.section = destination;
        }
    }

    fn top_bar(&mut self, ui: &mut Ui) {
        let can_save = if self.section == Section::Profiles {
            !self.config_files.is_empty()
        } else {
            self.selected_file().is_some_and(|file| !file.is_package)
        };
        let action = top_bar::show(
            ui,
            &self.status,
            self.preferences.language,
            can_save,
            self.has_dirty_files(),
        );
        match action {
            Some(TopBarAction::Save) => self.save_current(),
            Some(TopBarAction::Reload) => self.reload_workspace(),
            Some(TopBarAction::RestartEspanso) => {
                self.run_espanso_action(EspansoAction::Restart);
            }
            None => {}
        }
    }

    fn navigation(&mut self, ui: &mut Ui) {
        match navigation::show(
            ui,
            &mut self.section,
            &self.files,
            self.selected_file,
            self.preferences.language,
        ) {
            Some(NavigationAction::SelectFile(index)) => {
                self.selected_file = index;
                self.selected_snippet = 0;
            }
            Some(NavigationAction::AddFile) => self.new_file_dialog = true,
            Some(NavigationAction::Reload) => self.reload_workspace(),
            None => {}
        }
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
                    .fill(theme::palette(ui).paper)
                    .inner_margin(Margin::same(theme::PADDING_XL)),
            )
            .show(ui, |ui| self.snippet_editor(ui));
    }

    fn snippet_list(&mut self, ui: &mut Ui) {
        let compact = compact_layout(ui.ctx().content_rect().width());
        egui::Panel::left("snippet-list")
            .exact_size(if compact {
                compact_collection_width(
                    ui.ctx().content_rect().width(),
                    theme::SNIPPET_LIST_COMPACT_WIDTH,
                )
            } else {
                theme::SNIPPET_LIST_WIDTH
            })
            // egui's resize handle is exposed to AT-SPI as an unnamed focusable `unknown`
            // control. Fixed responsive widths avoid a dead keyboard/screen-reader stop.
            .resizable(false)
            .frame(
                Frame::new()
                    .fill(theme::palette(ui).panel)
                    .inner_margin(Margin::same(theme::PADDING_LG))
                    .stroke(Stroke::new(
                        theme::STROKE_STANDARD,
                        theme::palette(ui).border_subtle,
                    )),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let language = self.preferences.language;
                    section_heading(ui, i18n::text(language, TextKey::Snippets));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .button(i18n::text(language, TextKey::NewSnippet))
                            .clicked()
                        {
                            self.add_snippet();
                        }
                    });
                });
                let search_label = ui.label(i18n::text(self.preferences.language, TextKey::Search));
                let search_response = ui
                    .add(
                        singleline_text_edit(&mut self.search)
                            .id_salt("snippet-search")
                            .hint_text(i18n::text(self.preferences.language, TextKey::SearchHint))
                            .desired_width(f32::INFINITY),
                    )
                    .labelled_by(search_label.id);
                if self.focus_search {
                    search_response.request_focus();
                    self.focus_search = false;
                }
                ui.add_space(theme::SPACE_XS);

                ui.horizontal(|ui| {
                    let language = self.preferences.language;
                    let sort_label = ui.label(i18n::text(language, TextKey::SortBy));
                    let response = ComboBox::from_id_salt("snippet-sort")
                        .selected_text(i18n::text(
                            language,
                            self.preferences.snippet_sort.text_key(),
                        ))
                        .show_ui(ui, |ui| {
                            for sort in SnippetSort::ALL {
                                ui.selectable_value(
                                    &mut self.preferences.snippet_sort,
                                    sort,
                                    i18n::text(language, sort.text_key()),
                                );
                            }
                        });
                    response.response.labelled_by(sort_label.id);
                });
                ui.add_space(theme::SPACE_XS);

                let tags = snippet_library::search_terms(&self.files);
                if !tags.is_empty() {
                    let normalized_search = self.search.trim().to_lowercase();
                    let mut selected_tag = tags
                        .iter()
                        .find(|(tag, _)| tag.to_lowercase() == normalized_search)
                        .map(|(tag, _)| tag.clone());
                    let previous_tag = selected_tag.clone();
                    ui.horizontal(|ui| {
                        let language = self.preferences.language;
                        let tag_label = ui.label(i18n::text(language, TextKey::FilterByTag));
                        let response = ComboBox::from_id_salt("snippet-tag-filter")
                            .selected_text(
                                selected_tag
                                    .as_deref()
                                    .unwrap_or_else(|| i18n::text(language, TextKey::AllTags)),
                            )
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut selected_tag,
                                    None,
                                    i18n::text(language, TextKey::AllTags),
                                );
                                for (tag, count) in &tags {
                                    ui.selectable_value(
                                        &mut selected_tag,
                                        Some(tag.clone()),
                                        format!("{tag} ({count})"),
                                    );
                                }
                            });
                        response.response.labelled_by(tag_label.id);
                    });
                    if selected_tag != previous_tag {
                        self.search = selected_tag.unwrap_or_default();
                        self.focus_search = true;
                    }
                    ui.add_space(theme::SPACE_XS);
                }

                let searching = !self.search.trim().is_empty();
                let snippets = snippet_library::entries(
                    &self.files,
                    self.selected_file,
                    &self.search,
                    self.preferences.language,
                    self.preferences.snippet_sort,
                );
                if searching {
                    let result_count =
                        i18n::search_result_count(self.preferences.language, snippets.len());
                    let response = ui.label(
                        RichText::new(result_count.clone())
                            .small()
                            .color(theme::palette(ui).muted),
                    );
                    ui.ctx().accesskit_node_builder(response.id, move |node| {
                        node.set_label(result_count);
                        node.set_live(egui::accesskit::Live::Polite);
                    });
                }
                let mut clear_search = false;
                ScrollArea::vertical()
                    .id_salt("snippets")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if snippets.is_empty() && searching {
                            ui.add_space(theme::SPACE_LG);
                            ui.vertical_centered(|ui| {
                                section_heading(
                                    ui,
                                    i18n::text(self.preferences.language, TextKey::NoSearchResults),
                                );
                                ui.label(
                                    RichText::new(i18n::text(
                                        self.preferences.language,
                                        TextKey::NoSearchResultsDescription,
                                    ))
                                    .small()
                                    .color(theme::palette(ui).muted),
                                );
                                ui.add_space(theme::SPACE_MD);
                                clear_search = ui
                                    .button(i18n::text(
                                        self.preferences.language,
                                        TextKey::ClearSearch,
                                    ))
                                    .clicked();
                            });
                        }
                        for entry in snippets {
                            let selected = self.selected_file == entry.file_index
                                && self.selected_snippet == entry.snippet_index;
                            let response = snippet_card(
                                ui,
                                selected,
                                &entry.title,
                                &entry.triggers,
                                &entry.preview,
                                &entry.context,
                            );
                            if response.clicked() {
                                self.selected_file = entry.file_index;
                                self.selected_snippet = entry.snippet_index;
                                self.editor_tab = EditorTab::Content;
                            }
                            ui.add_space(theme::SPACE_SM);
                        }
                    });
                if clear_search {
                    self.search.clear();
                    self.focus_search = true;
                }

                ui.separator();
                ui.horizontal(|ui| {
                    let language = self.preferences.language;
                    if ui
                        .button(i18n::text(language, TextKey::Duplicate))
                        .clicked()
                    {
                        self.duplicate_snippet();
                    }
                    if ui
                        .button(
                            RichText::new(i18n::text(language, TextKey::Delete))
                                .color(theme::palette(ui).danger),
                        )
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
            let language = self.preferences.language;
            if centered_empty_state_action(
                ui,
                i18n::text(language, TextKey::NoSnippetsTitle),
                i18n::text(language, TextKey::NoSnippetsDescription),
                i18n::text(language, TextKey::CreateFirstSnippet),
            ) {
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

        ui.horizontal_wrapped(|ui| {
            let language = self.preferences.language;
            display_heading(ui, &localized_snippet_title(language, &snippet));
            if is_package {
                ui.label(
                    RichText::new(i18n::text(language, TextKey::ReadOnlyPackage))
                        .color(theme::palette(ui).amber),
                );
            }
        });
        ui.add_space(theme::SPACE_SM);
        ui.horizontal_wrapped(|ui| {
            let language = self.preferences.language;
            tab_button(
                ui,
                &mut self.editor_tab,
                EditorTab::Content,
                i18n::text(language, TextKey::Content),
            );
            tab_button(
                ui,
                &mut self.editor_tab,
                EditorTab::Variables,
                i18n::text(language, TextKey::Variables),
            );
            tab_button(
                ui,
                &mut self.editor_tab,
                EditorTab::Options,
                i18n::text(language, TextKey::AdvancedOptions),
            );
            tab_button(
                ui,
                &mut self.editor_tab,
                EditorTab::RawYaml,
                i18n::text(language, TextKey::RawYaml),
            );
        });
        ui.separator();

        if is_package {
            let language = self.preferences.language;
            ui.add_space(theme::SPACE_SM);
            callout(
                ui,
                theme::palette(ui).amber,
                i18n::text(language, TextKey::PackageEditorWarning),
            );
            if ui
                .button(i18n::text(language, TextKey::CopyThisSnippet))
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
                self.notify_storage_error(error);
            }
        }
    }

    fn content_editor(&mut self, ui: &mut Ui, snippet: &mut Snippet) {
        let language = self.preferences.language;
        labelled_two_column_field(
            ui,
            i18n::text(language, TextKey::DisplayName),
            i18n::text(language, TextKey::DisplayNameDescription),
            |ui, label_id| {
                let mut label = snippet.label.clone().unwrap_or_default();
                if ui
                    .add(
                        singleline_text_edit(&mut label)
                            .hint_text(i18n::text(language, TextKey::DisplayNameHint)),
                    )
                    .labelled_by(label_id)
                    .changed()
                {
                    snippet.label = (!label.trim().is_empty()).then_some(label);
                }
            },
        );
        ui.add_space(theme::SPACE_MD);

        let regex_mode = snippet.regex.is_some();
        labelled_two_column_field(
            ui,
            i18n::text(language, TextKey::Trigger),
            i18n::text(
                language,
                if regex_mode {
                    TextKey::RegexTriggerDescription
                } else {
                    TextKey::TriggerDescription
                },
            ),
            |ui, label_id| {
                let (regex_mode, _) = trigger_mode_selector(ui, language, snippet);
                if regex_mode {
                    let mut regex = snippet.regex.clone().unwrap_or_default();
                    if ui
                        .add(
                            singleline_text_edit(&mut regex)
                                .hint_text(i18n::text(language, TextKey::RegexTriggerHint)),
                        )
                        .labelled_by(label_id)
                        .changed()
                    {
                        snippet.regex = Some(regex);
                        snippet.trigger = None;
                        snippet.triggers.clear();
                    }
                } else {
                    let mut triggers = snippet.trigger_list().join(", ");
                    if ui
                        .add(singleline_text_edit(&mut triggers).hint_text(":sig, :signature"))
                        .labelled_by(label_id)
                        .changed()
                    {
                        snippet.set_trigger_list(triggers.split(',').map(str::to_string).collect());
                    }
                }
            },
        );
        ui.add_space(theme::SPACE_LG);

        labelled_two_column_field(
            ui,
            i18n::text(language, TextKey::ExpansionType),
            "",
            |ui, label_id| {
                let mut kind = snippet.content_kind();
                let kind_response = ComboBox::from_id_salt("content-kind")
                    .selected_text(i18n::content_kind_label(language, kind))
                    .show_ui(ui, |ui| {
                        for candidate in ContentKind::ALL {
                            ui.selectable_value(
                                &mut kind,
                                candidate,
                                i18n::content_kind_label(language, candidate),
                            );
                        }
                    });
                kind_response.response.labelled_by(label_id);
                if kind != snippet.content_kind() {
                    snippet.set_content_kind(kind);
                }
            },
        );

        ui.add_space(theme::SPACE_SM);
        if snippet.content_kind() == ContentKind::Html {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(i18n::text(language, TextKey::HtmlEditing)).strong());
                unambiguous_selectable_value(
                    ui,
                    &mut self.html_source_mode,
                    false,
                    i18n::text(language, TextKey::Composer),
                );
                unambiguous_selectable_value(
                    ui,
                    &mut self.html_source_mode,
                    true,
                    i18n::text(language, TextKey::Source),
                );
                ui.label(
                    RichText::new(i18n::text(language, TextKey::SafeHtmlNotice))
                        .small()
                        .color(theme::palette(ui).muted),
                );
            });
        }
        editor_toolbar(ui, language, snippet);
        ui.add_space(theme::SPACE_SM);
        match snippet.content_kind() {
            ContentKind::Image => self.image_editor(ui, snippet),
            ContentKind::Form => self.form_editor(ui, snippet),
            kind => {
                let content_label =
                    ui.label(RichText::new(i18n::text(language, TextKey::Content)).strong());
                ui.add(
                    multiline_text_edit(snippet.content_mut())
                        .font(FontId::new(
                            theme::TEXT_BODY,
                            if kind == ContentKind::Html && !self.html_source_mode {
                                FontFamily::Proportional
                            } else {
                                FontFamily::Monospace
                            },
                        ))
                        .desired_rows(14)
                        .desired_width(f32::INFINITY)
                        .hint_text(match kind {
                            ContentKind::Html => i18n::text(language, TextKey::HtmlContentHint),
                            ContentKind::Markdown => {
                                i18n::text(language, TextKey::MarkdownContentHint)
                            }
                            _ => i18n::text(language, TextKey::PlainContentHint),
                        }),
                )
                .labelled_by(content_label.id);
                self.content_preview(ui, snippet);
            }
        }
    }

    fn image_editor(&mut self, ui: &mut Ui, snippet: &mut Snippet) {
        let language = self.preferences.language;
        labelled_two_column_field(
            ui,
            i18n::text(language, TextKey::ImagePath),
            i18n::text(language, TextKey::ImagePathHint),
            |ui, label_id| {
                ui.add(
                    singleline_text_edit(snippet.content_mut())
                        .hint_text(i18n::text(language, TextKey::ImagePathHint))
                        .desired_width(f32::INFINITY),
                )
                .labelled_by(label_id);
                if ui
                    .button(i18n::text(language, TextKey::ChooseImage))
                    .clicked()
                    && let Some(path) = rfd::FileDialog::new()
                        .add_filter(
                            i18n::text(language, TextKey::Image),
                            &["png", "jpg", "jpeg", "gif", "webp"],
                        )
                        .pick_file()
                {
                    *snippet.content_mut() = path.to_string_lossy().into_owned();
                }
            },
        );
        ui.add_space(theme::SPACE_MD);
        let path = snippet.content();
        if !path.is_empty() && !path.contains("$CONFIG") {
            let file_name = Path::new(path)
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or(path);
            ui.add(
                egui::Image::from_uri(format!("file://{path}"))
                    .alt_text(format!(
                        "{}: {file_name}",
                        i18n::text(language, TextKey::ImagePreview)
                    ))
                    .max_size(egui::vec2(
                        theme::IMAGE_PREVIEW_MAX_SIZE[0],
                        theme::IMAGE_PREVIEW_MAX_SIZE[1],
                    )),
            );
        } else {
            callout(
                ui,
                theme::palette(ui).accent,
                i18n::text(language, TextKey::ImageCompatibility),
            );
        }
    }

    fn form_editor(&mut self, ui: &mut Ui, snippet: &mut Snippet) {
        let language = self.preferences.language;
        callout(
            ui,
            theme::palette(ui).accent,
            i18n::text(language, TextKey::FormEditorDescription),
        );
        let form_content_label = ui.label(i18n::text(language, TextKey::FormContent));
        ui.add(
            multiline_text_edit(snippet.content_mut())
                .font(FontId::new(theme::TEXT_BODY, FontFamily::Monospace))
                .desired_rows(9)
                .desired_width(f32::INFINITY)
                .hint_text(i18n::text(language, TextKey::FormContentHint)),
        )
        .labelled_by(form_content_label.id);
        ui.add_space(theme::SPACE_MD);
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new(i18n::text(language, TextKey::FormFields))
                    .strong()
                    .size(theme::TEXT_SECTION),
            );
            if ui
                .button(i18n::text(language, TextKey::AddFormField))
                .clicked()
            {
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
                responsive_detail_actions(
                    ui,
                    |ui| {
                        ui.label(
                            RichText::new(format!("[[{name}]]"))
                                .family(FontFamily::Monospace)
                                .strong(),
                        );
                        ui.label(i18n::form_field_kind_label(language, &field.kind()));
                        if let Some(default) = &field.default {
                            ui.label(
                                RichText::new(i18n::default_value_text(language, default))
                                    .small()
                                    .color(theme::palette(ui).muted),
                            );
                        }
                    },
                    |ui| {
                        if context_row_button(ui, i18n::text(language, TextKey::Edit), &name)
                            .clicked()
                        {
                            self.form_field_editor = Some(FormFieldEditor {
                                original_name: Some(name.clone()),
                                name: name.clone(),
                                field: field.clone(),
                            });
                        }
                        if context_row_button(ui, i18n::text(language, TextKey::Delete), &name)
                            .clicked()
                        {
                            remove = Some(name.clone());
                        }
                    },
                );
            });
        }
        if let Some(name) = remove {
            snippet.form_fields.shift_remove(&name);
        }
    }

    fn content_preview(&mut self, ui: &mut Ui, snippet: &Snippet) {
        let language = self.preferences.language;
        ui.add_space(theme::SPACE_LG);
        ui.label(
            RichText::new(i18n::text(language, TextKey::LivePreview))
                .strong()
                .size(theme::TEXT_SECTION),
        );
        Frame::new()
            .fill(theme::palette(ui).panel)
            .stroke(Stroke::new(
                theme::STROKE_STANDARD,
                theme::palette(ui).border_subtle,
            ))
            .corner_radius(theme::RADIUS_CARD)
            .inner_margin(Margin::same(theme::PADDING_LG))
            .show(ui, |ui| match snippet.content_kind() {
                ContentKind::Markdown => {
                    CommonMarkViewer::new().show(ui, &mut self.markdown_cache, snippet.content());
                }
                ContentKind::Html => {
                    ui.label(
                        RichText::new(i18n::text(language, TextKey::SafeTextPreview))
                            .small()
                            .color(theme::palette(ui).muted),
                    );
                    let preview = html_editor::safe_preview(language, snippet.content());
                    if preview.trim().is_empty() {
                        ui.label(
                            RichText::new(i18n::text(language, TextKey::NoPreviewContent))
                                .color(theme::palette(ui).muted),
                        );
                    } else {
                        ui.label(preview);
                    }
                    if self.html_source_mode {
                        ui.collapsing(i18n::text(language, TextKey::GeneratedSource), |ui| {
                            ui.code(snippet.content());
                        });
                    }
                }
                _ => {
                    ui.label(snippet.content());
                }
            });
    }

    fn variables_editor(&mut self, ui: &mut Ui, snippet: &mut Snippet) {
        let language = self.preferences.language;
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(i18n::text(language, TextKey::QuickAdd)).strong());
            for kind in [
                "date",
                "clipboard",
                "choice",
                "form",
                "random",
                "echo",
                "shell",
                "script",
            ] {
                if ui
                    .button(format!("＋ {}", i18n::variable_kind_label(language, kind)))
                    .clicked()
                {
                    self.variable_editor =
                        Some(VariableEditor::new(VariableScope::Local, kind, language));
                }
            }
        });
        ui.add_space(theme::SPACE_MD);
        if snippet.vars.is_empty() {
            centered_empty_state(
                ui,
                i18n::text(language, TextKey::NoVariablesTitle),
                i18n::text(language, TextKey::NoVariablesDescription),
            );
        }

        let mut remove = None;
        let variables = snippet.vars.clone();
        for (index, variable) in variables.iter().enumerate() {
            Frame::group(ui.style()).show(ui, |ui| {
                responsive_detail_actions(
                    ui,
                    |ui| {
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new(&variable.name)
                                    .strong()
                                    .size(theme::TEXT_BODY),
                            );
                            ui.label(
                                RichText::new(format!("{}  {}", variable.kind, variable.token()))
                                    .family(FontFamily::Monospace)
                                    .color(theme::palette(ui).accent),
                            );
                        });
                    },
                    |ui| {
                        if context_row_button(
                            ui,
                            i18n::text(language, TextKey::InsertIntoContent),
                            &variable.name,
                        )
                        .clicked()
                        {
                            snippet.insert_token(&variable.token());
                        }
                        if context_row_button(
                            ui,
                            i18n::text(language, TextKey::Edit),
                            &variable.name,
                        )
                        .clicked()
                        {
                            self.variable_editor = Some(VariableEditor {
                                scope: VariableScope::Local,
                                index: Some(index),
                                variable: variable.clone(),
                                insert_in_content: false,
                            });
                        }
                        if context_row_button(
                            ui,
                            i18n::text(language, TextKey::Delete),
                            &variable.name,
                        )
                        .clicked()
                        {
                            remove = Some(index);
                        }
                    },
                );
                variable_summary(ui, language, variable);
            });
            ui.add_space(theme::SPACE_SM);
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
                theme::palette(ui).amber,
                i18n::text(language, TextKey::ScriptVariableWarning),
            );
        }
    }

    fn options_editor(&mut self, ui: &mut Ui, snippet: &mut Snippet) {
        let language = self.preferences.language;
        section_heading(ui, i18n::text(language, TextKey::TriggerConditions));
        option_checkbox(
            ui,
            language,
            &mut snippet.word,
            i18n::text(language, TextKey::WholeWord),
            i18n::text(language, TextKey::WholeWordDescription),
        );
        option_checkbox(
            ui,
            language,
            &mut snippet.left_word,
            i18n::text(language, TextKey::LeftWord),
            i18n::text(language, TextKey::LeftWordDescription),
        );
        option_checkbox(
            ui,
            language,
            &mut snippet.right_word,
            i18n::text(language, TextKey::RightWord),
            i18n::text(language, TextKey::RightWordDescription),
        );
        ui.add_space(theme::SPACE_LG);
        section_heading(ui, i18n::text(language, TextKey::LetterCase));
        option_checkbox(
            ui,
            language,
            &mut snippet.propagate_case,
            i18n::text(language, TextKey::PropagateCase),
            i18n::text(language, TextKey::PropagateCaseDescription),
        );
        labelled_two_column_field(
            ui,
            i18n::text(language, TextKey::UppercaseStyle),
            i18n::text(language, TextKey::UppercaseStyleDescription),
            |ui, label_id| {
                let mut style = snippet.uppercase_style.clone().unwrap_or_default();
                let response = ComboBox::from_id_salt("uppercase-style")
                    .selected_text(if style.is_empty() {
                        i18n::text(language, TextKey::Standard)
                    } else {
                        &style
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut style,
                            String::new(),
                            i18n::text(language, TextKey::Standard),
                        );
                        ui.selectable_value(
                            &mut style,
                            "capitalize_words".into(),
                            i18n::text(language, TextKey::CapitalizeWords),
                        );
                        ui.selectable_value(
                            &mut style,
                            "capitalize".into(),
                            i18n::text(language, TextKey::CapitalizeFirst),
                        );
                    });
                response.response.labelled_by(label_id);
                snippet.uppercase_style = (!style.is_empty()).then_some(style);
            },
        );
        ui.add_space(theme::SPACE_LG);
        section_heading(ui, i18n::text(language, TextKey::ExpansionMethod));
        labelled_two_column_field(
            ui,
            i18n::text(language, TextKey::ForceMode),
            i18n::text(language, TextKey::ForceModeDescription),
            |ui, label_id| {
                let mut mode = snippet.force_mode.clone().unwrap_or_default();
                let response = ComboBox::from_id_salt("force-mode")
                    .selected_text(if mode.is_empty() {
                        i18n::text(language, TextKey::Automatic)
                    } else {
                        &mode
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut mode,
                            String::new(),
                            i18n::text(language, TextKey::Automatic),
                        );
                        ui.selectable_value(
                            &mut mode,
                            "clipboard".into(),
                            i18n::text(language, TextKey::Clipboard),
                        );
                        ui.selectable_value(
                            &mut mode,
                            "keys".into(),
                            i18n::text(language, TextKey::KeyInjection),
                        );
                    });
                response.response.labelled_by(label_id);
                snippet.force_mode = (!mode.is_empty()).then_some(mode);
            },
        );
        if snippet.content_kind() == ContentKind::Markdown {
            option_checkbox(
                ui,
                language,
                &mut snippet.paragraph,
                i18n::text(language, TextKey::NoMarkdownParagraph),
                i18n::text(language, TextKey::NoMarkdownParagraphDescription),
            );
        }
        ui.add_space(theme::SPACE_LG);
        section_heading(ui, i18n::text(language, TextKey::Search));
        let mut terms = snippet.search_terms.join(", ");
        labelled_two_column_field(
            ui,
            i18n::text(language, TextKey::SearchKeywords),
            i18n::text(language, TextKey::CommaSeparated),
            |ui, label_id| {
                if ui
                    .add(
                        singleline_text_edit(&mut terms)
                            .hint_text(i18n::text(language, TextKey::SearchKeywordsHint)),
                    )
                    .labelled_by(label_id)
                    .changed()
                {
                    snippet.search_terms = terms
                        .split(',')
                        .map(str::trim)
                        .filter(|term| !term.is_empty())
                        .map(str::to_string)
                        .collect();
                }
            },
        );
    }

    fn raw_yaml_editor(&mut self, ui: &mut Ui) {
        let language = self.preferences.language;
        let apply_clicked = {
            let Some(file) = self.files.get_mut(self.selected_file) else {
                return;
            };
            if file.had_comments {
                callout(
                    ui,
                    theme::palette(ui).amber,
                    i18n::text(language, TextKey::RawYamlFormattingWarning),
                );
            }
            let raw_yaml_label = ui.label(
                RichText::new(file.relative_path.display().to_string())
                    .family(FontFamily::Monospace),
            );
            let changed = yaml_editor::editor(ui, &mut file.raw_yaml, 28)
                .labelled_by(raw_yaml_label.id)
                .changed();
            if changed {
                file.dirty = true;
            }
            let mut apply_clicked = false;
            ui.horizontal_wrapped(|ui| {
                apply_clicked = ui
                    .button(i18n::text(language, TextKey::ValidateAndApplyYaml))
                    .clicked();
                ui.label(
                    RichText::new(i18n::text(language, TextKey::ValidateAgainOnSave))
                        .small()
                        .color(theme::palette(ui).muted),
                );
            });
            apply_clicked
        };
        if apply_clicked {
            let result = self.files[self.selected_file].apply_raw_yaml();
            match result {
                Ok(()) => self.notify(
                    MessageKind::Success,
                    i18n::text(language, TextKey::YamlValid),
                ),
                Err(error) => self.notify_storage_error(error),
            }
        }
    }

    fn profiles_view(&mut self, ui: &mut Ui) {
        let compact = compact_layout(ui.ctx().content_rect().width());
        egui::Panel::left("profile-list")
            .exact_size(if compact {
                compact_collection_width(
                    ui.ctx().content_rect().width(),
                    theme::PROFILE_LIST_COMPACT_WIDTH,
                )
            } else {
                theme::PROFILE_LIST_WIDTH
            })
            .resizable(false)
            .frame(
                Frame::new()
                    .fill(theme::palette(ui).panel)
                    .inner_margin(Margin::same(theme::PADDING_LG))
                    .stroke(Stroke::new(
                        theme::STROKE_STANDARD,
                        theme::palette(ui).border_subtle,
                    )),
            )
            .show(ui, |ui| {
                let language = self.preferences.language;
                section_heading(ui, i18n::text(language, TextKey::ProfileListTitle));
                ui.label(
                    RichText::new("config/*.yml")
                        .family(FontFamily::Monospace)
                        .small()
                        .color(theme::palette(ui).muted),
                );
                ui.separator();
                ScrollArea::vertical()
                    .id_salt("profile-file-list")
                    .show(ui, |ui| {
                        for (index, file) in self.config_files.iter().enumerate() {
                            let dirty = if file.dirty { " •" } else { "" };
                            let kind = if file.is_default {
                                i18n::text(language, TextKey::DefaultProfile)
                            } else if file.profile.has_filter() {
                                i18n::text(language, TextKey::AppProfile)
                            } else {
                                i18n::text(language, TextKey::MissingFilter)
                            };
                            if selection_list_row(
                                ui,
                                format!("{}{dirty}\n{kind}", file.display_name),
                                self.selected_config == index,
                            )
                            .clicked()
                            {
                                self.selected_config = index;
                            }
                        }
                    });
                ui.add_space(theme::SPACE_SM);
                if ui
                    .add_sized(
                        [ui.available_width(), theme::CONTROL_HEIGHT],
                        Button::new(i18n::text(language, TextKey::AddProfile)),
                    )
                    .clicked()
                {
                    self.new_config_dialog = true;
                }
            });

        egui::CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(theme::palette(ui).paper)
                    .inner_margin(Margin::same(theme::PADDING_XL)),
            )
            .show(ui, |ui| {
                if self.config_files.is_empty() {
                    let language = self.preferences.language;
                    if centered_empty_state_action(
                        ui,
                        i18n::text(language, TextKey::NoProfilesTitle),
                        i18n::text(language, TextKey::NoProfilesDescription),
                        i18n::text(language, TextKey::AddFirstProfile),
                    ) {
                        self.new_config_dialog = true;
                    }
                    return;
                }

                let index = self
                    .selected_config
                    .min(self.config_files.len().saturating_sub(1));
                self.selected_config = index;
                let file = &self.config_files[index];
                responsive_detail_actions(
                    ui,
                    |ui| {
                        ui.vertical(|ui| {
                            display_heading(ui, &file.display_name);
                            ui.label(
                                RichText::new(file.relative_path.display().to_string())
                                    .family(FontFamily::Monospace)
                                    .small()
                                    .color(theme::palette(ui).muted),
                            );
                        });
                    },
                    |ui| {
                        unambiguous_selectable_value(
                            ui,
                            &mut self.profile_raw_yaml,
                            false,
                            i18n::text(self.preferences.language, TextKey::Visual),
                        );
                        unambiguous_selectable_value(
                            ui,
                            &mut self.profile_raw_yaml,
                            true,
                            i18n::text(self.preferences.language, TextKey::RawYaml),
                        );
                    },
                );
                ui.separator();

                if self.profile_raw_yaml {
                    self.profile_raw_editor(ui, index);
                } else {
                    self.profile_visual_editor(ui, index);
                }
            });
    }

    fn profile_visual_editor(&mut self, ui: &mut Ui, index: usize) {
        let language = self.preferences.language;
        let is_default = self.config_files[index].is_default;
        let original = self.config_files[index].profile.clone();
        let mut profile = original.clone();

        profile_editor::visual_editor(ui, language, is_default, &mut profile);

        if profile != original {
            self.config_files[index].profile = profile;
            if let Err(error) = self.config_files[index].refresh_raw_from_profile() {
                self.notify_storage_error(error);
            }
        }
    }

    fn profile_raw_editor(&mut self, ui: &mut Ui, index: usize) {
        let language = self.preferences.language;
        let apply_clicked = {
            let file = &mut self.config_files[index];
            if file.had_comments {
                callout(
                    ui,
                    theme::palette(ui).amber,
                    i18n::text(language, TextKey::ProfileRawYamlNotice),
                );
            }
            let raw_yaml_label = ui.label(i18n::text(language, TextKey::RawYaml));
            let changed = yaml_editor::editor(ui, &mut file.raw_yaml, 30)
                .labelled_by(raw_yaml_label.id)
                .changed();
            if changed {
                file.dirty = true;
            }
            ui.button(i18n::text(language, TextKey::ValidateAndApplyYaml))
                .clicked()
        };
        if apply_clicked {
            match self.config_files[index].apply_raw_yaml() {
                Ok(()) => self.notify(
                    MessageKind::Success,
                    i18n::text(language, TextKey::YamlValid),
                ),
                Err(error) => self.notify_storage_error(error),
            }
        }
    }

    fn globals_view(&mut self, ui: &mut Ui) {
        centered_content_panel(ui, "globals-scroll", |ui| {
            let language = self.preferences.language;
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    display_heading(ui, i18n::text(language, TextKey::Globals));
                    ui.label(i18n::text(language, TextKey::GlobalsDescription));
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .button(i18n::text(language, TextKey::AddVariable))
                        .clicked()
                    {
                        self.variable_editor =
                            Some(VariableEditor::new(VariableScope::Global, "echo", language));
                    }
                });
            });
            ui.separator();
            let Some(file) = self.selected_file() else {
                centered_empty_state(
                    ui,
                    i18n::text(language, TextKey::NoFileTitle),
                    i18n::text(language, TextKey::SelectConfigFolderDescription),
                );
                return;
            };
            let variables = file.document.global_vars.clone();
            let mut remove = None;
            for (index, variable) in variables.iter().enumerate() {
                Frame::group(ui.style()).show(ui, |ui| {
                    responsive_detail_actions(
                        ui,
                        |ui| {
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new(&variable.name)
                                        .strong()
                                        .size(theme::TEXT_SECTION),
                                );
                                ui.label(
                                    RichText::new(format!(
                                        "{}  {}",
                                        variable.kind,
                                        variable.token()
                                    ))
                                    .family(FontFamily::Monospace)
                                    .color(theme::palette(ui).accent),
                                );
                            });
                        },
                        |ui| {
                            if context_row_button(
                                ui,
                                i18n::text(language, TextKey::Edit),
                                &variable.name,
                            )
                            .clicked()
                            {
                                self.variable_editor = Some(VariableEditor {
                                    scope: VariableScope::Global,
                                    index: Some(index),
                                    variable: variable.clone(),
                                    insert_in_content: false,
                                });
                            }
                            if context_row_button(
                                ui,
                                i18n::text(language, TextKey::Delete),
                                &variable.name,
                            )
                            .clicked()
                            {
                                remove = Some(index);
                            }
                        },
                    );
                    variable_summary(ui, language, variable);
                });
                ui.add_space(theme::SPACE_SM);
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
        centered_content_panel(ui, "diagnostics-scroll", |ui| {
            let language = self.preferences.language;
            display_heading(ui, i18n::text(language, TextKey::DiagnosticsTitle));
            ui.label(i18n::text(language, TextKey::DiagnosticsDescription));
            ui.separator();
            let Some(file) = self.selected_file() else {
                return;
            };
            let diagnostics = file.document.diagnostics();
            if diagnostics.is_empty() {
                callout(
                    ui,
                    theme::palette(ui).accent,
                    i18n::text(language, TextKey::NoProblems),
                );
                return;
            }
            for diagnostic in diagnostics {
                let color = match diagnostic.level {
                    DiagnosticLevel::Error => theme::palette(ui).danger,
                    DiagnosticLevel::Warning => theme::palette(ui).amber,
                };
                Frame::group(ui.style()).show(ui, |ui| {
                    let diagnostic_text = i18n::diagnostic_text(language, &diagnostic.kind);
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            RichText::new(match diagnostic.level {
                                DiagnosticLevel::Error => i18n::text(language, TextKey::Error),
                                DiagnosticLevel::Warning => i18n::text(language, TextKey::Warning),
                            })
                            .color(color)
                            .strong(),
                        );
                        ui.label(&diagnostic_text);
                    });
                    if let Some(index) = diagnostic.snippet_index {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if context_row_button(
                                ui,
                                i18n::text(language, TextKey::Open),
                                &diagnostic_text,
                            )
                            .clicked()
                            {
                                self.selected_snippet = index;
                                self.section = Section::Library;
                            }
                        });
                    }
                });
                ui.add_space(theme::SPACE_SM);
            }
        });
    }

    fn settings_view(&mut self, ui: &mut Ui) {
        centered_content_panel(ui, "settings-scroll", |ui| {
            settings_editor::appearance_and_accessibility(ui, &mut self.preferences);
            let language = self.preferences.language;
            if let Some(action) =
                settings_editor::config_folder(ui, language, &self.preferences.config_root)
            {
                self.handle_settings_action(action);
            }
            if let Some(action) = settings_editor::espanso_service(ui, language, &self.status) {
                self.handle_settings_action(action);
            }
            let selected_file = self.selected_file();
            let can_export = selected_file.is_some();
            let can_modify = selected_file.is_some_and(|file| !file.is_package);
            if let Some(action) = settings_editor::backup_and_migration(
                ui,
                language,
                self.preferences.config_root.is_dir(),
                can_export,
                can_modify,
            ) {
                self.handle_settings_action(action);
            }
            ui.add_space(theme::SPACE_LG);
            section_heading(ui, i18n::text(language, TextKey::HistoryTitle));
            if let Some(relative) = self.selected_file().map(|file| file.relative_path.clone()) {
                self.history_list(ui, &relative);
            } else {
                ui.label(i18n::text(language, TextKey::SelectHistoryFile));
            }
            if let Some(action) = settings_editor::delete_file_action(ui, language, can_modify) {
                self.handle_settings_action(action);
            }
        });
    }

    fn handle_settings_action(&mut self, action: settings_editor::SettingsAction) {
        let language = self.preferences.language;
        match action {
            settings_editor::SettingsAction::ChooseConfigRoot => self.choose_config_root(),
            settings_editor::SettingsAction::OpenConfigRoot => {
                if let Err(error) = open::that(&self.preferences.config_root) {
                    self.notify(
                        MessageKind::Error,
                        i18n::open_failed_text(language, &error.to_string()),
                    );
                }
            }
            settings_editor::SettingsAction::Espanso(action) => self.run_espanso_action(action),
            settings_editor::SettingsAction::RefreshStatus => self.status = espanso::detect(),
            settings_editor::SettingsAction::BackupAll => {
                if let Some(destination) = rfd::FileDialog::new()
                    .set_title(i18n::text(language, TextKey::BackupDestination))
                    .pick_folder()
                {
                    match storage::create_backup_snapshot(
                        &self.preferences.config_root,
                        &destination,
                    ) {
                        Ok(path) => self.notify(
                            MessageKind::Success,
                            i18n::backup_created_text(language, &path.display().to_string()),
                        ),
                        Err(error) => self.notify_storage_error(error),
                    }
                }
            }
            settings_editor::SettingsAction::ExportCsv => {
                if let Some(file) = self.selected_file()
                    && let Some(destination) = rfd::FileDialog::new()
                        .set_file_name(format!("{}.csv", file.display_name))
                        .add_filter("CSV", &["csv"])
                        .save_file()
                {
                    match storage::export_csv(file, &destination) {
                        Ok(()) => self.notify(
                            MessageKind::Success,
                            i18n::text(language, TextKey::CsvExported),
                        ),
                        Err(error) => self.notify_storage_error(error),
                    }
                }
            }
            settings_editor::SettingsAction::ImportCsv => {
                if let Some(source) = rfd::FileDialog::new()
                    .add_filter("CSV", &["csv"])
                    .pick_file()
                    && let Some(file) = self.selected_file_mut()
                {
                    match storage::import_csv(file, &source) {
                        Ok(count) => self.notify(
                            MessageKind::Success,
                            i18n::csv_imported_text(language, count),
                        ),
                        Err(error) => self.notify_storage_error(error),
                    }
                }
            }
            settings_editor::SettingsAction::DeleteSelectedFile => {
                self.pending_delete = Some(PendingDelete::File);
            }
        }
    }

    fn history_list(&mut self, ui: &mut Ui, relative_path: &Path) {
        let language = self.preferences.language;
        match storage::list_history(&self.preferences.config_root, relative_path) {
            Ok(entries) if entries.is_empty() => {
                ui.label(
                    RichText::new(i18n::text(language, TextKey::NoHistory))
                        .color(theme::palette(ui).muted),
                );
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
                        if context_button_enabled(
                            ui,
                            can_restore,
                            i18n::text(language, TextKey::RestoreVersion),
                            &entry.timestamp,
                        )
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
                        RichText::new(i18n::text(language, TextKey::HistoryDisabled))
                            .small()
                            .color(theme::palette(ui).amber),
                    );
                }
            }
            Err(error) => {
                ui.label(
                    RichText::new(i18n::storage_error_text(language, &error))
                        .color(theme::palette(ui).danger),
                );
            }
        }
    }

    fn about_view(&mut self, ui: &mut Ui) {
        centered_content_panel(ui, "about-scroll", |ui| {
            let language = self.preferences.language;
            display_heading(ui, "Espanso GUI");
            ui.label(
                RichText::new(i18n::text(language, TextKey::ProductTagline))
                    .size(theme::TEXT_SECTION),
            );
            ui.add_space(theme::SPACE_LG);
            ui.label(i18n::text(language, TextKey::AboutDescription));
            ui.add_space(theme::SPACE_MD);
            callout(
                ui,
                theme::palette(ui).amber,
                i18n::text(language, TextKey::UnofficialNotice),
            );
            ui.add_space(theme::SPACE_MD);
            ui.label(format!("{}: MIT", i18n::text(language, TextKey::License)));
            ui.label(format!(
                "{}: {}",
                i18n::text(language, TextKey::Version),
                env!("CARGO_PKG_VERSION")
            ));
            ui.label(i18n::text(language, TextKey::Implementation));
            ui.add_space(theme::SPACE_LG);
            if ui
                .button(i18n::text(language, TextKey::OpenEspansoDocs))
                .clicked()
            {
                self.open_espanso_documentation();
            }
        });
    }

    fn empty_workspace(&mut self, ui: &mut Ui) {
        centered_content_panel(ui, "empty-workspace-scroll", |ui| {
            let language = self.preferences.language;
            centered_empty_state(
                ui,
                i18n::text(language, TextKey::ConnectTitle),
                i18n::text(language, TextKey::ConnectDescription),
            );
            ui.add_space(theme::SPACE_MD);
            if !self.status.installed {
                callout(
                    ui,
                    theme::palette(ui).amber,
                    i18n::text(language, TextKey::EspansoInstallRequired),
                );
                ui.add_space(theme::SPACE_MD);
                ui.vertical_centered(|ui| {
                    if ui
                        .add(primary_button(
                            ui,
                            i18n::text(language, TextKey::OpenEspansoSetup),
                        ))
                        .clicked()
                    {
                        self.open_espanso_documentation();
                    }
                });
                ui.add_space(theme::SPACE_LG);
            }
            section_heading(ui, i18n::text(language, TextKey::ConfigLocation));
            wrapped_path_label(ui, &self.preferences.config_root);
            ui.label(
                RichText::new(i18n::text(language, TextKey::InitializeHelp))
                    .small()
                    .color(theme::palette(ui).muted),
            );
            ui.add_space(theme::SPACE_MD);
            ui.horizontal_wrapped(|ui| {
                if ui
                    .add(primary_button(
                        ui,
                        i18n::text(language, TextKey::ChooseConfigFolder),
                    ))
                    .clicked()
                {
                    self.choose_config_root();
                }
                if ui
                    .button(i18n::text(language, TextKey::InitializeHere))
                    .clicked()
                {
                    self.initialize_config();
                }
            });
            if let Some(error) = &self.load_error {
                ui.add_space(theme::SPACE_MD);
                callout(
                    ui,
                    theme::palette(ui).danger,
                    &i18n::storage_error_text(language, error),
                );
            }
        });
    }

    fn modal_windows(&mut self, ui: &mut Ui) {
        // Render one modal at a time so focus ownership is deterministic even when, for example,
        // an application-close request arrives while an editor dialog is already open.
        if self.confirm_close {
            self.close_confirmation(ui);
        } else if self.conflict_dialog.is_some() {
            self.conflict_window(ui);
        } else if self.pending_restore.is_some() {
            self.restore_confirmation(ui);
        } else if self.pending_delete.is_some() {
            self.delete_confirmation(ui);
        } else if self.variable_editor.is_some() {
            self.variable_window(ui);
        } else if self.form_field_editor.is_some() {
            self.form_field_window(ui);
        } else if self.new_config_dialog {
            self.new_config_window(ui);
        } else if self.new_file_dialog {
            self.new_file_window(ui);
        }
    }

    fn new_file_window(&mut self, ui: &mut Ui) {
        if !self.new_file_dialog {
            return;
        }
        let mut create = false;
        let language = self.preferences.language;
        let dismissed = show_modal(
            ui,
            "new-file",
            i18n::text(language, TextKey::NewMatchFileTitle),
            |ui| {
                set_responsive_modal_width(ui, theme::MODAL_WIDTH_SM);
                let file_name_label = ui.label(i18n::text(language, TextKey::FileName));
                ui.add(singleline_text_edit(&mut self.new_file_name).hint_text("work"))
                    .labelled_by(file_name_label.id);
                ui.label(
                    RichText::new(i18n::text(language, TextKey::NewMatchFileDescription))
                        .small()
                        .color(theme::palette(ui).muted),
                );
                modal_actions(ui, |ui| {
                    if ui
                        .add(primary_button(ui, i18n::text(language, TextKey::Create)))
                        .clicked()
                    {
                        create = true;
                    }
                    if ui.button(i18n::text(language, TextKey::Cancel)).clicked() {
                        self.new_file_dialog = false;
                    }
                });
            },
        );
        if dismissed {
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
        let mut create = false;
        let language = self.preferences.language;
        let dismissed = show_modal(
            ui,
            "new-config",
            i18n::text(language, TextKey::NewProfileTitle),
            |ui| {
                set_responsive_modal_width(ui, theme::MODAL_WIDTH_MD);
                let file_name_label = ui.label(i18n::text(language, TextKey::FileName));
                ui.add(singleline_text_edit(&mut self.new_config_name).hint_text("telegram"))
                    .labelled_by(file_name_label.id);
                ui.label(
                    RichText::new(i18n::text(language, TextKey::NewProfileDescription))
                        .small()
                        .color(theme::palette(ui).muted),
                );
                modal_actions(ui, |ui| {
                    if ui
                        .add(primary_button(ui, i18n::text(language, TextKey::Create)))
                        .clicked()
                    {
                        create = true;
                    }
                    if ui.button(i18n::text(language, TextKey::Cancel)).clicked() {
                        self.new_config_dialog = false;
                    }
                });
            },
        );
        if dismissed {
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
        let mut apply = false;
        let mut cancel = false;
        let language = self.preferences.language;
        let dismissed = show_modal(
            ui,
            "conflict-resolution",
            i18n::text(language, TextKey::ConflictTitle),
            |ui| {
                set_responsive_modal_size(ui, theme::MODAL_WIDTH_WIDE, theme::MODAL_HEIGHT_TALL);
                callout(
                    ui,
                    theme::palette(ui).amber,
                    i18n::text(language, TextKey::ConflictIntroduction),
                );
                ui.add_space(theme::SPACE_SM);
                if dialog.conflict.plan.conflicts.is_empty() {
                    callout(
                        ui,
                        theme::palette(ui).accent,
                        i18n::text(language, TextKey::NoOverlappingChanges),
                    );
                } else {
                    ui.label(i18n::conflict_count_text(
                        language,
                        dialog.conflict.plan.conflicts.len(),
                    ));
                    ui.separator();
                    ScrollArea::vertical()
                        .id_salt("conflict-fields")
                        .max_height(ui.available_height().min(theme::CONFLICT_LIST_MAX_HEIGHT))
                        .show(ui, |ui| {
                            for (index, conflict) in
                                dialog.conflict.plan.conflicts.iter().enumerate()
                            {
                                let missing = i18n::text(language, TextKey::DeletedValue);
                                let unavailable = i18n::text(language, TextKey::UnavailableValue);
                                Frame::group(ui.style()).show(ui, |ui| {
                                    ui.label(
                                        RichText::new(&conflict.label)
                                            .family(FontFamily::Monospace)
                                            .strong(),
                                    );
                                    ui.label(
                                        RichText::new(format!(
                                            "{}: {}",
                                            i18n::text(language, TextKey::BaseValue),
                                            conflict.base_summary(missing, unavailable)
                                        ))
                                        .small()
                                        .color(theme::palette(ui).muted),
                                    );
                                    ui.horizontal(|ui| {
                                        context_selectable_value(
                                            ui,
                                            &mut dialog.choices[index],
                                            ResolutionChoice::Local,
                                            i18n::text(language, TextKey::UseLocal),
                                            &conflict.label,
                                        );
                                        ui.code(conflict.local_summary(missing, unavailable));
                                    });
                                    ui.horizontal(|ui| {
                                        context_selectable_value(
                                            ui,
                                            &mut dialog.choices[index],
                                            ResolutionChoice::Disk,
                                            i18n::text(language, TextKey::UseDisk),
                                            &conflict.label,
                                        );
                                        ui.code(conflict.disk_summary(missing, unavailable));
                                    });
                                });
                                ui.add_space(theme::SPACE_SM);
                            }
                        });
                }
                ui.separator();
                modal_actions(ui, |ui| {
                    if ui
                        .add(primary_button(
                            ui,
                            i18n::text(language, TextKey::MergeAndSave),
                        ))
                        .clicked()
                    {
                        apply = true;
                    }
                    if ui.button(i18n::text(language, TextKey::Cancel)).clicked() {
                        cancel = true;
                    }
                });
            },
        );

        if apply {
            let root = self.preferences.config_root.clone();
            let result = match dialog.target {
                ConflictTarget::Match(index) => self.files.get_mut(index).map_or_else(
                    || Err(storage::StorageIssue::MissingTargetFile.into()),
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
                    || Err(storage::StorageIssue::MissingTargetFile.into()),
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
                    i18n::merge_saved_text(language, &receipt.hash[..8]),
                ),
                Err(error) => self.notify_storage_error(error),
            }
        } else if !dismissed && !cancel {
            self.conflict_dialog = Some(dialog);
        }
    }

    fn restore_confirmation(&mut self, ui: &mut Ui) {
        let Some(pending) = self.pending_restore.clone() else {
            return;
        };
        let mut restore = false;
        let mut cancel = false;
        let language = self.preferences.language;
        let dismissed = show_modal(
            ui,
            "restore-history",
            i18n::text(language, TextKey::RestoreHistoryTitle),
            |ui| {
                set_responsive_modal_width(ui, theme::MODAL_WIDTH_LG);
                ui.label(i18n::restore_target_text(
                    language,
                    &pending.relative_path.display().to_string(),
                    &pending.timestamp,
                ));
                callout(
                    ui,
                    theme::palette(ui).amber,
                    i18n::text(language, TextKey::RestoreWarning),
                );
                modal_actions(ui, |ui| {
                    if ui
                        .add(primary_button(
                            ui,
                            i18n::text(language, TextKey::BackupAndRestore),
                        ))
                        .clicked()
                    {
                        restore = true;
                    }
                    if ui.button(i18n::text(language, TextKey::Cancel)).clicked() {
                        cancel = true;
                    }
                });
            },
        );
        if restore {
            match storage::restore_history(
                &self.preferences.config_root,
                &pending.relative_path,
                &pending.backup_path,
            ) {
                Ok(_) => {
                    self.pending_restore = None;
                    self.reload_workspace();
                    self.notify(
                        MessageKind::Success,
                        i18n::text(language, TextKey::RestoreComplete),
                    );
                }
                Err(error) => {
                    self.pending_restore = None;
                    self.notify_storage_error(error);
                }
            }
        } else if cancel || dismissed {
            self.pending_restore = None;
        }
    }

    fn variable_window(&mut self, ui: &mut Ui) {
        let Some(mut editor) = self.variable_editor.take() else {
            return;
        };
        let mut save = false;
        let mut cancel = false;
        let language = self.preferences.language;
        let title = if editor.index.is_some() {
            i18n::text(language, TextKey::EditVariableTitle)
        } else {
            i18n::text(language, TextKey::AddVariableTitle)
        };
        let dismissed = show_modal(ui, "variable-editor", title, |ui| {
            set_responsive_modal_width(ui, theme::MODAL_WIDTH_XL);
            ui.horizontal_wrapped(|ui| {
                ui.vertical(|ui| {
                    let name_label = ui
                        .label(RichText::new(i18n::text(language, TextKey::VariableName)).strong());
                    ui.add(
                        singleline_text_edit(&mut editor.variable.name).hint_text("my_variable"),
                    )
                    .labelled_by(name_label.id);
                });
                ui.vertical(|ui| {
                    let kind_label =
                        ui.label(RichText::new(i18n::text(language, TextKey::Kind)).strong());
                    let old_kind = editor.variable.kind.clone();
                    let kind_response = ComboBox::from_id_salt("variable-kind")
                        .selected_text(i18n::variable_kind_label(language, &editor.variable.kind))
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
                                    i18n::variable_kind_label(language, kind),
                                );
                            }
                        });
                    kind_response.response.labelled_by(kind_label.id);
                    if editor.variable.kind != old_kind {
                        let name = editor.variable.name.clone();
                        editor.variable = localized_new_variable(language, &editor.variable.kind);
                        editor.variable.name = name;
                    }
                });
            });
            ui.separator();
            variable_parameters(ui, language, &mut editor.variable);
            ui.separator();
            let mut dependencies = editor.variable.depends_on.join(", ");
            labelled_two_column_field(
                ui,
                i18n::text(language, TextKey::Dependencies),
                i18n::text(language, TextKey::DependenciesDescription),
                |ui, label_id| {
                    if ui
                        .add(singleline_text_edit(&mut dependencies).hint_text("first, second"))
                        .labelled_by(label_id)
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
                    i18n::text(language, TextKey::InsertVariableToken),
                );
            }
            modal_actions(ui, |ui| {
                if ui
                    .add(primary_button(
                        ui,
                        i18n::text(language, TextKey::SaveVariable),
                    ))
                    .clicked()
                {
                    save = true;
                }
                if ui.button(i18n::text(language, TextKey::Cancel)).clicked() {
                    cancel = true;
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
                    i18n::text(language, TextKey::InvalidVariableName),
                );
                self.variable_editor = Some(editor);
                return;
            }
            self.apply_variable_editor(editor);
        } else if !dismissed && !cancel {
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
            i18n::text(self.preferences.language, TextKey::VariableSaved),
        );
    }

    fn form_field_window(&mut self, ui: &mut Ui) {
        let Some(mut editor) = self.form_field_editor.take() else {
            return;
        };
        let mut save = false;
        let mut cancel = false;
        let language = self.preferences.language;
        let dismissed = show_modal(
            ui,
            "form-field-editor",
            i18n::text(language, TextKey::FormFieldTitle),
            |ui| {
                set_responsive_modal_width(ui, theme::MODAL_WIDTH_LG);
                let name_label = ui.label(i18n::text(language, TextKey::FieldName));
                ui.add(singleline_text_edit(&mut editor.name))
                    .labelled_by(name_label.id);
                let original_kind = editor.field.kind();
                let mut kind = original_kind.clone();
                let kind_label = ui.label(i18n::text(language, TextKey::InputType));
                let kind_response = ComboBox::from_id_salt("form-field-kind")
                    .selected_text(i18n::form_field_kind_label(language, &kind))
                    .show_ui(ui, |ui| {
                        for candidate in FormFieldKind::ALL {
                            let label = i18n::form_field_kind_label(language, &candidate);
                            ui.selectable_value(&mut kind, candidate, label);
                        }
                    });
                kind_response.response.labelled_by(kind_label.id);
                if kind != original_kind {
                    editor.field.set_kind(&kind);
                }
                let initial_value_label = ui.label(i18n::text(language, TextKey::InitialValue));
                let mut default = editor.field.default.clone().unwrap_or_default();
                if ui
                    .add(singleline_text_edit(&mut default))
                    .labelled_by(initial_value_label.id)
                    .changed()
                {
                    editor.field.default = (!default.is_empty()).then_some(default);
                }
                if matches!(kind, FormFieldKind::Choice | FormFieldKind::List) {
                    let choices_label = ui.label(i18n::text(language, TextKey::ChoicesPerLine));
                    let mut values = editor.field.values.join("\n");
                    if ui
                        .add(multiline_text_edit(&mut values).desired_rows(6))
                        .labelled_by(choices_label.id)
                        .changed()
                    {
                        editor.field.values = values.lines().map(str::to_string).collect();
                    }
                }
                modal_actions(ui, |ui| {
                    if ui
                        .add(primary_button(ui, i18n::text(language, TextKey::SaveField)))
                        .clicked()
                    {
                        save = true;
                    }
                    if ui.button(i18n::text(language, TextKey::Cancel)).clicked() {
                        cancel = true;
                    }
                });
            },
        );
        if save {
            let name = editor.name.trim().to_string();
            if name.is_empty()
                || !name
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || character == '_')
            {
                self.notify(
                    MessageKind::Error,
                    i18n::text(language, TextKey::InvalidFieldName),
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
        } else if !dismissed && !cancel {
            self.form_field_editor = Some(editor);
        }
    }

    fn delete_confirmation(&mut self, ui: &mut Ui) {
        let Some(pending) = self.pending_delete else {
            return;
        };
        let mut keep_open = true;
        let language = self.preferences.language;
        let dismissed = show_modal(
            ui,
            "delete-confirmation",
            i18n::text(language, TextKey::DeleteConfirmationTitle),
            |ui| {
                set_responsive_modal_width(ui, theme::MODAL_WIDTH_MD);
                ui.label(i18n::text(
                    language,
                    match pending {
                        PendingDelete::Snippet => TextKey::DeleteSnippetQuestion,
                        PendingDelete::File => TextKey::DeleteFileQuestion,
                    },
                ));
                modal_actions(ui, |ui| {
                    if ui
                        .add(danger_button(
                            ui,
                            i18n::text(language, TextKey::ConfirmDelete),
                        ))
                        .clicked()
                    {
                        match pending {
                            PendingDelete::Snippet => self.delete_selected_snippet(),
                            PendingDelete::File => self.delete_selected_file(),
                        }
                        keep_open = false;
                    }
                    if ui.button(i18n::text(language, TextKey::Cancel)).clicked() {
                        keep_open = false;
                    }
                });
            },
        );
        if !keep_open || dismissed {
            self.pending_delete = None;
        }
    }

    fn close_confirmation(&mut self, ui: &mut Ui) {
        if !self.confirm_close {
            return;
        }
        let language = self.preferences.language;
        let dismissed = show_modal(
            ui,
            "close-confirmation",
            i18n::text(language, TextKey::UnsavedChangesTitle),
            |ui| {
                set_responsive_modal_width(ui, theme::MODAL_WIDTH_MD);
                ui.label(i18n::text(language, TextKey::DiscardChangesQuestion));
                modal_actions(ui, |ui| {
                    if ui
                        .add(danger_button(
                            ui,
                            i18n::text(language, TextKey::DiscardAndExit),
                        ))
                        .clicked()
                    {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        self.confirm_close = false;
                        for file in &mut self.files {
                            file.dirty = false;
                        }
                    }
                    if ui
                        .button(i18n::text(language, TextKey::ReturnToEditor))
                        .clicked()
                    {
                        self.confirm_close = false;
                    }
                });
            },
        );
        if dismissed {
            self.confirm_close = false;
        }
    }
}

impl EspansoGuiApp {
    fn render(&mut self, ui: &mut Ui) {
        ui.ctx()
            .accesskit_node_builder(egui::accesskit_root_id(), |node| {
                node.set_label("Espanso GUI");
            });
        self.keyboard_shortcuts(ui);
        let close_requested = ui.ctx().input(|input| input.viewport().close_requested());
        if close_requested && self.has_dirty_files() && !self.confirm_close {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.confirm_close = true;
        }

        egui::CentralPanel::default()
            .frame(Frame::new().fill(theme::palette(ui).paper))
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
}

impl eframe::App for EspansoGuiApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.render(ui);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, APP_STORAGE_KEY, &self.preferences);
    }
}

fn tab_button(ui: &mut Ui, current: &mut EditorTab, value: EditorTab, label: &str) {
    let _ = unambiguous_selectable_value(ui, current, value, label);
}

fn option_checkbox(
    ui: &mut Ui,
    language: Language,
    value: &mut Option<bool>,
    label: &str,
    description: &str,
) {
    let mut enabled = value.unwrap_or(false);
    labelled_two_column_field(ui, label, description, |ui, label_id| {
        if ui
            .checkbox(&mut enabled, i18n::text(language, TextKey::Enabled))
            .labelled_by(label_id)
            .changed()
        {
            *value = enabled.then_some(true);
        }
    });
}

fn message_bar(ui: &mut Ui, message: &Message) {
    let color = match message.kind {
        MessageKind::Success => theme::palette(ui).accent,
        MessageKind::Info => theme::palette(ui).info,
        MessageKind::Error => theme::palette(ui).danger,
    };
    let live = match message.kind {
        MessageKind::Error => egui::accesskit::Live::Assertive,
        MessageKind::Success | MessageKind::Info => egui::accesskit::Live::Polite,
    };
    live_message_bar(ui, &message.text, color, live);
}

fn localized_new_snippet(language: Language) -> Snippet {
    Snippet::with_template(
        i18n::text(language, TextKey::NewSnippetLabel),
        i18n::text(language, TextKey::NewSnippetContent),
    )
}

fn localized_snippet_title(language: Language, snippet: &Snippet) -> String {
    snippet
        .title()
        .unwrap_or_else(|| i18n::text(language, TextKey::UntitledSnippet).into())
}

fn localized_new_variable(language: Language, kind: &str) -> Variable {
    let mut variable = Variable::new(kind);
    match kind {
        "echo" => variable.set_param("echo", i18n::text(language, TextKey::ExampleValue)),
        "random" => variable.set_string_list(
            "choices",
            &[
                i18n::text(language, TextKey::ExampleCandidateOne).into(),
                i18n::text(language, TextKey::ExampleCandidateTwo).into(),
            ],
        ),
        "choice" => variable.set_string_list(
            "values",
            &[
                i18n::text(language, TextKey::ExampleCandidateOne).into(),
                i18n::text(language, TextKey::ExampleCandidateTwo).into(),
            ],
        ),
        "form" => variable.set_param("layout", i18n::text(language, TextKey::ExampleFormLayout)),
        _ => {}
    }
    variable
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    fn accessibility_fixture(language: Language) -> EspansoGuiApp {
        let root = PathBuf::from("/tmp/espanso-gui-accessibility-fixture");
        let match_yaml = r#"global_vars:
  - name: today
    type: date
    params:
      format: "%Y-%m-%d"
  - name: greeting
    type: echo
    params:
      echo: hello
matches:
  - trigger: ":form"
    label: "Form example"
    search_terms: [forms, demo]
    form: "Name: [[name]] / Choice: [[choice]]"
    form_fields:
      name:
        default: Ada
      choice:
        type: choice
        values: [One, Two]
      future:
        type: future_widget
  - trigger: ":form"
    label: "Duplicate diagnostic"
    replace: "{{missing_variable}}"
"#;
        let profile_yaml = r#"filter_title: Browser
inject_delay: 1
max_form_width: 640
"#;
        let files = vec![WorkspaceFile {
            relative_path: PathBuf::from("match/base.yml"),
            display_name: "base".into(),
            document: crate::model::MatchFile::from_yaml(match_yaml).expect("match fixture"),
            raw_yaml: match_yaml.into(),
            base_yaml: match_yaml.into(),
            saved_hash: "fixture".into(),
            modified_ms: 0,
            is_package: false,
            dirty: false,
            had_comments: false,
        }];
        let config_files = vec![ConfigFile {
            relative_path: PathBuf::from("config/browser.yml"),
            display_name: "browser".into(),
            profile: crate::model::ConfigProfile::from_yaml(profile_yaml).expect("profile fixture"),
            raw_yaml: profile_yaml.into(),
            base_yaml: profile_yaml.into(),
            saved_hash: "fixture".into(),
            modified_ms: 0,
            is_default: false,
            dirty: false,
            had_comments: false,
        }];
        let preferences = Preferences {
            config_root: root.clone(),
            language,
            ui_scale: 1.0,
            snippet_sort: SnippetSort::FileOrder,
            appearance: theme::Appearance::System,
        };
        let status = EspansoStatus {
            config_root: root,
            ..EspansoStatus::default()
        };
        EspansoGuiApp::from_loaded(preferences, status, files, config_files, None)
    }

    fn accessibility_update_at_size(
        app: &mut EspansoGuiApp,
        screen_size: egui::Vec2,
    ) -> egui::accesskit::TreeUpdate {
        let context = egui::Context::default();
        context.enable_accesskit();
        theme::install(&context);
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, screen_size)),
            ..Default::default()
        };
        let mut warmup = context.run_ui(input.clone(), |ui| app.render(ui));
        warmup.textures_delta.clear();
        let mut output = context.run_ui(input, |ui| app.render(ui));
        output.textures_delta.clear();
        output
            .platform_output
            .accesskit_update
            .expect("accessibility should be enabled")
    }

    fn accessibility_update(app: &mut EspansoGuiApp) -> egui::accesskit::TreeUpdate {
        accessibility_update_at_size(
            app,
            egui::vec2(theme::DEFAULT_WINDOW_SIZE[0], theme::DEFAULT_WINDOW_SIZE[1]),
        )
    }

    fn accessibility_nodes(
        update: &egui::accesskit::TreeUpdate,
    ) -> HashMap<egui::accesskit::NodeId, &egui::accesskit::Node> {
        update.nodes.iter().map(|(id, node)| (*id, node)).collect()
    }

    fn accessible_name(
        id: egui::accesskit::NodeId,
        nodes: &HashMap<egui::accesskit::NodeId, &egui::accesskit::Node>,
    ) -> String {
        let node = nodes.get(&id).expect("accessibility node");
        if let Some(label) = node.label().filter(|label| !label.trim().is_empty()) {
            return label.trim().to_owned();
        }
        node.labelled_by()
            .iter()
            .filter_map(|label_id| nodes.get(label_id))
            .filter_map(|label| label.label().or_else(|| label.value()))
            .map(str::trim)
            .filter(|label| !label.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn focusable_nodes_from<'a>(
        root: egui::accesskit::NodeId,
        nodes: &HashMap<egui::accesskit::NodeId, &'a egui::accesskit::Node>,
    ) -> Vec<(egui::accesskit::NodeId, &'a egui::accesskit::Node)> {
        fn visit<'a>(
            id: egui::accesskit::NodeId,
            nodes: &HashMap<egui::accesskit::NodeId, &'a egui::accesskit::Node>,
            output: &mut Vec<(egui::accesskit::NodeId, &'a egui::accesskit::Node)>,
        ) {
            let node = nodes.get(&id).expect("complete initial accessibility tree");
            if node.supports_action(egui::accesskit::Action::Focus) {
                output.push((id, node));
            }
            for child in node.children() {
                visit(*child, nodes, output);
            }
        }

        let mut output = Vec::new();
        visit(root, nodes, &mut output);
        output
    }

    fn focusable_nodes(
        update: &egui::accesskit::TreeUpdate,
    ) -> Vec<(egui::accesskit::NodeId, &egui::accesskit::Node)> {
        let nodes = accessibility_nodes(update);
        let root = update.tree.as_ref().expect("initial tree").root;
        focusable_nodes_from(root, &nodes)
    }

    fn focus_labels(app: &mut EspansoGuiApp) -> Vec<String> {
        let update = accessibility_update(app);
        assert_named_horizontal_bounds(&update, theme::DEFAULT_WINDOW_SIZE[0]);
        focus_labels_from_update(&update)
    }

    fn focus_labels_at_size(app: &mut EspansoGuiApp, screen_size: egui::Vec2) -> Vec<String> {
        let update = accessibility_update_at_size(app, screen_size);
        assert_named_horizontal_bounds(&update, screen_size.x);
        focus_labels_from_update(&update)
    }

    fn focus_labels_from_update(update: &egui::accesskit::TreeUpdate) -> Vec<String> {
        let nodes = accessibility_nodes(update);
        focusable_nodes(update)
            .into_iter()
            .map(|(id, _)| accessible_name(id, &nodes))
            .collect()
    }

    fn assert_named_horizontal_bounds(update: &egui::accesskit::TreeUpdate, screen_width: f32) {
        let nodes = accessibility_nodes(update);
        for (id, node) in &update.nodes {
            if node.role() == egui::accesskit::Role::TextRun {
                continue;
            }
            let name = accessible_name(*id, &nodes);
            if name.is_empty() {
                continue;
            }
            let Some(bounds) = node.bounds() else {
                continue;
            };
            assert!(
                bounds.x0 >= -1.0 && bounds.x1 <= f64::from(screen_width) + 1.0,
                "named UI node {name:?} overflows horizontally: {bounds:?} in {screen_width}"
            );
        }
    }

    fn label_position(labels: &[String], visible_label: &str) -> usize {
        labels
            .iter()
            .position(|label| label.starts_with(visible_label))
            .unwrap_or_else(|| panic!("missing focus label {visible_label:?} in {labels:#?}"))
    }

    fn maximum_zoom_screen_size() -> egui::Vec2 {
        egui::vec2(
            theme::MINIMUM_WINDOW_SIZE[0] / theme::UI_SCALE_MAX,
            theme::MINIMUM_WINDOW_SIZE[1] / theme::UI_SCALE_MAX,
        )
    }

    fn screen_size_at_scale(scale: f32) -> egui::Vec2 {
        egui::vec2(
            theme::MINIMUM_WINDOW_SIZE[0] / scale,
            theme::MINIMUM_WINDOW_SIZE[1] / scale,
        )
    }

    fn assert_modal_accessibility(
        app: &mut EspansoGuiApp,
        expected_title: &str,
        expected_controls: &[String],
    ) -> HashSet<String> {
        assert_modal_accessibility_at_size(
            app,
            egui::vec2(theme::DEFAULT_WINDOW_SIZE[0], theme::DEFAULT_WINDOW_SIZE[1]),
            expected_title,
            expected_controls,
        )
    }

    fn assert_modal_accessibility_at_size(
        app: &mut EspansoGuiApp,
        screen_size: egui::Vec2,
        expected_title: &str,
        expected_controls: &[String],
    ) -> HashSet<String> {
        let update = accessibility_update_at_size(app, screen_size);
        assert_named_horizontal_bounds(&update, screen_size.x);
        let (dialog_id, dialog) = update
            .nodes
            .iter()
            .map(|(id, node)| (*id, node))
            .find(|(_, node)| node.role() == egui::accesskit::Role::Dialog)
            .expect("dialog node");
        assert_eq!(dialog.label(), Some(expected_title));
        assert!(dialog.is_modal());

        let nodes = accessibility_nodes(&update);
        let focusable = focusable_nodes_from(dialog_id, &nodes);
        for (id, node) in &focusable {
            let Some(bounds) = node.bounds() else {
                continue;
            };
            let height = bounds.y1 - bounds.y0;
            let width = bounds.x1 - bounds.x0;
            assert!(
                height + 1.0 >= f64::from(theme::CONTROL_HEIGHT),
                "dialog control {:?} is only {height:.1}px high in {expected_title:?}: {bounds:?}",
                accessible_name(*id, &nodes)
            );
            assert!(
                width + 1.0 >= f64::from(theme::CONTROL_MIN_WIDTH),
                "dialog control {:?} is only {width:.1}px wide in {expected_title:?}: {bounds:?}",
                accessible_name(*id, &nodes)
            );
        }
        let labels = focusable
            .into_iter()
            .map(|(id, _)| accessible_name(id, &nodes))
            .collect::<HashSet<_>>();
        assert!(
            labels.iter().all(|label| !label.is_empty()),
            "unnamed dialog control in {expected_title:?}: {labels:#?}"
        );
        for expected in expected_controls {
            assert!(
                labels.contains(expected),
                "missing dialog control {expected:?} in {expected_title:?}: {labels:#?}"
            );
        }
        labels
    }

    fn conflict_fixture() -> ExternalConflict {
        let base = serde_yaml_ng::from_str("matches:\n  - replace: base\n").expect("base YAML");
        let local = serde_yaml_ng::from_str("matches:\n  - replace: local\n").expect("local YAML");
        let disk = serde_yaml_ng::from_str("matches:\n  - replace: disk\n").expect("disk YAML");
        ExternalConflict {
            remote_yaml: "matches:\n  - replace: disk\n".into(),
            remote_hash: "fixture".into(),
            plan: crate::conflict::MergePlan::new(&base, &local, &disk),
        }
    }

    #[test]
    fn new_snippet_template_follows_the_selected_language() {
        let japanese = localized_new_snippet(Language::Japanese);
        assert_eq!(japanese.label.as_deref(), Some("新しいスニペット"));
        assert_eq!(japanese.content(), "ここに展開するテキストを入力");

        let english = localized_new_snippet(Language::English);
        assert_eq!(english.label.as_deref(), Some("New snippet"));
        assert_eq!(english.content(), "Enter replacement text here");

        let mut untitled = Snippet::new();
        untitled.trigger = None;
        assert_eq!(
            localized_snippet_title(Language::English, &untitled),
            "Untitled snippet"
        );
    }

    #[test]
    fn snippet_search_spans_all_files_without_changing_the_unfiltered_list() {
        let app = accessibility_fixture(Language::English);
        let mut files = app.files;
        files[0].document.matches[0].search_terms = vec!["Forms".into(), "Work".into()];
        let remote_yaml = r#"matches:
  - trigger: ":remote"
    label: "Remote result"
    replace: "Found in another file"
"#;
        let mut remote = files[0].clone();
        remote.relative_path = PathBuf::from("match/remote.yml");
        remote.display_name = "remote".into();
        remote.document = crate::model::MatchFile::from_yaml(remote_yaml).unwrap();
        remote.document.matches[0].search_terms = vec!["work".into()];
        files.push(remote);

        let unfiltered =
            snippet_library::entries(&files, 0, "", Language::English, SnippetSort::FileOrder);
        assert_eq!(unfiltered.len(), 2);
        assert!(unfiltered.iter().all(|entry| entry.file_index == 0));

        let results = snippet_library::entries(
            &files,
            0,
            "  REMOTE  ",
            Language::English,
            SnippetSort::FileOrder,
        );
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].file_index, 1);
        assert_eq!(results[0].snippet_index, 0);
        assert_eq!(results[0].title, "Remote result");
        assert!(results[0].context.contains("match/remote.yml"));

        let sorted = snippet_library::entries(&files, 0, ":", Language::English, SnippetSort::Name);
        assert_eq!(
            sorted
                .iter()
                .map(|entry| entry.title.as_str())
                .collect::<Vec<_>>(),
            ["Duplicate diagnostic", "Form example", "Remote result"]
        );
        assert_eq!(
            snippet_library::search_terms(&files),
            vec![("Forms".into(), 1), ("Work".into(), 2)]
        );
    }

    #[test]
    fn empty_search_results_are_explained_and_keyboard_recoverable_in_both_languages() {
        for language in Language::ALL {
            let mut app = accessibility_fixture(language);
            app.search = "definitely-not-present".into();
            let update = accessibility_update(&mut app);
            let result_count = i18n::search_result_count(language, 0);
            let live_nodes = update
                .nodes
                .iter()
                .map(|(_, node)| node)
                .filter(|node| node.live().is_some())
                .map(|node| (node.label(), node.live()))
                .collect::<Vec<_>>();
            assert!(
                live_nodes.iter().any(|(label, live)| {
                    *label == Some(&result_count) && *live == Some(egui::accesskit::Live::Polite)
                }),
                "missing localized live result count {result_count:?} in {live_nodes:#?}"
            );
            let focus_labels = focus_labels_from_update(&update);
            for key in [TextKey::FilterByTag, TextKey::ClearSearch] {
                assert!(
                    focus_labels
                        .iter()
                        .any(|label| label == i18n::text(language, key)),
                    "missing {key:?} in {language:?}: {focus_labels:#?}"
                );
            }
        }
    }

    #[test]
    fn disconnected_workspace_exposes_bilingual_next_steps_at_maximum_zoom() {
        for language in Language::ALL {
            let mut app = accessibility_fixture(language);
            app.files.clear();
            app.status.installed = false;
            let labels = focus_labels_at_size(&mut app, maximum_zoom_screen_size());
            assert!(
                labels.iter().all(|label| !label.is_empty()),
                "unnamed onboarding control in {language:?}: {labels:#?}"
            );
            for key in [
                TextKey::OpenEspansoSetup,
                TextKey::ChooseConfigFolder,
                TextKey::InitializeHere,
            ] {
                assert!(
                    labels
                        .iter()
                        .any(|label| label == i18n::text(language, key)),
                    "missing onboarding action {key:?} in {language:?}: {labels:#?}"
                );
            }
            assert!(
                labels
                    .iter()
                    .all(|label| label != i18n::text(language, TextKey::AddFile)),
                "file creation should stay hidden until a workspace is connected: {labels:#?}"
            );
        }
    }

    #[test]
    fn wide_navigation_labels_and_shortcuts_fit_without_truncation() {
        let context = egui::Context::default();
        theme::install(&context);
        let available_text_width = theme::NAVIGATION_WIDTH
            - 2.0 * f32::from(theme::PADDING_LG)
            - 2.0 * f32::from(theme::PADDING_MD);

        let mut output = context.run_ui(egui::RawInput::default(), |ui| {
            let font = egui::TextStyle::Button.resolve(ui.style());
            for language in Language::ALL {
                for (key, shortcut) in [
                    (TextKey::Snippets, navigation::command_shortcut("1")),
                    (TextKey::Profiles, navigation::command_shortcut("2")),
                    (TextKey::Globals, navigation::command_shortcut("3")),
                    (TextKey::Diagnostics, navigation::command_shortcut("4")),
                    (TextKey::SettingsNav, navigation::command_shortcut("5")),
                    (TextKey::About, String::new()),
                ] {
                    let label = i18n::text(language, key);
                    let label_width = ui
                        .painter()
                        .layout_no_wrap(label.into(), font.clone(), theme::palette(ui).ink)
                        .size()
                        .x;
                    let shortcut_width = ui
                        .painter()
                        .layout_no_wrap(shortcut.clone(), font.clone(), theme::palette(ui).ink)
                        .size()
                        .x;
                    let atom_gaps = 2.0 * ui.spacing().icon_spacing;
                    assert!(
                        label_width + shortcut_width + atom_gaps <= available_text_width,
                        "{language:?} {key:?} needs {:.1}px but only {available_text_width:.1}px is available",
                        label_width + shortcut_width + atom_gaps
                    );
                }
            }
        });
        output.textures_delta.clear();
    }

    #[test]
    fn snippet_search_hint_fits_the_input_in_both_languages() {
        let context = egui::Context::default();
        theme::install(&context);
        let available_text_width = theme::SNIPPET_LIST_COMPACT_WIDTH
            - 2.0 * f32::from(theme::PADDING_LG)
            - 2.0 * f32::from(theme::PADDING_MD);

        let mut output = context.run_ui(egui::RawInput::default(), |ui| {
            let font = egui::TextStyle::Body.resolve(ui.style());
            for language in Language::ALL {
                let hint = i18n::text(language, TextKey::SearchHint);
                let width = ui
                    .painter()
                    .layout_no_wrap(hint.into(), font.clone(), theme::palette(ui).muted)
                    .size()
                    .x;
                assert!(
                    width <= available_text_width,
                    "{language:?} search hint needs {width:.1}px but only {available_text_width:.1}px is available"
                );
            }
        });
        output.textures_delta.clear();
    }

    #[test]
    fn short_field_descriptions_fit_the_wide_label_column() {
        let context = egui::Context::default();
        theme::install(&context);
        let mut output = context.run_ui(egui::RawInput::default(), |ui| {
            let font = egui::TextStyle::Small.resolve(ui.style());
            for language in Language::ALL {
                for key in [
                    TextKey::DisplayNameDescription,
                    TextKey::LanguageDescription,
                ] {
                    let label = i18n::text(language, key);
                    let width = ui
                        .painter()
                        .layout_no_wrap(label.into(), font.clone(), theme::palette(ui).muted)
                        .size()
                        .x;
                    assert!(
                        width <= theme::FIELD_LABEL_WIDTH,
                        "{language:?} {key:?} needs {width:.1}px but the wide field label column provides only {:.1}px",
                        theme::FIELD_LABEL_WIDTH
                    );
                }
            }
        });
        output.textures_delta.clear();
    }

    #[test]
    fn long_file_list_keeps_navigation_footer_actions_visible() {
        for language in Language::ALL {
            let mut app = accessibility_fixture(language);
            let template = app.files[0].clone();
            app.files = (0..32)
                .map(|index| {
                    let mut file = template.clone();
                    file.display_name = format!("file-{index:02}");
                    file.relative_path = PathBuf::from(format!("match/file-{index:02}.yml"));
                    file
                })
                .collect();

            let screen_size =
                egui::vec2(theme::DEFAULT_WINDOW_SIZE[0], theme::MINIMUM_WINDOW_SIZE[1]);
            let update = accessibility_update_at_size(&mut app, screen_size);
            let nodes = accessibility_nodes(&update);
            let focusable = focusable_nodes(&update);
            for key in [TextKey::AddFile, TextKey::SettingsNav, TextKey::About] {
                let expected = i18n::text(language, key);
                let (_, node) = focusable
                    .iter()
                    .find(|(id, _)| accessible_name(*id, &nodes).starts_with(expected))
                    .unwrap_or_else(|| {
                        panic!("missing footer action {expected:?} in {language:?}")
                    });
                let bounds = node.bounds().expect("footer action bounds");
                assert!(
                    bounds.y0 >= -1.0 && bounds.y1 <= f64::from(screen_size.y) + 1.0,
                    "footer action {expected:?} is outside the viewport: {bounds:?}"
                );
            }
            let add_file_name = i18n::text(language, TextKey::AddFile);
            let add_file_bounds = focusable
                .iter()
                .find(|(id, _)| accessible_name(*id, &nodes).starts_with(add_file_name))
                .and_then(|(_, node)| node.bounds())
                .expect("add-file bounds");
            let version = format!("v{}", env!("CARGO_PKG_VERSION"));
            let version_bounds = nodes
                .values()
                .find_map(|node| {
                    (node.label().or_else(|| node.value()) == Some(version.as_str()))
                        .then(|| node.bounds())
                        .flatten()
                })
                .expect("version bounds");
            assert!(
                add_file_bounds.y1 + f64::from(theme::SPACE_XS) <= version_bounds.y0,
                "add-file and version surfaces overlap: {add_file_bounds:?} / {version_bounds:?}"
            );
        }
    }

    #[test]
    fn compact_section_selector_fits_every_localized_value_without_truncation() {
        let context = egui::Context::default();
        theme::install(&context);
        let maximum_zoom_width = theme::MINIMUM_WINDOW_SIZE[0] / theme::UI_SCALE_MAX;
        let selector_row_width = theme::COMPACT_SECTION_SELECTOR_WIDTH
            + theme::COMPACT_FILE_SELECTOR_WIDTH
            + f32::from(theme::PADDING_COMPACT);
        let available_row_width = maximum_zoom_width - 2.0 * f32::from(theme::PADDING_MD);
        assert!(
            selector_row_width <= available_row_width,
            "compact selectors need {selector_row_width:.1}px but maximum zoom provides only {available_row_width:.1}px"
        );
        let mut output = context.run_ui(egui::RawInput::default(), |ui| {
            let font = egui::TextStyle::Button.resolve(ui.style());
            for language in Language::ALL {
                for section in [
                    Section::Library,
                    Section::Profiles,
                    Section::Globals,
                    Section::Diagnostics,
                    Section::Settings,
                    Section::About,
                ] {
                    let label = i18n::text(language, section.text_key());
                    let label_width = ui
                        .painter()
                        .layout_no_wrap(label.into(), font.clone(), theme::palette(ui).ink)
                        .size()
                        .x;
                    let required_width = label_width
                        + 2.0 * ui.spacing().button_padding.x
                        + ui.spacing().icon_width
                        + ui.spacing().icon_spacing;
                    assert!(
                        required_width <= theme::COMPACT_SECTION_SELECTOR_WIDTH,
                        "{language:?} {section:?} needs {required_width:.1}px but the compact selector provides only {:.1}px",
                        theme::COMPACT_SECTION_SELECTOR_WIDTH
                    );
                }
            }
        });
        output.textures_delta.clear();
    }

    #[test]
    fn compact_top_bar_keeps_visible_status_text_at_maximum_zoom() {
        let context = egui::Context::default();
        theme::install(&context);
        let available_width = theme::MINIMUM_WINDOW_SIZE[0] / theme::UI_SCALE_MAX
            - 2.0 * f32::from(theme::PADDING_LG);
        let mut output = context.run_ui(egui::RawInput::default(), |ui| {
            let display_font = FontId::new(theme::TEXT_DISPLAY, FontFamily::Proportional);
            let section_font = FontId::new(theme::TEXT_SECTION, FontFamily::Proportional);
            let body_font = egui::TextStyle::Button.resolve(ui.style());
            let small_font = egui::TextStyle::Small.resolve(ui.style());
            let fixed_width = ui
                .painter()
                .layout_no_wrap("E/".into(), display_font, theme::palette(ui).accent)
                .size()
                .x
                + ui
                    .painter()
                    .layout_no_wrap(
                        "Espanso GUI".into(),
                        section_font,
                        theme::palette(ui).ink,
                    )
                    .size()
                    .x;

            for language in Language::ALL {
                for status_key in [TextKey::ConnectedShort, TextKey::NotDetectedShort] {
                    let status_width = ui
                        .painter()
                        .layout_no_wrap(
                            i18n::text(language, status_key).into(),
                            small_font.clone(),
                            theme::palette(ui).ink,
                        )
                        .size()
                        .x;
                    let save_width = ui
                        .painter()
                        .layout_no_wrap(
                            i18n::text(language, TextKey::Save).into(),
                            body_font.clone(),
                            theme::palette(ui).ink,
                        )
                        .size()
                        .x;
                    let required_width = fixed_width
                        + status_width
                        + save_width
                        + 4.0 * f32::from(theme::PADDING_MD)
                        + 3.0 * theme::SPACE_MD;
                    assert!(
                        required_width <= available_width,
                        "{language:?} {status_key:?} top bar needs {required_width:.1}px but only {available_width:.1}px is available"
                    );
                }
            }
        });
        output.textures_delta.clear();
    }

    #[test]
    fn new_variable_templates_follow_the_selected_language() {
        assert_eq!(
            localized_new_variable(Language::Japanese, "echo").param_str("echo"),
            "値"
        );
        assert_eq!(
            localized_new_variable(Language::English, "choice").param_strings("values"),
            vec!["Candidate 1", "Candidate 2"]
        );
        assert_eq!(
            localized_new_variable(Language::English, "form").param_str("layout"),
            "Name: [[name]]"
        );
    }

    #[test]
    fn application_styles_use_semantic_theme_colors() {
        for source in [
            include_str!("app.rs"),
            include_str!("html_editor.rs"),
            include_str!("navigation.rs"),
            include_str!("profile_editor.rs"),
            include_str!("settings_editor.rs"),
            include_str!("snippet_editor.rs"),
            include_str!("top_bar.rs"),
            include_str!("ui_components.rs"),
            include_str!("variable_editor.rs"),
            include_str!("yaml_editor.rs"),
        ] {
            assert!(!source.contains(concat!("Color32::from_", "rgb")));
            assert!(!source.contains(concat!("Color32::from_", "gray")));
            assert!(!source.contains(concat!("Color32::", "WHITE")));
        }
    }

    #[test]
    fn primary_views_expose_named_controls_in_stable_navigation_order() {
        for language in Language::ALL {
            for section in [
                Section::Library,
                Section::Profiles,
                Section::Globals,
                Section::Diagnostics,
                Section::Settings,
                Section::About,
            ] {
                let mut app = accessibility_fixture(language);
                app.section = section;
                let labels = focus_labels(&mut app);
                assert!(
                    labels.iter().all(|label| !label.is_empty()),
                    "unnamed focusable control in {language:?} {section:?}: {labels:#?}"
                );

                let ordered_navigation = [
                    TextKey::Save,
                    TextKey::Reload,
                    TextKey::Snippets,
                    TextKey::Profiles,
                    TextKey::Globals,
                    TextKey::Diagnostics,
                ]
                .map(|key| label_position(&labels, i18n::text(language, key)));
                assert!(
                    ordered_navigation.windows(2).all(|pair| pair[0] < pair[1]),
                    "unstable navigation order in {language:?} {section:?}: {labels:#?}"
                );
            }
        }
    }

    #[test]
    fn primary_view_controls_keep_comfortable_hit_targets() {
        for language in Language::ALL {
            for section in [
                Section::Library,
                Section::Profiles,
                Section::Globals,
                Section::Diagnostics,
                Section::Settings,
                Section::About,
            ] {
                let mut app = accessibility_fixture(language);
                app.section = section;
                let update = accessibility_update(&mut app);
                let nodes = accessibility_nodes(&update);
                for (id, node) in focusable_nodes(&update) {
                    let Some(bounds) = node.bounds() else {
                        continue;
                    };
                    let height = bounds.y1 - bounds.y0;
                    let width = bounds.x1 - bounds.x0;
                    assert!(
                        height + 1.0 >= f64::from(theme::CONTROL_HEIGHT),
                        "control {:?} is only {height:.1}px high in {language:?} {section:?}: {bounds:?}",
                        accessible_name(id, &nodes)
                    );
                    assert!(
                        width + 1.0 >= f64::from(theme::CONTROL_MIN_WIDTH),
                        "control {:?} is only {width:.1}px wide in {language:?} {section:?}: {bounds:?}",
                        accessible_name(id, &nodes)
                    );
                }
            }
        }
    }

    #[test]
    fn wide_navigation_and_file_rows_expose_their_selected_state() {
        for language in Language::ALL {
            let mut app = accessibility_fixture(language);
            let update = accessibility_update(&mut app);
            let nodes = accessibility_nodes(&update);
            let toggled = focusable_nodes(&update)
                .into_iter()
                .filter_map(|(id, node)| {
                    node.toggled()
                        .map(|state| (accessible_name(id, &nodes), state))
                })
                .collect::<Vec<_>>();
            let selected_section = i18n::text(language, TextKey::Snippets);

            assert!(
                toggled.iter().any(|(name, state)| {
                    name.starts_with(selected_section) && *state == egui::accesskit::Toggled::True
                }),
                "selected navigation state missing in {language:?}: {toggled:#?}"
            );
            assert!(
                toggled.iter().any(|(name, state)| {
                    name.starts_with("base") && *state == egui::accesskit::Toggled::True
                }),
                "selected file state missing in {language:?}: {toggled:#?}"
            );
            let profiles = i18n::text(language, TextKey::Profiles);
            assert!(
                toggled.iter().any(|(name, state)| {
                    name.starts_with(profiles) && *state == egui::accesskit::Toggled::False
                }),
                "unselected navigation state missing in {language:?}: {toggled:#?}"
            );
        }
    }

    #[test]
    fn maximum_zoom_keeps_primary_views_and_large_dialog_actions_exposed() {
        let screen_size = maximum_zoom_screen_size();
        for language in Language::ALL {
            for section in [
                Section::Library,
                Section::Profiles,
                Section::Globals,
                Section::Diagnostics,
                Section::Settings,
                Section::About,
            ] {
                let mut app = accessibility_fixture(language);
                app.section = section;
                app.files[0].dirty = true;
                let update = accessibility_update_at_size(&mut app, screen_size);
                assert_named_horizontal_bounds(&update, screen_size.x);
                let labels = focus_labels_from_update(&update);
                assert!(
                    labels.iter().all(|label| !label.is_empty()),
                    "unnamed 200%-zoom control in {language:?} {section:?}: {labels:#?}"
                );
                let nodes = accessibility_nodes(&update);
                let workspace_name = i18n::text(language, TextKey::Workspace);
                let section_selector = focusable_nodes(&update)
                    .into_iter()
                    .find(|(id, _)| accessible_name(*id, &nodes) == workspace_name)
                    .unwrap_or_else(|| {
                        panic!(
                            "missing compact workspace selector in {language:?} {section:?}: {labels:#?}"
                        )
                    });
                assert_eq!(
                    section_selector.1.value(),
                    Some(i18n::text(language, section.text_key())),
                    "compact workspace selector does not expose its current section"
                );
                let (primary_name, primary) = match section {
                    Section::Library | Section::Profiles | Section::Globals | Section::Settings => {
                        let key = match section {
                            Section::Library => TextKey::Search,
                            Section::Profiles => TextKey::Visual,
                            Section::Globals => TextKey::AddVariable,
                            Section::Settings => TextKey::Language,
                            _ => unreachable!(),
                        };
                        let name = i18n::text(language, key);
                        let node = focusable_nodes(&update)
                            .into_iter()
                            .find(|(id, _)| accessible_name(*id, &nodes).starts_with(name))
                            .map(|(_, node)| node)
                            .unwrap_or_else(|| {
                                panic!(
                                    "missing primary action {name:?} in {language:?} {section:?}: {labels:#?}"
                                )
                            });
                        (name, node)
                    }
                    Section::Diagnostics | Section::About => {
                        let node = update
                            .nodes
                            .iter()
                            .map(|(_, node)| node)
                            .find(|node| {
                                node.role() == egui::accesskit::Role::Heading
                                    && node.level() == Some(1)
                            })
                            .unwrap_or_else(|| {
                                panic!("missing page heading in {language:?} {section:?}")
                            });
                        (i18n::text(language, section.text_key()), node)
                    }
                };
                let bounds = primary
                    .bounds()
                    .unwrap_or_else(|| panic!("primary surface {primary_name:?} has no bounds"));
                assert!(
                    bounds.y0 < f64::from(screen_size.y) && bounds.y1 > 0.0,
                    "primary surface {primary_name:?} starts below the maximum-zoom viewport in {language:?} {section:?}: {bounds:?}"
                );
            }

            let mut variable_app = accessibility_fixture(language);
            variable_app.variable_editor =
                Some(VariableEditor::new(VariableScope::Global, "echo", language));
            assert_modal_accessibility_at_size(
                &mut variable_app,
                screen_size,
                i18n::text(language, TextKey::AddVariableTitle),
                &[TextKey::SaveVariable, TextKey::Cancel]
                    .map(|key| i18n::text(language, key).to_owned()),
            );

            let mut form_app = accessibility_fixture(language);
            form_app.form_field_editor = Some(FormFieldEditor {
                original_name: None,
                name: "field".into(),
                field: FormField::default(),
            });
            assert_modal_accessibility_at_size(
                &mut form_app,
                screen_size,
                i18n::text(language, TextKey::FormFieldTitle),
                &[TextKey::SaveField, TextKey::Cancel]
                    .map(|key| i18n::text(language, key).to_owned()),
            );

            let mut conflict_app = accessibility_fixture(language);
            let conflict = conflict_fixture();
            conflict_app.conflict_dialog = Some(ConflictDialog {
                target: ConflictTarget::Match(0),
                choices: vec![ResolutionChoice::Local; conflict.plan.conflicts.len()],
                conflict,
            });
            assert_modal_accessibility_at_size(
                &mut conflict_app,
                screen_size,
                i18n::text(language, TextKey::ConflictTitle),
                &[TextKey::MergeAndSave, TextKey::Cancel]
                    .map(|key| i18n::text(language, key).to_owned()),
            );
        }
    }

    #[test]
    fn supported_scale_checkpoints_keep_every_primary_view_horizontally_exposed() {
        for scale in [0.8, 1.0, 1.5, 2.0] {
            let screen_size = screen_size_at_scale(scale);
            let percentage = scale * 100.0;
            for language in Language::ALL {
                for section in [
                    Section::Library,
                    Section::Profiles,
                    Section::Globals,
                    Section::Diagnostics,
                    Section::Settings,
                    Section::About,
                ] {
                    let mut app = accessibility_fixture(language);
                    app.section = section;
                    app.files[0].dirty = true;
                    let update = accessibility_update_at_size(&mut app, screen_size);
                    assert_named_horizontal_bounds(&update, screen_size.x);
                    let labels = focus_labels_from_update(&update);
                    assert!(
                        labels.iter().all(|label| !label.is_empty()),
                        "unnamed control at {percentage:.0}% scale in {language:?} {section:?}: {labels:#?}"
                    );
                }
            }
        }
    }

    #[test]
    fn repeated_editor_actions_have_unique_contextual_names() {
        for language in Language::ALL {
            let mut app = accessibility_fixture(language);
            app.section = Section::Library;
            let library = focus_labels(&mut app);
            for (key, target) in [
                (TextKey::Delete, "name"),
                (TextKey::Edit, "name"),
                (TextKey::Delete, "choice"),
                (TextKey::Edit, "choice"),
                (TextKey::Delete, "future"),
                (TextKey::Edit, "future"),
            ] {
                assert!(
                    library.contains(&format!("{}: {target}", i18n::text(language, key))),
                    "missing contextual form action in {language:?}: {library:#?}"
                );
            }

            app.section = Section::Diagnostics;
            let diagnostics = focus_labels(&mut app);
            let open_prefix = format!("{}: ", i18n::text(language, TextKey::Open));
            let open_actions = diagnostics
                .iter()
                .filter(|label| label.starts_with(&open_prefix))
                .collect::<Vec<_>>();
            assert_eq!(open_actions.len(), 2, "{language:?}: {diagnostics:#?}");
            assert_eq!(
                open_actions.iter().copied().collect::<HashSet<_>>().len(),
                open_actions.len(),
                "duplicate diagnostic action names in {language:?}: {diagnostics:#?}"
            );

            app.section = Section::Profiles;
            let profiles = focus_labels(&mut app);
            let override_suffix = format!(": {}", i18n::text(language, TextKey::Override));
            let overrides = profiles
                .iter()
                .filter(|label| label.ends_with(&override_suffix))
                .collect::<Vec<_>>();
            assert!(overrides.len() >= 8, "{language:?}: {profiles:#?}");
            assert_eq!(
                overrides.iter().copied().collect::<HashSet<_>>().len(),
                overrides.len(),
                "duplicate profile override names in {language:?}: {profiles:#?}"
            );
        }
    }

    #[test]
    fn scale_slider_and_value_editor_share_the_visible_accessible_name() {
        for language in Language::ALL {
            let mut app = accessibility_fixture(language);
            app.section = Section::Settings;
            let labels = focus_labels(&mut app);
            let scale_label = i18n::text(language, TextKey::UiScale);
            assert_eq!(
                labels.iter().filter(|label| *label == scale_label).count(),
                2,
                "slider and percentage editor must both be named in {language:?}: {labels:#?}"
            );
        }
    }

    #[test]
    fn every_dialog_blocks_global_navigation_and_editor_shortcuts() {
        for dialog_kind in 0..8 {
            let mut app = accessibility_fixture(Language::English);
            match dialog_kind {
                0 => app.new_file_dialog = true,
                1 => app.new_config_dialog = true,
                2 => {
                    app.variable_editor = Some(VariableEditor::new(
                        VariableScope::Global,
                        "echo",
                        Language::English,
                    ));
                }
                3 => {
                    app.form_field_editor = Some(FormFieldEditor {
                        original_name: None,
                        name: "field".into(),
                        field: FormField::default(),
                    });
                }
                4 => app.pending_delete = Some(PendingDelete::Snippet),
                5 => {
                    app.pending_restore = Some(PendingRestore {
                        relative_path: PathBuf::from("match/base.yml"),
                        backup_path: PathBuf::from("backup/base.yml"),
                        timestamp: "2026-08-16T12:00:00Z".into(),
                    });
                }
                6 => {
                    let conflict = conflict_fixture();
                    app.conflict_dialog = Some(ConflictDialog {
                        target: ConflictTarget::Match(0),
                        choices: vec![ResolutionChoice::Local; conflict.plan.conflicts.len()],
                        conflict,
                    });
                }
                7 => app.confirm_close = true,
                _ => unreachable!(),
            }
            assert!(app.has_open_modal());

            let original_snippet_count = app.files[0].document.matches.len();
            let modifiers = egui::Modifiers::COMMAND;
            let mut input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1_440.0, 900.0),
                )),
                ..Default::default()
            };
            for key in [Key::N, Key::Num5] {
                input.events.push(egui::Event::Key {
                    key,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers,
                });
            }

            let context = egui::Context::default();
            let mut output = context.run_ui(input, |ui| app.render(ui));
            output.textures_delta.clear();

            assert_eq!(
                app.files[0].document.matches.len(),
                original_snippet_count,
                "dialog {dialog_kind} allowed Cmd/Ctrl+N to mutate the editor"
            );
            assert_eq!(
                app.section,
                Section::Library,
                "dialog {dialog_kind} allowed Cmd/Ctrl+5 to navigate behind it"
            );
        }
    }

    #[test]
    fn application_dialogs_are_named_grouped_and_actionable() {
        for language in Language::ALL {
            let mut app = accessibility_fixture(language);
            app.new_file_dialog = true;
            assert_modal_accessibility(
                &mut app,
                i18n::text(language, TextKey::NewMatchFileTitle),
                &[TextKey::FileName, TextKey::Create, TextKey::Cancel]
                    .map(|key| i18n::text(language, key).to_owned()),
            );

            let mut app = accessibility_fixture(language);
            app.new_config_dialog = true;
            assert_modal_accessibility(
                &mut app,
                i18n::text(language, TextKey::NewProfileTitle),
                &[TextKey::FileName, TextKey::Create, TextKey::Cancel]
                    .map(|key| i18n::text(language, key).to_owned()),
            );

            let mut app = accessibility_fixture(language);
            app.variable_editor =
                Some(VariableEditor::new(VariableScope::Global, "echo", language));
            assert_modal_accessibility(
                &mut app,
                i18n::text(language, TextKey::AddVariableTitle),
                &[
                    TextKey::VariableName,
                    TextKey::Kind,
                    TextKey::FixedValue,
                    TextKey::Dependencies,
                    TextKey::SaveVariable,
                    TextKey::Cancel,
                ]
                .map(|key| i18n::text(language, key).to_owned()),
            );

            let mut app = accessibility_fixture(language);
            app.form_field_editor = Some(FormFieldEditor {
                original_name: None,
                name: "field".into(),
                field: FormField::default(),
            });
            assert_modal_accessibility(
                &mut app,
                i18n::text(language, TextKey::FormFieldTitle),
                &[
                    TextKey::FieldName,
                    TextKey::InputType,
                    TextKey::InitialValue,
                    TextKey::SaveField,
                    TextKey::Cancel,
                ]
                .map(|key| i18n::text(language, key).to_owned()),
            );

            let mut app = accessibility_fixture(language);
            app.pending_delete = Some(PendingDelete::Snippet);
            assert_modal_accessibility(
                &mut app,
                i18n::text(language, TextKey::DeleteConfirmationTitle),
                &[TextKey::ConfirmDelete, TextKey::Cancel]
                    .map(|key| i18n::text(language, key).to_owned()),
            );

            let mut app = accessibility_fixture(language);
            app.pending_restore = Some(PendingRestore {
                relative_path: PathBuf::from("match/base.yml"),
                backup_path: PathBuf::from("backup/base.yml"),
                timestamp: "2026-08-16T12:00:00Z".into(),
            });
            assert_modal_accessibility(
                &mut app,
                i18n::text(language, TextKey::RestoreHistoryTitle),
                &[TextKey::BackupAndRestore, TextKey::Cancel]
                    .map(|key| i18n::text(language, key).to_owned()),
            );

            let mut app = accessibility_fixture(language);
            let conflict = conflict_fixture();
            app.conflict_dialog = Some(ConflictDialog {
                target: ConflictTarget::Match(0),
                choices: vec![ResolutionChoice::Local; conflict.plan.conflicts.len()],
                conflict,
            });
            let conflict_path = "matches[0].replace";
            assert_modal_accessibility(
                &mut app,
                i18n::text(language, TextKey::ConflictTitle),
                &[
                    format!(
                        "{}: {conflict_path}",
                        i18n::text(language, TextKey::UseLocal)
                    ),
                    format!(
                        "{}: {conflict_path}",
                        i18n::text(language, TextKey::UseDisk)
                    ),
                    i18n::text(language, TextKey::MergeAndSave).to_owned(),
                    i18n::text(language, TextKey::Cancel).to_owned(),
                ],
            );

            let mut app = accessibility_fixture(language);
            app.confirm_close = true;
            assert_modal_accessibility(
                &mut app,
                i18n::text(language, TextKey::UnsavedChangesTitle),
                &[TextKey::DiscardAndExit, TextKey::ReturnToEditor]
                    .map(|key| i18n::text(language, key).to_owned()),
            );
        }
    }

    #[test]
    fn dialogs_and_operation_messages_expose_accessible_semantics() {
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut output = context.run_ui(Default::default(), |ui| {
            let _ = snippet_card(
                ui,
                true,
                "Snippet title",
                ":trigger",
                "Replacement preview",
                "Plain text",
            );
            show_modal(ui, "test-dialog", "Test dialog", |ui| {
                let _ = ui.button("Confirm");
            });
            message_bar(
                ui,
                &Message {
                    kind: MessageKind::Error,
                    text: "Test error".into(),
                },
            );
        });
        output.textures_delta.clear();
        let update = output
            .platform_output
            .accesskit_update
            .expect("accessibility should be enabled");

        let dialog = update
            .nodes
            .iter()
            .map(|(_, node)| node)
            .find(|node| node.role() == egui::accesskit::Role::Dialog)
            .expect("dialog node");
        assert_eq!(dialog.label(), Some("Test dialog"));
        assert!(dialog.is_modal());

        let live_error = update
            .nodes
            .iter()
            .map(|(_, node)| node)
            .find(|node| node.live() == Some(egui::accesskit::Live::Assertive))
            .expect("assertive error announcement");
        assert_eq!(live_error.value(), Some("Test error"));

        let snippet = update
            .nodes
            .iter()
            .map(|(_, node)| node)
            .find(|node| {
                node.label() == Some("Snippet title. :trigger. Plain text. Replacement preview")
            })
            .expect("named snippet button");
        assert_eq!(snippet.role(), egui::accesskit::Role::Button);
        assert_eq!(snippet.toggled(), Some(egui::accesskit::Toggled::True));
    }
}
