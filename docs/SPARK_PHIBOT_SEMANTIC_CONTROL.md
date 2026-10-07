# PhiCade Rung 47 — SPARK Phi-Bot Semantic Control

Rung 47 moves from a human proof action to the first bounded **Phi-Bot** control
surface for canonical SPARK.

The transport architecture does not change:

```text
Phi-Bot
   ↓
PhiCade AuthorityPolicy
   ↓
PixelForge Runtime Bridge v1 / JSONL
   ↓
SPARK Threshold adapter
   ↓
SPARK action-engine.js
```

## Explicit Phi-Bot grant

PhiCade adds a SPARK-specific grant with exactly these gameplay buttons:

```text
UP
DOWN
LEFT
RIGHT
DASH_UP
DASH_DOWN
DASH_LEFT
DASH_RIGHT
PULSE
```

The grant exposes no system commands, no axes, and permits at most one action
per frame.

This is intentionally narrower than a generic gamepad grant.

## Qualified action grammar

The PhiCade mapper translates only the granted buttons:

```text
UP/DOWN/LEFT/RIGHT
  -> SPARK MOVE { x, y }

DASH_UP/DASH_DOWN/DASH_LEFT/DASH_RIGHT
  -> SPARK DASH { x, y }

PULSE
  -> SPARK PULSE
```

SPARK still owns movement speed, collision, dash timing, cooldowns, vessel power
behavior, damage, progression, and all other game rules.

## Bounded semantic observation

Rung 47 introduces:

```text
phicade.spark-semantic-observation.v1
```

It is a deterministic projection of the JSON-safe SPARK bridge observation,
designed for agent reasoning without pretending a framebuffer exists.

The profile includes only:

- tick;
- room, world, phase, and vessel form;
- player X/Y, HP/max HP, and dash-ready state;
- vessel power name and cooldown;
- enemy count;
- nearest enemy only;
- visible fragment count;
- exit directions;
- allowed SPARK bridge actions;
- cumulative dash and power counters.

The profile deliberately does not copy the full SPARK run, inventory, save state,
lore state, hidden room data, D1 data, Memory Arcade state, or unrelated campaign
internals into the agent context.

## Qualification sequence

The first Phi-Bot proof uses one explicitly granted agent:

```text
phi-spark-rung47
```

and performs:

```text
RIGHT
  ↓
SPARK_PLAYER_MOVED

DASH_RIGHT
  ↓
SPARK_DASH_STARTED

PULSE
  ↓
SPARK_PULSE_USED
```

Each action must be accepted independently by PhiCade authority before
translation and submission.

The final proof additionally requires:

- real positive X movement;
- exactly one recorded dash;
- dash cooldown active;
- exactly one recorded power use;
- Lumen Pulse cooldown active;
- cumulative semantic event equality;
- a 64-character SPARK runtime hash.

Receipt schema:

```text
phicade.pixelforge-spark-phibot-qualification.v1
```

Default artifact:

```text
artifacts/phicade-pixelforge-spark-phibot-qualification.json
```

## What this proves

Rung 47 proves that an explicitly granted Phi-Bot can receive a bounded semantic
SPARK observation and execute MOVE, DASH, and PULSE through the already-qualified
PhiCade -> PixelForge -> SPARK chain.

## What this does not prove

This rung does not yet connect an LLM/model provider to SPARK, nor does it claim
autonomous full-game play, rendering, audio, exact replay, save mutation,
Sideways, Duel, co-op, Memory Arcade control, or Unreal control.

The next rung can finally attach the existing bounded Agent Session / model-policy
stack to this semantic SPARK profile and run a tiny governed local-model
playtest.
