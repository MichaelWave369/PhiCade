#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORK="${ROOT}/.qualification/scummvm"
SOURCE="${WORK}/ScummVM"
TAG="v2026.3.0"
REVISION="fed42f2068dcafc6aafa1c28c77e4c88def74b66"
CORE="${SOURCE}/backends/platform/libretro/scummvm_libretro.so"
RECEIPT="${ROOT}/artifacts/scummvm-qualification.json"

rm -rf "${WORK}"
mkdir -p "${WORK}" "${ROOT}/artifacts"

git clone --depth 1 --branch "${TAG}" https://github.com/scummvm/scummvm.git "${SOURCE}"

actual_revision="$(git -C "${SOURCE}" rev-parse HEAD)"
if [[ "${actual_revision}" != "${REVISION}" ]]; then
  echo "ScummVM revision mismatch: expected ${REVISION}, got ${actual_revision}" >&2
  exit 1
fi

make -C "${SOURCE}/backends/platform/libretro" \
  noengine \
  -j2 \
  FORCE_OPENGLNONE=1

if [[ ! -f "${CORE}" ]]; then
  echo "ScummVM libretro core not produced at ${CORE}" >&2
  exit 1
fi

cargo run --quiet -p phicade-libretro --bin qualify_scummvm -- \
  --core "${CORE}" \
  --frames 180 \
  --receipt "${RECEIPT}"

echo "ScummVM qualification receipt: ${RECEIPT}"
sha256sum "${CORE}"
