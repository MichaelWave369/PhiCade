# Rung 15 Benchmark Suite v1

Rung 15 expands Φ-Agent Gym from one frozen Game Boy task into a two-task
source-first benchmark suite.

Suite ID:

\`phicade-agent-gym-suite-v1\`

The suite manifest lives at:

\`benchmarks/suite-v1.json\`

## Why a second task

A single environment can reveal behavior, but it can also reward a narrow shortcut.

Task A always starts in the upper-left region and is solved by moving RIGHT then
DOWN.

Mirror Dash reverses that geometry and is solved by moving LEFT then UP.

The two tasks intentionally share the same visual grammar, control surface, movement
rate, scorer, success radius, and model-visible objective.

The changed variable is spatial/control direction.

This does not make Suite v1 a comprehensive game benchmark. It makes it materially
harder to confuse one directional habit with general visual control competence.

## Task A — Move the Block to the X

Task ID:

\`move-block-to-x-v1\`

Frozen properties:

- start: (16, 24)
- target: (136, 112)
- oracle: RIGHT for 60 frames, then DOWN for 44 frames
- source SHA-256:
  \`0c82f65532030d64bf022539002f334716a5aae822672b3cdfca511906e51e6f\`
- ROM SHA-256:
  \`353e69e859f50f5ef14f0221e386b18b8194f771cc603696530a59617593c59e\`

## Task B — Mirror Dash

Task ID:

\`move-block-to-x-mirror-v1\`

Frozen properties:

- start: (136, 112)
- target: (16, 24)
- oracle: LEFT for 60 frames, then UP for 44 frames
- source SHA-256:
  \`fc10866cf7166f74f1ae41f8957f3b6057d8bf710731e42a02cbdb9087feb658\`
- ROM SHA-256:
  \`278a8106343fe1688a1370c0575578417744c96ae52568ab1e97f446dc222bfb\`

## Shared task contract

Both tasks use:

- Game Boy / SameBoy,
- D-pad only,
- 2 pixels of movement per emulated frame,
- rendered RGBA framebuffer observation,
- solid 8×8 player sprite,
- visible X target,
- Manhattan-distance progress,
- 0–1000 score,
- success radius <= 4 pixels,
- 120-frame warmup,
- NO-INPUT negative control,
- deterministic oracle positive control,
- exact replay control.

The scorer never reads emulator RAM.

## Runtime task registry

\`phicade-runtime\` now owns the benchmark registry.

Each task entry freezes:

- introduction-suite ID,
- task ID,
- title,
- source SHA-256,
- ROM SHA-256,
- start coordinate,
- target coordinate,
- initial distance,
- success radius,
- warmup frames,
- model-visible prompt,
- deterministic oracle legs.

Runtime lookup is by exact ROM SHA-256.

As of Rung 19, suite membership is stored separately from immutable task identity.
Tasks A and B retain their original task/source/ROM identities and remain the exact
Suite v1 population, while the Suite v2 membership list reuses them unchanged.

A filename, display title, or directory path does not make a ROM a benchmark task.

## Compatibility wrappers

The original \`AGENT_GYM_*\` constants and helper functions remain available as
compatibility wrappers around Task A.

New benchmark execution should use the task registry and generic scorer.

This preserves old receipt/test code without making Task A a hidden privileged
special case.

## Qualification

The existing \`agent_gym_qualify\` binary now accepts:

\`--task <task-id>\`

For each suite member CI:

1. assembles the ROM from source using pinned RGBDS,
2. boots it through the frozen SameBoy build,
3. warms to the frozen start,
4. verifies exact start geometry from rendered pixels,
5. runs a full-duration NO-INPUT control,
6. requires zero progress,
7. restores the same serialized start state,
8. executes the task registry oracle,
9. requires score >= 980/1000 and task success,
10. restores and replays the same oracle,
11. requires identical final framebuffer SHA-256,
12. requires observed source/ROM SHA-256 to exactly match the registry.

Qualification schema:

\`phicade.agent-gym-qualification.v2\`

Task A receipt:

\`artifacts/agent-gym-qualification.json\`

Mirror Dash receipt:

\`artifacts/agent-gym-mirror-qualification.json\`

## Native benchmark execution

Native PhiCade now resolves the loaded ROM through the registry.

For any registered suite task:

- BENCH TASK is eligible,
- CAMPAIGN is eligible,
- warmup comes from the task entry,
- start geometry comes from the task entry,
- score target comes from the task entry,
- task-success comes from the task-specific scorer,
- gameplay receipts bind that task's source/ROM hashes,
- campaigns pin that exact task,
- campaign continuation rejects task drift.

## Model prompt

The Ollama provider also resolves benchmark prompts through the same registry.

Both Suite v1 tasks currently expose the same visible objective:

> move the solid square block onto the visible X target using the D-pad

No coordinates, oracle trajectory, score formula, RAM, or hidden state are exposed.

Non-benchmark ROMs receive no benchmark task prompt.

## Campaigns and Comparison Lab

Campaign receipts remain task-local.

Comparison Lab already requires the same benchmark ID and source/ROM hashes, so a
Task A campaign cannot be directly compared against a Mirror Dash campaign.

That is intentional.

Rung 15 establishes multiple independently frozen tasks.

Rungs 16–17 add explicit Suite Reports and Suite Comparison above task-local
Comparison Lab. Rung 19 adds Suite v2 without modifying this v1 task population.

## What Rung 15 proves

Rung 15 proves that PhiCade's benchmark machinery is no longer structurally tied
to one ROM.

It does not prove that two mirrored tasks are a broad measure of game-playing
intelligence.

It establishes the registry, qualification, runtime, campaign, and UI seams needed
to grow the suite without creating a new bespoke evidence path for every task.


## Rung 19 preservation note

Suite v1 is still exactly two tasks. Its ID, manifest, task hashes, and historical
report/comparison namespace are unchanged.

See `docs/BENCHMARK_SUITE_V2.md` for the versioned membership model and Wall
Detour.
