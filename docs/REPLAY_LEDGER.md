# Rung 5 Replay Ledger

PhiCade Replay Ledger turns a gameplay claim into a reproducible artifact.

The v1 replay format records normalized Action Bus events at the exact emulated
frame where the native host applied them, binds them to core/game provenance, and
adds periodic hashes so verification can identify the first observed divergence.

## Replay schema

Schema: `phicade.replay.v1`

A replay binds:

- ROM SHA-256
- core name and reported version
- core binary SHA-256
- starting emulated frame
- ending emulated frame
- initial serialized core state
- initial frontend input mask
- frame-stamped `ActionEnvelope` events
- periodic checkpoints
- final serialized-state SHA-256
- final framebuffer SHA-256

The initial core state is embedded as base64 so the replay is self-contained with
respect to emulator state. It does **not** embed the ROM, BIOS, or emulator core.

## Exact applied frame

The UI's frame value is not accepted blindly as replay truth.

When actions cross the native runtime boundary during recording, PhiCade stamps
each accepted action with the core's current emulated frame immediately before
that action is applied.

The original ActionSource remains in the ledger as provenance. During verifier
execution the actions are reissued with the `Replay` source.

## Frontend input mask

libretro save states do not necessarily contain the frontend's held-button mask.

Replay v1 therefore records the host input mask alongside the initial state and
every checkpoint. Verification restores that mask before the first replay frame
and compares it at checkpoints.

This closes the case where recording starts while a direction/button is already
held.

## Checkpoints

The desktop recorder currently checkpoints every 60 emulated frames.

Each checkpoint records:

- emulated frame
- SHA-256 of serialized core state
- SHA-256 of the rendered RGBA framebuffer
- host input mask

The final frame is always checkpointed when recording stops, even when it does
not land exactly on the periodic interval.

## Recording constraints

Replay v1 deliberately keeps the timeline linear.

While recording:

- speed must remain 1x
- save-state is blocked
- load-state is blocked
- rewind is blocked
- per-game profile changes are blocked
- reset is allowed and recorded
- ordinary button/axis input is allowed and recorded
- screenshots and battery-RAM flushes remain host observations/persistence and do
  not enter the replay input tape

Future replay schemas can model explicit branches. v1 does not pretend branches
are a simple linear tape.

## Content-addressed export

Stopping a recording serializes the ledger and computes SHA-256 over those exact
JSON bytes.

The files are stored under the ROM fingerprint namespace:

```text
replays/<rom-sha256>/<replay-sha256>.replay.json
replays/<rom-sha256>/<replay-sha256>.receipt.json
```

The replay hash therefore names the exact bytes being verified.

## Receipt

Schema: `phicade.replay-receipt.v1`

A receipt records:

- replay SHA-256
- ROM/core provenance
- frame range
- action count
- checkpoint count
- verification result, when verification has been run
- first observed divergence frame and expected/actual hashes on failure

An exported replay begins as unverified. Running VERIFY LAST rewrites only the
paired receipt; the content-addressed replay file does not change.

## Verification

Verification:

1. checks replay schema and provenance,
2. verifies the running ROM/core binary identity,
3. snapshots the user's current live session,
4. restores the replay's initial core state and frontend input mask,
5. replays frame-stamped actions,
6. compares periodic state/frame/input checkpoints,
7. compares the final state and framebuffer hashes,
8. restores the user's pre-verification live state, input mask, save RAM, rewind
   buffer, and framebuffer cache.

A successful verification returns `pass`.

A mismatch returns `diverged` with the first checkpoint frame at which the fork
was observed. This is checkpoint-granularity localization, not a claim that the
exact causal instruction occurred on that frame.

## CI qualification

The SameBoy qualification job includes a Replay Ledger positive and negative
control.

Positive control:

- warm up the frozen SameBoy core
- record a synthetic action stream containing button transitions and RESET
- checkpoint every 30 frames
- restore the initial state
- replay the tape
- require every checkpoint and final hash to match

Negative control:

- clone the same replay
- replace the recorded RESET with a different button action
- run verification again
- require the divergence detector to report a fork

CI fails if either exact replay fails or the deliberately corrupted replay escapes
detection.
