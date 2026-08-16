# アーキテクチャ

[English](../ARCHITECTURE.md) | [日本語](ARCHITECTURE.md)

Espanso GUIは`eframe`と`egui`で構築された単一のネイティブRustアプリケーションです。

## コンポーネント

- `src/app.rs`：アプリケーション状態、画面composition、ダイアログ、検証表示、ユーザー操作。
- `src/conflict.rs`：再帰的なthree-way YAML mergeとフィールド単位の競合選択。
- `src/i18n.rs`：型付き日本語／英語UIカタログ。
- `src/lossless_yaml.rs`：変更していないソースのバイト列を保持するsequence／mapping patch。
- `src/model.rs`：Espanso match file、content type、variable、form field、semantic diagnosticのserde model。
- `src/html_editor.rs`：安全なHTML composer fragmentと、不活性なtext preview変換。
- `src/navigation.rs`：画面幅に応じたワークスペースナビゲーション、選択ファイル表示、高さを制限したファイル一覧、型付きナビゲーション操作。
- `src/preferences.rs`：旧版と互換性のあるアプリ設定の永続化と、UI拡大率の解析／表示。
- `src/profile_editor.rs`：アプリ別／既定profileのcontrol、上書きsemantics、responsive field表示。
- `src/settings_editor.rs`：外観、表示言語、UI拡大率、キーボード操作案内の表示。
- `src/snippet_editor.rs`：snippet trigger mode選択とcontent別editor toolbar。
- `src/snippet_library.rs`：副作用のない全ファイル検索、標準tag集約、表示entry生成、安定した表示順のsort。
- `src/storage.rs`：match／config profileに限定したfilesystem access、YAML読み込み、atomic save、hashによる同時変更検出、backup history、restore、recoverable deletion、snapshot、CSV変換。
- `src/espanso.rs`：Espanso検出と明示的な`start`、`stop`、`restart`、`status`操作だけを扱う最小process boundary。
- `src/theme.rs`：visual systemとcross-platformの日本語対応system font検出。
- `src/top_bar.rs`：画面幅に応じたアプリ名／状態／操作の表示と、型付き保存／再読込／再起動操作。
- `src/ui_components.rs`：responsive panel、label付きfield layout、modal shell、action button、status surface、snippet card、empty state、live message表示の再利用component。
- `src/variable_editor.rs`：対応する全Espanso variable typeのvisual parameter editorとsummary。
- `src/yaml_editor.rs`：共通のcache付きRaw YAML editorと、contrastを確保したsyntax highlighting。
- `src/yaml_syntax.rs`：source色分けとlossless patchで共有する、引用符を認識したYAML comment境界判定。

## データフロー

```text
Espanso match/*.yml
        │ 読み込み + hash
        ▼
WorkspaceFile ── serde model ── visual editor
        │                           │
        │ raw YAML                  │ structured mutation
        └──────────────┬────────────┘
                       ▼
          validate + disk hash比較
                       │
          変更あり？ three-way merge dialog
                       │
      最新disk状態をbackup + atomic write
                       ▼
               EspansoがYAMLを再読込
```

本アプリはEspansoをlink、embed、fork、patchしません。互換性は公開されたYAMLとローカルの`espanso`実行ファイルだけを通じて実現します。

## ビジュアルシステム

ネイティブ`egui`アプリケーションのためCSS layerはありません。`src/theme.rs`がdesign tokenの唯一の情報源に相当し、semantic foreground／surface／state／tint color、4段階のtypography、4段階のgap spacing、4ポイントgridのpadding、stroke、control、list、field、panel、modal、window、content geometry、対応UI scale範囲を保持します。アプリケーションコードは数値を直書きせずtoken名を使用し、source-level regression testでその境界を維持します。

共通部品はレベル1の表示タイトル、レベル2のセクション見出し、塗りつぶした主要／破壊的ボタン、対象名を含む繰り返し操作、画面幅に応じた詳細／操作行、左揃えの選択一覧、右揃えのダイアログ操作、表示領域内に収まる構造化モーダル、中央配置でスクロール可能な長文パネルを担当します。単独の選択肢は高コントラストの選択面とチェック印を併用するため、色だけに頼らず状態を判別でき、読み上げ名も変わりません。詳細／操作行は広い幅で操作を右揃えにし、狭い幅では内容の下へ折り返します。ナビゲーション操作部は個別の固定値ではなく、パネルの実際の利用可能幅を使います。これにより、編集画面ごとにスタイルを繰り返さず、文字階層、選択表示、読みやすい操作文、行／ダイアログ配置、応答型ページ幅を一貫させます。アプリの描画処理には、ネイティブウィンドウを作らず画面全体のAccessKitテストに使える、副作用のないUI入口もあります。

## 安全性の不変条件

1. 構造化書き込みは、選択された`match`／`config`directory配下の`.yml`／`.yaml`に限定します。
2. 親directory traversal、absolute relative path、symlink escapeを拒否します。
3. disk上のhashが変わった場合はthree-way mergeを開き、確認後に2回目のhash checkを行って競合を防ぎます。
4. 上書きまたは復元前に、既存ファイルをapp所有の永続履歴へcopyします。
5. ファイル削除はapp所有の復元directoryへ移動します。
6. Hub package fileはvisual editorでread-onlyです。
7. shell／script variableの本文は保存しますが、Espanso GUIは実行しません。

## YAML互換性

既知のfieldは型付きです。`#[serde(flatten)]` mapはfile、match、variable、form field、profile levelの未知keyを保持します。構造化保存では変更したsequence itemまたはmapping valueだけをserializeし、original sourceへspliceするため、無関係なcommentとformattingはバイト単位で保持されます。編集したfragmentはnormalizeされる場合がありますが、変更前の完全なファイルはhistoryに保持されます。
