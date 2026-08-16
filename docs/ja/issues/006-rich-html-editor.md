---
title: 任意で利用できるHTMLリッチテキストエディターを追加する
labels: enhancement, rich-text
---

[English](../../issues/006-rich-html-editor.md) | [日本語](006-rich-html-editor.md)

bold、italic、link、list、color、heading、imageなど一般的なHTML snippet向けのformatting-oriented composerを追加します。source modeを残し、予測可能でportableなHTMLを生成します。

previewはscriptを実行せず、remote active contentを自動取得してはいけません。

## 現在の状況（2026-08-16）

実装済みです。composerから強調、見出し、link、番号付き／番号なしlist、色、local imageの予測可能でportableな断片を挿入でき、source modeも直接編集できます。preview表示前にactive elementとremote resource URLを除去し、scriptやremote contentがpreviewへ渡らないことをtestで検証します。
