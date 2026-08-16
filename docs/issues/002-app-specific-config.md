---
title: Add visual app-specific Espanso configuration profiles
labels: enhancement, configuration
---

[English](002-app-specific-config.md) | [日本語](../ja/issues/002-app-specific-config.md)

Add a visual editor for files under `config/`, including application filters, enable/disable behavior, injection backend, delays, search shortcuts, and form size limits.

The implementation must use public documented Espanso configuration only and remain fully inside this repository.

## Current status (2026-08-16)

Implemented. The profile workspace edits `default.yml` and app-specific `config/*.yml` files through visual controls for filters, enablement, injection, delays, shortcuts, clipboard/status behavior, and form limits. Unknown fields round-trip, Raw YAML remains available, and all writes use validation, concurrent-change detection, and automatic history.
