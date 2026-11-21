#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WORK="$ROOT/.qualification/dosbox-pure"
SOURCE="$WORK/upstream"
REVISION="e39497e7c2a32c67c26e6598dd14f6d58c384bfd"
CORE="$SOURCE/dosbox_pure_libretro.so"
FIXTURE="$WORK/PHITEST.dosz"
RECEIPT="$ROOT/artifacts/dosbox-pure-original-fixture.json"

rm -rf "$WORK"
mkdir -p "$WORK" "$ROOT/artifacts"
git init --quiet "$SOURCE"
git -C "$SOURCE" remote add origin https://github.com/libretro/dosbox-pure.git
git -C "$SOURCE" fetch --quiet --depth 1 origin "$REVISION"
git -C "$SOURCE" checkout --quiet --detach FETCH_HEAD
actual="$(git -C "$SOURCE" rev-parse HEAD)"
if [[ "$actual" != "$REVISION" ]]; then
  echo "DOSBox Pure source mismatch: expected $REVISION, got $actual" >&2
  exit 1
fi

make -C "$SOURCE" -j2
test -f "$CORE" || { echo "DOSBox Pure core build missing" >&2; exit 1; }

nasm -f bin "$ROOT/fixtures/dos/phitest.asm" -o "$WORK/PHITEST.COM"
python3 "$ROOT/scripts/build-dos-fixture.py" \
  --com "$WORK/PHITEST.COM" --output "$FIXTURE"

cargo run --locked --quiet -p phicade-libretro --features dos-lab-input --bin qualify_dosbox_pure -- \
  --core "$CORE" --fixture "$FIXTURE" --frames 240 --receipt "$RECEIPT"

echo "Original fixture receipt: $RECEIPT"
echo "Pinned DOSBox Pure source: $REVISION"
echo "No copyrighted game or third-party core binary is uploaded."
