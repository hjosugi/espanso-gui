---
title: キーボード操作、スクリーンリーダー監査、ローカライズ基盤を完成させる
labels: accessibility, i18n, enhancement
---

[English](../../issues/005-accessibility-localization.md) | [日本語](005-accessibility-localization.md)

Windows、macOS、Linuxですべてのeditor／dialogを監査します。安定したfocus order、accessible name、contrast検証、拡大可能なUI text、日本語／英語から始めるlocalization catalogを追加します。

## 受け入れ条件

- すべての主要view／modalに安定したkeyboard順、有用なaccessible nameがあり、名称のないfocusable application controlがない。
- navigation、tab、list row、mode selectorが判別しやすいselected stateと、読みやすいforeground／background contrastを公開する。
- application text、操作message、diagnostic、repository document、contributor向けtemplateを英語／日本語で網羅する。
- 共通ビジュアルシステムの文字サイズを4種類以内に制限し、補助文を18ポイント以上に保ち、間隔、余白、操作部サイズ、フォーカス、意味別の色を共通トークンで提供する。
- 80%、100%、150%、200%のapp scaleで主要flowを使用でき、必要なactionが隠れない。
- [release監査手順](../ACCESSIBILITY_AUDIT.md)の全項目について、WindowsのNarrator、macOSのVoiceOver、LinuxのOrcaがそれぞれ合格する。
- 最終commitで`cargo fmt --check`、`cargo test --all-targets`、`cargo clippy --all-targets -- -D warnings`が成功する。

## 現在の検証状況

日英の画面／ダイアログ、フォーカス順、ポインター入力、レスポンシブ配置、文字体系、余白、選択状態、WCAGコントラストの自動回帰テストは実装済みです。4段階の拡大率、最大拡大時の初期表示操作、方式／タブ選択肢の常時見えるボタン境界、ポインターで選択できる正規表現方式、色に依存しない選択印、コンパクト一覧での検索案内文の収まり、1行／複数行editorの内側余白、読みやすい操作部／menu／modal寸法、常時見えるscroll handle、多数のファイルが下部操作を押し出さないことも検証します。対応する全content／variable種類、diagnostic例、profile、未知YAML値をまとめた単一の標準監査fixtureもrepositoryに含み、そのmatrixが読込可能かつlosslessであることを自動テストで固定します。Linuxの診断確認でネイティブアクセシビリティツリーの不具合を発見して修正しましたが、人間が読み上げを聞きながら手順書を完走する監査には至っていません。Windows NarratorとmacOS VoiceOverは未検証です。3環境の実機監査がすべて合格するまでこのIssueは未完了のままにし、自動テストだけからプラットフォーム対応完了を推定しません。
