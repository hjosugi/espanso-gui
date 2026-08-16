---
title: Preserve YAML comments during structured edits
labels: enhancement, data-safety, yaml
---

[English](001-lossless-yaml-comments.md) | [日本語](../ja/issues/001-lossless-yaml-comments.md)

Structured editing currently retains unknown keys but normalizes YAML and may remove or move comments. Replace full serde reserialization with a lossless syntax-tree patcher for known fields.

Acceptance criteria:

- Comments outside and inside unchanged matches remain byte-for-byte stable.
- Editing one match does not reformat unrelated matches.
- Existing automatic backup behavior remains in place.
- Fixtures cover block scalars, anchors, quoted values, and comments at multiple nesting levels.

## Current status (2026-08-16)

Implemented. Structured match/profile edits patch only changed YAML fragments; unchanged matches and fields remain byte-for-byte stable. Automatic pre-write history remains active, and regression fixtures cover comments, block scalars, anchors, quoting, `#` characters inside quoted profile values, complete replacement of a changed block-scalar profile field, Windows CRLF, and files without a final line ending. A deliberately edited fragment can still be reformatted, so the UI discloses that limit and Raw YAML remains available for exact source edits.
