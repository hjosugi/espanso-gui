---
title: 構造化編集でYAMLコメントを保持する
labels: enhancement, data-safety, yaml
---

[English](../../issues/001-lossless-yaml-comments.md) | [日本語](001-lossless-yaml-comments.md)

現在の構造化編集は未知のkeyを保持しますが、YAMLを正規化し、commentを削除または移動する場合があります。全体をserdeで再serializeする方式を、既知fieldだけを変更するlossless syntax-tree patcherへ置き換えます。

受け入れ条件：

- 変更していないmatchの外側／内側のcommentがバイト単位で変化しない。
- 1つのmatchを編集しても無関係なmatchを書式変更しない。
- 既存の自動backup動作を維持する。
- block scalar、anchor、quoted value、複数階層のcommentをfixtureで検証する。

## 現在の状況（2026-08-16）

実装済みです。スニペット／プロファイルの構造化編集では変更したYAML断片だけを更新し、変更していない項目はバイト単位で保持します。書き込み前の自動履歴を維持し、コメント、ブロックスカラー、アンカー、引用形式、プロファイル引用値内の`#`、変更したブロックスカラー項目の完全置換、WindowsのCRLF、末尾改行なしのファイルを回帰テストで検証します。意図的に編集した断片は再整形される場合があるため、UIで制限を明示し、厳密なソース編集用にRaw YAMLも残しています。
