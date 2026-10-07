#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PIXELFORGE_ROOT="${1:-}"

if [[ -z "$PIXELFORGE_ROOT" ]]; then
  echo "Usage: bash ./scripts/qualify-pixelforge.sh /path/to/parallax-pixelforge" >&2
  exit 2
fi

cd "$ROOT"
cargo run -p phicade-runtime --example qualify_pixelforge -- \
  --pixelforge-root "$PIXELFORGE_ROOT" \
  --out "$ROOT/artifacts/pixelforge-cartridge-qualification.json"
