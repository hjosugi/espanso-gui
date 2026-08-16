# Accessibility and localization

[English](ACCESSIBILITY.md) | [日本語](ja/ACCESSIBILITY.md)

Espanso GUI uses the same `egui` widget tree and AccessKit integration on Windows, macOS, and Linux. Native controls keep a deterministic declaration-order focus sequence; no platform-specific focus overrides are used.

## Keyboard map

| Action | Shortcut |
| --- | --- |
| Open snippets, profiles, globals, diagnostics, settings | <kbd>Cmd/Ctrl</kbd>+<kbd>1</kbd> through <kbd>5</kbd> |
| Focus snippet search | <kbd>Cmd/Ctrl</kbd>+<kbd>F</kbd> |
| Save the active match/config file | <kbd>Cmd/Ctrl</kbd>+<kbd>S</kbd> |
| Create a snippet | <kbd>Cmd/Ctrl</kbd>+<kbd>N</kbd> |
| Close the active dialog | <kbd>Esc</kbd> |
| Move through controls | <kbd>Tab</kbd> / <kbd>Shift</kbd>+<kbd>Tab</kbd> |

The search input, snippet cards, snippet content/options, image preview, variable parameters, form fields, profile settings, Raw YAML editors, language selector, and UI scale control have explicit accessible names or label relationships. The UI-scale slider and its editable percentage value are separate named controls, so neither becomes an unnamed focus stop. Non-empty search queries span every loaded match file; localized result counts are named polite live announcements, and an empty result set provides a keyboard-focusable clear action. Snippet cards expose a button role plus their selected state, title, trigger, type, source file for cross-file results, and preview, and use a dedicated interactive node that accepts assistive-technology focus. Buttons use visible action names rather than icon-only labels; repeated row actions add the target field, variable, or diagnostic to their accessible name. Keyboard focus uses a two-point accent outline against the active control surface, including navigation buttons and custom snippet cards. Dialog containers expose a dialog role and stable title and structurally own their named fields and keyboard-focusable actions. Operation results use polite live announcements, while errors are assertive.

Dialogs use egui's modal accessibility layer and input backdrop. While a dialog is open, pointer events and global application shortcuts cannot activate the editor behind it; <kbd>Esc</kbd> is consumed by the topmost dialog only. Automated tree tests render every primary view in Japanese and English, verify the common declaration-order navigation prefix and effective names (including `labelled_by` relationships), and inspect all eight application-dialog variants as modal groups with named descendant controls. The primary-view matrix is repeated at 80%, 100%, 150%, and 200% scale using the minimum-window-equivalent logical viewport; variable, form, and conflict actions are also rendered at the 540×360-point 200% checkpoint. At every checkpoint, each named non-text-run UI node's horizontal bounds must remain inside the viewport; this guards against controls or labelled surfaces that exist in the accessibility tree but are visually cut off. A separate input regression exercises all eight dialog states with navigation and editor shortcut events and proves that the background state remains unchanged.

The disconnected onboarding view is also rendered in both languages at that maximum-zoom viewport. Its setup-guide, configuration-folder, and safe-initialization actions must remain named, focusable, and reachable without a pointer.

## Display

