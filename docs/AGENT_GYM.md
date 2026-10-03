# Rung 11 Φ-Agent Gym

Rung 11 adds a source-first Game Boy benchmark environment for measuring
pixel-grounded controller competence.

It does not yet claim that any specific model is good at the task.

It proves that PhiCade now has a deterministic, auditable environment in which
such a claim can later be measured.

## Task

**Move the solid 8×8 player block to the visible X target.**

Frozen geometry:

- start: screen (16, 24)
- target: screen (136, 112)
- movement: 2 pixels per emulated frame
- controls: D-pad only
- system: Game Boy
- observation: rendered framebuffer only

The benchmark source lives at:

`benchmarks/agent-gym/main.asm`

The task contract lives at:

`benchmarks/agent-gym/manifest.json`

## Source-first ROM

CI assembles the benchmark ROM from source using the same pinned RGBDS toolchain
already used in SameBoy qualification.

The built ROM is not accepted as an opaque fixture.

The qualification receipt binds:

- source SHA-256
- assembled ROM SHA-256
- SameBoy binary SHA-256
- core name/version

## Pixel-grounded scorer

The scorer never reads game RAM.

It searches the rendered RGBA framebuffer for the solid 8×8 player sprite and
computes Manhattan distance to the frozen visible target coordinate.

```text
distance = |player.x - target.x| + |player.y - target.y|

score = normalized progress from initial distance to final distance
        on a 0..1000 scale
```

Success requires a final distance of at most 4 pixels.

## Benchmark self-qualification

Before a real model may be compared on the gym, the gym itself must pass three
controls.

### NO-INPUT negative control

From a frozen starting state, run the full benchmark duration with no input.

Required:

- final player position unchanged
- progress = 0

### Oracle positive control

From the same frozen state, a deterministic script moves RIGHT then DOWN.

Required:

- final distance <= 4 pixels
- score >= 980/1000

### Deterministic replay control

Restore the same frozen state and execute the oracle again.

Required:

- final framebuffer SHA-256 exactly matches the first oracle run

## Receipt

Schema:

`phicade.agent-gym-qualification.v1`

The receipt records:

- benchmark source hash
- assembled ROM hash
- SameBoy hash and identity
- start/end emulated frames
- start player coordinate
- target coordinate
- initial distance
- no-input final coordinate/distance/progress
- oracle final coordinate/distance/progress
- oracle score
- oracle success
- negative-control PASS
- positive-control PASS
- deterministic-replay PASS
- both oracle final framebuffer hashes

## What Rung 11 proves

Rung 11 proves the benchmark environment and scorer.

It does **not** yet prove a particular Ollama model can solve the task.

The next model-benchmark layer can reuse this exact ROM, scorer, frame budget, and
receipt vocabulary while binding the run to a Rung 10-qualified model digest.
