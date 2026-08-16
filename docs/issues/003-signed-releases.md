---
title: Configure Windows signing and macOS notarization
labels: release, security, platform
---

[English](003-signed-releases.md) | [日本語](../ja/issues/003-signed-releases.md)

Add optional secret-backed signing to the release workflow.

Acceptance criteria:

- Unsigned forks still build without secrets.
- Windows artifacts are Authenticode-signed when credentials are present.
- macOS application and disk image are signed, hardened, submitted for notarization, and stapled.
- Release notes state signing status and checksums remain attached.

## Current verification

The release workflow, signing scripts, credential gates, post-signature verification, checksum publication, and bilingual signing-status notes are implemented and covered by repository tests. The latest published release remains unsigned because the required Windows certificate and Apple signing/notarization secrets are not configured. Keep this issue open until a tagged release is produced with real credentials and both platform signatures are verified on the published artifacts.
