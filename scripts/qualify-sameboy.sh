#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORK="${PHICADE_QUALIFY_WORK:-$ROOT/.qualification/sameboy}"
RGBDS_TAG="v1.0.3"
SAMEBOY_REV="213a12ce93d66b105a113debd9396306066a7cfc"
FIXTURE_URL="https://github.com/mattcurrie/dmg-acid2/releases/download/v1.0/dmg-acid2.gb"

rm -rf "$WORK"
mkdir -p "$WORK"

printf '==> building pinned RGBDS %s\n' "$RGBDS_TAG"
git clone --depth 1 --branch "$RGBDS_TAG" https://github.com/gbdev/rgbds.git "$WORK/rgbds"
make -C "$WORK/rgbds" -j2
make -C "$WORK/rgbds" PREFIX="$WORK/rgbds-install" install

printf '==> building SameBoy %s\n' "$SAMEBOY_REV"
git clone https://github.com/LIJI32/SameBoy.git "$WORK/SameBoy"
git -C "$WORK/SameBoy" checkout "$SAMEBOY_REV"
PATH="$WORK/rgbds-install/bin:$PATH" make -C "$WORK/SameBoy" -j2 CONF=release libretro

CORE="$(find "$WORK/SameBoy" -type f -name 'sameboy_libretro.so' -print -quit)"
if [[ -z "$CORE" ]]; then
  echo "sameboy_libretro.so was not produced" >&2
  find "$WORK/SameBoy" -maxdepth 4 -type f -name '*libretro*' -print >&2 || true
  exit 1
fi

printf '==> downloading MIT-licensed dmg-acid2 v1.0 fixture\n'
curl --fail --location --retry 3 "$FIXTURE_URL" --output "$WORK/dmg-acid2.gb"

printf '==> running governed libretro smoke qualification\n'
mkdir -p "$ROOT/artifacts"
cargo run -p phicade-libretro --bin qualify -- \
  --core "$CORE" \
  --rom "$WORK/dmg-acid2.gb" \
  --frames 240 \
  --receipt "$ROOT/artifacts/sameboy-qualification.json"

printf '==> running Replay Ledger exact/divergence qualification\n'
cargo run -p phicade-libretro --bin replay_qualify -- \
  --core "$CORE" \
  --rom "$WORK/dmg-acid2.gb" \
  --receipt "$ROOT/artifacts/replay-qualification.json"

printf '==> running Phi-Bot governed-seat qualification\n'
cargo run -p phicade-libretro --bin phibot_qualify -- \
  --core "$CORE" \
  --rom "$WORK/dmg-acid2.gb" \
  --receipt "$ROOT/artifacts/phibot-qualification.json"

printf '==> receipts:\n'
printf '    %s\n' "$ROOT/artifacts/sameboy-qualification.json"
printf '    %s\n' "$ROOT/artifacts/replay-qualification.json"
printf '    %s\n' "$ROOT/artifacts/phibot-qualification.json"
