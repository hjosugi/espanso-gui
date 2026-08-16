# 互換性

[English](../COMPATIBILITY.md) | [日本語](COMPATIBILITY.md)

## 対応OS

| プラットフォーム | ビルド対象 | リリースパッケージ |
| --- | --- | --- |
| Windows | x86_64 | ネイティブインストーラー／パッケージ |
| macOS | CIはApple Silicon、ソースはIntelにも対応 | `.app`／ディスクイメージ |
| Linux | x86_64 | 対応環境ではAppImageとdistribution package |

CI matrixはWindows、macOS、Linuxでcompileとtestを実行します。

## Espanso

Espanso GUIは`espanso.org/docs`に記載されたEspanso 2設定形式を対象とします。独自インストールは`espanso path`で検出し、失敗した場合は既定の設定場所を利用します。

視覚的に編集できるmatch content：

- `replace`
- `markdown`と`paragraph`
- `html`
- `image_path`
- shorthand `form`と`form_fields`
- 検索キーワードと全ファイル対象タグとして表示する`search_terms`

視覚的に編集できるvariable：

- `format`、`offset`、`locale`、`tz`を含む`date`
- `clipboard`
- `echo`
- `random`
- `choice`
- `shell`
- `script`
- `form`
- `global`

未知または新しく追加されたfieldはdata modelに保持され、Raw YAMLから変更できます。

視覚的に編集できるアプリ別設定：

- title、executable、class、OS filter
- enable／disableとinjection backend
- word、key、clipboard、paste delay
- paste／search shortcut
- form size、clipboard preservation、icon、notification option

## 0.2の既知の制限

- 変更されていないstructured YAML itemとprofile fieldはバイト単位で保持されますが、編集したfragmentはserializeされ、自身の書式が変わる場合があります。以前のファイルは自動backupに保持されます。
- HTML previewは意図的にtext-onlyで、scriptを実行せずremote resourceも取得しません。最終的なrich-text renderingはEspansoと対象applicationに依存します。
- rich-text injectionの動作は最終的にEspansoと対象applicationが決定します。
- 任意のproject signing identityが設定されていない場合、Windows／macOS packageは未署名です。実際の状態は各releaseに記載されます。
- AccessKit連携と自動contrast checkはplatform間で共通ですが、Narrator、VoiceOver、Orcaは手動release auditが必要です。
