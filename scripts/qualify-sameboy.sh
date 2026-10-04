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

printf '==> assembling source-first Phi-Agent Gym task C / Wall Detour\n'
WALL_SRC="$ROOT/benchmarks/agent-gym-wall-detour/main.asm"
WALL_OBJ="$WORK/phi-agent-gym-wall-detour.o"
WALL_ROM="$WORK/phi-agent-gym-wall-detour.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$WALL_OBJ" "$WALL_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$WALL_ROM" "$WALL_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHIWALLGYM" "$WALL_ROM"

printf '==> assembling source-first Phi-Agent Gym task D / Temporal Cue LEFT\n'
TEMP_LEFT_SRC="$ROOT/benchmarks/agent-gym-temporal-left/main.asm"
TEMP_LEFT_OBJ="$WORK/phi-agent-gym-temporal-left.o"
TEMP_LEFT_ROM="$WORK/phi-agent-gym-temporal-left.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$TEMP_LEFT_OBJ" "$TEMP_LEFT_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$TEMP_LEFT_ROM" "$TEMP_LEFT_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHITEMPLEFT" "$TEMP_LEFT_ROM"

printf '==> assembling source-first Phi-Agent Gym task E / Temporal Cue RIGHT\n'
TEMP_RIGHT_SRC="$ROOT/benchmarks/agent-gym-temporal-right/main.asm"
TEMP_RIGHT_OBJ="$WORK/phi-agent-gym-temporal-right.o"
TEMP_RIGHT_ROM="$WORK/phi-agent-gym-temporal-right.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$TEMP_RIGHT_OBJ" "$TEMP_RIGHT_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$TEMP_RIGHT_ROM" "$TEMP_RIGHT_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHITEMPRIGHT" "$TEMP_RIGHT_ROM"

printf '==> assembling source-first Phi-Agent Gym task F / Relay Rooms LEFT\n'
RELAY_LEFT_SRC="$ROOT/benchmarks/agent-gym-relay-left/main.asm"
RELAY_LEFT_OBJ="$WORK/phi-agent-gym-relay-left.o"
RELAY_LEFT_ROM="$WORK/phi-agent-gym-relay-left.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$RELAY_LEFT_OBJ" "$RELAY_LEFT_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$RELAY_LEFT_ROM" "$RELAY_LEFT_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHIRELAYLEFT" "$RELAY_LEFT_ROM"

printf '==> assembling source-first Phi-Agent Gym task G / Relay Rooms RIGHT\n'
RELAY_RIGHT_SRC="$ROOT/benchmarks/agent-gym-relay-right/main.asm"
RELAY_RIGHT_OBJ="$WORK/phi-agent-gym-relay-right.o"
RELAY_RIGHT_ROM="$WORK/phi-agent-gym-relay-right.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$RELAY_RIGHT_OBJ" "$RELAY_RIGHT_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$RELAY_RIGHT_ROM" "$RELAY_RIGHT_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHIRELAYRIGHT" "$RELAY_RIGHT_ROM"

printf '==> assembling source-first Phi-Agent Gym task H / Key Gate LEFT\n'
KEY_LEFT_SRC="$ROOT/benchmarks/agent-gym-key-gate-left/main.asm"
KEY_LEFT_OBJ="$WORK/phi-agent-gym-key-gate-left.o"
KEY_LEFT_ROM="$WORK/phi-agent-gym-key-gate-left.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$KEY_LEFT_OBJ" "$KEY_LEFT_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$KEY_LEFT_ROM" "$KEY_LEFT_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHIKEYLEFT" "$KEY_LEFT_ROM"

printf '==> assembling source-first Phi-Agent Gym task I / Key Gate RIGHT\n'
KEY_RIGHT_SRC="$ROOT/benchmarks/agent-gym-key-gate-right/main.asm"
KEY_RIGHT_OBJ="$WORK/phi-agent-gym-key-gate-right.o"
KEY_RIGHT_ROM="$WORK/phi-agent-gym-key-gate-right.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$KEY_RIGHT_OBJ" "$KEY_RIGHT_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$KEY_RIGHT_ROM" "$KEY_RIGHT_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHIKEYRIGHT" "$KEY_RIGHT_ROM"

printf '==> assembling source-first Phi-Agent Gym task J / Power Chain LEFT\n'
POWER_LEFT_SRC="$ROOT/benchmarks/agent-gym-power-chain-left/main.asm"
POWER_LEFT_OBJ="$WORK/phi-agent-gym-power-chain-left.o"
POWER_LEFT_ROM="$WORK/phi-agent-gym-power-chain-left.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$POWER_LEFT_OBJ" "$POWER_LEFT_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$POWER_LEFT_ROM" "$POWER_LEFT_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHIPOWERLEFT" "$POWER_LEFT_ROM"

printf '==> assembling source-first Phi-Agent Gym task K / Power Chain RIGHT\n'
POWER_RIGHT_SRC="$ROOT/benchmarks/agent-gym-power-chain-right/main.asm"
POWER_RIGHT_OBJ="$WORK/phi-agent-gym-power-chain-right.o"
POWER_RIGHT_ROM="$WORK/phi-agent-gym-power-chain-right.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$POWER_RIGHT_OBJ" "$POWER_RIGHT_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$POWER_RIGHT_ROM" "$POWER_RIGHT_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHIPOWERRIGHT" "$POWER_RIGHT_ROM"

printf '==> assembling source-first Phi-Agent Gym task L / Branch Selector TRIANGLE\n'
BRANCH_TRI_SRC="$ROOT/benchmarks/agent-gym-branch-selector-triangle/main.asm"
BRANCH_TRI_OBJ="$WORK/phi-agent-gym-branch-selector-triangle.o"
BRANCH_TRI_ROM="$WORK/phi-agent-gym-branch-selector-triangle.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$BRANCH_TRI_OBJ" "$BRANCH_TRI_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$BRANCH_TRI_ROM" "$BRANCH_TRI_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHIBRANCHTRI" "$BRANCH_TRI_ROM"

printf '==> assembling source-first Phi-Agent Gym task M / Branch Selector SQUARE\n'
BRANCH_SQ_SRC="$ROOT/benchmarks/agent-gym-branch-selector-square/main.asm"
BRANCH_SQ_OBJ="$WORK/phi-agent-gym-branch-selector-square.o"
BRANCH_SQ_ROM="$WORK/phi-agent-gym-branch-selector-square.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$BRANCH_SQ_OBJ" "$BRANCH_SQ_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$BRANCH_SQ_ROM" "$BRANCH_SQ_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHIBRANCHSQ" "$BRANCH_SQ_ROM"

printf '==> assembling source-first Phi-Agent Gym task N / Nested TRIANGLE-CIRCLE\n'
NESTED_TC_SRC="$ROOT/benchmarks/agent-gym-nested-branch-triangle-circle/main.asm"
NESTED_TC_OBJ="$WORK/phi-agent-gym-nested-triangle-circle.o"
NESTED_TC_ROM="$WORK/phi-agent-gym-nested-triangle-circle.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$NESTED_TC_OBJ" "$NESTED_TC_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$NESTED_TC_ROM" "$NESTED_TC_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHINTCIRCLE" "$NESTED_TC_ROM"

printf '==> assembling source-first Phi-Agent Gym task O / Nested TRIANGLE-CROSS\n'
NESTED_TX_SRC="$ROOT/benchmarks/agent-gym-nested-branch-triangle-cross/main.asm"
NESTED_TX_OBJ="$WORK/phi-agent-gym-nested-triangle-cross.o"
NESTED_TX_ROM="$WORK/phi-agent-gym-nested-triangle-cross.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$NESTED_TX_OBJ" "$NESTED_TX_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$NESTED_TX_ROM" "$NESTED_TX_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHINTCROSS" "$NESTED_TX_ROM"

printf '==> assembling source-first Phi-Agent Gym task P / Nested SQUARE-CIRCLE\n'
NESTED_SC_SRC="$ROOT/benchmarks/agent-gym-nested-branch-square-circle/main.asm"
NESTED_SC_OBJ="$WORK/phi-agent-gym-nested-square-circle.o"
NESTED_SC_ROM="$WORK/phi-agent-gym-nested-square-circle.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$NESTED_SC_OBJ" "$NESTED_SC_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$NESTED_SC_ROM" "$NESTED_SC_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHINSCIRCLE" "$NESTED_SC_ROM"

printf '==> assembling source-first Phi-Agent Gym task Q / Nested SQUARE-CROSS\n'
NESTED_SX_SRC="$ROOT/benchmarks/agent-gym-nested-branch-square-cross/main.asm"
NESTED_SX_OBJ="$WORK/phi-agent-gym-nested-square-cross.o"
NESTED_SX_ROM="$WORK/phi-agent-gym-nested-square-cross.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$NESTED_SX_OBJ" "$NESTED_SX_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$NESTED_SX_ROM" "$NESTED_SX_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHINSCROSS" "$NESTED_SX_ROM"

printf '==> assembling source-first Phi-Agent Gym task R / Binding Memory NORMAL-TRIANGLE\n'
BIND_NT_SRC="$ROOT/benchmarks/agent-gym-binding-memory-normal-triangle/main.asm"
BIND_NT_OBJ="$WORK/phi-agent-gym-binding-memory-normal-triangle.o"
BIND_NT_ROM="$WORK/phi-agent-gym-binding-memory-normal-triangle.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$BIND_NT_OBJ" "$BIND_NT_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$BIND_NT_ROM" "$BIND_NT_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHIBINDNT" "$BIND_NT_ROM"

