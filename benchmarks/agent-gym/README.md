# Φ-Agent Gym

Φ-Agent Gym is PhiCade's source-first Game Boy benchmark fixture for measuring
pixel-grounded controller behavior.

## Task

**Move the solid 8×8 block to the visible X target.**

The ROM exposes only ordinary Game Boy D-pad input.

- player start: screen `(16, 24)`
- target: screen `(136, 112)`
- movement: 2 pixels per emulated frame
- observation: rendered framebuffer
- score: normalized Manhattan-distance progress, 0–1000

The model is not given benchmark RAM, object coordinates, or hidden game state.

## Why this exists

Rung 10 proves that an exact local model digest can:

- accept image input,
- produce valid structured output,
- pass a synthetic visual probe.

That does **not** establish gameplay ability.

Φ-Agent Gym provides a deterministic environment where later model-specific runs
can be compared using the same core, task, frame budget, scoring rule, and receipt
shape.

## Source-first build

The ROM is assembled from `main.asm` with the same pinned RGBDS toolchain used by
the SameBoy qualification job.

CI does not trust a prebuilt benchmark binary. It records both:

- source SHA-256
- assembled ROM SHA-256

## Harness controls

The current qualification harness validates the benchmark itself before any real
model is graded.

### NO-INPUT negative control

The frozen starting state is advanced without input.

Expected result:

- zero progress

### Oracle positive control

A deterministic D-pad script moves:

1. RIGHT to the target X coordinate
2. DOWN to the target Y coordinate

Expected result:

- success within 4 pixels
- score >= 980/1000

### Deterministic replay control

The oracle is replayed from the exact same serialized starting state.

Expected result:

- identical final framebuffer SHA-256

## Scoring

The harness locates the solid player sprite from the rendered RGBA framebuffer.
The target coordinate is part of the frozen benchmark manifest.

```text
distance = |player.x - target.x| + |player.y - target.y|

progress = initial_distance - final_distance

score = clamp(progress / initial_distance, 0..1) * 1000
```

The scorer never reads emulator RAM.

## License

The Φ-Agent Gym source is part of PhiCade and is licensed under MIT.
