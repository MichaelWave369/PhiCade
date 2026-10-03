# Rung 4 Session Machinery

PhiCade Rung 4 adds persistent session features without giving the emulator core,
UI, or future agent seats alternate control paths.

## Namespaces

Every running game is identified locally by the SHA-256 of the selected ROM bytes.

That fingerprint namespaces:

- battery RAM: `saves/<game-sha256>.srm`
- per-game profile: `profiles/<game-sha256>.json`
- screenshots: `screenshots/<game-sha256>/frame-XXXXXXXXXXXX.png`
- save states: `states/<game-sha256>/<core-name>-<core-version>/slot-N.state`

Save states are therefore bound to both the game fingerprint and the active core
identity/version. A filename is never treated as identity.

## Battery RAM

When a core exposes `RETRO_MEMORY_SAVE_RAM`, PhiCade:

1. restores matching persisted RAM after the game is loaded,
2. flushes RAM periodically while the session runs,
3. allows an explicit host flush,
4. flushes again on normal session shutdown/drop.

Games with no save RAM produce no empty placeholder file.

## Save/load state

Save and load requests are `ActionEnvelope` system commands. The native host handles
the persistence boundary, while the core provides `retro_serialize` /
`retro_unserialize`.

State files begin with a PhiCade format marker plus the host frame number followed
by the opaque core state bytes.

Slots are 0 through 9.

## Rewind

Rewind is also a governed `ActionEnvelope` system command.

The host snapshots serialized core state at the profile's configured interval and
keeps only a bounded window. The default profile stores roughly ten seconds of
history at one snapshot every 30 emulated frames.

A rewind command restores the newest snapshot at or before the requested lookback
and discards snapshots from the abandoned future.

## Fast-forward

Per-game profiles allow only 1x, 2x, or 4x.

Fast-forward advances multiple emulated frames during one host display tick.
Intermediate audio is not presented to Web Audio; the desktop UI mutes fast-forward
audio rather than producing misleading pitch-shifted output.

## Screenshots

Screenshots are encoded from the actual last RGBA frame returned by the qualified
core host and stored as PNG files in the game fingerprint namespace.

Screenshot capture is a host observation action, not emulator input, so it does not
enter the Action Bus.

## Per-game profile

The default profile is:

```json
{
  "fastForward": 1,
  "rewindSeconds": 10,
  "rewindIntervalFrames": 30,
  "saveSlot": 0
}
```

Profiles are validated before persistence. Current hard bounds are:

- fast-forward: 1x, 2x, or 4x
- rewind depth: 2 through 60 seconds
- rewind interval: 10 through 120 frames
- state slot: 0 through 9

## Qualification

The SameBoy qualification receipt was upgraded to
`phicade.core-qualification.v2`.

After the normal video/audio smoke run, CI:

1. serializes SameBoy state,
2. advances 30 frames,
3. hashes the resulting video frame,
4. restores the serialized state and host frame number,
5. advances the same 30 frames again,
6. requires the final frame hash to match exactly.

This does not claim every future core is deterministic. It proves the frozen SameBoy
configuration used by this rung can perform the state round-trip required by
save/load and rewind.