- UI scale is persisted and adjustable from 80% to 200% without restarting.
- At narrow logical widths, including high app or operating-system scaling, navigation becomes one named section selector above the workspace, redundant top-bar actions move into that workspace row, list panels narrow, labelled fields stack, compact empty states reduce only their otherwise decorative top offset, long configuration paths wrap, and modal width and height are capped to the logical viewport. The selector exposes its current section as an accessible value. At 200% zoom the collection panel reserves at least 320 points for the detail editor while retaining its own 200-point minimum, and the first primary surface of every view remains above the initial fold. Even the wide conflict editor scrolls instead of forcing its former fixed 760×560-point minimum outside the window.
- Typography is limited to four sizes: 32-point display titles, 24-point section headings, 20-point body/control/monospace text, and 18-point supporting text. Display and section titles use distinct shared components and expose level-one and level-two heading semantics to assistive technology. The 18-point supporting role replaces egui's smaller secondary-text default. Every single-line and multi-line input uses shared 16×12-point internal padding instead of the compact toolkit default.
- Text receives four points of additional line spacing. Check, radio, and related control icons use a 24-point outer size with a 12-point gap; sliders and selectors use shared readable widths instead of the toolkit's compact defaults. Modal windows reserve 24-point internal margins and menus reserve 12 points.
- Vertical rhythm uses a shared 4/12/16/24-point spacing scale. Insets use the same 4-point grid through shared 8/12/16/24/32/40-point padding tokens. Font, gap, margin, tint, stroke, control, panel, list, field, image-preview, modal, and scale values come from the shared design-token module, with a source-level test rejecting visual literals in UI code. Navigation and file/profile selection rows consume one consistent available width and keep their labels left-aligned, while intentional empty states and their actions remain centered. Wide forms reserve a 340-point label column so common Japanese and English descriptions do not leave an orphaned final word; forms stack vertically below 660 points. The wide file list has a bounded scrolling region so adding many configuration files cannot push its add-file action or footer navigation below the viewport; bounds tests also require the add-file and version surfaces to remain separated.
- Scroll regions reserve a persistent, foreground-colored 12-point solid bar with a minimum 48-point handle instead of hiding a two-point floating handle until hover. This keeps long libraries and editors discoverable and easier to operate with a pointer.
- Settings, diagnostics, global variables, and About use one centered, scrollable content layout with a 1,040-point maximum width; narrow views retain the full available width.
- The bundled text palette, including muted placeholder/supporting text, informational, warning, error, and composited disabled-control colors, is checked in unit tests for WCAG AA normal-text contrast on the paper, panel, sidebar, input, inactive-control, and composited badge/callout surfaces. Interactive borders and the focus outline are checked at 3:1 across their actual widget surfaces.
- Raw YAML keys, quoted values, comments, and ordinary text use that same contrast-tested semantic palette in both appearances. Highlighting is cached and never changes the source bytes.
- System, light, and dark appearances share semantic color roles. Selected controls use a strong accent surface with an explicit contrast-tested foreground; selected snippet cards also carry an accent edge indicator. Filled primary and destructive actions use their own contrast-tested foreground. Compact connection state remains visible as localized text instead of a color-only dot, and localized font-width tests keep it inside the maximum-zoom top bar. Dialog actions share one right-aligned layout.
- The application accessibility root is named `Espanso GUI`; list selection buttons retain their visible label and pressed state in the accessibility tree.
- Japanese-capable system fonts are selected without changing the user's operating-system font scaling.

## Localization

The typed localization catalog starts with Japanese and English. Language selection is persisted. Navigation, library and profile lists, the full snippet and profile editors, diagnostics, settings, history, empty states, connection state, operation feedback, and all modal dialogs use catalog keys. Variable and form builders localize their types, field labels, help text, validation errors, generated placeholders, and summary text. Semantic diagnostics are language-neutral in the model and rendered by the selected catalog. New user-facing surfaces should add both strings and remain covered by the catalog completeness test.

Public repository documentation uses English source documents with mirrored Japanese versions under `docs/ja/` and `.ja.md` files at the repository root. Every pair has a visible language switch. GitHub issue and pull-request forms use bilingual copy in the same form. Integration tests reject a missing language pair, missing switch, broken issue-spec front matter, invalid template YAML, or missing Japanese AppStream metadata.

## Manual release audit

Before a stable release, exercise the shared focus sequence with Narrator on Windows, VoiceOver on macOS, and Orca on Linux. Verify the search label, navigation buttons, editor tabs, profile controls, merge choices, confirmation dialogs, and 200% scaling. Record platform-specific defects in this repository only. Use [ACCESSIBILITY_AUDIT.md](ACCESSIBILITY_AUDIT.md) for the required platform matrix, test flows, evidence, and exit criteria.
