# リリースの署名とnotarization

[English](../SIGNING.md) | [日本語](SIGNING.md)

リリースワークフローは常にWindows、macOS、Linuxでビルドします。署名は任意です。forkやローカルのメンテナーは認証情報なしで未署名ビルドを公開でき、各GitHub Releaseにはプラットフォームごとの状態が明記されます。

コントリビューターや自動化は、他者に代わって証明書契約、Apple Developer契約、その他の法的条件に同意してはいけません。

## Windows Authenticode

次のrepository secretを両方設定するか、どちらも設定しないでください。

- `WINDOWS_CERTIFICATE_BASE64`：Base64でencodeしたPKCS#12／PFX署名証明書
- `WINDOWS_CERTIFICATE_PASSWORD`：PFXファイルのpassword

両方が利用できる場合、workflowは証明書をrunnerの一時directoryへdecodeし、packaging前にapplication executableへ署名し、生成した`.exe`／`.msi`installerへ署名し、Windows SDKの`signtool`ですべての署名を検証します。さらにPowerShellのAuthenticode statusがvalidであること、全artifactのsigner thumbprintが設定PFXと一致すること、timestamp certificateが存在することを要求します。署名にはSHA-256とRFC 3161 timestampを使います。一時証明書ファイルを削除し、memory上の証明書は`finally` blockで破棄します。

## macOS Developer IDとnotarization

次の6つのrepository secretをすべて設定するか、どれも設定しないでください。

- `APPLE_CERTIFICATE`：Base64でencodeしたDeveloper ID Application PKCS#12 file
- `APPLE_CERTIFICATE_PASSWORD`：PKCS#12 fileのpassword
- `APPLE_SIGNING_IDENTITY`：完全なDeveloper ID Application identity
- `APPLE_ID`：`notarytool`で使用するApple ID
- `APPLE_PASSWORD`：そのApple IDのapp-specific password
- `APPLE_TEAM_ID`：Apple Developer Team ID

6つすべてがある場合、`cargo-packager`は証明書をimportし、hardened runtimeとsecure timestampを使ってappを署名し、notarizationへsubmitし、accepted ticketをstapleします。workflowは生成したDMGも`notarytool`へsubmitし、`Accepted` responseを要求してstapleし、appとdisk imageのticketを検証します。両artifactが設定されたDeveloper ID identity／Team IDを使用し、app signatureにhardened-runtime flagがあることも確認します。両署名にはsecure timestampが必要です。appは`spctl`のactive Gatekeeper policyを通過し、`hdiutil`がsubmit前とstaple後のdisk imageを検証します。packaging commandは署名済みappを検証用に残すため`app`と`dmg`の両方を要求し、release assetとしてはDMGだけを公開します。

## 失敗時の動作とリリースノート

- 一部だけ設定されたplatform credentialは、曖昧な状態のartifactを作らずpackaging前に失敗します。
- credentialがない場合は許容され、明示的に未署名のartifactを生成します。
- package jobは小さなstatus recordをuploadします。publish jobは全platformのrecordを要求し、3件すべてをrelease noteへ追加します。
- `SHA256SUMS`は署名状態にかかわらず、公開packageすべてを対象にします。
- packaging jobにはread-only `GITHUB_TOKEN`を渡し、tagで制限されたpublish jobだけに、このrepositoryのreleaseを作成／更新する`contents: write`を付与します。
- checkoutはGitHub credentialをlocal Git configurationへ保存しません。publish commandにはtagで制限されたtokenを`GH_TOKEN`だけで渡します。

値を表示せず、署名credentialの存在を確認できます。

```sh
gh secret list --repo hjosugi/espanso-gui
```

credentialを使う経路は、repository ownerが証明書とApple notarization credentialを設定した後でのみ完全に検証できます。

## 自動受け入れテスト

`tests/release_workflow.rs`はrelease workflowをparseし、package jobがread-onlyであること、未署名buildが`SIGNING_ENABLED`に依存しないこと、不完全なcredential setが失敗すること、各signing actionが完全なsetでgateされることを検証します。Windows executableと両installer形式がsignerへ渡ること、保持されたmacOS appとDMGがdistribution checkへ入ること、tag公開時に3件のsigning status recordと`SHA256SUMS`が添付されることも確認します。別のscript checkはsigner identity、timestamp、hardened runtime、Gatekeeper、stapling、disk-image integrityを要求します。これらのtestは実credentialを必要とせず、また公開せずにworkflow構造を検証します。実際のsigned releaseが最終acceptance gateです。
