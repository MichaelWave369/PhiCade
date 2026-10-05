# Content Descriptor v1

PhiCade content should describe what the operator selected without assuming that
every launchable thing is a single console ROM.

Schema:

`phicade.content-descriptor.v1`

## Locator kinds

Content Descriptor v1 supports three locator shapes:

- `FILE` — a single file such as a ROM, `.scummvm` hook, package, or cartridge.
- `DIRECTORY` — a folder that a runtime may inspect as one content root.
- `LAUNCH_TARGET` — a runtime-native target identifier.

A descriptor also carries:

- `displayName`
- optional `system`
- optional `runtimeHint`

## Runtime hint is not authority

`runtimeHint` is routing metadata only.

A descriptor may suggest `scummvm`, `pixelforge-bridge`, or another provider,
but the hint does not install, authorize, select, or launch that runtime. Runtime
selection remains governed by PhiCade/PhiOS policy and qualification evidence.

## Backward compatibility

The existing `GameImage` API remains intact.

A file descriptor with a `system` can be losslessly projected into a
`GameImage`, and every `GameImage` can be projected into a Content Descriptor.

The default `EmulatorCore::load_content()` implementation uses exactly this
compatibility bridge:

```text
Content Descriptor
      |
      | FILE + system
      v
   GameImage
      |
      v
 legacy EmulatorCore::load_game()
```

Directory and launch-target descriptors fail closed with `UnsupportedImage`
on legacy emulator cores.

This means current SameBoy behavior does not change merely because the
runtime-neutral content layer exists.

## Examples

### Existing Game Boy ROM

```json
{
  "schema": "phicade.content-descriptor.v1",
  "displayName": "Demo",
  "locator": {
    "kind": "FILE",
    "path": "roms/demo.gb"
  },
  "system": "GAME_BOY",
  "runtimeHint": null
}
```

### Future ScummVM game directory

```json
{
  "schema": "phicade.content-descriptor.v1",
  "displayName": "Monkey Island",
  "locator": {
    "kind": "DIRECTORY",
    "path": "games/monkey-island"
  },
  "system": null,
  "runtimeHint": "scummvm"
}
```

### Future ScummVM launcher target

```json
{
  "schema": "phicade.content-descriptor.v1",
  "displayName": "Monkey Island",
  "locator": {
    "kind": "LAUNCH_TARGET",
    "target": "monkey"
  },
  "system": null,
  "runtimeHint": "scummvm"
}
```

### Future PixelForge cartridge

A PixelForge cartridge can remain a `FILE` locator and carry a
`runtimeHint` such as `pixelforge-bridge`. The content contract does not need
to pretend that cartridge is a SNES/GB/etc. system image.

## Design rule

**Content description is separate from runtime capability and separate from
authority.**

- Content Descriptor says what was selected.
- Runtime Capability Manifest says what a provider can do.
- Governance decides whether that provider may handle that content.
