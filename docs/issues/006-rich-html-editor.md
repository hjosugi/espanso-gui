---
title: Add an optional visual HTML rich-text composer
labels: enhancement, rich-text
---

[English](006-rich-html-editor.md) | [日本語](../ja/issues/006-rich-html-editor.md)

Add a formatting-oriented composer for common HTML snippets—bold, italic, links, lists, colors, headings, and images—while keeping source mode and generating predictable portable HTML.

The preview must not execute scripts or load remote active content automatically.

## Current status (2026-08-16)

Implemented. The composer inserts predictable portable fragments for emphasis, headings, links, ordered/unordered lists, colors, and local images, while source mode remains directly editable. Preview sanitization strips active elements and remote-resource URLs before rendering; tests verify that scripts and remote content are never exposed to the preview.
