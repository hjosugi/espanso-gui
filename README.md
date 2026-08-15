# Espanso GUI

<img src="icons/icon.png" alt="Espanso GUI icon" width="128" height="128">

**Espanso GUI** is a polished, cross-platform visual editor for [Espanso](https://espanso.org/), written entirely in Rust.

EspansoのYAML設定を直接覚えなくても、スニペット、変数、フォーム、Markdown、HTML、画像を視覚的に作成・管理できます。Windows、macOS、Linuxで同じコードベースを使用します。

> [!IMPORTANT]
> This is an independent, unofficial project. It is not affiliated with, endorsed by, or supported by Espanso or its maintainers. Please report Espanso GUI problems to this repository only—not to the Espanso project.

## Features

- Three-pane snippet library with full-text search, labels, aliases, regex triggers, duplication, and safe deletion
- Plain text, Markdown with live preview, HTML, image, and interactive form match editors
- Point-and-click variable builder:
  - date/time, offsets, locale, and timezone
  - clipboard and fixed values
  - choice dialogs and random choices
  - shell commands and scripts
  - forms and global-variable references
- Easy `{{variable}}` insertion into snippet content
- Form field builder for text, multiline, choice, and list controls
- Word-boundary, case propagation, injection mode, search terms, and cursor hints
- Global variables, unresolved-variable diagnostics, and duplicate-trigger warnings
- Raw YAML editor for advanced and future Espanso options
- Existing unknown YAML fields are retained when structured data is loaded and saved
- Optimistic concurrency checks to prevent overwriting edits from another program
- Automatic pre-save backups and recoverable file deletion
- CSV import/export and full configuration snapshots
- Espanso detection, status, start, stop, and restart controls
- Espanso Hub package files are read-only; individual matches can be copied to a user file
- No telemetry, cloud service, account, or background network access

## Install

Prebuilt packages will be attached to each GitHub Release:

- Windows: installer/package produced by `cargo-packager`
- macOS: application bundle/disk image produced by `cargo-packager`
- Linux: platform packages such as AppImage and Debian package

The first unsigned preview may trigger an operating-system security warning. Signing and notarization status is documented in each release.

Espanso itself is not bundled. Install and start Espanso separately, then launch Espanso GUI.

## Build from source

Requirements:

- Rust 1.95 or later
- Native build tools for your platform
- On Linux, the normal X11/Wayland development libraries required by `winit`

```sh
git clone https://github.com/hjosugi/espanso-gui.git
cd espanso-gui
cargo run --release
```

Run the quality suite:

```sh
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

Build native installers:

```sh
cargo install cargo-packager --locked
cargo packager --release
```

## Data safety

Espanso GUI edits only the configuration folder selected in the app. Before overwriting a file, it verifies that the on-disk content still matches the version originally loaded.

- Automatic backups: `<espanso-config>/.espanso-gui/backups/`
- Recoverable deletions: `<espanso-config>/.espanso-gui/trash/`
- Manual snapshots: destination selected by the user

Structured editing preserves unknown YAML keys, but it may reformat the file and reposition or remove comments. The original commented file remains in the automatic backup. Use the Raw YAML tab when comment layout must remain untouched.

Shell and script variables execute local commands when their Espanso trigger runs. Espanso GUI never executes those commands while editing; nevertheless, only save commands you have reviewed and trust.

## Supported Espanso syntax

The editor currently models the Espanso 2 match format, including `trigger`, `triggers`, `regex`, `replace`, `markdown`, `html`, `image_path`, `form`, `form_fields`, `vars`, `global_vars`, word boundaries, case propagation, labels, search terms, force mode, and Markdown paragraph behavior. Unknown fields are kept for forward compatibility.

See [Compatibility](docs/COMPATIBILITY.md) for details and known limitations.

## Project boundaries

Espanso GUI reads public Espanso documentation to remain compatible but does not modify, fork, contact, open issues against, or submit pull requests to the Espanso project. Development and support stay entirely in `hjosugi/espanso-gui`.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). The architecture is described in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), and icon usage in [docs/BRANDING.md](docs/BRANDING.md).

## License

MIT. See [LICENSE](LICENSE).

“Espanso” is used descriptively to identify configuration compatibility. All trademarks belong to their respective owners.
