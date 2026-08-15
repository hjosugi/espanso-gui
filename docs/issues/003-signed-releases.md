---
title: Configure Windows signing and macOS notarization
labels: release, security, platform
---

Add optional secret-backed signing to the release workflow.

Acceptance criteria:

- Unsigned forks still build without secrets.
- Windows artifacts are Authenticode-signed when credentials are present.
- macOS application and disk image are signed, hardened, submitted for notarization, and stapled.
- Release notes state signing status and checksums remain attached.
