# Espanso GUI

[English](README.md) | [日本語](README.ja.md)

<img src="icons/icon.png" alt="Espanso GUI アイコン" width="128" height="128">

**Espanso GUI** は、すべてRustで実装された、[Espanso](https://espanso.org/)用の洗練されたクロスプラットフォーム・ビジュアルエディターです。

EspansoのYAML設定を直接覚えなくても、スニペット、変数、フォーム、Markdown、HTML、画像を視覚的に作成・管理できます。Windows、macOS、Linuxを同じコードベースでサポートします。

> [!IMPORTANT]
> 本プロジェクトは独立した非公式プロジェクトです。Espansoおよびそのメンテナーとの提携、承認、サポート関係はありません。Espanso GUIの問題はEspansoプロジェクトではなく、このリポジトリだけに報告してください。

## 機能

- 全スニペットファイルの横断全文検索、Espanso標準`search_terms`によるタグ絞り込み、保存されるYAML順／名前順／トリガー順の並べ替え、ラベル、別名、正規表現トリガー、複製、安全な削除を備えた3ペインのスニペットライブラリ
- プレーンテキスト、ライブプレビュー付きMarkdown、HTML、画像、対話型フォームの編集
- 強調、見出し、リンク、リスト、色、ローカル画像に対応し、いつでもソースを確認できる安全なHTMLビジュアルエディター
- 画面操作で作成できる変数：
  - 日時、オフセット、ロケール、タイムゾーン
  - クリップボードと固定値
  - 選択ダイアログとランダム選択
  - シェルコマンドとスクリプト
  - フォームとグローバル変数参照
- スニペット本文への`{{variable}}`挿入
- テキスト、複数行、選択、リストに対応したフォーム項目ビルダー
- 単語境界、大文字・小文字の引き継ぎ、挿入方式、検索語、カーソル位置の指定
- グローバル変数、未解決変数の診断、重複トリガーの警告
- Espansoの高度な設定や将来の設定項目を扱い、コメントを保持した直接編集ができる構文色分け付きRaw YAMLエディター
- 構造化データを読み書きしても既存の未知のYAMLフィールドを保持
- アンカー、ブロックスカラー、引用形式を含む、変更されていないYAMLのコメント保持編集
- フィルター、バックエンド、遅延、ショートカット、フォームサイズなどを扱うアプリ別設定画面
- 他のプログラムによる変更を検出した場合の、フィールド単位の選択を備えたローカルthree-way merge
- 永続的なローカル履歴、ワンクリック復元、復元可能なファイル削除
- CSV入出力と設定全体のスナップショット
- Espansoの検出、状態確認、開始、停止、再起動
- 日本語／英語UI、システム／ライト／ダーク外観、32／24／20／18ポイントの文字体系、40×48ポイントの操作部、入力／buttonの16×12ポイント内側余白、常時表示scroll bar、80～200%で自動調整するレイアウト、キーボード操作
- Espanso Hubパッケージは読み取り専用とし、個別スニペットをユーザーファイルへコピー可能
- テレメトリー、クラウドサービス、アカウント、バックグラウンド通信なし

## インストール

ビルド済みパッケージは各GitHub Releaseに添付されます。

- Windows：`cargo-packager`で作成したインストーラー／パッケージ
- macOS：`cargo-packager`で作成したアプリケーションバンドル／ディスクイメージ
- Linux：AppImage、Debianパッケージなどの対応パッケージ

未署名のビルドでは、OSのセキュリティ警告が表示される場合があります。リリースワークフローは、Windows Authenticode署名とmacOS Developer ID署名、notarization、staplingを任意で利用できます。各プラットフォームの実際の状態はリリースごとに明記されます。詳しくは[リリースの署名とnotarization](docs/ja/SIGNING.md)を参照してください。

Espanso本体は同梱されません。Espansoを別途インストールして起動した後、Espanso GUIを起動してください。

## ソースからビルド

必要なもの：

- Rust 1.95以降
- 各プラットフォームのネイティブビルドツール
- Linuxでは`winit`が通常必要とするX11／Wayland開発ライブラリ

```sh
git clone https://github.com/hjosugi/espanso-gui.git
cd espanso-gui
cargo run --release
```

品質チェック：

```sh
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

ネイティブインストーラーの作成：

```sh
cargo install cargo-packager --locked
cargo packager --release
```

## データの安全性

Espanso GUIが編集するのは、アプリ内で選択された設定フォルダーだけです。ファイルを上書きする前に、ディスク上の内容が最初に読み込んだ版と一致していることを確認します。

- 自動バックアップ：`<espanso-config>/.espanso-gui/backups/`
- 復元可能な削除：`<espanso-config>/.espanso-gui/trash/`
- 手動スナップショット：使用中のEspanso設定フォルダ外からユーザーが選択した保存先

構造化編集では、変更したYAMLシーケンス項目とトップレベルのプロファイル項目だけを差し替えます。変更していないコメント、アンカー、ブロックスカラー、引用形式、未知のフィールドはバイト単位で保持されます。ただし、変更したYAML断片は整形される場合があります。元のファイルは自動バックアップに残り、編集断片の書式を厳密に保つ必要がある場合はRaw YAMLタブを利用できます。

シェル変数とスクリプト変数は、Espansoのトリガー実行時にローカルコマンドを実行します。Espanso GUIは編集中にそれらのコマンドを実行しませんが、内容を確認し信頼できるコマンドだけを保存してください。

## 対応するEspanso構文

現在のエディターはEspanso 2のmatch形式に対応し、`trigger`、`triggers`、`regex`、`replace`、`markdown`、`html`、`image_path`、`form`、`form_fields`、`vars`、`global_vars`、単語境界、大文字・小文字の引き継ぎ、ラベル、検索語、force mode、Markdownの段落動作を扱います。アプリ別の`config/*.yml`フィルターと主要設定も編集できます。未知のフィールドは将来互換性のため保持されます。

詳細と既知の制限は[互換性](docs/ja/COMPATIBILITY.md)を参照してください。

## プロジェクトの境界

Espanso GUIは互換性維持のため公開されたEspanso文書を参照しますが、Espansoプロジェクトの変更、fork、連絡、Issue作成、Pull Request作成は行いません。開発とサポートはすべて`hjosugi/espanso-gui`内で行います。

## コントリビューション

[CONTRIBUTING.ja.md](CONTRIBUTING.ja.md)を参照してください。設計は[アーキテクチャ](docs/ja/ARCHITECTURE.md)、リリース署名は[署名](docs/ja/SIGNING.md)、アクセシビリティは[アクセシビリティ](docs/ja/ACCESSIBILITY.md)、ネイティブ監査は[アクセシビリティ監査](docs/ja/ACCESSIBILITY_AUDIT.md)、aText／DashとのUX比較は[UXベンチマーク](docs/ja/UX_BENCHMARK.md)、アイコン利用は[ブランド](docs/ja/BRANDING.md)に記載しています。

## ライセンス

MIT。[LICENSE](LICENSE)を参照してください。

「Espanso」は設定互換性を示すために説明的に使用しています。各商標はそれぞれの権利者に帰属します。
