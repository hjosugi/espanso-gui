# Release signing and notarization

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
the Windows SDK `signtool`. Signatures use SHA-256 and an RFC 3161 timestamp.
The temporary certificate file is removed in a `finally` block.

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
the app and disk image tickets.

## Failure behavior and release notes

- A partially configured platform fails before packaging instead of silently
  producing an artifact with an ambiguous status.
- Missing credentials are allowed and produce explicitly unsigned artifacts.
- Package jobs upload a small status record. The publish job requires one from
  every platform and appends all three records to the release notes.
- `SHA256SUMS` covers every published package regardless of signing status.

Signing credentials can be checked without revealing values:

```sh
gh secret list --repo hjosugi/espanso-gui
```

The credential-backed paths can only be fully validated after the repository
owner provisions certificates and Apple notarization credentials.
