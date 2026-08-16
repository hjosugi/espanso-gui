# Accessibility release audit

[English](ACCESSIBILITY_AUDIT.md) | [日本語](ja/ACCESSIBILITY_AUDIT.md)

Use this runbook to complete the native assistive-technology portion of [issue #9](https://github.com/hjosugi/espanso-gui/issues/9). Run it against a release build on each operating system; an audit on one platform does not stand in for the other two.

## Test record

Fill one row for every release candidate. `Pass` means every flow below was exercised on the named native platform. Link defects only to this repository.

| Platform | OS version | Assistive technology | App version/commit | Result | Tester/date | Defects |
| --- | --- | --- | --- | --- | --- | --- |
| Windows |  | Narrator |  | Not run |  |  |
| macOS |  | VoiceOver |  | Not run |  |  |
| Linux |  | Orca |  | Not run |  |  |

## Preparation

1. Build or install the same release candidate that will be published.
2. Start with a disposable Espanso configuration containing:
   - plain, Markdown, HTML, image, and form matches;
   - local and global variables of every supported type;
   - `default.yml` and one app-specific profile;
   - a deliberate duplicate trigger and undefined variable for diagnostics;
   - one unknown YAML field and one unknown form-field `type` to verify preservation.
3. Record the OS display scaling, desktop/session type, assistive-technology version, app version, and commit in the table above.
4. Run the automated baseline before the native audit:

   ```sh
   cargo fmt --check
   cargo test --all-targets
   cargo clippy --all-targets -- -D warnings
   ```

## Keyboard and focus sequence

Perform each flow without a pointer. At every step, confirm the focus indicator is visible, the order follows the visual/declaration order, and no control is skipped or visited twice unexpectedly.

- Use <kbd>Cmd/Ctrl</kbd>+<kbd>1</kbd> through <kbd>5</kbd> to open snippets, profiles, globals, diagnostics, and settings.
- Use <kbd>Cmd/Ctrl</kbd>+<kbd>F</kbd> to focus snippet search, then type and clear a query.
- Use <kbd>Cmd/Ctrl</kbd>+<kbd>N</kbd> to create a snippet and <kbd>Cmd/Ctrl</kbd>+<kbd>S</kbd> to save it.
- Traverse the file list, snippet list, content/variables/options/Raw YAML tabs, toolbar, preview, and destructive actions.
- Traverse the profile list, visual/Raw YAML switch, every optional override, boolean selector, and numeric input.
- Traverse global variables, diagnostics, settings/history, About, and the disconnected-workspace screen.
- Open each dialog: new match file, new profile, variable editor, form-field editor, delete confirmation, restore confirmation, merge conflict, and unsaved-exit confirmation.
- While a dialog is open, verify background controls and global shortcuts do not activate. Confirm <kbd>Esc</kbd> closes only the topmost dialog and focus returns to a sensible control.

## Screen-reader names, roles, values, and state

With Narrator, VoiceOver, or Orca enabled, verify that speech and the accessibility inspector expose:

- a useful name for search, snippet content, image path, form content, Raw YAML, variable parameters, form fields, profile settings, language, and UI scale;
- the selected state for navigation, editor tabs, visual/Raw YAML switches, and option controls;
- the current value and enabled/disabled state for text, numeric, checkbox, slider, and combo-box controls;
- visible dialog titles and action names, with focus contained inside the active modal;
- polite announcement of operation results and immediate announcement of errors without moving focus;
- diagnostic severity and message text in the selected language;
- `Unsupported type: <name>` (or its Japanese equivalent) for an unknown form-field type without altering that YAML value.

Do not accept a result that announces only generic roles such as “edit” or “button” where a visible field/action name exists.

## Scaling and layout

Repeat the primary snippet, profile, variable, form, conflict, and settings flows at 80%, 100%, 150%, and 200% app scale.

- Text must remain readable without overlap or truncation that hides meaning.
- Focused controls and dialog actions must remain reachable by keyboard.
- Long content must scroll; it must not push actions permanently outside the window.
- Verify both Japanese and English at 200%, including long profile descriptions and modal copy.
- Repeat the 200% pass with the OS text/display scaling used by the tester; record that value.

## Localization and data preservation

1. In Japanese, visit every screen and dialog and confirm there is no unintended English-only prose apart from product names, YAML keys, command names, paths, and format examples.
2. Switch to English without restarting and repeat the same traversal.
3. Create a new snippet in each language and verify its initial display name and replacement text follow the selected language.
4. Trigger diagnostics, operation messages, variable summaries, form summaries, and conflict text in each language.
5. Open and close a visual editor without changing the fixture's unknown YAML field or unknown form-field type, save another intentional edit, and verify both unknown values remain present.

## Contrast and visual-system evidence

The unit test `theme::tests::text_palette_meets_wcag_aa_contrast_on_primary_surfaces` enforces a 4.5:1 minimum for the normal text palette—including muted placeholder/supporting, informational, warning, and error colors—on the paper, panel, sidebar, recessed input, inactive-control, and composited badge/callout surfaces, and for explicit foreground colors on accent and destructive actions. It also enforces 3:1 contrast for interactive borders and the two-point keyboard-focus outline across the actual widget-state surfaces. `theme::tests::secondary_text_never_falls_back_to_tiny_default_sizes` fixes the smallest reading style at 18 points and limits the type scale to four sizes. `theme::tests::controls_and_insets_keep_comfortable_shared_dimensions` keeps standard controls at least 40×48 points with 16×12-point button padding; contextual row actions, editor tools, single-line and multi-line fields, check/radio icons, menus, modals, and persistent scroll handles use the shared comfortable geometry instead of compact exceptions. Whole-view and dialog AccessKit bounds tests then require every focusable application control to remain at least 48 points high. `ui_components::tests::selected_snippet_cards_use_the_contrast_tested_text_color` keeps all text on the tinted selected-card surface on the 4.5:1-tested ink token while accent remains available through the frame and edge indicator. `theme::tests::semantic_palette_follows_the_selected_theme` verifies light/dark palette selection and the strong selected-control foreground/background pair. `theme::tests::ui_code_uses_design_tokens_instead_of_visual_literals` prevents UI code from bypassing shared typography, spacing, padding, tint, stroke, and geometry values, while `app::tests::application_styles_use_semantic_theme_colors` rejects RGB values embedded outside the theme. Responsive-layout tests prove that 200% zoom and even viewports smaller than the supported minimum cannot make modal minimums exceed the logical viewport. Application-level AccessKit tests render all six primary views in both languages, resolve effective names through label relationships, verify the common navigation prefix, require unique contextual row actions, and inspect all eight dialog variants as named modal groups with named descendant controls. During the native audit, inspect the focus indicator's full shape, disabled controls, warning/error callouts, selection states, centered long-form pages, and text rendered by the operating system; record any failure that the automated checks cannot cover.

On 2026-08-16, Linux/X11 diagnostic passes were run against isolated D-Bus and AT-SPI registries with a disposable configuration. The initial release-build bridge smoke exposed 54 application nodes; search, navigation, file/profile selections, editor fields, and tabs had useful names and states, selected controls were reported as pressed, and no unnamed focusable non-window control remained.

A follow-up development-build pass used Orca 50.2 with AT-SPI 2.60.6. Orca generated speech for the application name and the names, roles, selected states, values, and enabled states of controls across the snippet, profile, global-variable, diagnostic, settings, and About views. AT-SPI focus actions succeeded for every enabled control examined, including all six custom snippet cards. New-file, new-profile, variable, form-field, and delete-confirmation dialogs kept focus inside the modal. The pass found and fixed three native-tree defects: snippet cards that announced but rejected assistive-technology focus, repeated generic profile-override names, and repeated row actions such as `Open`, `Edit`, and `Delete` without their target context.

The subsequent application-tree regression pass found and fixed two additional structural defects: the editable percentage subcontrol of UI scale had no direct name in the raw AccessKit tree, and the synthetic dialog node had a title and modal state but did not own its fields and actions. Both now have direct label relationships and are covered in the bilingual full-view/dialog tests. Page and section titles now expose heading roles with levels one and two rather than appearing only as generic text runs. The primary-view matrix now runs at 80%, 100%, 150%, and 200% scale; the large variable, form, and conflict dialogs also render in the 540×360-point logical viewport that represents 200% zoom at the minimum supported window. Named non-text-run node bounds are checked against each horizontal viewport; this detected and fixed clipped form/variable row actions, profile mode selectors, and the expansion-type control. A vertical first-fold assertion then detected and fixed compact headers that pushed the initial Settings control out of view. An input-level regression drives navigation and editor shortcut events through every dialog state and verifies that modal ownership leaves the underlying section and snippet collection unchanged.

A release-build visual pass used a disposable Japanese configuration. At a 1440×900-point logical viewport, the 32-point page title, 24-point list/section headings, 20-point body and controls, and 18-point supporting copy remained distinct and readable; list cards fill one consistent column, navigation rows, fields, and action buttons share stable left edges, and selected rows retain high-contrast foregrounds without overlap.

A follow-up development-build visual pass repeated the connected editor at the 1440×720 minimum-height checkpoint and at 200% scale. At 100%, the bounded file list alone scrolled while Add file, version, Settings, and About remained separated; at 200%, both compact selectors fit on one row, the localized search placeholder remained fully visible, selected controls retained their check marks, and wrapped tabs plus the first editor surface stayed inside the initial viewport. Light and dark passes also confirmed the 16×12-point editor insets and persistent high-contrast scroll handles. The repository release binary was then rebuilt and passed an isolated launch smoke test; the full optimized test matrix also passed.

This remains diagnostic evidence rather than a `Pass` in the Linux row: the nested X11 harness could not reliably synthesize repeated Tab navigation, and it did not exercise the complete release-build flow matrix with a human listener. Windows Narrator and macOS VoiceOver also remain untested.

## Exit criteria

Issue #9's native audit is complete only when all three platform rows are `Pass`, every defect found during the run is fixed or explicitly deferred with rationale, and the full automated baseline succeeds on the final commit.