printf '==> assembling source-first Phi-Agent Gym task S / Binding Memory NORMAL-SQUARE\n'
BIND_NS_SRC="$ROOT/benchmarks/agent-gym-binding-memory-normal-square/main.asm"
BIND_NS_OBJ="$WORK/phi-agent-gym-binding-memory-normal-square.o"
BIND_NS_ROM="$WORK/phi-agent-gym-binding-memory-normal-square.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$BIND_NS_OBJ" "$BIND_NS_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$BIND_NS_ROM" "$BIND_NS_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHIBINDNS" "$BIND_NS_ROM"

printf '==> assembling source-first Phi-Agent Gym task T / Binding Memory SWAPPED-TRIANGLE\n'
BIND_ST_SRC="$ROOT/benchmarks/agent-gym-binding-memory-swapped-triangle/main.asm"
BIND_ST_OBJ="$WORK/phi-agent-gym-binding-memory-swapped-triangle.o"
BIND_ST_ROM="$WORK/phi-agent-gym-binding-memory-swapped-triangle.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$BIND_ST_OBJ" "$BIND_ST_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$BIND_ST_ROM" "$BIND_ST_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHIBINDST" "$BIND_ST_ROM"

printf '==> assembling source-first Phi-Agent Gym task U / Binding Memory SWAPPED-SQUARE\n'
BIND_SS_SRC="$ROOT/benchmarks/agent-gym-binding-memory-swapped-square/main.asm"
BIND_SS_OBJ="$WORK/phi-agent-gym-binding-memory-swapped-square.o"
BIND_SS_ROM="$WORK/phi-agent-gym-binding-memory-swapped-square.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$BIND_SS_OBJ" "$BIND_SS_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$BIND_SS_ROM" "$BIND_SS_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHIBINDSS" "$BIND_SS_ROM"

printf '==> assembling source-first Phi-Agent Gym task V / Compositional Recall NORMAL-TRIANGLE-MATCH\n'
COMP_NTM_SRC="$ROOT/benchmarks/agent-gym-compositional-recall-normal-triangle-match/main.asm"
COMP_NTM_OBJ="$WORK/phi-agent-gym-compositional-recall-normal-triangle-match.o"
COMP_NTM_ROM="$WORK/phi-agent-gym-compositional-recall-normal-triangle-match.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$COMP_NTM_OBJ" "$COMP_NTM_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$COMP_NTM_ROM" "$COMP_NTM_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHICRNTM" "$COMP_NTM_ROM"

printf '==> assembling source-first Phi-Agent Gym task W / Compositional Recall NORMAL-TRIANGLE-FLIP\n'
COMP_NTF_SRC="$ROOT/benchmarks/agent-gym-compositional-recall-normal-triangle-flip/main.asm"
COMP_NTF_OBJ="$WORK/phi-agent-gym-compositional-recall-normal-triangle-flip.o"
COMP_NTF_ROM="$WORK/phi-agent-gym-compositional-recall-normal-triangle-flip.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$COMP_NTF_OBJ" "$COMP_NTF_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$COMP_NTF_ROM" "$COMP_NTF_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHICRNTF" "$COMP_NTF_ROM"

printf '==> assembling source-first Phi-Agent Gym task X / Compositional Recall NORMAL-SQUARE-MATCH\n'
COMP_NSM_SRC="$ROOT/benchmarks/agent-gym-compositional-recall-normal-square-match/main.asm"
COMP_NSM_OBJ="$WORK/phi-agent-gym-compositional-recall-normal-square-match.o"
COMP_NSM_ROM="$WORK/phi-agent-gym-compositional-recall-normal-square-match.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$COMP_NSM_OBJ" "$COMP_NSM_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$COMP_NSM_ROM" "$COMP_NSM_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHICRNSM" "$COMP_NSM_ROM"

printf '==> assembling source-first Phi-Agent Gym task Y / Compositional Recall NORMAL-SQUARE-FLIP\n'
COMP_NSF_SRC="$ROOT/benchmarks/agent-gym-compositional-recall-normal-square-flip/main.asm"
COMP_NSF_OBJ="$WORK/phi-agent-gym-compositional-recall-normal-square-flip.o"
COMP_NSF_ROM="$WORK/phi-agent-gym-compositional-recall-normal-square-flip.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$COMP_NSF_OBJ" "$COMP_NSF_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$COMP_NSF_ROM" "$COMP_NSF_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHICRNSF" "$COMP_NSF_ROM"

printf '==> assembling source-first Phi-Agent Gym task Z / Compositional Recall SWAPPED-TRIANGLE-MATCH\n'
COMP_STM_SRC="$ROOT/benchmarks/agent-gym-compositional-recall-swapped-triangle-match/main.asm"
COMP_STM_OBJ="$WORK/phi-agent-gym-compositional-recall-swapped-triangle-match.o"
COMP_STM_ROM="$WORK/phi-agent-gym-compositional-recall-swapped-triangle-match.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$COMP_STM_OBJ" "$COMP_STM_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$COMP_STM_ROM" "$COMP_STM_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHICRSTM" "$COMP_STM_ROM"

printf '==> assembling source-first Phi-Agent Gym task AA / Compositional Recall SWAPPED-TRIANGLE-FLIP\n'
COMP_STF_SRC="$ROOT/benchmarks/agent-gym-compositional-recall-swapped-triangle-flip/main.asm"
COMP_STF_OBJ="$WORK/phi-agent-gym-compositional-recall-swapped-triangle-flip.o"
COMP_STF_ROM="$WORK/phi-agent-gym-compositional-recall-swapped-triangle-flip.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$COMP_STF_OBJ" "$COMP_STF_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$COMP_STF_ROM" "$COMP_STF_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHICRSTF" "$COMP_STF_ROM"

printf '==> assembling source-first Phi-Agent Gym task AB / Compositional Recall SWAPPED-SQUARE-MATCH\n'
COMP_SSM_SRC="$ROOT/benchmarks/agent-gym-compositional-recall-swapped-square-match/main.asm"
COMP_SSM_OBJ="$WORK/phi-agent-gym-compositional-recall-swapped-square-match.o"
COMP_SSM_ROM="$WORK/phi-agent-gym-compositional-recall-swapped-square-match.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$COMP_SSM_OBJ" "$COMP_SSM_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$COMP_SSM_ROM" "$COMP_SSM_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHICRSSM" "$COMP_SSM_ROM"

printf '==> assembling source-first Phi-Agent Gym task AC / Compositional Recall SWAPPED-SQUARE-FLIP\n'
COMP_SSF_SRC="$ROOT/benchmarks/agent-gym-compositional-recall-swapped-square-flip/main.asm"
COMP_SSF_OBJ="$WORK/phi-agent-gym-compositional-recall-swapped-square-flip.o"
COMP_SSF_ROM="$WORK/phi-agent-gym-compositional-recall-swapped-square-flip.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$COMP_SSF_OBJ" "$COMP_SSF_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$COMP_SSF_ROM" "$COMP_SSF_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHICRSSF" "$COMP_SSF_ROM"

printf '==> assembling source-first Sequential Rule normal-triangle-match-match\n'
SEQ_NTMM_SRC="$ROOT/benchmarks/agent-gym-sequential-rule-normal-triangle-match-match/main.asm"
SEQ_NTMM_OBJ="$WORK/agent-gym-sequential-rule-normal-triangle-match-match.o"
SEQ_NTMM_ROM="$WORK/agent-gym-sequential-rule-normal-triangle-match-match.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$SEQ_NTMM_OBJ" "$SEQ_NTMM_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$SEQ_NTMM_ROM" "$SEQ_NTMM_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHISNTMM" "$SEQ_NTMM_ROM"

printf '==> assembling source-first Sequential Rule normal-triangle-match-flip\n'
SEQ_NTMF_SRC="$ROOT/benchmarks/agent-gym-sequential-rule-normal-triangle-match-flip/main.asm"
SEQ_NTMF_OBJ="$WORK/agent-gym-sequential-rule-normal-triangle-match-flip.o"
SEQ_NTMF_ROM="$WORK/agent-gym-sequential-rule-normal-triangle-match-flip.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$SEQ_NTMF_OBJ" "$SEQ_NTMF_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$SEQ_NTMF_ROM" "$SEQ_NTMF_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHISNTMF" "$SEQ_NTMF_ROM"

printf '==> assembling source-first Sequential Rule normal-triangle-flip-match\n'
SEQ_NTFM_SRC="$ROOT/benchmarks/agent-gym-sequential-rule-normal-triangle-flip-match/main.asm"
SEQ_NTFM_OBJ="$WORK/agent-gym-sequential-rule-normal-triangle-flip-match.o"
SEQ_NTFM_ROM="$WORK/agent-gym-sequential-rule-normal-triangle-flip-match.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$SEQ_NTFM_OBJ" "$SEQ_NTFM_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$SEQ_NTFM_ROM" "$SEQ_NTFM_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHISNTFM" "$SEQ_NTFM_ROM"

printf '==> assembling source-first Sequential Rule normal-triangle-flip-flip\n'
SEQ_NTFF_SRC="$ROOT/benchmarks/agent-gym-sequential-rule-normal-triangle-flip-flip/main.asm"
SEQ_NTFF_OBJ="$WORK/agent-gym-sequential-rule-normal-triangle-flip-flip.o"
SEQ_NTFF_ROM="$WORK/agent-gym-sequential-rule-normal-triangle-flip-flip.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$SEQ_NTFF_OBJ" "$SEQ_NTFF_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$SEQ_NTFF_ROM" "$SEQ_NTFF_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHISNTFF" "$SEQ_NTFF_ROM"

