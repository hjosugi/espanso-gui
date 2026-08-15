#!/usr/bin/env python3
"""Add cargo-packager's macOS signing identity without logging the secret value."""

from __future__ import annotations

import json
import os
from pathlib import Path


identity = os.environ.get("APPLE_SIGNING_IDENTITY", "")
if not identity:
    raise SystemExit("APPLE_SIGNING_IDENTITY is required")

manifest = Path("Cargo.toml")
contents = manifest.read_text(encoding="utf-8")
section = "[package.metadata.packager.macos]"
if section in contents:
    raise SystemExit(f"{section} is already configured")

manifest.write_text(
    contents
    + f"\n{section}\n"
    + f"signing-identity = {json.dumps(identity)}\n",
    encoding="utf-8",
)
