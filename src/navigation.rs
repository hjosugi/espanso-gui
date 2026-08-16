use crate::i18n::{self, Language, TextKey};
use crate::storage::WorkspaceFile;
use crate::theme;
use crate::ui_components::{compact_layout, selection_list_row};
use eframe::egui::{self, Button, ComboBox, Frame, Margin, RichText, ScrollArea, Ui};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Section {
    Library,
    Profiles,
    Globals,
    Diagnostics,
    Settings,
    About,
}

impl Section {
    pub(crate) fn text_key(self) -> TextKey {
        match self {
            Self::Library => TextKey::Snippets,
            Self::Profiles => TextKey::Profiles,
            Self::Globals => TextKey::Globals,
            Self::Diagnostics => TextKey::Diagnostics,
            Self::Settings => TextKey::SettingsNav,
            Self::About => TextKey::About,
        }
    }

    fn uses_match_file(self) -> bool {
        matches!(self, Self::Library | Self::Globals | Self::Diagnostics)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NavigationAction {
    SelectFile(usize),
    AddFile,
    Reload,
}

pub(crate) fn show(
    ui: &mut Ui,
    section: &mut Section,
    files: &[WorkspaceFile],
    selected_file: usize,
    language: Language,
) -> Option<NavigationAction> {
    if compact_layout(ui.ctx().content_rect().width()) {
        compact_navigation(ui, section, files, selected_file, language)
    } else {
        wide_navigation(ui, section, files, selected_file, language)
    }
}

fn compact_navigation(
    ui: &mut Ui,
    section: &mut Section,
    files: &[WorkspaceFile],
    selected_file: usize,
    language: Language,
) -> Option<NavigationAction> {
    let mut action = None;
    egui::Panel::top("navigation-compact")
        .resizable(false)
        .frame(
            Frame::new()
                .fill(theme::palette(ui).sidebar)
                .inner_margin(Margin::symmetric(theme::PADDING_MD, theme::PADDING_COMPACT)),
        )
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.x = f32::from(theme::PADDING_COMPACT);
            ui.horizontal_wrapped(|ui| {
                let section_name = i18n::text(language, TextKey::Workspace);
                let section_value = i18n::text(language, section.text_key());
                let response = ComboBox::from_id_salt("compact-section")
                    .width(theme::COMPACT_SECTION_SELECTOR_WIDTH)
                    .selected_text(section_value)
                    .show_ui(ui, |ui| {
                        for destination in [
                            Section::Library,
                            Section::Profiles,
                            Section::Globals,
                            Section::Diagnostics,
                            Section::Settings,
                            Section::About,
                        ] {
                            ui.selectable_value(
                                section,
                                destination,
                                i18n::text(language, destination.text_key()),
                            );
                        }
                        ui.separator();
                        if ui.button(i18n::text(language, TextKey::Reload)).clicked() {
                            action = Some(NavigationAction::Reload);
                            ui.close();
                        }
                    });
                response.response.widget_info(|| {
                    let mut info =
                        egui::WidgetInfo::labeled(egui::WidgetType::ComboBox, true, section_name);
                    info.current_text_value = Some(section_value.to_owned());
                    info
                });
                if section.uses_match_file() && !files.is_empty() {
                    let file_name = i18n::text(language, TextKey::MatchFiles);
                    let mut next_file = selected_file;
                    let selected_text = files
                        .get(next_file)
                        .map(|file| {
                            let dirty = if file.dirty { " •" } else { "" };
                            format!("{}{dirty}", file.display_name)
                        })
                        .unwrap_or_else(|| i18n::text(language, TextKey::NoFileTitle).into());
                    let accessible_value = selected_text.clone();
                    let mut add_file = false;
                    let response = ComboBox::from_id_salt("compact-file-list")
                        .width(theme::COMPACT_FILE_SELECTOR_WIDTH)
                        .selected_text(selected_text)
                        .show_ui(ui, |ui| {
                            for (index, file) in files.iter().enumerate() {
                                let package = if file.is_package {
                                    format!(" · {}", i18n::text(language, TextKey::Package))
                                } else {
                                    String::new()
                                };
                                ui.selectable_value(
                                    &mut next_file,
                                    index,
                                    format!(
                                        "{} · {}{}",
                                        file.display_name,
                                        i18n::snippet_count(language, file.snippet_count()),
                                        package
                                    ),
                                );
                            }
                            if *section == Section::Library {
                                ui.separator();
                                if ui.button(i18n::text(language, TextKey::AddFile)).clicked() {
                                    add_file = true;
                                    ui.close();
                                }
                            }
                        });
                    response.response.widget_info(|| {
                        let mut info =
                            egui::WidgetInfo::labeled(egui::WidgetType::ComboBox, true, file_name);
                        info.current_text_value = Some(accessible_value.clone());
                        info
                    });
                    if next_file != selected_file {
                        action = Some(NavigationAction::SelectFile(next_file));
                    }
                    if add_file {
                        action = Some(NavigationAction::AddFile);
                    }
                }
            });
        });
    action
}

