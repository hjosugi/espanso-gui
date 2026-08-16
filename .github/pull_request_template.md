## Summary / 概要

Describe the user-visible outcome. / ユーザーから見える変更結果を説明してください。

## Validation / 検証

- [ ] `cargo fmt --check`
- [ ] `cargo test --all-targets`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] I considered Windows, macOS, and Linux behavior. / Windows、macOS、Linuxでの動作を考慮しました。
- [ ] I preserved unknown YAML fields and protected existing user data. / 未知のYAMLフィールドを保持し、既存のユーザーデータを保護しました。
- [ ] I updated Japanese and English user-facing copy and documentation together. / 利用者向け文言と文書の日本語／英語を同時に更新しました。
- [ ] This change does not contact or require a write to the Espanso upstream project. / Espanso上流への連絡や書き込みを必要としない変更です。

## Data safety and compatibility / データ安全性と互換性

Describe configuration writes, migrations, backups, and Espanso syntax affected by this change. / この変更が影響する設定書き込み、移行、バックアップ、Espanso構文を説明してください。
