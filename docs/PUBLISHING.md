# Publishing

The public repository is `hjosugi/espanso-gui`. Releases are produced entirely by GitHub Actions from version tags.

## Intended remote state

- Repository: `hjosugi/espanso-gui`
- Visibility: public
- Default branch: `main`
- Release tags: `v<semantic-version>` matching `Cargo.toml`
- Issue tracker: enabled
- Discussions/wiki/projects: optional
- Espanso upstream: no writes, issues, pull requests, discussions, or contact

## Publish sequence

1. Confirm the session working repository resolves to `hjosugi/espanso-gui`.
2. Review the complete diff, license, independence notice, and release limitations.
3. Update `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`, AppStream metadata, and `docs/releases/v<version>.md` together.
4. Push `main` to `hjosugi/espanso-gui:main`.
5. Wait for the cross-platform CI matrix to pass.
6. Dispatch the Release workflow on `main` as a packaging rehearsal and wait for all three package jobs.
7. Create and push the annotated version tag only after both workflows succeed.
8. Let the tag-triggered release workflow build artifacts and create the GitHub Release.
9. Download the assets, verify `SHA256SUMS`, and check the signing-status section in the published notes.
10. Update this repository's issues. Do not contact Espanso upstream.

The original 0.1.0 bootstrap helper remains available for repository recovery, but routine releases use the workflow above.

```sh
./scripts/publish-hjosugi-repository.sh repository
# Wait for the main-branch CI matrix to pass.
./scripts/publish-hjosugi-repository.sh release
```

The helper refuses a dirty tree, a branch other than `main`, a mismatched `origin`, or a release tag whose commit has not passed the latest `main` CI run. It is intentionally fixed to `hjosugi/espanso-gui` and contains no Espanso-upstream operations.

If release signing identities are unavailable, the workflow publishes explicitly unsigned artifacts. Credential setup and the precise signed path are documented in [SIGNING.md](SIGNING.md).
