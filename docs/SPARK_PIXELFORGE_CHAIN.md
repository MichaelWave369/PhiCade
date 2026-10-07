# PhiCade Rung 46 — Govern SPARK Through PixelForge

Rung 46 is the first full three-project qualification:

```text
PhiCade
   ↓
PixelForge
   ↓
SPARK
```

PhiCade remains the controller-authority and evidence layer. PixelForge remains
the runtime bridge/transport layer. SPARK remains the game authority.

## Pinned sources

PixelForge:

```text
8790c3b00b5184136fb8fa6a2fbdabe834eeca22
```

SPARK bridge source:

```text
fae7879820bef63a550fea486b2defbc3cee5304
```

These correspond to the merged PixelForge v5.34 JSON-safe SPARK pin and the
qualified SPARK Threshold bridge source.

## Runtime path

PhiCade launches PixelForge's:

```text
scripts/serve_external_spark_runtime.mjs
```

with the pinned SPARK checkout as `--spark-root`.

PixelForge then loads SPARK's own `runtime/spark-threshold-adapter-v1.js`,
wraps it with `createBridgeV1()`, and serves Runtime Bridge v1 over the
existing JSONL transport.

PhiCade never imports or rewrites SPARK gameplay rules.

## Governed action proof

The first full-chain action is deliberately small:

```text
Human seat 1 presses RIGHT
        ↓
PhiCade ActionEnvelope
        ↓
PhiCade AuthorityPolicy
        ↓
accepted
        ↓
qualification mapper
        ↓
SPARK MOVE { x: 1, y: 0 }
        ↓
PixelForge Runtime Bridge v1
        ↓
SPARK action-engine.js
        ↓
SPARK_PLAYER_MOVED
```

The mapper is qualification-specific. It does not define a universal SPARK
controller grammar.

## Required initial state

The exact SPARK observation must report:

```text
room:       threshold
form:       spark
position:   (480, 390)
max health: 112
```

The 112 HP includes SPARK's canonical starter Bark Ward.

## PASS conditions

The qualification requires the exact source revisions, a valid SPARK descriptor,
controller registration, the canonical Threshold observation, PhiCade authority
acceptance, a queued MOVE RIGHT at tick 0, exactly one `SPARK_PLAYER_MOVED`
event, an identical semantic event from `events(0)`, positive X movement with
stable Y/room, final tick 1, and a 64-character hex runtime hash.

Receipt schema:

```text
phicade.pixelforge-spark-chain-qualification.v1
```

Default receipt:

```text
artifacts/phicade-pixelforge-spark-qualification.json
```

## Private-repository evidence topology

SPARK remains private, so public PhiCade CI does not attempt to checkout it with
a default repository token.

The exact three-repository qualification runs from the private SPARK repository:

```text
private SPARK workflow
   ├── pinned SPARK
   ├── public PixelForge
   └── public PhiCade candidate
```

This avoids storing a cross-repository PAT while preserving exact source pins.

Public PhiCade CI still compiles and unit-tests the adapter and qualification
mapping.

## Capability boundary

Rung 46 proves only one governed semantic SPARK state transition. It does not
claim framebuffer/audio streaming, exact replay, restorable snapshots, campaign
save authority, co-op control, Sideways control, Duel control, Memory Arcade
launching through PhiCade, full-game agent Autodrive, or Unreal runtime control.

## Next rung

After the full chain is merged and green, the next useful expansion is not
another transport layer. Widen the qualified SPARK action surface carefully,
starting with DASH and PULSE, then expose a bounded semantic observation profile
suitable for Phi-Bot playtesting.
