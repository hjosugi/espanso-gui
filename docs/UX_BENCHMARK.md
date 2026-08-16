# UX benchmark

[English](UX_BENCHMARK.md) | [日本語](ja/UX_BENCHMARK.md)

This document records the product-level comparison requested during the 0.2 usability review. It is a directional benchmark, not a claim that the products have identical scope. The comparison was reviewed on 2026-08-16 against the official [aText product page](https://www.trankynam.com/atext/), [aText documentation](https://www.trankynam.com/atext/doc/), [Dash product page](https://kapeli.com/dash), and [Dash user guide](https://kapeli.com/dash_guide).

## Baseline comparison

| Workflow | Espanso GUI | aText | Dash | UX decision |
| --- | --- | --- | --- | --- |
| Find and reuse content | Cross-file full-text search, tag filtering, and persistent sorting | Search window and grouped snippets | Search, tags, and snippet management | Keep search permanently visible and keyboard-focusable; never limit results to the selected YAML file. |
| Create dynamic snippets | Espanso variables, dates, clipboard, choices, forms, scripts, and cursor placement | Fields, date/time, clipboard, scripting, and editable expansion fields | Placeholders, date/time, clipboard, and scripting | Prefer a visual builder, but always retain Raw YAML for syntax that is not yet modeled. |
| Recognize the current item | Strong navigation/file selection, full-width cards, selected-card indicator, persistent button boundaries for tabs/modes, and explicit check-marked selection | Native selected rows and groups | Native selected rows and tags | Selection must remain recognizable without hover and must not rely on a low-contrast tint alone. |
| Read and operate the editor | Four-size 32/24/20/18-point type scale, standard 40×48-point controls, 16×12-point input/button insets, persistent scroll bars, adaptive list/detail widths, 80–200% scaling, and syntax-colored Raw YAML | Native desktop layout | Native macOS layout with code syntax highlighting | Supporting copy stays at least 18 points; contextual and toolbar actions do not use compact-button exceptions; the editor retains a practical width at maximum zoom. |
| Rich content | Plain text, Markdown preview, safe HTML composer/source, local images, and forms | Formatted text, images, and fields | Code-oriented snippets and placeholders | Keep active HTML and remote resources out of preview; make the source representation inspectable. |
| Protect user data | Concurrent-change detection, three-way merge, automatic history, recoverable deletion, snapshots, and unknown-YAML preservation | Backup/export and folder-based sync workflows | Dropbox/iCloud sync workflows | Data preservation remains a differentiator; sync must not be added without explicit conflict handling and recoverability. |
| Cross-platform use | Shared Rust UI for Windows, macOS, and Linux | Windows and macOS | macOS | Keep shared behavior first and require an explicit fallback for platform-specific features. |

## Remaining gaps

- Espanso GUI has no built-in cloud sync. Users can place configuration in their own synchronized workflow, but first-class sync needs an explicit security, conflict, and recovery design.
- Raw YAML highlights keys, quoted values, and comments with the tested light/dark semantic palette. It is intentionally a YAML editor rather than a multi-language code browser or Dash-style documentation set.
- Espanso GUI does not ship a curated built-in snippet library comparable to aText's built-in examples. It edits the user's Espanso configuration and read-only Espanso Hub package files instead.
- Native Narrator, VoiceOver, and Orca release audits are still required; automated accessibility coverage does not replace them.

## Release UX gate

A change does not pass this benchmark merely because it renders. Verify that the primary action is obvious, the selected item is unmistakable, body and supporting text remain readable, controls share a stable alignment and target size, keyboard order matches visual order, empty/error states explain the next action, and user data remains recoverable. Record platform-specific exceptions in the compatibility document.
