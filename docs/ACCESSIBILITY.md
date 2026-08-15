# Accessibility and localization

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

The search input has an explicit accessible label relationship. Buttons use visible action names rather than icon-only labels. Dialogs have stable titles, visible field labels, and keyboard-focusable actions.

## Display

- UI scale is persisted and adjustable from 80% to 200% without restarting.
- The bundled palette is checked in unit tests for WCAG AA normal-text contrast on the primary surfaces.
- Japanese-capable system fonts are selected without changing the user's operating-system font scaling.

## Localization

The typed localization catalog starts with Japanese and English. Language selection is persisted. Navigation, save/reload state, search, connection state, and accessibility settings use catalog keys; new user-facing surfaces should add both strings and remain covered by the catalog completeness test.

## Manual release audit

Before a stable release, exercise the shared focus sequence with Narrator on Windows, VoiceOver on macOS, and Orca on Linux. Verify the search label, navigation buttons, editor tabs, profile controls, merge choices, confirmation dialogs, and 200% scaling. Record platform-specific defects in this repository only.
