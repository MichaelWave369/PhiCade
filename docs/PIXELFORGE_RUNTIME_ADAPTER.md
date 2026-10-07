# PixelForge Runtime Adapter v1

## Purpose

Rung 44 established the local PhiCade ↔ PixelForge Runtime Bridge v1 seam over
PixelForge's bounded JSONL transport.

Rung 45 keeps that transport unchanged and upgrades the qualification target from
the deterministic reference counter to a real PixelForge cartridge artifact:

```text
The Legend of More Bounce
scene: Bouncehome Grove
```

The split remains:

```text
PixelForge builds / owns cartridge semantics
                |
                v
PixelForge Runtime Bridge v1
                |
          JSONL Transport v1
                |
                v
PhiCade governs the session
                |
     +----------+----------+
     |          |          |
   Human      Phi-Bot    Replay/later
```

Neither project absorbs the other.

## Pinned PixelForge source

The real-cartridge qualification target is the exact merged PixelForge v5.33
revision:

```text
8fda48073b5d2ea6273df54c47c194bdac8231d6
```

PhiCade refuses qualification if the local PixelForge checkout is at a different
revision.

The earlier v5.32 counter proof remains useful as the transport-level historical
qualification. Rung 45 proves that the same seam can carry actual game state.

## Cartridge target

PixelForge launches:

```text
scripts/serve_cartridge_runtime.mjs
```

with:

```text
--cartridge the-legend-of-more-bounce
--scene bouncehome-grove
```

That runtime loads the existing PixelForge scene packet:

```text
games/the-legend-of-more-bounce/runtime/
bouncehome-grove.runtime-scene.v5.11.json
```

The scene's real spawn and collision data decide movement outcomes.

## Transport

The adapter still uses:

- `pixelforge.runtime-transport.request.v1`
- `pixelforge.runtime-transport.response.v1`

It validates response schema, request/response IDs, Runtime Bridge protocol
identity, Runtime Bridge version, game/runtime identity, and explicit remote
errors.

No new transport protocol was introduced for the cartridge.

## Capability projection

The adapter declares:

- execution model: **BRIDGED_RUNTIME**
- governed actions: **SUPPORTED**
- external process lifecycle: **SUPPORTED**
- semantic events: **SUPPORTED**
- frame step: **SUPPORTED** only when the PixelForge descriptor is externally
  stepped or satisfies the Runtime Bridge deterministic compatibility rule

It still leaves these **UNSUPPORTED** until separately proven:

- rendered framebuffer,
- audio stream,
- reset,
- restorable state snapshots,
- exact Replay v1,
- persistent save data,
- game detection.

A callable `snapshot()` method is not promoted into emulator-style restorable
state support.

## Authority proof

The qualification constructs a normal PhiCade human Action Bus envelope:

```text
seat 1 / RIGHT pressed
```

The existing PhiCade `AuthorityPolicy` must accept that action first.

Only then does the narrow qualification mapper translate it into the cartridge's
own PixelForge action grammar:

```json
{
  "type": "MOVE",
  "actorId": "more-bounce",
  "params": { "direction": "RIGHT" }
}
```

That mapper is specific to the first cartridge qualification. It is not a
universal PixelForge action grammar and does not define SPARK's future controls.

## Qualification chain

The pinned proof requires:

1. exact PixelForge v5.33 source revision,
2. launch the real cartridge JSONL server,
3. validate `describe()`,
4. require gameId `the-legend-of-more-bounce`,
5. require runtimeVersion `legend-bouncehome/1`,
6. register the PhiCade controller,
7. observe Bouncehome Grove at player spawn `(2.5, 8.5)`,
8. authorize one RIGHT action through PhiCade authority,
9. submit the translated PixelForge MOVE intent,
10. advance the cartridge runtime,
11. require a `PLAYER_MOVED` semantic event,
12. consume the same event through `events(0)`,
13. observe the player at `(3.5, 8.5)`,
14. capture a non-empty cartridge runtime hash,
15. write a PASS receipt.

Receipt:

```text
artifacts/pixelforge-cartridge-qualification.json
```

Schema:

```text
phicade.pixelforge-cartridge-qualification.v1
```

## Local qualification

From PhiCade:

```bash
bash ./scripts/qualify-pixelforge.sh /path/to/parallax-pixelforge
```

Or directly on Windows PowerShell:

```powershell
cargo run -p phicade-runtime --example qualify_pixelforge -- --pixelforge-root "C:\path\to\parallax-pixelforge"
```

The checkout must be at the pinned revision and Node must be on PATH.

## What PASS proves

PASS proves that PhiCade can govern an action that crosses the real PixelForge
Runtime Bridge into an actual cartridge scene, that PixelForge's own scene data
controls the outcome, and that semantic events plus runtime hash return across
the same seam.

## What it does not prove

Rung 45 does not claim:

- SPARK is connected yet,
- framebuffer/audio streaming exists,
- PixelForge snapshots are restorable,
- exact replay exists for bridged cartridges,
- arbitrary PixelForge games share one action grammar,
- agent Autodrive is enabled for arbitrary cartridges,
- Unreal is bridged.

Those remain later capability-specific rungs.

## Next rung

SPARK becomes the next intended large consumer.

The correct next proof is not to copy SPARK into PhiCade. PixelForge should expose
one deliberately narrow SPARK incarnation through Runtime Bridge v1, then PhiCade
should qualify that exact incarnation through the already-proven governed seam.


## Rung 46: SPARK consumer

The first large consumer is now canonical **SPARK: The Substrate**.

PhiCade does not import SPARK directly. It launches the pinned PixelForge v5.34
external-SPARK server, which loads SPARK's own Threshold adapter and existing
Descent engine.

The first proof remains deliberately narrow:

```text
PhiCade RIGHT
   -> AuthorityPolicy
   -> SPARK MOVE { x: 1, y: 0 }
   -> PixelForge Runtime Bridge v1
   -> SPARK action-engine
   -> SPARK_PLAYER_MOVED
   -> semantic event + runtime hash
```

See `docs/SPARK_PIXELFORGE_CHAIN.md`.

This does not widen the previously conservative capability claims. Framebuffer,
audio, exact replay, restorable snapshots, persistent save authority, and
full-game agent control remain unqualified.
