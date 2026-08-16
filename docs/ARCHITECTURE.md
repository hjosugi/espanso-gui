# Architecture

[English](ARCHITECTURE.md) | [日本語](ja/ARCHITECTURE.md)

Espanso GUI is a single native Rust application built with `eframe` and `egui`.

## Components

- `src/app.rs`: application state, screen composition, dialogs, validation presentation, and user-initiated actions.
- `src/conflict.rs`: recursive three-way YAML merge and field-level conflict selection.
- `src/i18n.rs`: typed Japanese and English UI catalogs.
- `src/lossless_yaml.rs`: sequence and mapping patching that retains unchanged source bytes.
- `src/model.rs`: serde model for Espanso match files, content types, variables, form fields, and semantic diagnostics.
- `src/html_editor.rs`: safe HTML composer fragments and inert text-preview conversion.
- `src/navigation.rs`: responsive workspace navigation, selected-file presentation, bounded file-list layout, and typed navigation actions.
- `src/preferences.rs`: backward-compatible persisted application preferences and UI-scale parsing/formatting.
- `src/profile_editor.rs`: app/default profile controls, override semantics, and responsive field presentation.
- `src/settings_editor.rs`: appearance, language, UI-scale, and keyboard-help presentation.
- `src/snippet_editor.rs`: snippet trigger-mode selection and content-specific editor toolbar.
- `src/snippet_library.rs`: side-effect-free cross-file search, native tag aggregation, presentation entries, and stable display sorting.
- `src/storage.rs`: bounded filesystem access for match and config profiles, YAML loading, atomic save, hash-based concurrency detection, backup history, restoration, recoverable deletion, snapshots, and CSV conversion.
- `src/espanso.rs`: minimal process boundary for Espanso discovery and explicit `start`, `stop`, `restart`, and `status` actions.
- `src/theme.rs`: visual system and cross-platform Japanese-capable system-font discovery.
- `src/top_bar.rs`: responsive application identity/status/actions presentation and typed save, reload, and restart actions.
- `src/ui_components.rs`: reusable responsive panels, labelled-field layouts, modal shells, action buttons, status surfaces, snippet cards, empty states, and live-message presentation.
- `src/variable_editor.rs`: visual parameter editors and summaries for every supported Espanso variable type.
- `src/yaml_editor.rs`: shared, cached Raw YAML text editor and contrast-safe syntax highlighting.
- `src/yaml_syntax.rs`: shared quote-aware YAML comment-boundary scanning for source highlighting and lossless patches.

## Data flow

```text
Espanso match/*.yml
        │ read + hash
        ▼
WorkspaceFile ── serde model ── visual editors
        │                           │
        │ raw YAML                  │ structured mutation
        └──────────────┬────────────┘
                       ▼
          validate + compare disk hash
                       │
          changed? three-way merge dialog
                       │
      backup latest disk state + atomic write
                       ▼
               Espanso reloads YAML
```

The app does not link to, embed, fork, or patch Espanso. Compatibility is through documented YAML and the local `espanso` executable only.

## Visual system

This is a native `egui` application, so it has no CSS layer. `src/theme.rs` is the equivalent design-token source of truth: it owns semantic foreground, surface, state and tint colors; four-level typography; four-level gap spacing; four-point-grid padding; stroke, control, list, field, panel, modal, window, and content geometry; and the supported UI-scale range. Application code names those tokens instead of embedding visual literals, and source-level regression tests enforce that boundary.

Shared application helpers own level-one display titles, level-two section headings, filled primary/destructive buttons, target-aware repeated actions, responsive detail/action rows, left-aligned selection lists, right-aligned modal actions, structurally grouped and viewport-bounded modal shells, and centered scrollable long-form panels. Standalone selected options pair the high-contrast selected surface with a check mark, so mode is not conveyed by color alone while the accessible name remains stable. Detail/action rows keep actions right-aligned at wide widths and move them to a wrapped row below their content at narrow widths. Navigation controls use the panel's actual available width instead of separate fixed values. This keeps typography hierarchy, selection treatment, readable action text, row and dialog alignment, and responsive page width consistent without repeating widget styling in each editor. The application renderer also has a side-effect-free UI entrypoint used by whole-view AccessKit tests without constructing a native window.

## Safety invariants

1. Structured writes are restricted to `.yml` and `.yaml` files beneath the selected `match` and `config` directories.
2. Parent traversal, absolute relative paths, and symlink escapes are rejected.
3. A changed on-disk hash opens a three-way merge; a second hash check prevents racing the confirmed merge.
4. Existing files are copied to persistent app-owned history before overwrite or restoration.
5. File deletion moves data to an app-owned recovery directory.
6. Hub package files are read-only in the visual editor.
7. Shell and script variable bodies are stored but never executed by Espanso GUI.

## YAML compatibility

Known fields are typed. `#[serde(flatten)]` maps retain unknown keys at the file, match, variable, form-field, and profile levels. Structured saves serialize only changed sequence items or mapping values and splice them into the original source, so unrelated comments and formatting remain byte for byte. An edited fragment can be normalized; the previous complete file is retained in history.
