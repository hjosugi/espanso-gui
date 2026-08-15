# Repository instructions

- This is an independent, unofficial Espanso-compatible GUI. Never contact, file an issue against, open a pull request against, or otherwise write to the Espanso upstream project on behalf of this repository.
- Espanso documentation and public source may be used read-only for compatibility research.
- Keep implementation code in Rust. Do not introduce a JavaScript or TypeScript frontend without an explicit repository-owner decision.
- Support Windows, macOS, and Linux in shared code. Platform-specific behavior must have a clear fallback.
- Treat Espanso configuration as user data: validate paths, detect concurrent changes, back up before overwrite, and prefer recoverable deletion.
- Preserve unknown YAML fields. Call out any operation that reformats or cannot losslessly retain comments.
- Run `cargo fmt --check`, `cargo test --all-targets`, and `cargo clippy --all-targets -- -D warnings` before handoff.
