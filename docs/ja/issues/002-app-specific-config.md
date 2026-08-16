---
title: Espansoのアプリ別設定を視覚的に編集する
labels: enhancement, configuration
---

[English](../../issues/002-app-specific-config.md) | [日本語](002-app-specific-config.md)

application filter、enable／disable、injection backend、delay、search shortcut、form size limitを含む、`config/`配下のファイル向けvisual editorを追加します。

実装は公開されたEspanso設定だけを使用し、このrepository内で完結しなければなりません。

## 現在の状況（2026-08-16）

実装済みです。profile workspaceから`default.yml`とアプリ別`config/*.yml`を開き、filter、有効状態、注入方式、遅延、shortcut、clipboard／status動作、form上限を視覚的に編集できます。未知fieldを往復保持し、Raw YAMLも利用でき、全書き込みでvalidation、同時変更検出、自動履歴を使います。
