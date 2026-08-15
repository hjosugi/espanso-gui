#!/usr/bin/env bash
set -euo pipefail

: "${APPLE_ID:?APPLE_ID is required}"
: "${APPLE_PASSWORD:?APPLE_PASSWORD is required}"
: "${APPLE_TEAM_ID:?APPLE_TEAM_ID is required}"

shopt -s nullglob
app_bundles=(dist/*.app)
disk_images=(dist/*.dmg)

if ((${#app_bundles[@]} != 1)); then
  echo "Expected exactly one macOS app bundle, found ${#app_bundles[@]}" >&2
  exit 1
fi
if ((${#disk_images[@]} != 1)); then
  echo "Expected exactly one macOS disk image, found ${#disk_images[@]}" >&2
  exit 1
fi

app_bundle="${app_bundles[0]}"
disk_image="${disk_images[0]}"

codesign --verify --deep --strict --verbose=2 "${app_bundle}"
xcrun stapler validate --verbose "${app_bundle}"
codesign --verify --strict --verbose=2 "${disk_image}"

notary_result="${RUNNER_TEMP}/espanso-gui-dmg-notary.json"
xcrun notarytool submit "${disk_image}" \
  --apple-id "${APPLE_ID}" \
  --password "${APPLE_PASSWORD}" \
  --team-id "${APPLE_TEAM_ID}" \
  --wait \
  --output-format json > "${notary_result}"

python - "${notary_result}" <<'PY'
import json
import pathlib
import sys

result = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if result.get("status") != "Accepted":
    raise SystemExit(f"DMG notarization was not accepted: {result}")
PY

xcrun stapler staple --verbose "${disk_image}"
xcrun stapler validate --verbose "${disk_image}"
