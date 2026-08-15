---
title: Preserve YAML comments during structured edits
labels: enhancement, data-safety, yaml
---

Structured editing currently retains unknown keys but normalizes YAML and may remove or move comments. Replace full serde reserialization with a lossless syntax-tree patcher for known fields.

Acceptance criteria:

- Comments outside and inside unchanged matches remain byte-for-byte stable.
- Editing one match does not reformat unrelated matches.
- Existing automatic backup behavior remains in place.
- Fixtures cover block scalars, anchors, quoted values, and comments at multiple nesting levels.
