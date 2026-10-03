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

# SameBoy's boot-ROM build invokes build/pb12 from nested parallel make jobs.
# On GitHub-hosted runners that helper can be observed before its executable
# mode is stable, producing a transient "Permission denied". Build it once
# up front, make the executable mode explicit, then continue the normal
# parallel libretro build.
printf '==> prebuilding SameBoy pb12 boot-ROM helper\n'
mkdir -p "$WORK/SameBoy/build"
${CC:-cc} -std=c99 -Wall -Werror \
  "$WORK/SameBoy/BootROMs/pb12.c" \
  -o "$WORK/SameBoy/build/pb12"
chmod 0755 "$WORK/SameBoy/build/pb12"
test -x "$WORK/SameBoy/build/pb12"

PATH="$WORK/rgbds-install/bin:$PATH" make -C "$WORK/SameBoy" -j2 CONF=release libretro

CORE="$(find "$WORK/SameBoy" -type f -name 'sameboy_libretro.so' -print -quit)"
if [[ -z "$CORE" ]]; then
  echo "sameboy_libretro.so was not produced" >&2
  find "$WORK/SameBoy" -maxdepth 4 -type f -name '*libretro*' -print >&2 || true
  exit 1
fi

printf '==> downloading MIT-licensed dmg-acid2 v1.0 fixture\n'
curl --fail --location --retry 3 "$FIXTURE_URL" --output "$WORK/dmg-acid2.gb"

printf '==> assembling source-first Phi-Agent Gym task A\n'
GYM_SRC="$ROOT/benchmarks/agent-gym/main.asm"
GYM_OBJ="$WORK/phi-agent-gym.o"
GYM_ROM="$WORK/phi-agent-gym.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$GYM_OBJ" "$GYM_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$GYM_ROM" "$GYM_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHIAGENTGYM" "$GYM_ROM"

printf '==> assembling source-first Phi-Agent Gym task B / Mirror Dash\n'
MIRROR_SRC="$ROOT/benchmarks/agent-gym-mirror/main.asm"
MIRROR_OBJ="$WORK/phi-agent-gym-mirror.o"
MIRROR_ROM="$WORK/phi-agent-gym-mirror.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$MIRROR_OBJ" "$MIRROR_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$MIRROR_ROM" "$MIRROR_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHIMIRRORGYM" "$MIRROR_ROM"

printf '==> benchmark suite hashes\n'
sha256sum "$GYM_SRC" "$GYM_ROM" "$MIRROR_SRC" "$MIRROR_ROM"

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

printf '==> running Agent Driver Protocol qualification\n'
cargo run -p phicade-libretro --bin driver_qualify -- \
  --core "$CORE" \
  --rom "$WORK/dmg-acid2.gb" \
  --receipt "$ROOT/artifacts/agent-driver-qualification.json"

printf '==> running governed Autodrive qualification\n'
cargo run -p phicade-libretro --bin autodrive_qualify -- \
  --core "$CORE" \
  --rom "$WORK/dmg-acid2.gb" \
  --receipt "$ROOT/artifacts/autodrive-qualification.json"

printf '==> running Phi-Agent Gym task A qualification\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$GYM_ROM" \
  --source "$GYM_SRC" \
  --task "move-block-to-x-v1" \
  --receipt "$ROOT/artifacts/agent-gym-qualification.json"

printf '==> running Phi-Agent Gym task B / Mirror Dash qualification\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$MIRROR_ROM" \
  --source "$MIRROR_SRC" \
  --task "move-block-to-x-mirror-v1" \
  --receipt "$ROOT/artifacts/agent-gym-mirror-qualification.json"

printf '==> receipts:\n'
printf '    %s\n' "$ROOT/artifacts/sameboy-qualification.json"
printf '    %s\n' "$ROOT/artifacts/replay-qualification.json"
printf '    %s\n' "$ROOT/artifacts/phibot-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-driver-qualification.json"
printf '    %s\n' "$ROOT/artifacts/autodrive-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-mirror-qualification.json"