printf '==> assembling source-first Sequential Rule normal-square-match-match\n'
SEQ_NSMM_SRC="$ROOT/benchmarks/agent-gym-sequential-rule-normal-square-match-match/main.asm"
SEQ_NSMM_OBJ="$WORK/agent-gym-sequential-rule-normal-square-match-match.o"
SEQ_NSMM_ROM="$WORK/agent-gym-sequential-rule-normal-square-match-match.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$SEQ_NSMM_OBJ" "$SEQ_NSMM_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$SEQ_NSMM_ROM" "$SEQ_NSMM_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHISNSMM" "$SEQ_NSMM_ROM"

printf '==> assembling source-first Sequential Rule normal-square-match-flip\n'
SEQ_NSMF_SRC="$ROOT/benchmarks/agent-gym-sequential-rule-normal-square-match-flip/main.asm"
SEQ_NSMF_OBJ="$WORK/agent-gym-sequential-rule-normal-square-match-flip.o"
SEQ_NSMF_ROM="$WORK/agent-gym-sequential-rule-normal-square-match-flip.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$SEQ_NSMF_OBJ" "$SEQ_NSMF_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$SEQ_NSMF_ROM" "$SEQ_NSMF_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHISNSMF" "$SEQ_NSMF_ROM"

printf '==> assembling source-first Sequential Rule normal-square-flip-match\n'
SEQ_NSFM_SRC="$ROOT/benchmarks/agent-gym-sequential-rule-normal-square-flip-match/main.asm"
SEQ_NSFM_OBJ="$WORK/agent-gym-sequential-rule-normal-square-flip-match.o"
SEQ_NSFM_ROM="$WORK/agent-gym-sequential-rule-normal-square-flip-match.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$SEQ_NSFM_OBJ" "$SEQ_NSFM_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$SEQ_NSFM_ROM" "$SEQ_NSFM_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHISNSFM" "$SEQ_NSFM_ROM"

printf '==> assembling source-first Sequential Rule normal-square-flip-flip\n'
SEQ_NSFF_SRC="$ROOT/benchmarks/agent-gym-sequential-rule-normal-square-flip-flip/main.asm"
SEQ_NSFF_OBJ="$WORK/agent-gym-sequential-rule-normal-square-flip-flip.o"
SEQ_NSFF_ROM="$WORK/agent-gym-sequential-rule-normal-square-flip-flip.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$SEQ_NSFF_OBJ" "$SEQ_NSFF_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$SEQ_NSFF_ROM" "$SEQ_NSFF_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHISNSFF" "$SEQ_NSFF_ROM"

printf '==> assembling source-first Sequential Rule swapped-triangle-match-match\n'
SEQ_STMM_SRC="$ROOT/benchmarks/agent-gym-sequential-rule-swapped-triangle-match-match/main.asm"
SEQ_STMM_OBJ="$WORK/agent-gym-sequential-rule-swapped-triangle-match-match.o"
SEQ_STMM_ROM="$WORK/agent-gym-sequential-rule-swapped-triangle-match-match.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$SEQ_STMM_OBJ" "$SEQ_STMM_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$SEQ_STMM_ROM" "$SEQ_STMM_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHISSTMM" "$SEQ_STMM_ROM"

printf '==> assembling source-first Sequential Rule swapped-triangle-match-flip\n'
SEQ_STMF_SRC="$ROOT/benchmarks/agent-gym-sequential-rule-swapped-triangle-match-flip/main.asm"
SEQ_STMF_OBJ="$WORK/agent-gym-sequential-rule-swapped-triangle-match-flip.o"
SEQ_STMF_ROM="$WORK/agent-gym-sequential-rule-swapped-triangle-match-flip.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$SEQ_STMF_OBJ" "$SEQ_STMF_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$SEQ_STMF_ROM" "$SEQ_STMF_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHISSTMF" "$SEQ_STMF_ROM"

printf '==> assembling source-first Sequential Rule swapped-triangle-flip-match\n'
SEQ_STFM_SRC="$ROOT/benchmarks/agent-gym-sequential-rule-swapped-triangle-flip-match/main.asm"
SEQ_STFM_OBJ="$WORK/agent-gym-sequential-rule-swapped-triangle-flip-match.o"
SEQ_STFM_ROM="$WORK/agent-gym-sequential-rule-swapped-triangle-flip-match.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$SEQ_STFM_OBJ" "$SEQ_STFM_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$SEQ_STFM_ROM" "$SEQ_STFM_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHISSTFM" "$SEQ_STFM_ROM"

printf '==> assembling source-first Sequential Rule swapped-triangle-flip-flip\n'
SEQ_STFF_SRC="$ROOT/benchmarks/agent-gym-sequential-rule-swapped-triangle-flip-flip/main.asm"
SEQ_STFF_OBJ="$WORK/agent-gym-sequential-rule-swapped-triangle-flip-flip.o"
SEQ_STFF_ROM="$WORK/agent-gym-sequential-rule-swapped-triangle-flip-flip.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$SEQ_STFF_OBJ" "$SEQ_STFF_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$SEQ_STFF_ROM" "$SEQ_STFF_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHISSTFF" "$SEQ_STFF_ROM"

printf '==> assembling source-first Sequential Rule swapped-square-match-match\n'
SEQ_SSMM_SRC="$ROOT/benchmarks/agent-gym-sequential-rule-swapped-square-match-match/main.asm"
SEQ_SSMM_OBJ="$WORK/agent-gym-sequential-rule-swapped-square-match-match.o"
SEQ_SSMM_ROM="$WORK/agent-gym-sequential-rule-swapped-square-match-match.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$SEQ_SSMM_OBJ" "$SEQ_SSMM_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$SEQ_SSMM_ROM" "$SEQ_SSMM_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHISSSMM" "$SEQ_SSMM_ROM"

printf '==> assembling source-first Sequential Rule swapped-square-match-flip\n'
SEQ_SSMF_SRC="$ROOT/benchmarks/agent-gym-sequential-rule-swapped-square-match-flip/main.asm"
SEQ_SSMF_OBJ="$WORK/agent-gym-sequential-rule-swapped-square-match-flip.o"
SEQ_SSMF_ROM="$WORK/agent-gym-sequential-rule-swapped-square-match-flip.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$SEQ_SSMF_OBJ" "$SEQ_SSMF_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$SEQ_SSMF_ROM" "$SEQ_SSMF_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHISSSMF" "$SEQ_SSMF_ROM"

printf '==> assembling source-first Sequential Rule swapped-square-flip-match\n'
SEQ_SSFM_SRC="$ROOT/benchmarks/agent-gym-sequential-rule-swapped-square-flip-match/main.asm"
SEQ_SSFM_OBJ="$WORK/agent-gym-sequential-rule-swapped-square-flip-match.o"
SEQ_SSFM_ROM="$WORK/agent-gym-sequential-rule-swapped-square-flip-match.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$SEQ_SSFM_OBJ" "$SEQ_SSFM_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$SEQ_SSFM_ROM" "$SEQ_SSFM_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHISSSFM" "$SEQ_SSFM_ROM"

printf '==> assembling source-first Sequential Rule swapped-square-flip-flip\n'
SEQ_SSFF_SRC="$ROOT/benchmarks/agent-gym-sequential-rule-swapped-square-flip-flip/main.asm"
SEQ_SSFF_OBJ="$WORK/agent-gym-sequential-rule-swapped-square-flip-flip.o"
SEQ_SSFF_ROM="$WORK/agent-gym-sequential-rule-swapped-square-flip-flip.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$SEQ_SSFF_OBJ" "$SEQ_SSFF_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$SEQ_SSFF_ROM" "$SEQ_SSFF_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHISSSFF" "$SEQ_SSFF_ROM"

printf '==> assembling source-first Selective Context a-circle-triangle-match\n'
CTX_A_CIR_TM_SRC="$ROOT/benchmarks/agent-gym-selective-context-a-circle-triangle-match/main.asm"
CTX_A_CIR_TM_OBJ="$WORK/agent-gym-selective-context-a-circle-triangle-match.o"
CTX_A_CIR_TM_ROM="$WORK/agent-gym-selective-context-a-circle-triangle-match.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$CTX_A_CIR_TM_OBJ" "$CTX_A_CIR_TM_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$CTX_A_CIR_TM_ROM" "$CTX_A_CIR_TM_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHIACITM" "$CTX_A_CIR_TM_ROM"

printf '==> assembling source-first Selective Context a-circle-triangle-flip\n'
CTX_A_CIR_TF_SRC="$ROOT/benchmarks/agent-gym-selective-context-a-circle-triangle-flip/main.asm"
CTX_A_CIR_TF_OBJ="$WORK/agent-gym-selective-context-a-circle-triangle-flip.o"
CTX_A_CIR_TF_ROM="$WORK/agent-gym-selective-context-a-circle-triangle-flip.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$CTX_A_CIR_TF_OBJ" "$CTX_A_CIR_TF_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$CTX_A_CIR_TF_ROM" "$CTX_A_CIR_TF_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHIACITF" "$CTX_A_CIR_TF_ROM"

