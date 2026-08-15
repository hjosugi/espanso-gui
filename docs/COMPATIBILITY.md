# Compatibility

## Operating systems

| Platform | Build target | Release package |
| --- | --- | --- |
| Windows | x86_64 | Native installer/package |
| macOS | Apple Silicon in CI; source supports Intel | `.app` / disk image |
| Linux | x86_64 | AppImage and distribution package where supported |

The CI matrix compiles and tests on Windows, macOS, and Linux.

## Espanso

Espanso GUI targets the Espanso 2 configuration format documented at `espanso.org/docs`. It discovers custom installations through `espanso path` and supports default configuration locations as a fallback.

Supported visual match content:

- `replace`
- `markdown` and `paragraph`
- `html`
- `image_path`
- shorthand `form` and `form_fields`

Supported visual variables:

- `date`, including `format`, `offset`, `locale`, and `tz`
- `clipboard`
- `echo`
- `random`
- `choice`
- `shell`
- `script`
- `form`
- `global`

Unknown or newly introduced fields remain in the data model and can be changed through Raw YAML.

Supported visual app-specific configuration:

- title, executable, class, and operating-system filters
- enable/disable and injection backend selection
- word, key, clipboard, and paste delays
- paste and search shortcuts
- form size, clipboard preservation, icon, and notification options

## Known limitations in 0.2

- Unchanged structured YAML items and profile fields are preserved byte for byte, but an edited fragment is serialized and may change its own formatting. Automatic backups retain the prior file.
- HTML preview is deliberately text-only and never runs scripts or fetches remote resources; final rich-text rendering depends on Espanso and the target application.
- Rich-text injection behavior is ultimately determined by Espanso and the target application.
- Windows and macOS packages remain unsigned when optional project signing identities are not configured; every release states the actual status.
- AccessKit integration and automated contrast checks are shared across platforms, while Narrator, VoiceOver, and Orca still require a manual release audit.
