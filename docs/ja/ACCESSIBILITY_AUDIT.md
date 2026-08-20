# アクセシビリティ・リリース監査

[English](../ACCESSIBILITY_AUDIT.md) | [日本語](ACCESSIBILITY_AUDIT.md)

この手順書は[Issue #9](https://github.com/hjosugi/espanso-gui/issues/9)のネイティブ支援技術監査を完了するために使用します。各OSのrelease buildで実行してください。1つのplatformの監査で残り2つを代替することはできません。

## テスト記録

release candidateごとに1行を記入します。`Pass`は、以下のflowを指定native platformですべて実行したことを意味します。不具合のリンク先はこのrepositoryだけにしてください。

| プラットフォーム | OSバージョン | 支援技術 | アプリ版／commit | 結果 | 担当／日付 | 不具合 |
| --- | --- | --- | --- | --- | --- | --- |
| Windows |  | Narrator |  | 未実施 |  |  |
| macOS |  | VoiceOver |  | 未実施 |  |  |
| Linux |  | Orca |  | 未実施 |  |  |

## 準備

1. 公開予定のrelease candidateと同一のbuildを作成またはinstallします。
2. `tests/fixtures/accessibility/`をrepository外の新しい破棄可能なdirectoryへcopyし、
   そのcopyをEspanso GUIのconfiguration folderとして選択します。repository内のfixtureを
   直接編集しないでください。標準fixtureには次が含まれます。
   - plain、Markdown、HTML、image、form、regular-expression、multi-triggerのmatch
   - 対応する全種類のlocal／global variable
   - `default.yml`と1つのapp-specific profile
   - diagnostic確認用の意図的なduplicate triggerとundefined variable
   - preservation確認用の未知file、match、profile、form-field、form-field `type`の値
   shellとscriptの例は編集中には実行されません。copyしたcommandを確認して信頼できる場合を除き、
   audit中にtriggerしないでください。
3. OS display scaling、desktop／session type、支援技術version、app version、commitを上表へ記録します。
4. native audit前に自動baselineを実行します。

   ```sh
   cargo fmt --check
   cargo test --all-targets
   cargo clippy --all-targets -- -D warnings
   ```

merge-conflict dialogを確認するには、Espanso GUIで同じsnippetを編集した後、copyした
`match/audit.yml`を外部から変更してappでsaveします。各platform passの後にcopyしたfixtureを
元へ戻し、すべてのtesterが同一byte列から開始するようにしてください。

## キーボードとフォーカス順

各flowをpointerなしで実行します。すべての手順でfocus indicatorが見えること、visual／declaration orderに従うこと、操作部が不意に飛ばされたり2回訪問されたりしないことを確認します。

- <kbd>Cmd/Ctrl</kbd>+<kbd>1</kbd>～<kbd>5</kbd>でsnippets、profiles、globals、diagnostics、settingsを開きます。
- <kbd>Cmd/Ctrl</kbd>+<kbd>F</kbd>でsnippet searchへfocusし、queryを入力して消去します。
- <kbd>Cmd/Ctrl</kbd>+<kbd>N</kbd>でsnippetを作成し、<kbd>Cmd/Ctrl</kbd>+<kbd>S</kbd>で保存します。
- file list、snippet list、content／variables／options／Raw YAML tab、toolbar、preview、destructive actionを順に移動します。
- profile list、visual／Raw YAML切替、全optional override、boolean selector、numeric inputを順に移動します。
- global variables、diagnostics、settings／history、About、disconnected workspace画面を順に移動します。
- new match file、new profile、variable editor、form-field editor、delete confirmation、restore confirmation、merge conflict、unsaved-exit confirmationの各dialogを開きます。
- dialog表示中にbackground controlやglobal shortcutが動作しないことを確認します。<kbd>Esc</kbd>が最前面のdialogだけを閉じ、妥当なcontrolへfocusが戻ることを確認します。

## スクリーンリーダーの名前、役割、値、状態

Narrator、VoiceOver、Orcaを有効にして、読み上げとaccessibility inspectorが次を公開することを確認します。

- search、snippet content、image path、form content、Raw YAML、variable parameter、form field、profile setting、language、UI scaleの有用な名前
- navigation、editor tab、visual／Raw YAML切替、option controlのselected state
- text、numeric、checkbox、slider、combo-box controlの現在値とenabled／disabled state
- 見えるdialog titleとaction name、およびactive modal内に限定されたfocus
- focusを移動しない操作結果のpolite announcementと、errorの即時announcement
- 選択言語によるdiagnostic severityとmessage text
- 未知のform-field typeに対する`Unsupported type: <name>`または日本語の同等表示と、そのYAML値が変更されないこと

見えるfield／action名があるにもかかわらず「edit」「button」のようなgeneric roleだけを読み上げる結果は合格にしません。

## 拡大率とレイアウト

主要なsnippet、profile、variable、form、conflict、settings flowを80%、100%、150%、200%のapp scaleで繰り返します。

- 意味を隠す重なりや切り捨てがなく、文字を読めること。
- focus中のcontrolとdialog actionへkeyboardで到達できること。
- 長いcontentはscrollし、actionをwindow外へ恒久的に押し出さないこと。
- 200%で日本語と英語の両方を確認し、長いprofile descriptionとmodal copyも確認すること。
- testerが使用するOS text／display scalingでも200%確認を繰り返し、その値を記録すること。

## ローカライズとデータ保持

1. 日本語で全screen／dialogを訪問し、product name、YAML key、command name、path、format example以外に意図しない英語だけの文章がないことを確認します。
2. 再起動せず英語へ切り替え、同じ確認を繰り返します。
3. 各言語で新しいsnippetを作り、初期display nameとreplacement textが選択言語に従うことを確認します。
4. 各言語でdiagnostic、operation message、variable summary、form summary、conflict textを発生させます。
5. fixtureの未知YAML fieldと未知form-field typeを変更せずvisual editorを開閉し、別の意図した変更を保存して両方の未知値が残ることを確認します。

## コントラストとビジュアルシステムの証拠

単体テスト`theme::tests::text_palette_meets_wcag_aa_contrast_on_primary_surfaces`は、placeholder／補助、情報、警告、エラーを含む通常の文字色について、本文、パネル、サイドバー、入力欄、無効操作面、合成したバッジ／注意枠の面で4.5:1以上を要求し、主要操作／破壊的操作上の専用文字色も検証します。実際の操作状態の面に対する境界線と2ポイントのキーボードフォーカス表示には3:1を要求します。`theme::tests::secondary_text_never_falls_back_to_tiny_default_sizes`は最小文字を18ポイントに固定し、文字体系を4種類に制限します。`theme::tests::controls_and_insets_keep_comfortable_shared_dimensions`は標準操作部を40×48ポイント以上、ボタン内余白を16×12ポイントに保ち、行内操作、編集ツール、1行／複数行入力欄、check／radio icon、menu、modal、常時表示scroll handleも小型例外を作らず共通寸法を使うことを確認します。さらに、画面全体／ダイアログのAccessKit境界テストで、フォーカス可能な全操作部が高さ48ポイント以上を保つことを検証します。`ui_components::tests::selected_snippet_cards_use_the_contrast_tested_text_color`は色付き選択カード面の全テキストを4.5:1検証済みの文字色へ揃え、アクセント色を枠／端の印で維持します。`theme::tests::semantic_palette_follows_the_selected_theme`は明るい／暗い配色の選択と、強い選択操作部の前景／背景の組を検証します。`theme::tests::ui_code_uses_design_tokens_instead_of_visual_literals`はUIコードが共通の文字、間隔、余白、色、線、寸法を回避することを防ぎ、`app::tests::application_styles_use_semantic_theme_colors`はテーマ外のRGB値を拒否します。応答型配置テストは200%拡大と対応最小値より小さい表示領域でも、ダイアログの最小値が論理表示領域を超えないことを確認します。アプリケーション単位のAccessKitテストは全6主要画面を日英両言語で描画し、ラベル関係から有効名を解決し、共通ナビゲーション接頭辞を確認し、文脈付き行操作の一意性を要求し、全8種類のダイアログを名前付き子操作部を持つ名前付きモーダルグループとして検査します。実機監査ではフォーカス表示の全体形状、無効操作部、警告／エラー枠、選択状態、中央揃えの長文ページ、OSが描画する文字を確認し、自動検査では扱えない失敗を記録してください。

2026-08-16に、破棄可能な設定を使い、隔離したD-Bus／AT-SPI registry上でLinux／X11 diagnostic passを実行しました。最初のrelease-build bridge smokeでは54個のapplication nodeを確認しました。search、navigation、file／profile selection、editor field、tabは有用なname／stateを持ち、選択中controlはpressedとして報告され、window以外のfocusable controlに名称のないものは残りませんでした。

続くdevelopment-build passではOrca 50.2とAT-SPI 2.60.6を使用しました。Orcaはapplication nameに加え、snippet、profile、global-variable、diagnostic、settings、About viewの各controlについてname、role、selected state、value、enabled stateのspeechを生成しました。6枚すべてのcustom snippet cardを含め、調べたenabled controlすべてでAT-SPI focus actionが成功しました。new-file、new-profile、variable、form-field、delete-confirmation dialogはfocusをmodal内に保ちました。このpassで、読み上げるが支援技術focusを拒否するsnippet card、繰り返されるgenericなprofile-override name、対象contextを含まない`Open`／`Edit`／`Delete`などのrow actionという3件のnative-tree defectを発見し、修正しました。

その後のapplication-tree regression passでさらに2件の構造的問題を発見し、修正しました。UI scaleの編集可能percentage subcontrolにraw AccessKit tree上の直接nameがなく、synthetic dialog nodeはtitle／modal stateを持つ一方でfields／actionsを所有していませんでした。現在は両方に直接label relationがあり、日英のfull-view／dialog testで検証されます。page／section titleはgeneric text runだけでなくlevel 1／level 2 heading roleを公開します。primary-view matrixは80%、100%、150%、200%で実行し、大きなvariable／form／conflict dialogも最小対応windowの200% zoomに相当する540×360ポイントで描画します。名前のある非text-run nodeのboundsを各horizontal viewportに対して検証し、切れていたform／variable row action、profile mode selector、expansion type controlを検出・修正しました。さらに初期viewportのvertical assertionで、Settingsの最初のcontrolを画面外へ押し出していたcompact headerも検出・修正しました。input-level regressionは全dialog stateでnavigation／editor shortcut eventを送り、modal ownershipにより背後のsection／snippet collectionが変わらないことを検証します。

リリースビルドの見た目確認では、破棄可能な日本語設定を使用しました。1440×900ポイントの論理表示領域で、32ポイントのページタイトル、24ポイントの一覧／セクション見出し、20ポイントの本文／操作部、18ポイントの補助文は明確に区別でき、読みやすい状態を保ちました。一覧カードは一貫した列幅を満たし、ナビゲーション行、入力欄、操作ボタンは安定した左端を共有し、選択行は重なりなく高コントラストの文字色を維持しました。

追加のdevelopment build見た目確認では接続済みeditorを最小高の1440×720 checkpointと200%表示で繰り返しました。100%ではファイル一覧だけがスクロールし、「ファイルを追加」、version、「設定」、Aboutは分離したままでした。200%では2つのcompact selectorが1行に収まり、日英の検索placeholderは全体が見え、選択操作部のチェック印を維持し、折り返したtabと最初のeditor surfaceは初期viewport内に残りました。light／dark確認でもeditorの16×12ポイント内側余白と常時表示の高contrast scroll handleを確認しました。その後repositoryのrelease binaryを再buildして分離環境で起動smoke testを行い、最適化済みの全test matrixも合格しました。

これはLinux行の`Pass`ではなく診断上の証拠です。nested X11 harnessでは繰り返すTab navigationを確実にsynthesizeできず、人間のlistenerによる完全なrelease-build flow matrixを実行していません。Windows NarratorとmacOS VoiceOverも未検証です。

## 完了条件

Issue #9のnative auditは、3つすべてのplatform行が`Pass`になり、発見した全不具合が修正済みまたは理由付きで明示的に延期され、最終commitで自動baselineがすべて成功した場合にのみ完了します。
