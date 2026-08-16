# Release signing and notarization

[English](SIGNING.md) | [日本語](ja/SIGNING.md)

The release workflow always builds on Windows, macOS, and Linux. Signing is
optional: forks and local maintainers can publish unsigned builds without
configuring credentials, and every GitHub Release records the status of each
platform explicitly.

No contributor or automation should accept a certificate agreement, Apple
Developer agreement, or other legal terms on another person's behalf.

## Windows Authenticode

Configure both repository secrets, or configure neither:

- `WINDOWS_CERTIFICATE_BASE64`: Base64-encoded PKCS#12/PFX signing certificate
- `WINDOWS_CERTIFICATE_PASSWORD`: Password for the PFX file

When both are available, the workflow decodes the certificate into the runner's
temporary directory, signs the application executable before packaging, signs
the generated `.exe` and `.msi` installers, and verifies every signature with
the Windows SDK `signtool`. It additionally requires PowerShell's Authenticode
status to be valid, checks that every artifact's signer thumbprint matches the
configured PFX, and rejects a missing timestamp certificate. Signatures use
SHA-256 and an RFC 3161 timestamp. The temporary certificate file is removed
and the in-memory certificate is disposed in a `finally` block.

## macOS Developer ID and notarization

Configure all six repository secrets, or configure none:

- `APPLE_CERTIFICATE`: Base64-encoded Developer ID Application PKCS#12 file
- `APPLE_CERTIFICATE_PASSWORD`: Password for the PKCS#12 file
- `APPLE_SIGNING_IDENTITY`: Full Developer ID Application identity
- `APPLE_ID`: Apple ID used by `notarytool`
- `APPLE_PASSWORD`: App-specific password for that Apple ID
- `APPLE_TEAM_ID`: Apple Developer Team ID

With all six present, `cargo-packager` imports the certificate, signs the app
with hardened runtime and a secure timestamp, submits the app for notarization,
and staples the accepted ticket. The workflow then submits the generated DMG
with `notarytool`, requires an `Accepted` response, staples it, and validates
the app and disk image tickets. It also verifies that both artifacts use the
configured Developer ID identity and Team ID, and that the app signature has
the hardened-runtime flag. Both signatures must expose secure timestamps, the
app must pass the active Gatekeeper policy through `spctl`, and `hdiutil`
verifies the disk image both before submission and after stapling. The packaging
command requests both `app` and `dmg` so that the signed app remains available
for these checks; only the DMG is published as a release asset.

## Failure behavior and release notes

- A partially configured platform fails before packaging instead of silently
  producing an artifact with an ambiguous status.
- Missing credentials are allowed and produce explicitly unsigned artifacts.
- Package jobs upload a small status record. The publish job requires one from
  every platform and appends all three records to the release notes.
- `SHA256SUMS` covers every published package regardless of signing status.
- Packaging jobs receive a read-only `GITHUB_TOKEN`; only the tag-gated publish
  job receives `contents: write` to create or update this repository's release.
- Checkout never persists GitHub credentials into the local Git configuration;
  the publish command receives its tag-gated token only through `GH_TOKEN`.

Signing credentials can be checked without revealing values:

```sh
gh secret list --repo hjosugi/espanso-gui
```

The credential-backed paths can only be fully validated after the repository
owner provisions certificates and Apple notarization credentials.

## Automated acceptance coverage

`tests/release_workflow.rs` parses the release workflow and verifies that
package jobs remain read-only, unsigned builds do not depend on
`SIGNING_ENABLED`, partial credential sets fail, and every signing action is
gated on a complete set. It also verifies that the Windows executable and both
installer formats enter the signer, that the retained macOS app and DMG enter
the distribution checks, and that tagged publishing attaches all three signing
status records plus `SHA256SUMS`. Separate script checks require signer identity,
timestamps, hardened runtime, Gatekeeper, stapling, and disk-image integrity.
These tests validate workflow structure without requiring or exposing real
credentials; a genuine signed release is still the final acceptance gate.
