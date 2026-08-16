---
title: Add sync-aware conflict resolution and history
labels: enhancement, data-safety
---

[English](004-sync-conflicts.md) | [日本語](../ja/issues/004-sync-conflicts.md)

Build a local three-way merge experience for Espanso configuration directories synchronized through iCloud Drive, Dropbox, OneDrive, Google Drive, Git, or network folders.

No cloud account integration or telemetry is required. Conflict resolution must remain local and show field-level differences before writing.

## Current status (2026-08-16)

Implemented locally without cloud APIs. Before a save, the application compares the loaded base, local edit, and current disk content; independent changes merge automatically, while overlapping fields receive explicit local/disk choices. The latest disk version is backed up before the resolved file is written, and saved history can be restored with another pre-restore backup.
