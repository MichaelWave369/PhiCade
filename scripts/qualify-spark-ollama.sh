#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PIXELFORGE_ROOT="${1:-}"
SPARK_ROOT="${2:-}"
MODEL="${3:-}"
TURNS="${4:-3}"

if [[ -z "$PIXELFORGE_ROOT" || -z "$SPARK_ROOT" || -z "$MODEL" ]]; then
  echo "Usage: bash ./scripts/qualify-spark-ollama.sh /path/to/parallax-pixelforge /path/to/SparkTheSubstrate MODEL [turns]" >&2
  exit 2
fi

cd "$ROOT"
cargo run --manifest-path src-tauri/Cargo.toml --example qualify_spark_ollama -- \
  --pixelforge-root "$PIXELFORGE_ROOT" \
  --spark-root "$SPARK_ROOT" \
  --model "$MODEL" \
  --turns "$TURNS" \
  --out "$ROOT/artifacts/phicade-spark-ollama-playtest.json"
