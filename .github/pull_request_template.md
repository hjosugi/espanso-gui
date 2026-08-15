## Summary

Describe the user-visible outcome.

## Validation

- [ ] `cargo fmt --check`
- [ ] `cargo test --all-targets`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] I considered Windows, macOS, and Linux behavior.
- [ ] I preserved unknown YAML fields and protected existing user data.
- [ ] This change does not contact or require a write to the Espanso upstream project.

## Data safety and compatibility

Describe configuration writes, migrations, backups, and Espanso syntax affected by this change.