printf '==> assembling source-first Selective Context a-circle-square-match\n'
CTX_A_CIR_SM_SRC="$ROOT/benchmarks/agent-gym-selective-context-a-circle-square-match/main.asm"
CTX_A_CIR_SM_OBJ="$WORK/agent-gym-selective-context-a-circle-square-match.o"
CTX_A_CIR_SM_ROM="$WORK/agent-gym-selective-context-a-circle-square-match.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$CTX_A_CIR_SM_OBJ" "$CTX_A_CIR_SM_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$CTX_A_CIR_SM_ROM" "$CTX_A_CIR_SM_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHIACISM" "$CTX_A_CIR_SM_ROM"

printf '==> assembling source-first Selective Context a-circle-square-flip\n'
CTX_A_CIR_SF_SRC="$ROOT/benchmarks/agent-gym-selective-context-a-circle-square-flip/main.asm"
CTX_A_CIR_SF_OBJ="$WORK/agent-gym-selective-context-a-circle-square-flip.o"
CTX_A_CIR_SF_ROM="$WORK/agent-gym-selective-context-a-circle-square-flip.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$CTX_A_CIR_SF_OBJ" "$CTX_A_CIR_SF_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$CTX_A_CIR_SF_ROM" "$CTX_A_CIR_SF_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHIACISF" "$CTX_A_CIR_SF_ROM"

printf '==> assembling source-first Selective Context a-cross-triangle-match\n'
CTX_A_CRS_TM_SRC="$ROOT/benchmarks/agent-gym-selective-context-a-cross-triangle-match/main.asm"
CTX_A_CRS_TM_OBJ="$WORK/agent-gym-selective-context-a-cross-triangle-match.o"
CTX_A_CRS_TM_ROM="$WORK/agent-gym-selective-context-a-cross-triangle-match.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$CTX_A_CRS_TM_OBJ" "$CTX_A_CRS_TM_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$CTX_A_CRS_TM_ROM" "$CTX_A_CRS_TM_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHIACRTM" "$CTX_A_CRS_TM_ROM"

printf '==> assembling source-first Selective Context a-cross-triangle-flip\n'
CTX_A_CRS_TF_SRC="$ROOT/benchmarks/agent-gym-selective-context-a-cross-triangle-flip/main.asm"
CTX_A_CRS_TF_OBJ="$WORK/agent-gym-selective-context-a-cross-triangle-flip.o"
CTX_A_CRS_TF_ROM="$WORK/agent-gym-selective-context-a-cross-triangle-flip.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$CTX_A_CRS_TF_OBJ" "$CTX_A_CRS_TF_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$CTX_A_CRS_TF_ROM" "$CTX_A_CRS_TF_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHIACRTF" "$CTX_A_CRS_TF_ROM"

printf '==> assembling source-first Selective Context a-cross-square-match\n'
CTX_A_CRS_SM_SRC="$ROOT/benchmarks/agent-gym-selective-context-a-cross-square-match/main.asm"
CTX_A_CRS_SM_OBJ="$WORK/agent-gym-selective-context-a-cross-square-match.o"
CTX_A_CRS_SM_ROM="$WORK/agent-gym-selective-context-a-cross-square-match.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$CTX_A_CRS_SM_OBJ" "$CTX_A_CRS_SM_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$CTX_A_CRS_SM_ROM" "$CTX_A_CRS_SM_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHIACRSM" "$CTX_A_CRS_SM_ROM"

printf '==> assembling source-first Selective Context a-cross-square-flip\n'
CTX_A_CRS_SF_SRC="$ROOT/benchmarks/agent-gym-selective-context-a-cross-square-flip/main.asm"
CTX_A_CRS_SF_OBJ="$WORK/agent-gym-selective-context-a-cross-square-flip.o"
CTX_A_CRS_SF_ROM="$WORK/agent-gym-selective-context-a-cross-square-flip.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$CTX_A_CRS_SF_OBJ" "$CTX_A_CRS_SF_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$CTX_A_CRS_SF_ROM" "$CTX_A_CRS_SF_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHIACRSF" "$CTX_A_CRS_SF_ROM"

printf '==> assembling source-first Selective Context b-circle-triangle-match\n'
CTX_B_CIR_TM_SRC="$ROOT/benchmarks/agent-gym-selective-context-b-circle-triangle-match/main.asm"
CTX_B_CIR_TM_OBJ="$WORK/agent-gym-selective-context-b-circle-triangle-match.o"
CTX_B_CIR_TM_ROM="$WORK/agent-gym-selective-context-b-circle-triangle-match.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$CTX_B_CIR_TM_OBJ" "$CTX_B_CIR_TM_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$CTX_B_CIR_TM_ROM" "$CTX_B_CIR_TM_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHIBCITM" "$CTX_B_CIR_TM_ROM"

printf '==> assembling source-first Selective Context b-circle-triangle-flip\n'
CTX_B_CIR_TF_SRC="$ROOT/benchmarks/agent-gym-selective-context-b-circle-triangle-flip/main.asm"
CTX_B_CIR_TF_OBJ="$WORK/agent-gym-selective-context-b-circle-triangle-flip.o"
CTX_B_CIR_TF_ROM="$WORK/agent-gym-selective-context-b-circle-triangle-flip.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$CTX_B_CIR_TF_OBJ" "$CTX_B_CIR_TF_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$CTX_B_CIR_TF_ROM" "$CTX_B_CIR_TF_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHIBCITF" "$CTX_B_CIR_TF_ROM"

printf '==> assembling source-first Selective Context b-circle-square-match\n'
CTX_B_CIR_SM_SRC="$ROOT/benchmarks/agent-gym-selective-context-b-circle-square-match/main.asm"
CTX_B_CIR_SM_OBJ="$WORK/agent-gym-selective-context-b-circle-square-match.o"
CTX_B_CIR_SM_ROM="$WORK/agent-gym-selective-context-b-circle-square-match.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$CTX_B_CIR_SM_OBJ" "$CTX_B_CIR_SM_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$CTX_B_CIR_SM_ROM" "$CTX_B_CIR_SM_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHIBCISM" "$CTX_B_CIR_SM_ROM"

printf '==> assembling source-first Selective Context b-circle-square-flip\n'
CTX_B_CIR_SF_SRC="$ROOT/benchmarks/agent-gym-selective-context-b-circle-square-flip/main.asm"
CTX_B_CIR_SF_OBJ="$WORK/agent-gym-selective-context-b-circle-square-flip.o"
CTX_B_CIR_SF_ROM="$WORK/agent-gym-selective-context-b-circle-square-flip.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$CTX_B_CIR_SF_OBJ" "$CTX_B_CIR_SF_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$CTX_B_CIR_SF_ROM" "$CTX_B_CIR_SF_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHIBCISF" "$CTX_B_CIR_SF_ROM"

printf '==> assembling source-first Selective Context b-cross-triangle-match\n'
CTX_B_CRS_TM_SRC="$ROOT/benchmarks/agent-gym-selective-context-b-cross-triangle-match/main.asm"
CTX_B_CRS_TM_OBJ="$WORK/agent-gym-selective-context-b-cross-triangle-match.o"
CTX_B_CRS_TM_ROM="$WORK/agent-gym-selective-context-b-cross-triangle-match.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$CTX_B_CRS_TM_OBJ" "$CTX_B_CRS_TM_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$CTX_B_CRS_TM_ROM" "$CTX_B_CRS_TM_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHIBCRTM" "$CTX_B_CRS_TM_ROM"

printf '==> assembling source-first Selective Context b-cross-triangle-flip\n'
CTX_B_CRS_TF_SRC="$ROOT/benchmarks/agent-gym-selective-context-b-cross-triangle-flip/main.asm"
CTX_B_CRS_TF_OBJ="$WORK/agent-gym-selective-context-b-cross-triangle-flip.o"
CTX_B_CRS_TF_ROM="$WORK/agent-gym-selective-context-b-cross-triangle-flip.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$CTX_B_CRS_TF_OBJ" "$CTX_B_CRS_TF_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$CTX_B_CRS_TF_ROM" "$CTX_B_CRS_TF_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHIBCRTF" "$CTX_B_CRS_TF_ROM"

printf '==> assembling source-first Selective Context b-cross-square-match\n'
CTX_B_CRS_SM_SRC="$ROOT/benchmarks/agent-gym-selective-context-b-cross-square-match/main.asm"
CTX_B_CRS_SM_OBJ="$WORK/agent-gym-selective-context-b-cross-square-match.o"
CTX_B_CRS_SM_ROM="$WORK/agent-gym-selective-context-b-cross-square-match.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$CTX_B_CRS_SM_OBJ" "$CTX_B_CRS_SM_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$CTX_B_CRS_SM_ROM" "$CTX_B_CRS_SM_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHIBCRSM" "$CTX_B_CRS_SM_ROM"

printf '==> assembling source-first Selective Context b-cross-square-flip\n'
CTX_B_CRS_SF_SRC="$ROOT/benchmarks/agent-gym-selective-context-b-cross-square-flip/main.asm"
CTX_B_CRS_SF_OBJ="$WORK/agent-gym-selective-context-b-cross-square-flip.o"
CTX_B_CRS_SF_ROM="$WORK/agent-gym-selective-context-b-cross-square-flip.gb"
PATH="$WORK/rgbds-install/bin:$PATH" rgbasm -o "$CTX_B_CRS_SF_OBJ" "$CTX_B_CRS_SF_SRC"
PATH="$WORK/rgbds-install/bin:$PATH" rgblink -o "$CTX_B_CRS_SF_ROM" "$CTX_B_CRS_SF_OBJ"
PATH="$WORK/rgbds-install/bin:$PATH" rgbfix -v -p 0x00 -t "PHIBCRSF" "$CTX_B_CRS_SF_ROM"

