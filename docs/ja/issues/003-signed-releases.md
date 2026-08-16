---
title: Windows署名とmacOS notarizationを設定する
labels: release, security, platform
---

[English](../../issues/003-signed-releases.md) | [日本語](003-signed-releases.md)

secretがある場合だけ署名する処理をrelease workflowへ追加します。

受け入れ条件：

- secretがないforkでも未署名buildを作成できる。
- credentialがある場合はWindows artifactをAuthenticodeで署名する。
- macOS applicationとdisk imageを署名、harden、notarizationへsubmitし、stapleする。
- release noteへ署名状態を記録し、checksumを添付する。

## 現在の検証状況

release workflow、署名script、credential gate、署名後検証、checksum公開、日英の署名状態noteは実装済みで、repository testでも検証しています。必要なWindows certificateとApple署名／notarization secretが設定されていないため、最新の公開releaseは未署名のままです。実credentialを使うtag releaseを作成し、公開artifact上で両platformの署名を検証できるまで、このIssueはopenのままにします。
