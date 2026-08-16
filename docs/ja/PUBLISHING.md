# 公開手順

[English](../PUBLISHING.md) | [日本語](PUBLISHING.md)

公開リポジトリは`hjosugi/espanso-gui`です。リリースはversion tagからGitHub Actionsだけで作成します。

## 意図するリモート状態

- リポジトリ：`hjosugi/espanso-gui`
- 公開範囲：public
- 既定ブランチ：`main`
- リリースタグ：`Cargo.toml`と一致する`v<semantic-version>`
- Issue tracker：有効
- Discussions／wiki／projects：任意
- Espanso上流：書き込み、Issue、Pull Request、Discussion、連絡を行わない

## 公開手順

1. セッションの作業リポジトリが`hjosugi/espanso-gui`であることを確認します。
2. 全差分、license、independence notice、release limitationを確認します。
3. `Cargo.toml`、`Cargo.lock`、`CHANGELOG.md`、AppStream metadata、`docs/releases/v<version>.md`を同時に更新します。
4. `main`を`hjosugi/espanso-gui:main`へpushします。
5. cross-platform CI matrixの成功を待ちます。
6. `main`でRelease workflowをpackaging rehearsalとしてdispatchし、3つのpackage jobすべてを待ちます。
7. 両workflowが成功してからannotated version tagを作成しpushします。
8. tag-triggered release workflowにartifactとGitHub Releaseを作成させます。
9. assetをdownloadし、`SHA256SUMS`と公開noteのsigning-status sectionを確認します。
10. このrepositoryのIssueを更新します。Espanso上流へは連絡しません。

最初の0.1.0 bootstrap helperはrepository recovery用に残していますが、通常のreleaseは上記workflowを使います。

```sh
./scripts/publish-hjosugi-repository.sh repository
# main branchのCI matrixが成功するまで待つ
./scripts/publish-hjosugi-repository.sh release
```

helperはdirty tree、`main`以外のbranch、一致しない`origin`、最新`main` CIを通っていないcommitのrelease tagを拒否します。対象は意図的に`hjosugi/espanso-gui`へ固定され、Espanso上流の操作は含みません。

release signing identityがない場合、workflowは明示的に未署名のartifactを公開します。credential setupと署名経路の詳細は[SIGNING.md](SIGNING.md)を参照してください。
