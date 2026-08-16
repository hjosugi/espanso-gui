# アクセシビリティとローカライズ

[English](../ACCESSIBILITY.md) | [日本語](ACCESSIBILITY.md)

Espanso GUIはWindows、macOS、Linuxで同じ`egui`ウィジェットツリーとAccessKit連携を使用します。ネイティブ操作部のフォーカス順は宣言順で決定され、プラットフォーム固有のフォーカス上書きは使用しません。

## キーボード操作

| 操作 | ショートカット |
| --- | --- |
| スニペット、プロファイル、グローバル変数、診断、設定を開く | <kbd>Cmd/Ctrl</kbd>+<kbd>1</kbd>～<kbd>5</kbd> |
| スニペット検索へフォーカス | <kbd>Cmd/Ctrl</kbd>+<kbd>F</kbd> |
| 選択中のmatch／configファイルを保存 | <kbd>Cmd/Ctrl</kbd>+<kbd>S</kbd> |
| スニペットを作成 | <kbd>Cmd/Ctrl</kbd>+<kbd>N</kbd> |
| 開いているダイアログを閉じる | <kbd>Esc</kbd> |
| 操作部間を移動 | <kbd>Tab</kbd>／<kbd>Shift</kbd>+<kbd>Tab</kbd> |

検索入力、スニペットカード、本文／オプション、画像プレビュー、変数パラメーター、フォーム項目、プロファイル設定、Raw YAMLエディター、言語選択、UI拡大率には明示的なアクセシブル名またはラベル関係があります。UI拡大率のsliderと編集可能なpercentage値は個別に名前を持つため、名称のないフォーカス位置にはなりません。入力中の検索は読み込み済みmatch fileをすべて横断し、ローカライズされた結果件数を名前付きpolite live announcementとして公開します。結果が0件の場合はkeyboardでfocusできるclear actionを表示します。スニペットカードはbutton roleに加え、選択状態、タイトル、トリガー、種類、横断検索結果のsource file、プレビューを公開し、支援技術からフォーカス可能な専用interactive nodeを使用します。ボタンはアイコンだけでなく見える操作名を使い、繰り返し表示される行操作には対象の項目、変数、診断をアクセシブル名へ加えます。キーボードフォーカスは、ナビゲーションボタンとカスタムスニペットカードを含む現在の操作面に対して2ポイントのaccent outlineで表示されます。ダイアログcontainerはdialog roleと安定したタイトルを公開し、名前のあるフィールドとkeyboard-focusable actionを構造的に所有します。操作結果はpolite、エラーはassertiveなlive announcementを使います。

ダイアログにはeguiのmodal accessibility layerとinput backdropを使用します。ダイアログ表示中は、pointer操作とアプリ全体のshortcutで背後のeditorを操作できません。<kbd>Esc</kbd>は最前面のダイアログだけが処理します。自動tree testは日英両言語ですべての主要画面を描画し、共通の宣言順navigation prefix、`labelled_by`を含む実効名、8種類すべてのapplication dialogが名前付きdescendant controlを持つmodal groupであることを検証します。全主要画面は最小window相当の論理viewportを使って80%、100%、150%、200%で繰り返し、変数／フォーム／競合操作も540×360ポイントの200% checkpointで描画します。各checkpointで名前のある非text-run UI nodeのhorizontal boundsがviewport内に残ることを必須とし、accessibility treeには存在しても見た目では切れているcontrol／label付きsurfaceを防ぎます。別の入力回帰テストは8種類すべてのdialog stateでnavigation／editor shortcut eventを実行し、背景状態が変わらないことを確認します。

未接続のonboarding画面も、最大拡大率の同じviewportで日英両言語を描画します。setup guide、configuration folder、安全な初期化の各actionは、名前を持ち、focus可能で、pointerなしでも到達できなければなりません。

## 表示

