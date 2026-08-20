---
title: Complete keyboard navigation, screen-reader audit, and localization framework
labels: accessibility, i18n, enhancement
---

[English](005-accessibility-localization.md) | [日本語](../ja/issues/005-accessibility-localization.md)

Audit all editors and dialogs on Windows, macOS, and Linux. Add stable focus order, accessible names, contrast verification, scalable UI text, and a localization catalog beginning with Japanese and English.

## Acceptance criteria

- Every primary view and modal has a stable keyboard order, useful accessible names, and no unnamed focusable application controls.
- Navigation, tabs, list rows, and mode selectors expose an unambiguous selected state with readable foreground/background contrast.
- English and Japanese cover all application text, operation messages, diagnostics, repository documentation, and contributor-facing templates.
- The shared visual system uses no more than four text sizes, keeps supporting text at 18 points or larger, and supplies shared spacing, padding, control-size, focus, and semantic-color tokens.
- Primary flows remain usable at 80%, 100%, 150%, and 200% app scale without hiding required actions.
- Narrator on Windows, VoiceOver on macOS, and Orca on Linux each pass the complete native audit in [the release audit runbook](../ACCESSIBILITY_AUDIT.md).
- `cargo fmt --check`, `cargo test --all-targets`, and `cargo clippy --all-targets -- -D warnings` pass on the final commit.

## Current verification

Automated bilingual view/dialog, focus-order, pointer-input, responsive-layout, typography, spacing, selection, and WCAG contrast regressions are implemented. They now cover four scale checkpoints, maximum-zoom first-fold actions, persistent button boundaries for mode/tab choices, a pointer-selectable regular-expression mode, stable non-color selection markers, localized search-hint fit in the compact list, padded single-line and multi-line editors, readable control/menu/modal geometry, visible scroll handles, and a long file list that cannot displace footer actions. The repository also provides one canonical disposable audit fixture with every supported content and variable kind, diagnostic cases, profiles, and unknown YAML values; an automated test keeps that matrix loadable and lossless. A Linux diagnostic pass found and fixed native accessibility-tree defects, but it did not complete the runbook with a human listener. Windows Narrator and macOS VoiceOver remain untested. Keep this issue open until all three native audit rows pass; do not infer platform completion from the automated suite.
