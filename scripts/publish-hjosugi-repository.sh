#!/usr/bin/env bash
set -euo pipefail

readonly REPOSITORY="hjosugi/espanso-gui"
readonly EXPECTED_HTTPS="https://github.com/hjosugi/espanso-gui.git"
readonly EXPECTED_SSH="git@github.com:hjosugi/espanso-gui.git"

die() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

require_target_checkout() {
  command -v gh >/dev/null || die "GitHub CLI (gh) is required"
  command -v rg >/dev/null || die "ripgrep (rg) is required"
  command -v python3 >/dev/null || die "python3 is required"
  [[ "$(git rev-parse --show-toplevel)" == "$(pwd -P)" ]] || die "run from the repository root"
  [[ "$(git branch --show-current)" == "main" ]] || die "the current branch must be main"
  [[ -z "$(git status --porcelain)" ]] || die "commit or remove local changes before publishing"
  gh auth status >/dev/null
}

ensure_exact_origin() {
  local origin
  if origin="$(git remote get-url origin 2>/dev/null)"; then
    case "$origin" in
      "$EXPECTED_HTTPS"|"$EXPECTED_SSH") ;;
      *) die "origin is $origin, expected exactly $EXPECTED_HTTPS or $EXPECTED_SSH" ;;
    esac
  else
    git remote add origin "$EXPECTED_HTTPS"
  fi
}

create_labels() {
  local specification name color description
  local -a labels=(
    "bug|d73a4a|Something is not working"
    "enhancement|a2eeef|New or improved user-facing behavior"
    "triage|fbca04|Needs review and prioritization"
    "data-safety|b60205|Backups, recovery, conflicts, or user-data integrity"
    "yaml|5319e7|Espanso YAML parsing or serialization"
    "configuration|1d76db|Espanso configuration support"
    "release|0e8a16|Packaging and publishing"
    "security|b60205|Security hardening"
    "platform|c5def5|Windows, macOS, or Linux-specific work"
    "accessibility|7057ff|Keyboard, screen reader, contrast, or scalable UI"
    "i18n|d4c5f9|Localization and language support"
    "rich-text|f9d0c4|Markdown, HTML, image, or formatting editor"
  )
  for specification in "${labels[@]}"; do
    IFS='|' read -r name color description <<<"$specification"
    gh label create "$name" --repo "$REPOSITORY" --color "$color" --description "$description" --force
  done
}

create_issues() {
  local issue_file title labels body
  for issue_file in docs/issues/*.md; do
    title="$(sed -n 's/^title: //p' "$issue_file")"
    labels="$(sed -n 's/^labels: //p' "$issue_file")"
    [[ -n "$title" && -n "$labels" ]] || die "invalid issue draft: $issue_file"
    if gh issue list --repo "$REPOSITORY" --state all --limit 1000 --json title --jq '.[].title' | rg -Fxq -- "$title"; then
      printf 'issue already exists: %s\n' "$title"
      continue
    fi
    body="$(awk 'BEGIN { separators=0 } /^---$/ { separators++; next } separators >= 2 { print }' "$issue_file")"
    gh issue create --repo "$REPOSITORY" --title "$title" --label "$labels" --body "$body"
  done
}

publish_repository() {
  require_target_checkout
  if ! gh repo view "$REPOSITORY" >/dev/null 2>&1; then
    gh repo create "$REPOSITORY" --public --source . --description "A polished cross-platform visual editor for Espanso, written in Rust"
  fi
  ensure_exact_origin
  gh repo edit "$REPOSITORY" --visibility public --enable-issues --enable-wiki=false --enable-projects=false
  git push --set-upstream origin main
  create_labels
  create_issues
  printf 'repository stage complete: https://github.com/%s\n' "$REPOSITORY"
}

publish_release_tag() {
  local version tag ci_conclusion ci_sha head_sha
  require_target_checkout
  ensure_exact_origin
  version="$(cargo metadata --no-deps --format-version 1 | python3 -c 'import json,sys; print(json.load(sys.stdin)["packages"][0]["version"])')"
  tag="v${version}"
  head_sha="$(git rev-parse HEAD)"
  ci_conclusion="$(gh run list --repo "$REPOSITORY" --workflow CI --branch main --limit 1 --json conclusion --jq '.[0].conclusion')"
  ci_sha="$(gh run list --repo "$REPOSITORY" --workflow CI --branch main --limit 1 --json headSha --jq '.[0].headSha')"
  [[ "$ci_conclusion" == "success" ]] || die "latest main CI conclusion is '$ci_conclusion', not success"
  [[ "$ci_sha" == "$head_sha" ]] || die "latest successful CI does not belong to the current commit"
  if ! git rev-parse "$tag" >/dev/null 2>&1; then
    git tag --annotate "$tag" --message "Espanso GUI ${tag}"
  fi
  [[ "$(git rev-list -n 1 "$tag")" == "$head_sha" ]] || die "$tag does not point at the current commit"
  git push origin "refs/tags/${tag}"
  printf 'release tag pushed; inspect only this repository:\n'
  printf '  gh run list --repo %s --workflow Release\n' "$REPOSITORY"
}

case "${1:-}" in
  repository) publish_repository ;;
  release) publish_release_tag ;;
  *) die "usage: $0 repository|release" ;;
esac