- UI拡大率は保存され、再起動せず80～200%で変更できます。
- アプリまたはOSの高い拡大率を含む狭い論理幅では、ナビゲーションを作業領域上部の名前付き画面選択1つへまとめ、重複する上部操作を同じ行へ移し、一覧パネルを狭め、ラベル付き項目を縦に並べます。空状態の装飾的な上余白だけを抑え、長い設定パスを折り返し、ダイアログの幅／高さを論理表示領域内へ制限します。選択部は現在の画面をアクセシブルな値として公開します。200%拡大時は一覧パネル自体の最小200ポイントを保ちながら詳細エディターへ最低320ポイントを残し、全画面の最初の主要内容を初期表示内に置きます。広い競合エディターも、従来の760×560ポイント固定最小サイズを画面外へ押し出さず、スクロール表示します。
- 文字サイズは4種類だけです。表示タイトルは32ポイント、セクション見出しは24ポイント、本文／操作部／等幅文字は20ポイント、補助文は18ポイントです。表示／セクションタイトルは個別の共通コンポーネントを使い、支援技術へレベル1／レベル2の見出し情報を公開します。18ポイントの補助文字はeguiの小さな既定文字を置き換えます。1行／複数行の全入力欄はツールキット既定の狭い余白を使わず、左右16／上下12ポイントの共通内側余白を使用します。
- 文字には4ポイントの追加行間を設けます。check／radio等の操作iconは外寸24ポイント、文字との間隔12ポイントを使い、sliderとselectorもツールキット既定の狭い幅を使いません。modal windowの内側余白は24ポイント、menuは12ポイントです。
- 縦方向のリズムには4／12／16／24ポイントの共通間隔を使用します。内側余白には8／12／16／24／32／40ポイントのトークンを使います。フォント、間隔、余白、色、線、操作部、パネル、一覧、入力欄、画像プレビュー、ダイアログ、拡大率の値は共通デザイントークンから取得し、UIコード内の個別指定はソーステストで拒否されます。ナビゲーションとファイル／プロファイル選択行は利用可能な幅を一貫して使い、ラベルを左揃えにし、空表示と明示的な操作だけを中央揃えにします。横長フォームは340ポイントのラベル列を確保し、日英の一般的な説明文で末尾だけが次の行へ残らないよう検証します。幅660ポイント未満では縦配置へ切り替えます。横長表示のファイル一覧は高さを制限したスクロール領域に収めるため、設定ファイルが増えてもファイル追加操作や下部ナビゲーションが画面外へ押し出されません。境界テストはファイル追加操作とversion表示が離れていることも要求します。
- スクロール領域はhoverまで隠れる2ポイントのfloating handleを使わず、高contrastな前景色で常時領域を確保する12ポイントのsolid barと最小48ポイントのhandleを使います。長い一覧とeditorを見つけやすくし、pointerでも操作しやすくします。
- 設定、診断、グローバル変数、Aboutは、最大幅1,040ポイントの中央揃えでスクロール可能な共通配置を使用します。狭い表示では利用可能な全幅を維持します。
- placeholder／補助、情報、警告、エラー、無効操作部の合成後文字色を含む配色は、本文、パネル、サイドバー、入力欄、無効操作面、合成したバッジ／注意枠の通常文字について、単体テストでWCAG AAコントラストを検査します。操作部の境界線とフォーカス表示は、実際の操作面上で3:1を検査します。
- Raw YAMLのkey、quoted value、comment、通常textは、light／dark双方で同じcontrast検証済みsemantic paletteを使います。色分け結果はcacheされ、source byteを変更しません。
- システム／ライト／ダーク外観は意味別の色を共有します。選択された操作部は強いアクセント面とコントラスト検証済みの専用文字色を使い、選択中のスニペットカードには左端の印も表示します。主要／破壊的操作も専用の検証済み文字色を使います。コンパクト表示の接続状態は色だけの点にせず日英の文字を表示し、最大拡大時も上部バー内へ収まることを実フォント幅で検証します。ダイアログ操作は共通の右揃え配置です。
- application accessibility rootの名前は`Espanso GUI`です。list selection buttonは見えるlabelとpressed stateをaccessibility treeに保持します。
- OS側のfont scalingを変更せず、日本語を表示できるsystem fontを選択します。

## ローカライズ

型付き翻訳カタログは日本語と英語に対応します。選択言語は保存されます。ナビゲーション、ライブラリ／プロファイル一覧、スニペット／プロファイル全編集画面、診断、設定、履歴、empty state、接続状態、操作結果、すべてのmodal dialogはcatalog keyを使います。変数／フォームビルダーは種類、field label、help text、validation error、生成placeholder、summary textを翻訳します。semantic diagnosticはmodel内では言語非依存で、選択中のcatalogから表示します。新しい利用者向け画面では日英両方の文字列を追加し、catalog completeness testの対象にしてください。

公開repository文書は英語原本に加え、`docs/ja/`とrepository rootの`.ja.md`に日本語版を配置します。すべての文書ペアに見える言語切替があります。GitHubのIssue／Pull Request入力画面は同じform内に日英両方を表示します。integration testは言語版の欠落、切替の欠落、Issue仕様front matterの破損、template YAMLの構文エラー、日本語AppStream metadataの欠落を拒否します。

## 手動リリース監査

安定版リリース前に、WindowsのNarrator、macOSのVoiceOver、LinuxのOrcaで共通focus sequenceを確認してください。検索label、navigation button、editor tab、profile control、merge choice、confirmation dialog、200%表示を検証します。platform固有の不具合はこのrepositoryだけに記録します。必須のplatform matrix、test flow、evidence、exit criteriaは[ACCESSIBILITY_AUDIT.md](ACCESSIBILITY_AUDIT.md)を使用してください。
