# Contributing

Thank you for improving Espanso GUI.

## Scope and upstream boundary

This repository is an independent compatibility tool. Keep all project discussion, bug reports, and pull requests in `hjosugi/espanso-gui`.

Do not contact Espanso maintainers on this project's behalf. Do not open Espanso issues or pull requests, post on Espanso discussions, or request upstream changes for Espanso GUI. Public Espanso documentation and source may be consulted read-only when compatibility research is necessary.

## Development

Use stable Rust 1.95 or later.

```sh
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

Changes that write configuration files must include tests for path containment, conflict handling, and recovery. New structured YAML fields should use `#[serde(flatten)]` or equivalent handling so unknown options are not silently discarded.

## Pull requests

- Keep changes focused.
- Add or update tests.
- Explain user-data migration or compatibility effects.
- Do not include real personal Espanso configurations in fixtures.
- UI copy should remain understandable without prior YAML knowledge.

By contributing, you agree that your contribution is licensed under the MIT License.
