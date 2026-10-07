#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PIXELFORGE_ROOT="${1:-}"
SPARK_ROOT="${2:-}"

if [[ -z "$PIXELFORGE_ROOT" || -z "$SPARK_ROOT" ]]; then
  echo "Usage: bash ./scripts/qualify-spark-chain.sh /path/to/parallax-pixelforge /path/to/SparkTheSubstrate" >&2
  exit 2
fi

cd "$ROOT"
cargo run -p phicade-runtime --example qualify_spark_chain -- \
  --pixelforge-root "$PIXELFORGE_ROOT" \
  --spark-root "$SPARK_ROOT" \
  --out "$ROOT/artifacts/phicade-pixelforge-spark-qualification.json"
