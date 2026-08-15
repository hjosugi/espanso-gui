# Architecture

Espanso GUI is a single native Rust application built with `eframe` and `egui`.

## Components

- `src/app.rs`: application state, navigation, editors, dialogs, validation presentation, and user-initiated actions.
- `src/model.rs`: serde model for Espanso match files, content types, variables, form fields, and semantic diagnostics.
- `src/storage.rs`: bounded filesystem access, YAML loading, atomic save, hash-based conflict detection, backups, recoverable deletion, snapshots, and CSV conversion.
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
          backup old file + atomic write
                       ▼
               Espanso reloads YAML
```

The app does not link to, embed, fork, or patch Espanso. Compatibility is through documented YAML and the local `espanso` executable only.

## Safety invariants

1. Structured writes are restricted to `.yml` and `.yaml` files beneath the selected `match` directory.
2. Parent traversal, absolute relative paths, and symlink escapes are rejected.
3. A changed on-disk hash blocks saving until the user reloads.
4. Existing files are copied to an app-owned backup before overwrite.
5. File deletion moves data to an app-owned recovery directory.
6. Hub package files are read-only in the visual editor.
7. Shell and script variable bodies are stored but never executed by Espanso GUI.

## YAML compatibility

Known fields are typed. `#[serde(flatten)]` maps retain unknown keys at the file, match, variable, and form-field levels. Serde serialization may normalize formatting and comments; the original byte-for-byte file is retained in the pre-save backup.