printf '==> verifying Selective Context build-path uniqueness\n'
CTX_SOURCE_PATH_COUNT=$(printf '%s\n' "$CTX_A_CIR_TM_SRC" "$CTX_A_CIR_TF_SRC" "$CTX_A_CIR_SM_SRC" "$CTX_A_CIR_SF_SRC" "$CTX_A_CRS_TM_SRC" "$CTX_A_CRS_TF_SRC" "$CTX_A_CRS_SM_SRC" "$CTX_A_CRS_SF_SRC" "$CTX_B_CIR_TM_SRC" "$CTX_B_CIR_TF_SRC" "$CTX_B_CIR_SM_SRC" "$CTX_B_CIR_SF_SRC" "$CTX_B_CRS_TM_SRC" "$CTX_B_CRS_TF_SRC" "$CTX_B_CRS_SM_SRC" "$CTX_B_CRS_SF_SRC" | sort -u | wc -l)
CTX_ROM_PATH_COUNT=$(printf '%s\n' "$CTX_A_CIR_TM_ROM" "$CTX_A_CIR_TF_ROM" "$CTX_A_CIR_SM_ROM" "$CTX_A_CIR_SF_ROM" "$CTX_A_CRS_TM_ROM" "$CTX_A_CRS_TF_ROM" "$CTX_A_CRS_SM_ROM" "$CTX_A_CRS_SF_ROM" "$CTX_B_CIR_TM_ROM" "$CTX_B_CIR_TF_ROM" "$CTX_B_CIR_SM_ROM" "$CTX_B_CIR_SF_ROM" "$CTX_B_CRS_TM_ROM" "$CTX_B_CRS_TF_ROM" "$CTX_B_CRS_SM_ROM" "$CTX_B_CRS_SF_ROM" | sort -u | wc -l)
if [ "$CTX_SOURCE_PATH_COUNT" -ne 16 ] || [ "$CTX_ROM_PATH_COUNT" -ne 16 ]; then
  echo "Selective Context provenance paths are not unique: sources=$CTX_SOURCE_PATH_COUNT roms=$CTX_ROM_PATH_COUNT" >&2
  exit 1
fi

printf '==> benchmark suite hashes\n'
sha256sum "$GYM_SRC" "$GYM_ROM" "$MIRROR_SRC" "$MIRROR_ROM" "$WALL_SRC" "$WALL_ROM" "$TEMP_LEFT_SRC" "$TEMP_LEFT_ROM" "$TEMP_RIGHT_SRC" "$TEMP_RIGHT_ROM" "$RELAY_LEFT_SRC" "$RELAY_LEFT_ROM" "$RELAY_RIGHT_SRC" "$RELAY_RIGHT_ROM" "$KEY_LEFT_SRC" "$KEY_LEFT_ROM" "$KEY_RIGHT_SRC" "$KEY_RIGHT_ROM" "$POWER_LEFT_SRC" "$POWER_LEFT_ROM" "$POWER_RIGHT_SRC" "$POWER_RIGHT_ROM" "$BRANCH_TRI_SRC" "$BRANCH_TRI_ROM" "$BRANCH_SQ_SRC" "$BRANCH_SQ_ROM" "$NESTED_TC_SRC" "$NESTED_TC_ROM" "$NESTED_TX_SRC" "$NESTED_TX_ROM" "$NESTED_SC_SRC" "$NESTED_SC_ROM" "$NESTED_SX_SRC" "$NESTED_SX_ROM" "$BIND_NT_SRC" "$BIND_NT_ROM" "$BIND_NS_SRC" "$BIND_NS_ROM" "$BIND_ST_SRC" "$BIND_ST_ROM" "$BIND_SS_SRC" "$BIND_SS_ROM" "$COMP_NTM_SRC" "$COMP_NTM_ROM" "$COMP_NTF_SRC" "$COMP_NTF_ROM" "$COMP_NSM_SRC" "$COMP_NSM_ROM" "$COMP_NSF_SRC" "$COMP_NSF_ROM" "$COMP_STM_SRC" "$COMP_STM_ROM" "$COMP_STF_SRC" "$COMP_STF_ROM" "$COMP_SSM_SRC" "$COMP_SSM_ROM" "$COMP_SSF_SRC" "$COMP_SSF_ROM" "$SEQ_NTMM_SRC" "$SEQ_NTMM_ROM" "$SEQ_NTMF_SRC" "$SEQ_NTMF_ROM" "$SEQ_NTFM_SRC" "$SEQ_NTFM_ROM" "$SEQ_NTFF_SRC" "$SEQ_NTFF_ROM" "$SEQ_NSMM_SRC" "$SEQ_NSMM_ROM" "$SEQ_NSMF_SRC" "$SEQ_NSMF_ROM" "$SEQ_NSFM_SRC" "$SEQ_NSFM_ROM" "$SEQ_NSFF_SRC" "$SEQ_NSFF_ROM" "$SEQ_STMM_SRC" "$SEQ_STMM_ROM" "$SEQ_STMF_SRC" "$SEQ_STMF_ROM" "$SEQ_STFM_SRC" "$SEQ_STFM_ROM" "$SEQ_STFF_SRC" "$SEQ_STFF_ROM" "$SEQ_SSMM_SRC" "$SEQ_SSMM_ROM" "$SEQ_SSMF_SRC" "$SEQ_SSMF_ROM" "$SEQ_SSFM_SRC" "$SEQ_SSFM_ROM" "$SEQ_SSFF_SRC" "$SEQ_SSFF_ROM" "$CTX_A_CIR_TM_SRC" "$CTX_A_CIR_TM_ROM" "$CTX_A_CIR_TF_SRC" "$CTX_A_CIR_TF_ROM" "$CTX_A_CIR_SM_SRC" "$CTX_A_CIR_SM_ROM" "$CTX_A_CIR_SF_SRC" "$CTX_A_CIR_SF_ROM" "$CTX_A_CRS_TM_SRC" "$CTX_A_CRS_TM_ROM" "$CTX_A_CRS_TF_SRC" "$CTX_A_CRS_TF_ROM" "$CTX_A_CRS_SM_SRC" "$CTX_A_CRS_SM_ROM" "$CTX_A_CRS_SF_SRC" "$CTX_A_CRS_SF_ROM" "$CTX_B_CIR_TM_SRC" "$CTX_B_CIR_TM_ROM" "$CTX_B_CIR_TF_SRC" "$CTX_B_CIR_TF_ROM" "$CTX_B_CIR_SM_SRC" "$CTX_B_CIR_SM_ROM" "$CTX_B_CIR_SF_SRC" "$CTX_B_CIR_SF_ROM" "$CTX_B_CRS_TM_SRC" "$CTX_B_CRS_TM_ROM" "$CTX_B_CRS_TF_SRC" "$CTX_B_CRS_TF_ROM" "$CTX_B_CRS_SM_SRC" "$CTX_B_CRS_SM_ROM" "$CTX_B_CRS_SF_SRC" "$CTX_B_CRS_SF_ROM"

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

printf '==> running Phi-Agent Gym task C / Wall Detour qualification\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$WALL_ROM" \
  --source "$WALL_SRC" \
  --task "wall-detour-v1" \
  --receipt "$ROOT/artifacts/agent-gym-wall-detour-qualification.json"

printf '==> running Phi-Agent Gym task D / Temporal Cue LEFT qualification\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$TEMP_LEFT_ROM" \
  --source "$TEMP_LEFT_SRC" \
  --task "temporal-cue-left-v1" \
  --receipt "$ROOT/artifacts/agent-gym-temporal-left-qualification.json"

printf '==> running Phi-Agent Gym task E / Temporal Cue RIGHT qualification\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$TEMP_RIGHT_ROM" \
  --source "$TEMP_RIGHT_SRC" \
  --task "temporal-cue-right-v1" \
  --receipt "$ROOT/artifacts/agent-gym-temporal-right-qualification.json"

printf '==> running Temporal Cue pair memory-boundary qualification\n'
cargo run -p phicade-libretro --bin temporal_cue_qualify -- \
  --core "$CORE" \
  --left-rom "$TEMP_LEFT_ROM" \
  --right-rom "$TEMP_RIGHT_ROM" \
  --receipt "$ROOT/artifacts/temporal-cue-pair-qualification.json"

printf '==> running Phi-Agent Gym task F / Relay Rooms LEFT qualification\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$RELAY_LEFT_ROM" \
  --source "$RELAY_LEFT_SRC" \
  --task "relay-rooms-left-v1" \
  --receipt "$ROOT/artifacts/agent-gym-relay-left-qualification.json"

printf '==> running Phi-Agent Gym task G / Relay Rooms RIGHT qualification\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$RELAY_RIGHT_ROM" \
  --source "$RELAY_RIGHT_SRC" \
  --task "relay-rooms-right-v1" \
  --receipt "$ROOT/artifacts/agent-gym-relay-right-qualification.json"

printf '==> running Relay Rooms pair multi-room qualification\n'
cargo run -p phicade-libretro --bin relay_rooms_qualify -- \
  --core "$CORE" \
  --left-rom "$RELAY_LEFT_ROM" \
  --right-rom "$RELAY_RIGHT_ROM" \
  --receipt "$ROOT/artifacts/relay-rooms-pair-qualification.json"

printf '==> running Phi-Agent Gym task H / Key Gate LEFT qualification\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$KEY_LEFT_ROM" \
  --source "$KEY_LEFT_SRC" \
  --task "key-gate-left-v1" \
  --receipt "$ROOT/artifacts/agent-gym-key-gate-left-qualification.json"

