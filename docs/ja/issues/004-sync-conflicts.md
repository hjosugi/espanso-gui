---
title: 同期を考慮した競合解決と履歴を追加する
labels: enhancement, data-safety
---

[English](../../issues/004-sync-conflicts.md) | [日本語](004-sync-conflicts.md)

iCloud Drive、Dropbox、OneDrive、Google Drive、Git、network folderで同期されるEspanso設定directory向けに、local three-way mergeを使う競合解決画面を構築します。

cloud account連携やtelemetryは不要です。競合解決はlocalで完結し、書き込み前にfield単位の差分を表示しなければなりません。

## 現在の状況（2026-08-16）

cloud APIを使わずlocalで実装済みです。保存前に読み込み時点のbase、local編集、現在のdisk内容を比較し、独立した変更は自動merge、重なったfieldはlocal／diskの採用値を明示的に選択できます。解決後の書き込み前に最新disk版をbackupし、保存履歴の復元時にも復元前backupを作成します。