fn wide_navigation(
    ui: &mut Ui,
    section: &mut Section,
    files: &[WorkspaceFile],
    selected_file: usize,
    language: Language,
) -> Option<NavigationAction> {
    let mut action = None;
    egui::Panel::left("navigation-wide")
        .exact_size(theme::NAVIGATION_WIDTH)
        .resizable(false)
        .frame(
            Frame::new()
                .fill(theme::palette(ui).sidebar)
                .inner_margin(Margin::same(theme::PADDING_LG)),
        )
        .show(ui, |ui| {
            ui.label(
                RichText::new(i18n::text(language, TextKey::Workspace))
                    .small()
                    .color(theme::palette(ui).muted),
            );
            nav_button(
                ui,
                section,
                Section::Library,
                i18n::text(language, TextKey::Snippets),
                &command_shortcut("1"),
            );
            nav_button(
                ui,
                section,
                Section::Profiles,
                i18n::text(language, TextKey::Profiles),
                &command_shortcut("2"),
            );
            nav_button(
                ui,
                section,
                Section::Globals,
                i18n::text(language, TextKey::Globals),
                &command_shortcut("3"),
            );
            nav_button(
                ui,
                section,
                Section::Diagnostics,
                i18n::text(language, TextKey::Diagnostics),
                &command_shortcut("4"),
            );
            if !files.is_empty() {
                ui.add_space(theme::SPACE_LG);
                ui.label(
                    RichText::new("Espanso")
                        .small()
                        .color(theme::palette(ui).muted),
                );
                ui.add_space(theme::SPACE_XS);

                // Keep the flexible list above both the add-file action and the fixed footer.
                // The extra spacing token is a safety gap verified by the overlap regression.
                let add_file_height = theme::CONTROL_HEIGHT + theme::SPACE_SM;
                let file_list_height = (ui.available_height()
                    - theme::NAVIGATION_FOOTER_HEIGHT
                    - add_file_height
                    - 3.0 * theme::SPACE_SM)
                    .max(0.0);
                ScrollArea::vertical()
                    .id_salt("file-list")
                    .max_height(file_list_height)
                    .min_scrolled_height(theme::SPACE_XS)
                    .show(ui, |ui| {
                        for (index, file) in files.iter().enumerate() {
                            let dirty = if file.dirty { " •" } else { "" };
                            let package = if file.is_package {
                                format!("  {}", i18n::text(language, TextKey::Package))
                            } else {
                                String::new()
                            };
                            let label = format!(
                                "{}{}\n{}{}",
                                file.display_name,
                                dirty,
                                i18n::snippet_count(language, file.snippet_count()),
                                package
                            );
                            if selection_list_row(ui, label, selected_file == index).clicked() {
                                action = Some(NavigationAction::SelectFile(index));
                            }
                        }
                    });

                ui.add_space(theme::SPACE_SM);
                if ui
                    .add_sized(
                        [ui.available_width(), theme::CONTROL_HEIGHT],
                        Button::new(i18n::text(language, TextKey::AddFile)),
                    )
                    .clicked()
                {
                    action = Some(NavigationAction::AddFile);
                }
            }
            let available = ui.available_rect_before_wrap();
            let footer_top =
                (available.bottom() - theme::NAVIGATION_FOOTER_HEIGHT).max(available.top());
            let footer_rect = egui::Rect::from_min_max(
                egui::pos2(available.left(), footer_top),
                available.right_bottom(),
            );
            ui.scope_builder(
                egui::UiBuilder::new()
                    .id_salt("navigation-footer")
                    .max_rect(footer_rect),
                |ui| {
                    ui.spacing_mut().item_spacing.y = f32::from(theme::PADDING_COMPACT);
                    ui.label(
                        RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                            .small()
                            .color(theme::palette(ui).muted),
                    );
                    nav_button(
                        ui,
                        section,
                        Section::Settings,
                        i18n::text(language, TextKey::SettingsNav),
                        &command_shortcut("5"),
                    );
                    nav_button(
                        ui,
                        section,
                        Section::About,
                        i18n::text(language, TextKey::About),
                        "",
                    );
                },
            );
        });
    action
}

fn nav_button(ui: &mut Ui, current: &mut Section, value: Section, label: &str, shortcut: &str) {
    let selected = *current == value;
    let foreground = if selected {
        theme::palette(ui).on_action
    } else {
        theme::palette(ui).ink
    };
    let mut label = RichText::new(label).color(foreground);
    if selected {
        label = label.strong();
    }
    let button = if shortcut.is_empty() {
        Button::new(label).right_text(())
    } else {
        Button::new(label).right_text(RichText::new(shortcut).color(foreground))
    };
    let response = ui.add_sized(
        [ui.available_width(), theme::CONTROL_HEIGHT],
        button
            .truncate()
            .selected(selected)
            .frame(true)
            .frame_when_inactive(selected),
    );
    if response.clicked() {
        *current = value;
    }
}

pub(crate) fn command_shortcut(key: &str) -> String {
    if cfg!(target_os = "macos") {
        format!("⌘{key}")
    } else {
        format!("Ctrl+{key}")
    }
}
