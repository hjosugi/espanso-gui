# Architecture

Espanso GUI is a single native Rust application built with `eframe` and `egui`.

## Components

- `src/app.rs`: application state, navigation, editors, dialogs, validation presentation, and user-initiated actions.
- `src/conflict.rs`: recursive three-way YAML merge and field-level conflict selection.
- `src/i18n.rs`: typed Japanese and English UI catalogs.
- `src/lossless_yaml.rs`: sequence and mapping patching that retains unchanged source bytes.
- `src/model.rs`: serde model for Espanso match files, content types, variables, form fields, and semantic diagnostics.
- `src/storage.rs`: bounded filesystem access for match and config profiles, YAML loading, atomic save, hash-based concurrency detection, backup history, restoration, recoverable deletion, snapshots, and CSV conversion.
- `src/espanso.rs`: minimal process boundary for Espanso discovery and explicit `start`, `stop`, `restart`, and `status` actions.
- `src/theme.rs`: visual system and cross-platform Japanese-capable system-font discovery.

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
