# Publishing handoff

This repository is locally prepared for `hjosugi/espanso-gui` but has not been written to GitHub from the build session.

## Intended remote state

- Repository: `hjosugi/espanso-gui`
- Visibility: public
- Default branch: `main`
- Initial tag and release: `v0.1.0`
- Issue tracker: enabled
- Discussions/wiki/projects: optional; not required for 0.1.0
- Espanso upstream: no writes, issues, pull requests, discussions, or contact

## Publish sequence for an authorized target-repository session

1. Confirm the session working repository resolves to `hjosugi/espanso-gui`.
2. Review the complete diff, license, independence notice, and release limitations.
3. Create the public repository without a generated README or license.
4. Push local `main` to `hjosugi/espanso-gui:main`.
5. Apply repository topics and enable Issues.
6. Create the six issues from `docs/issues/` in this repository only.
7. Wait for the cross-platform CI matrix to pass.
8. Create and push the annotated `v0.1.0` tag only after CI succeeds.
9. Let the release workflow build artifacts and create the GitHub Release.
10. Verify checksums, artifact downloads, and the independence notice. Do not contact Espanso upstream.

The checked-in helper performs the exact repository-only mutations in two deliberate stages:

```sh
./scripts/publish-hjosugi-repository.sh repository
# Wait for the main-branch CI matrix to pass.
./scripts/publish-hjosugi-repository.sh release
```

The helper refuses a dirty tree, a branch other than `main`, a mismatched `origin`, or a release tag whose commit has not passed the latest `main` CI run. It is intentionally fixed to `hjosugi/espanso-gui` and contains no Espanso-upstream operations.

If release signing identities are unavailable, publish 0.1.0 as an explicitly unsigned preview or keep the Release in draft until the owner decides.