printf '==> running Phi-Agent Gym task I / Key Gate RIGHT qualification\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$KEY_RIGHT_ROM" \
  --source "$KEY_RIGHT_SRC" \
  --task "key-gate-right-v1" \
  --receipt "$ROOT/artifacts/agent-gym-key-gate-right-qualification.json"

printf '==> running Key Gate pair state-dependency qualification\n'
cargo run -p phicade-libretro --bin key_gate_qualify -- \
  --core "$CORE" \
  --left-rom "$KEY_LEFT_ROM" \
  --right-rom "$KEY_RIGHT_ROM" \
  --receipt "$ROOT/artifacts/key-gate-pair-qualification.json"

printf '==> running Power Chain pair ordered-causality qualification\n'
cargo run -p phicade-libretro --bin power_chain_qualify -- \
  --core "$CORE" \
  --left-rom "$POWER_LEFT_ROM" \
  --right-rom "$POWER_RIGHT_ROM" \
  --receipt "$ROOT/artifacts/power-chain-pair-qualification.json"

printf '==> running Phi-Agent Gym task J / Power Chain LEFT qualification\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$POWER_LEFT_ROM" \
  --source "$POWER_LEFT_SRC" \
  --task "power-chain-left-v1" \
  --receipt "$ROOT/artifacts/agent-gym-power-chain-left-qualification.json"

printf '==> running Phi-Agent Gym task K / Power Chain RIGHT qualification\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$POWER_RIGHT_ROM" \
  --source "$POWER_RIGHT_SRC" \
  --task "power-chain-right-v1" \
  --receipt "$ROOT/artifacts/agent-gym-power-chain-right-qualification.json"

printf '==> running Branch Selector pair conditional-branch qualification\n'
cargo run -p phicade-libretro --bin branch_selector_qualify -- \
  --core "$CORE" \
  --triangle-rom "$BRANCH_TRI_ROM" \
  --square-rom "$BRANCH_SQ_ROM" \
  --receipt "$ROOT/artifacts/branch-selector-pair-qualification.json"

printf '==> running Phi-Agent Gym task L / Branch Selector TRIANGLE qualification\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$BRANCH_TRI_ROM" \
  --source "$BRANCH_TRI_SRC" \
  --task "branch-selector-triangle-v1" \
  --receipt "$ROOT/artifacts/agent-gym-branch-selector-triangle-qualification.json"

printf '==> running Phi-Agent Gym task M / Branch Selector SQUARE qualification\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$BRANCH_SQ_ROM" \
  --source "$BRANCH_SQ_SRC" \
  --task "branch-selector-square-v1" \
  --receipt "$ROOT/artifacts/agent-gym-branch-selector-square-qualification.json"

printf '==> running Nested Branch Graph factorial qualification\n'
cargo run -p phicade-libretro --bin nested_branch_qualify -- \
  --core "$CORE" \
  --triangle-circle-rom "$NESTED_TC_ROM" \
  --triangle-cross-rom "$NESTED_TX_ROM" \
  --square-circle-rom "$NESTED_SC_ROM" \
  --square-cross-rom "$NESTED_SX_ROM" \
  --receipt "$ROOT/artifacts/nested-branch-qualification.json"

printf '==> running Phi-Agent Gym task N / Nested TRIANGLE-CIRCLE qualification\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$NESTED_TC_ROM" \
  --source "$NESTED_TC_SRC" \
  --task "nested-branch-triangle-circle-v1" \
  --receipt "$ROOT/artifacts/agent-gym-nested-branch-triangle-circle-qualification.json"

printf '==> running Phi-Agent Gym task O / Nested TRIANGLE-CROSS qualification\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$NESTED_TX_ROM" \
  --source "$NESTED_TX_SRC" \
  --task "nested-branch-triangle-cross-v1" \
  --receipt "$ROOT/artifacts/agent-gym-nested-branch-triangle-cross-qualification.json"

printf '==> running Phi-Agent Gym task P / Nested SQUARE-CIRCLE qualification\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$NESTED_SC_ROM" \
  --source "$NESTED_SC_SRC" \
  --task "nested-branch-square-circle-v1" \
  --receipt "$ROOT/artifacts/agent-gym-nested-branch-square-circle-qualification.json"

printf '==> running Phi-Agent Gym task Q / Nested SQUARE-CROSS qualification\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$NESTED_SX_ROM" \
  --source "$NESTED_SX_SRC" \
  --task "nested-branch-square-cross-v1" \
  --receipt "$ROOT/artifacts/agent-gym-nested-branch-square-cross-qualification.json"

printf '==> running Relational Binding Memory factorial qualification\n'
cargo run -p phicade-libretro --bin binding_memory_qualify -- \
  --core "$CORE" \
  --normal-triangle-rom "$BIND_NT_ROM" \
  --normal-square-rom "$BIND_NS_ROM" \
  --swapped-triangle-rom "$BIND_ST_ROM" \
  --swapped-square-rom "$BIND_SS_ROM" \
  --receipt "$ROOT/artifacts/binding-memory-qualification.json"

printf '==> running Phi-Agent Gym task R / Binding Memory NORMAL-TRIANGLE qualification\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$BIND_NT_ROM" \
  --source "$BIND_NT_SRC" \
  --task "binding-memory-normal-triangle-v1" \
  --receipt "$ROOT/artifacts/agent-gym-binding-memory-normal-triangle-qualification.json"

printf '==> running Phi-Agent Gym task S / Binding Memory NORMAL-SQUARE qualification\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$BIND_NS_ROM" \
  --source "$BIND_NS_SRC" \
  --task "binding-memory-normal-square-v1" \
  --receipt "$ROOT/artifacts/agent-gym-binding-memory-normal-square-qualification.json"

printf '==> running Phi-Agent Gym task T / Binding Memory SWAPPED-TRIANGLE qualification\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$BIND_ST_ROM" \
  --source "$BIND_ST_SRC" \
  --task "binding-memory-swapped-triangle-v1" \
  --receipt "$ROOT/artifacts/agent-gym-binding-memory-swapped-triangle-qualification.json"

printf '==> running Phi-Agent Gym task U / Binding Memory SWAPPED-SQUARE qualification\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$BIND_SS_ROM" \
  --source "$BIND_SS_SRC" \
  --task "binding-memory-swapped-square-v1" \
  --receipt "$ROOT/artifacts/agent-gym-binding-memory-swapped-square-qualification.json"

printf '==> running Compositional Recall 2x2x2 qualification\n'
cargo run -p phicade-libretro --bin compositional_recall_qualify -- \
  --core "$CORE" \
  --ntm-rom "$COMP_NTM_ROM" \
  --ntf-rom "$COMP_NTF_ROM" \
  --nsm-rom "$COMP_NSM_ROM" \
  --nsf-rom "$COMP_NSF_ROM" \
  --stm-rom "$COMP_STM_ROM" \
  --stf-rom "$COMP_STF_ROM" \
  --ssm-rom "$COMP_SSM_ROM" \
  --ssf-rom "$COMP_SSF_ROM" \
  --receipt "$ROOT/artifacts/compositional-recall-qualification.json"

printf '==> running Phi-Agent Gym task V / Compositional Recall normal-triangle-match\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$COMP_NTM_ROM" \
  --source "$COMP_NTM_SRC" \
  --task "compositional-recall-normal-triangle-match-v1" \
  --receipt "$ROOT/artifacts/agent-gym-normal-triangle-match-qualification.json"

printf '==> running Phi-Agent Gym task W / Compositional Recall normal-triangle-flip\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$COMP_NTF_ROM" \
  --source "$COMP_NTF_SRC" \
  --task "compositional-recall-normal-triangle-flip-v1" \
  --receipt "$ROOT/artifacts/agent-gym-normal-triangle-flip-qualification.json"

printf '==> running Phi-Agent Gym task X / Compositional Recall normal-square-match\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$COMP_NSM_ROM" \
  --source "$COMP_NSM_SRC" \
  --task "compositional-recall-normal-square-match-v1" \
  --receipt "$ROOT/artifacts/agent-gym-normal-square-match-qualification.json"

printf '==> running Phi-Agent Gym task Y / Compositional Recall normal-square-flip\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$COMP_NSF_ROM" \
  --source "$COMP_NSF_SRC" \
  --task "compositional-recall-normal-square-flip-v1" \
  --receipt "$ROOT/artifacts/agent-gym-normal-square-flip-qualification.json"

printf '==> running Phi-Agent Gym task Z / Compositional Recall swapped-triangle-match\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$COMP_STM_ROM" \
  --source "$COMP_STM_SRC" \
  --task "compositional-recall-swapped-triangle-match-v1" \
  --receipt "$ROOT/artifacts/agent-gym-swapped-triangle-match-qualification.json"

printf '==> running Phi-Agent Gym task AA / Compositional Recall swapped-triangle-flip\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$COMP_STF_ROM" \
  --source "$COMP_STF_SRC" \
  --task "compositional-recall-swapped-triangle-flip-v1" \
  --receipt "$ROOT/artifacts/agent-gym-swapped-triangle-flip-qualification.json"

printf '==> running Phi-Agent Gym task AB / Compositional Recall swapped-square-match\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$COMP_SSM_ROM" \
  --source "$COMP_SSM_SRC" \
  --task "compositional-recall-swapped-square-match-v1" \
  --receipt "$ROOT/artifacts/agent-gym-swapped-square-match-qualification.json"

