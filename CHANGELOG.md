# Changelog

All notable changes to Espanso GUI are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow semantic versioning.

## [Unreleased]

## [0.2.0] - 2026-08-15

### Added

- Visual app-specific configuration profiles with filters, backends, delays, shortcuts, and form sizing.
- Local three-way YAML conflict resolution with field-level local/disk choices and restorable history.
- Safe visual HTML composer with deterministic markup and a non-executing preview.
- Japanese and English UI, keyboard navigation, accessible search labeling, and 80–200% scaling.
- Optional Windows Authenticode and macOS Developer ID signing/notarization with per-release status reporting.

### Changed

- Structured match, global-variable, and profile edits now preserve all unchanged YAML bytes, including comments, anchors, block scalars, quoting, and unknown fields.
- Updated `sha2` to 0.11 with explicit lowercase digest encoding.

## [0.1.0] - 2026-08-15

### Added

- Native Rust GUI for Windows, macOS, and Linux.
- Visual editors for text, Markdown, HTML, images, and forms.
- Guided local and global variable builder.
- YAML diagnostics, Raw YAML editing, and forward-compatible unknown-field retention.
- Optimistic concurrency checks, atomic writes, automatic backups, and recoverable deletion.
- CSV import/export, manual snapshots, and Espanso service controls.
- Cross-platform CI and release packaging workflows.

[Unreleased]: https://github.com/hjosugi/espanso-gui/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/hjosugi/espanso-gui/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/hjosugi/espanso-gui/releases/tag/v0.1.0
