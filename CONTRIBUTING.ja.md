# コントリビューション

[English](CONTRIBUTING.md) | [日本語](CONTRIBUTING.ja.md)

Espanso GUIの改善にご協力いただきありがとうございます。

## 対象範囲と上流プロジェクトとの境界

このリポジトリは独立した互換ツールです。プロジェクトに関する議論、バグ報告、Pull Requestはすべて`hjosugi/espanso-gui`内で行ってください。

本プロジェクトのためにEspansoのメンテナーへ連絡しないでください。Espanso側でのIssueやPull Requestの作成、Discussionへの投稿、Espanso GUIのための上流変更依頼は禁止します。互換性調査が必要な場合に限り、公開されたEspansoの文書とソースを読み取り専用で参照できます。

## 開発

安定版Rust 1.95以降を使用してください。

```sh
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

設定ファイルを書き込む変更には、パス範囲、競合処理、復元についてのテストが必要です。新しい構造化YAMLフィールドでは、未知のオプションを暗黙に失わないよう`#[serde(flatten)]`または同等の処理を使用してください。

## Pull Request

- 変更範囲を明確に保ってください。
- テストを追加または更新してください。
- ユーザーデータの移行や互換性への影響を説明してください。
- 実在する個人のEspanso設定をfixtureへ含めないでください。
- UI文言はYAMLの事前知識がなくても理解できる表現にしてください。
- ユーザー向け文言と文書を追加・変更する場合は、日本語と英語を同時に更新してください。

コントリビューションを行うことで、その内容をMIT Licenseで提供することに同意したものとします。