printf '==> running Phi-Agent Gym task AC / Compositional Recall swapped-square-flip\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$COMP_SSF_ROM" \
  --source "$COMP_SSF_SRC" \
  --task "compositional-recall-swapped-square-flip-v1" \
  --receipt "$ROOT/artifacts/agent-gym-swapped-square-flip-qualification.json"

printf '==> running Sequential Rule 2x2x2x2 qualification\n'
cargo run -p phicade-libretro --bin sequential_rule_qualify -- \
  --core "$CORE" \
  --rom "sequential-rule-normal-triangle-match-match-v1=$SEQ_NTMM_ROM" \
  --rom "sequential-rule-normal-triangle-match-flip-v1=$SEQ_NTMF_ROM" \
  --rom "sequential-rule-normal-triangle-flip-match-v1=$SEQ_NTFM_ROM" \
  --rom "sequential-rule-normal-triangle-flip-flip-v1=$SEQ_NTFF_ROM" \
  --rom "sequential-rule-normal-square-match-match-v1=$SEQ_NSMM_ROM" \
  --rom "sequential-rule-normal-square-match-flip-v1=$SEQ_NSMF_ROM" \
  --rom "sequential-rule-normal-square-flip-match-v1=$SEQ_NSFM_ROM" \
  --rom "sequential-rule-normal-square-flip-flip-v1=$SEQ_NSFF_ROM" \
  --rom "sequential-rule-swapped-triangle-match-match-v1=$SEQ_STMM_ROM" \
  --rom "sequential-rule-swapped-triangle-match-flip-v1=$SEQ_STMF_ROM" \
  --rom "sequential-rule-swapped-triangle-flip-match-v1=$SEQ_STFM_ROM" \
  --rom "sequential-rule-swapped-triangle-flip-flip-v1=$SEQ_STFF_ROM" \
  --rom "sequential-rule-swapped-square-match-match-v1=$SEQ_SSMM_ROM" \
  --rom "sequential-rule-swapped-square-match-flip-v1=$SEQ_SSMF_ROM" \
  --rom "sequential-rule-swapped-square-flip-match-v1=$SEQ_SSFM_ROM" \
  --rom "sequential-rule-swapped-square-flip-flip-v1=$SEQ_SSFF_ROM" \
  --receipt "$ROOT/artifacts/sequential-rule-qualification.json"

printf '==> running Sequential Rule normal-triangle-match-match\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$SEQ_NTMM_ROM" \
  --source "$SEQ_NTMM_SRC" \
  --task "sequential-rule-normal-triangle-match-match-v1" \
  --receipt "$ROOT/artifacts/agent-gym-sequential-rule-normal-triangle-match-match-qualification.json"

printf '==> running Sequential Rule normal-triangle-match-flip\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$SEQ_NTMF_ROM" \
  --source "$SEQ_NTMF_SRC" \
  --task "sequential-rule-normal-triangle-match-flip-v1" \
  --receipt "$ROOT/artifacts/agent-gym-sequential-rule-normal-triangle-match-flip-qualification.json"

printf '==> running Sequential Rule normal-triangle-flip-match\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$SEQ_NTFM_ROM" \
  --source "$SEQ_NTFM_SRC" \
  --task "sequential-rule-normal-triangle-flip-match-v1" \
  --receipt "$ROOT/artifacts/agent-gym-sequential-rule-normal-triangle-flip-match-qualification.json"

printf '==> running Sequential Rule normal-triangle-flip-flip\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$SEQ_NTFF_ROM" \
  --source "$SEQ_NTFF_SRC" \
  --task "sequential-rule-normal-triangle-flip-flip-v1" \
  --receipt "$ROOT/artifacts/agent-gym-sequential-rule-normal-triangle-flip-flip-qualification.json"

printf '==> running Sequential Rule normal-square-match-match\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$SEQ_NSMM_ROM" \
  --source "$SEQ_NSMM_SRC" \
  --task "sequential-rule-normal-square-match-match-v1" \
  --receipt "$ROOT/artifacts/agent-gym-sequential-rule-normal-square-match-match-qualification.json"

printf '==> running Sequential Rule normal-square-match-flip\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$SEQ_NSMF_ROM" \
  --source "$SEQ_NSMF_SRC" \
  --task "sequential-rule-normal-square-match-flip-v1" \
  --receipt "$ROOT/artifacts/agent-gym-sequential-rule-normal-square-match-flip-qualification.json"

printf '==> running Sequential Rule normal-square-flip-match\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$SEQ_NSFM_ROM" \
  --source "$SEQ_NSFM_SRC" \
  --task "sequential-rule-normal-square-flip-match-v1" \
  --receipt "$ROOT/artifacts/agent-gym-sequential-rule-normal-square-flip-match-qualification.json"

printf '==> running Sequential Rule normal-square-flip-flip\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$SEQ_NSFF_ROM" \
  --source "$SEQ_NSFF_SRC" \
  --task "sequential-rule-normal-square-flip-flip-v1" \
  --receipt "$ROOT/artifacts/agent-gym-sequential-rule-normal-square-flip-flip-qualification.json"

printf '==> running Sequential Rule swapped-triangle-match-match\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$SEQ_STMM_ROM" \
  --source "$SEQ_STMM_SRC" \
  --task "sequential-rule-swapped-triangle-match-match-v1" \
  --receipt "$ROOT/artifacts/agent-gym-sequential-rule-swapped-triangle-match-match-qualification.json"

printf '==> running Sequential Rule swapped-triangle-match-flip\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$SEQ_STMF_ROM" \
  --source "$SEQ_STMF_SRC" \
  --task "sequential-rule-swapped-triangle-match-flip-v1" \
  --receipt "$ROOT/artifacts/agent-gym-sequential-rule-swapped-triangle-match-flip-qualification.json"

printf '==> running Sequential Rule swapped-triangle-flip-match\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$SEQ_STFM_ROM" \
  --source "$SEQ_STFM_SRC" \
  --task "sequential-rule-swapped-triangle-flip-match-v1" \
  --receipt "$ROOT/artifacts/agent-gym-sequential-rule-swapped-triangle-flip-match-qualification.json"

printf '==> running Sequential Rule swapped-triangle-flip-flip\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$SEQ_STFF_ROM" \
  --source "$SEQ_STFF_SRC" \
  --task "sequential-rule-swapped-triangle-flip-flip-v1" \
  --receipt "$ROOT/artifacts/agent-gym-sequential-rule-swapped-triangle-flip-flip-qualification.json"

printf '==> running Sequential Rule swapped-square-match-match\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$SEQ_SSMM_ROM" \
  --source "$SEQ_SSMM_SRC" \
  --task "sequential-rule-swapped-square-match-match-v1" \
  --receipt "$ROOT/artifacts/agent-gym-sequential-rule-swapped-square-match-match-qualification.json"

printf '==> running Sequential Rule swapped-square-match-flip\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$SEQ_SSMF_ROM" \
  --source "$SEQ_SSMF_SRC" \
  --task "sequential-rule-swapped-square-match-flip-v1" \
  --receipt "$ROOT/artifacts/agent-gym-sequential-rule-swapped-square-match-flip-qualification.json"

printf '==> running Sequential Rule swapped-square-flip-match\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$SEQ_SSFM_ROM" \
  --source "$SEQ_SSFM_SRC" \
  --task "sequential-rule-swapped-square-flip-match-v1" \
  --receipt "$ROOT/artifacts/agent-gym-sequential-rule-swapped-square-flip-match-qualification.json"

printf '==> running Sequential Rule swapped-square-flip-flip\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$SEQ_SSFF_ROM" \
  --source "$SEQ_SSFF_SRC" \
  --task "sequential-rule-swapped-square-flip-flip-v1" \
  --receipt "$ROOT/artifacts/agent-gym-sequential-rule-swapped-square-flip-flip-qualification.json"

printf '==> running Selective Context 2x2x2x2 qualification\n'
cargo run -p phicade-libretro --bin selective_context_qualify -- \
  --core "$CORE" \
  --rom "selective-context-a-circle-triangle-match-v1=$CTX_A_CIR_TM_ROM" \
  --rom "selective-context-a-circle-triangle-flip-v1=$CTX_A_CIR_TF_ROM" \
  --rom "selective-context-a-circle-square-match-v1=$CTX_A_CIR_SM_ROM" \
  --rom "selective-context-a-circle-square-flip-v1=$CTX_A_CIR_SF_ROM" \
  --rom "selective-context-a-cross-triangle-match-v1=$CTX_A_CRS_TM_ROM" \
  --rom "selective-context-a-cross-triangle-flip-v1=$CTX_A_CRS_TF_ROM" \
  --rom "selective-context-a-cross-square-match-v1=$CTX_A_CRS_SM_ROM" \
  --rom "selective-context-a-cross-square-flip-v1=$CTX_A_CRS_SF_ROM" \
  --rom "selective-context-b-circle-triangle-match-v1=$CTX_B_CIR_TM_ROM" \
  --rom "selective-context-b-circle-triangle-flip-v1=$CTX_B_CIR_TF_ROM" \
  --rom "selective-context-b-circle-square-match-v1=$CTX_B_CIR_SM_ROM" \
  --rom "selective-context-b-circle-square-flip-v1=$CTX_B_CIR_SF_ROM" \
  --rom "selective-context-b-cross-triangle-match-v1=$CTX_B_CRS_TM_ROM" \
  --rom "selective-context-b-cross-triangle-flip-v1=$CTX_B_CRS_TF_ROM" \
  --rom "selective-context-b-cross-square-match-v1=$CTX_B_CRS_SM_ROM" \
  --rom "selective-context-b-cross-square-flip-v1=$CTX_B_CRS_SF_ROM" \
  --receipt "$ROOT/artifacts/selective-context-routing-qualification.json"

