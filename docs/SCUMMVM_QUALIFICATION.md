# ScummVM 2026.3.0 Qualification

Rung 35 qualifies the official ScummVM libretro port as a real PhiCade runtime
without bundling commercial game data or distributing the ScummVM binary.

## Pinned upstream identity

- upstream: `scummvm/scummvm`
- release: `v2026.3.0`
- source commit: `fed42f2068dcafc6aafa1c28c77e4c88def74b66`
- license: `GPL-3.0-or-later`
- qualification profile: `phicade.scummvm-2026.3.0-launcher-qualified.v1`

CI shallow-clones that exact release and refuses to continue if HEAD does not
match the pinned commit.

PhiCade does not vendor or publish the resulting ScummVM core binary.

## Qualification build

The ScummVM libretro backend provides a `noengine` build target. PhiCade uses
that target with software rendering forced on and several optional heavyweight
features disabled.

This produces a launcher/backend core with no game engines and no commercial
game assets.

The qualification therefore tests the runtime integration itself, not any
copyrighted game.

## Host contract

PhiCade now supports the libretro environment calls required by the pinned
ScummVM launcher path, including:

- no-content launcher support;
- retrieval of the loaded core path;
- explicit audio/video enablement;
- keyboard callback registration acknowledgement.

The core may then be loaded with a null content pointer and run its launcher.

## Required PASS evidence

`qualify_scummvm` refuses to produce a PASS receipt unless all of the following
hold:

- core identity is exactly ScummVM `v2026.3.0`;
- ScummVM requested no-content support;
- the launcher boots through `retro_load_game(NULL)`;
- at least one software framebuffer is produced;
- final framebuffer geometry is non-zero;
- governed RIGHT press reaches the libretro joypad state;
- governed RIGHT release clears that state;
- governed LEFT_X reaches the libretro analog state;
- neutral LEFT_X clears that analog state;
- `retro_serialize_size()` reports exactly zero;
- an attempted state serialization is refused;
- libretro save RAM size reports exactly zero;
- the capability manifest leaves state snapshots, exact replay, and persistent
  save data unsupported.

The receipt includes the exact built core SHA-256.

## Capability profile

For the pinned ScummVM qualification profile:

| Capability | Status |
| --- | --- |
| frame step | QUALIFIED |
| rendered framebuffer | QUALIFIED |
| audio stream | SUPPORTED |
| governed actions | SUPPORTED |
| reset | SUPPORTED |
| state snapshots | UNSUPPORTED |
| exact replay | UNSUPPORTED |
| persistent save data | UNSUPPORTED |
| game detection | SUPPORTED |
| external process lifecycle | UNSUPPORTED |
| semantic events | UNSUPPORTED |

`gameDetection` is a supported runtime behavior, not a qualified claim in this
launcher-only test.

`persistentSaveData` remains unsupported in the PhiCade manifest for this rung
because the no-engine launcher qualification does not prove a governed native
game-save lifecycle. This is separate from ScummVM's own game save features.

## Content path

Rung 34 already allows generic libretro FILE descriptors without a `SystemId`.

That is the path later used for:

- `.scummvm` hook files;
- ordinary files inside valid ScummVM game folders.

ScummVM itself performs game detection from the supplied file or hook.

Rung 35 does not ship, download, or test any copyrighted game data.

## Replay semantics

ScummVM's pinned libretro source implements these as stubs:

- `retro_serialize_size() -> 0`
- `retro_serialize(...) -> false`
- `retro_unserialize(...) -> false`
- libretro save-memory size -> 0

PhiCade therefore does not expose SameBoy-style state-checkpointed exact replay
for ScummVM.

This is not treated as a defect. It is an explicit runtime capability
difference.

## Authority rule

ScummVM gains no special authority by being qualified.

Human, replay, script, and agent actions still enter through PhiCade's governed
Action Bus. The loaded runtime never receives a direct agent or controller
handle.
