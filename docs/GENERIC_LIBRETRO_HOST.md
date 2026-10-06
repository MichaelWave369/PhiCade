# Generic libretro Host v1

Rung 34 removes Game Boy-specific assumptions from PhiCade's libretro adapter
without claiming that every libretro core is equally capable.

## Content loading

The libretro adapter now accepts any `FILE` Content Descriptor and passes that
path to the loaded core.

`GameImage` remains supported for backward compatibility, but its `SystemId`
is no longer used as a libretro compatibility gate. The core itself decides
whether the supplied file is valid content.

This matters for runtimes such as ScummVM, whose libretro core can accept files
inside a valid game directory or dedicated `.scummvm` hook files.

`DIRECTORY` and `LAUNCH_TARGET` descriptors remain unsupported by the generic
libretro adapter in this rung.

## RetroPad input surface

The adapter now exposes the standard 16-button RetroPad IDs:

- B
- Y
- SELECT
- START
- UP
- DOWN
- LEFT
- RIGHT
- A
- X
- L / L1
- R / R1
- L2
- R2
- L3
- R3

Button actions continue to travel through PhiCade's existing governed Action Bus.

## Analog input surface

The libretro host now maps these Action Bus axis names:

- `LEFT_X` / `LX`
- `LEFT_Y` / `LY`
- `RIGHT_X` / `RX`
- `RIGHT_Y` / `RY`

Values remain the existing signed 16-bit Action Bus axis values and are surfaced
through the standard libretro analog device callback.

This is sufficient groundwork for ScummVM's documented RetroPad D-pad / left
analog cursor path without adding mouse, pointer, or keyboard devices yet.

## Capability probes

Generic libretro capability claims are runtime-observed:

- `stateSnapshots` becomes SUPPORTED only when the loaded core reports a
  non-zero serialize size.
- `persistentSaveData` becomes SUPPORTED only when the loaded core exposes
  non-zero save RAM.
- `exactReplay` remains UNSUPPORTED for generic cores.

SameBoy 1.0.3 remains the explicit qualified exception under its existing pinned
qualification profile.

The existence of `retro_serialize*` or memory ABI symbols alone is not treated
as proof that a core actually provides those capabilities.

## Non-goals for Rung 34

This rung does not add:

- ScummVM itself;
- automatic core installation or discovery;
- directories or launcher-target loading through libretro;
- mouse, pointer, keyboard, or text input;
- generic exact-replay claims;
- hardware-rendering contexts.

Those belong in later, separately qualified rungs.
