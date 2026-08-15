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

## Known limitations in 0.1

- Structured saves normalize YAML layout and do not preserve comment positions. Automatic backups retain the original.
- HTML has a source preview rather than a complete browser rendering because the final rendering depends on the target application.
- Rich-text injection behavior is ultimately determined by Espanso and the target application.
- Platform packages are unsigned until project signing identities are configured.
- App-specific Espanso configuration profiles are visible only through external files; their visual editor is planned separately.