printf '==> running Selective Context a-circle-triangle-match\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$CTX_A_CIR_TM_ROM" \
  --source "$CTX_A_CIR_TM_SRC" \
  --task "selective-context-a-circle-triangle-match-v1" \
  --receipt "$ROOT/artifacts/agent-gym-selective-context-a-circle-triangle-match-qualification.json"

printf '==> running Selective Context a-circle-triangle-flip\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$CTX_A_CIR_TF_ROM" \
  --source "$CTX_A_CIR_TF_SRC" \
  --task "selective-context-a-circle-triangle-flip-v1" \
  --receipt "$ROOT/artifacts/agent-gym-selective-context-a-circle-triangle-flip-qualification.json"

printf '==> running Selective Context a-circle-square-match\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$CTX_A_CIR_SM_ROM" \
  --source "$CTX_A_CIR_SM_SRC" \
  --task "selective-context-a-circle-square-match-v1" \
  --receipt "$ROOT/artifacts/agent-gym-selective-context-a-circle-square-match-qualification.json"

printf '==> running Selective Context a-circle-square-flip\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$CTX_A_CIR_SF_ROM" \
  --source "$CTX_A_CIR_SF_SRC" \
  --task "selective-context-a-circle-square-flip-v1" \
  --receipt "$ROOT/artifacts/agent-gym-selective-context-a-circle-square-flip-qualification.json"

printf '==> running Selective Context a-cross-triangle-match\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$CTX_A_CRS_TM_ROM" \
  --source "$CTX_A_CRS_TM_SRC" \
  --task "selective-context-a-cross-triangle-match-v1" \
  --receipt "$ROOT/artifacts/agent-gym-selective-context-a-cross-triangle-match-qualification.json"

printf '==> running Selective Context a-cross-triangle-flip\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$CTX_A_CRS_TF_ROM" \
  --source "$CTX_A_CRS_TF_SRC" \
  --task "selective-context-a-cross-triangle-flip-v1" \
  --receipt "$ROOT/artifacts/agent-gym-selective-context-a-cross-triangle-flip-qualification.json"

printf '==> running Selective Context a-cross-square-match\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$CTX_A_CRS_SM_ROM" \
  --source "$CTX_A_CRS_SM_SRC" \
  --task "selective-context-a-cross-square-match-v1" \
  --receipt "$ROOT/artifacts/agent-gym-selective-context-a-cross-square-match-qualification.json"

printf '==> running Selective Context a-cross-square-flip\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$CTX_A_CRS_SF_ROM" \
  --source "$CTX_A_CRS_SF_SRC" \
  --task "selective-context-a-cross-square-flip-v1" \
  --receipt "$ROOT/artifacts/agent-gym-selective-context-a-cross-square-flip-qualification.json"

printf '==> running Selective Context b-circle-triangle-match\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$CTX_B_CIR_TM_ROM" \
  --source "$CTX_B_CIR_TM_SRC" \
  --task "selective-context-b-circle-triangle-match-v1" \
  --receipt "$ROOT/artifacts/agent-gym-selective-context-b-circle-triangle-match-qualification.json"

printf '==> running Selective Context b-circle-triangle-flip\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$CTX_B_CIR_TF_ROM" \
  --source "$CTX_B_CIR_TF_SRC" \
  --task "selective-context-b-circle-triangle-flip-v1" \
  --receipt "$ROOT/artifacts/agent-gym-selective-context-b-circle-triangle-flip-qualification.json"

printf '==> running Selective Context b-circle-square-match\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$CTX_B_CIR_SM_ROM" \
  --source "$CTX_B_CIR_SM_SRC" \
  --task "selective-context-b-circle-square-match-v1" \
  --receipt "$ROOT/artifacts/agent-gym-selective-context-b-circle-square-match-qualification.json"

printf '==> running Selective Context b-circle-square-flip\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$CTX_B_CIR_SF_ROM" \
  --source "$CTX_B_CIR_SF_SRC" \
  --task "selective-context-b-circle-square-flip-v1" \
  --receipt "$ROOT/artifacts/agent-gym-selective-context-b-circle-square-flip-qualification.json"

printf '==> running Selective Context b-cross-triangle-match\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$CTX_B_CRS_TM_ROM" \
  --source "$CTX_B_CRS_TM_SRC" \
  --task "selective-context-b-cross-triangle-match-v1" \
  --receipt "$ROOT/artifacts/agent-gym-selective-context-b-cross-triangle-match-qualification.json"

printf '==> running Selective Context b-cross-triangle-flip\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$CTX_B_CRS_TF_ROM" \
  --source "$CTX_B_CRS_TF_SRC" \
  --task "selective-context-b-cross-triangle-flip-v1" \
  --receipt "$ROOT/artifacts/agent-gym-selective-context-b-cross-triangle-flip-qualification.json"

printf '==> running Selective Context b-cross-square-match\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$CTX_B_CRS_SM_ROM" \
  --source "$CTX_B_CRS_SM_SRC" \
  --task "selective-context-b-cross-square-match-v1" \
  --receipt "$ROOT/artifacts/agent-gym-selective-context-b-cross-square-match-qualification.json"

printf '==> running Selective Context b-cross-square-flip\n'
cargo run -p phicade-libretro --bin agent_gym_qualify -- \
  --core "$CORE" \
  --rom "$CTX_B_CRS_SF_ROM" \
  --source "$CTX_B_CRS_SF_SRC" \
  --task "selective-context-b-cross-square-flip-v1" \
  --receipt "$ROOT/artifacts/agent-gym-selective-context-b-cross-square-flip-qualification.json"

printf '==> receipts:\n'
printf '    %s\n' "$ROOT/artifacts/sameboy-qualification.json"
printf '    %s\n' "$ROOT/artifacts/replay-qualification.json"
printf '    %s\n' "$ROOT/artifacts/phibot-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-driver-qualification.json"
printf '    %s\n' "$ROOT/artifacts/autodrive-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-mirror-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-wall-detour-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-temporal-left-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-temporal-right-qualification.json"
printf '    %s\n' "$ROOT/artifacts/temporal-cue-pair-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-relay-left-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-relay-right-qualification.json"
printf '    %s\n' "$ROOT/artifacts/relay-rooms-pair-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-key-gate-left-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-key-gate-right-qualification.json"
printf '    %s\n' "$ROOT/artifacts/key-gate-pair-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-power-chain-left-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-power-chain-right-qualification.json"
printf '    %s\n' "$ROOT/artifacts/power-chain-pair-qualification.json"
printf '    %s\n' "$ROOT/artifacts/branch-selector-pair-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-branch-selector-triangle-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-branch-selector-square-qualification.json"
printf '    %s\n' "$ROOT/artifacts/nested-branch-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-nested-branch-triangle-circle-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-nested-branch-triangle-cross-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-nested-branch-square-circle-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-nested-branch-square-cross-qualification.json"
printf '    %s\n' "$ROOT/artifacts/binding-memory-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-binding-memory-normal-triangle-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-binding-memory-normal-square-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-binding-memory-swapped-triangle-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-binding-memory-swapped-square-qualification.json"
printf '    %s\n' "$ROOT/artifacts/compositional-recall-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-normal-triangle-match-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-normal-triangle-flip-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-normal-square-match-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-normal-square-flip-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-swapped-triangle-match-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-swapped-triangle-flip-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-swapped-square-match-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-swapped-square-flip-qualification.json"
printf '    %s\n' "$ROOT/artifacts/sequential-rule-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-sequential-rule-normal-triangle-match-match-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-sequential-rule-normal-triangle-match-flip-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-sequential-rule-normal-triangle-flip-match-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-sequential-rule-normal-triangle-flip-flip-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-sequential-rule-normal-square-match-match-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-sequential-rule-normal-square-match-flip-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-sequential-rule-normal-square-flip-match-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-sequential-rule-normal-square-flip-flip-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-sequential-rule-swapped-triangle-match-match-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-sequential-rule-swapped-triangle-match-flip-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-sequential-rule-swapped-triangle-flip-match-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-sequential-rule-swapped-triangle-flip-flip-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-sequential-rule-swapped-square-match-match-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-sequential-rule-swapped-square-match-flip-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-sequential-rule-swapped-square-flip-match-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-sequential-rule-swapped-square-flip-flip-qualification.json"
printf '    %s\n' "$ROOT/artifacts/selective-context-routing-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-selective-context-a-circle-triangle-match-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-selective-context-a-circle-triangle-flip-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-selective-context-a-circle-square-match-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-selective-context-a-circle-square-flip-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-selective-context-a-cross-triangle-match-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-selective-context-a-cross-triangle-flip-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-selective-context-a-cross-square-match-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-selective-context-a-cross-square-flip-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-selective-context-b-circle-triangle-match-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-selective-context-b-circle-triangle-flip-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-selective-context-b-circle-square-match-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-selective-context-b-circle-square-flip-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-selective-context-b-cross-triangle-match-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-selective-context-b-cross-triangle-flip-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-selective-context-b-cross-square-match-qualification.json"
printf '    %s\n' "$ROOT/artifacts/agent-gym-selective-context-b-cross-square-flip-qualification.json"
