# Changelog

[English](CHANGELOG.md) | [日本語](CHANGELOG.ja.md)

All notable changes to Espanso GUI are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow semantic versioning.

## [Unreleased]

## [0.3.0] - 2026-08-16

### Changed

- Expanded Japanese and English localization across the primary library, full snippet and profile editors, diagnostics, settings, history, onboarding, operation-feedback, variable-builder, form-builder, and modal-dialog surfaces.
- Localized the initial new-snippet template instead of embedding Japanese defaults in the language-neutral model.
- Localized new-variable examples, untitled fallbacks, date-format presets, conflict summaries, and storage/validation errors instead of embedding Japanese presentation text in core models.
- Localized empty Espanso command results and external-open failures, and report documentation-link launch failures instead of discarding them.
- Linked editor, profile, variable, form, Raw YAML, language, scale, image-preview, snippet-card, dialog, and live-status semantics into the AccessKit tree; added platform-correct shortcut hints and expanded WCAG AA palette/focus checks.
- Fixed custom snippet cards rejecting assistive-technology focus, and gave repeated profile overrides and row actions unique target-aware screen-reader names verified with Orca on Linux.
- Gave both UI-scale controls direct accessible names, made modal dialog nodes structurally own their fields and actions, and added bilingual whole-view AccessKit regression coverage for focus names/order and every dialog variant; all eight dialog states also have input tests proving that global navigation and editor shortcuts cannot fire behind them.
- Added compact navigation, narrower list panels, stacked field layouts, and viewport-bounded scrollable dialogs for high UI/OS scaling and narrow logical viewports; the wide conflict editor no longer forces a 760×560-point minimum at 200% scale, and maximum-zoom tree tests keep all primary views plus variable, form, and conflict actions exposed in both languages.
- Reduced only the compact empty-state top offset so useful setup content reaches the initial fold sooner at 200%, and made long configuration paths selectable and wrap-safe through one shared component.
- Consolidated colors and tint opacity, typography, spacing and padding, control/border geometry, panel/list/field/modal dimensions, filled and contextual actions, selection-list alignment, modal action alignment, and long-form page widths into reusable visual-system helpers; text now uses only 32/24/20/18-point roles, page titles and section headings have a clear hierarchy, navigation rows consume one consistent available width, and source-level tests reject visual literals outside the token module.
- Increased the global reading scale, control height, list-row height, panel padding, field widths, and navigation/list widths instead of applying isolated per-screen overrides; all single-line and multi-line editors now share comfortable 16×12-point internal padding.
- Replaced the remaining compact toolkit defaults with four-point text leading, 24-point check/radio icons, 12-point icon gaps, wider sliders/selectors, 24-point modal margins, and 12-point menu margins.
- Replaced hidden two-point floating scroll handles with persistent 12-point solid scroll bars and 48-point minimum handles.
- Increased muted placeholder/supporting-text contrast and now test it against the actual recessed input and inactive-control surfaces, not only page backgrounds.
- Routed every single-line field through one shared 16×12-point inset treatment and added whole-view plus dialog-tree regressions requiring every focusable application control to retain a 48-point hit height.
- Replaced the remaining compact editor and row-action buttons with the shared 20-point, 40×48-point control treatment; status badges now use the shared 16×12-point inset as well.
- Added named UI-node bounds regressions at the default viewport and at 200% zoom, then fixed overflowing form actions, variable actions, profile mode selectors, and expansion-type controls with a shared responsive detail/action layout.
- Made compact snippet/profile lists yield enough width for the detail editor at 200% zoom, and added a cached, contrast-safe Raw YAML highlighter for keys, quoted values, and comments.
- Shortened the localized search placeholder and validate its actual font width against the compact 200%-zoom input, preventing ellipsis in both languages.
- Replaced the multi-row compact navigation strip with one section selector, moved Reload out of the wrapping compact top bar, and verified that the selected section plus every view's first primary surface remain visible and exposed at 200% zoom.
- Replaced the compact top bar's color-only connection dot with localized visible status text, with a font-width regression proving both connection states fit in Japanese and English at the 200% minimum-width checkpoint.
- Moved responsive workspace navigation into its own presentation module with typed actions and bounded long-file-list scrolling; regression coverage keeps Add file, version, Settings, and About separated and inside the minimum-height viewport.
- Moved the responsive top bar into its own side-effect-free presentation module, returning typed save, reload, and Espanso-restart actions to the application state layer.
- Fixed lossless profile patching so a `#` inside a quoted value is never mistaken for an inline comment, a changed block-scalar field cannot leave stale continuation lines behind, and CRLF or a missing final line ending stays intact; the quote-aware scanner is shared with Raw YAML highlighting and comment-warning detection.
- Rejected full-backup destinations inside the active Espanso configuration tree to prevent recursive snapshots, and made timestamp collisions choose a distinct snapshot directory instead of overwriting an earlier export.
- Split settings presentation from filesystem/process side effects, and disable backup, CSV import/export, and file deletion when no valid writable target is available instead of accepting an action that can only fail or be ignored.
- Fixed the normal/regular-expression trigger selector so the chosen mode persists immediately; existing literal triggers are escaped and carried into regex mode instead of disappearing, every mode and tab retains a visible button boundary, and selected modes include a non-color check marker with stable accessibility names.
- Added persisted system, light, and dark appearances with contrast-tested semantic palettes; strengthened selected rows and tabs with a high-contrast accent fill, bold list labels, non-color check markers, and a selected-card edge indicator while keeping accessible names stable; kept navigation labels and shortcuts on one aligned row, and hid file creation until a workspace is connected.
- Raised disabled-control opacity from the toolkit default while retaining a visible inactive treatment, and test the composited disabled text against its actual inactive surface at WCAG AA contrast.
- Exposed page titles and section titles as level-one and level-two accessibility headings while retaining their 32-point and 24-point visual hierarchy.
- Refactored diagnostics into language-neutral model events and moved their presentation into the typed localization catalog.
- Refactored storage validation failures into typed issues rendered by the selected language.
- Isolated HTML fragment generation and the non-executing text preview from application UI state, with focused safety tests.
- Preserved unknown form-field types in visual editors until the user explicitly selects a supported replacement type.
- Added a native Narrator, VoiceOver, and Orca release-audit matrix with keyboard, screen-reader, scaling, localization, and preservation checks.
- Replaced floating editor dialogs with input-blocking modal layers so global shortcuts and background controls cannot fire behind a dialog.
- Restricted release-package jobs to a read-only GitHub token, granted release-write access only to the tag-gated publish job, retained the signed macOS app for post-package validation, and verified artifact identity, Team ID, secure timestamps, hardened runtime, Gatekeeper assessment, disk-image integrity, and notarization tickets.
- Strengthened Windows release verification to require every Authenticode signature to be valid, match the configured certificate thumbprint, include an RFC 3161 timestamp certificate, and pass all-signature verification.
- Disabled persisted checkout credentials and added Rust regression coverage for release permissions, unsigned packaging, complete credential gates, Windows binary/installer signing, macOS app retention and distribution verification, signing-status notes, and attached checksums.
- Localized the README, user and developer documentation, issue specifications, release notes, GitHub contribution templates, and AppStream metadata in Japanese and English, with regression coverage for missing documentation pairs.
- Expanded non-empty snippet searches across every loaded match file, identified each result's source path, added localized result counts and a recoverable no-results state, and exposed count updates as named polite accessibility announcements.
- Added persistent YAML-order, name, and trigger sorting for snippet lists without modifying the underlying Espanso files; older saved preferences retain YAML order by default.
- Presented Espanso-native `search_terms` as an all-file tag filter with localized, explicitly labelled controls and occurrence counts, avoiding GUI-only fields in user YAML.
- Reworked the disconnected workspace into a responsive bilingual onboarding view with Espanso installation guidance, an official-documentation action, the exact configuration target, and a clear non-overwrite explanation for initialization; shared external-link failure handling with About.
- Extracted cross-file search, native tag aggregation, sorting, and list-entry projection from application rendering into a side-effect-free `snippet_library` module.
- Extracted persisted preferences, settings presentation, trigger-mode and toolbar behavior, profile controls, variable-type controls, and responsive labelled fields into focused modules instead of continuing to grow the application-state module.
- Added a bilingual aText/Dash UX benchmark that records covered workflows, intentional product boundaries, remaining gaps, and the release UX gate.

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

[Unreleased]: https://github.com/hjosugi/espanso-gui/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/hjosugi/espanso-gui/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/hjosugi/espanso-gui/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/hjosugi/espanso-gui/releases/tag/v0.1.0
