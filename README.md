# PhiCade

> **Old worlds. New players.**

PhiCade is an open-source retro game runtime and emulator front end built around
one rule: controllers may propose actions; the runtime owns authority.

## Current status — Rung 23

PhiCade now has a balanced stateful object-dependency benchmark. A visible key
must be deliberately acquired with A before the same central gate can be
unlocked and the visible target reached. Suite v5 preserves every Suite v4 task
and adds opposite LEFT/RIGHT key placements under identical interaction rules.

Benchmark suites:

- **Suite v1** — Move the Block to the X + Mirror Dash
- **Suite v2** — the exact v1 tasks + **Wall Detour**
- **Suite v3** — the exact v2 tasks + **Temporal Cue: Left** + **Temporal Cue: Right**
- **Suite v4** — the exact v3 tasks + **Relay Rooms: Left** + **Relay Rooms: Right**
- **Suite v5** — the exact v4 tasks + **Key Gate: Left** + **Key Gate: Right**
- **Wall Detour** — target is directly right, but a visible wall forces a
  DOWN → RIGHT → UP route through a lower gap
- **Temporal Cue pair** — opposite initial cues lead to an identical later
  decision screen that requires opposite correct choices
- **Relay Rooms pair** — opposite briefing cues survive a shared wall-detour
  corridor before an identical terminal scene requires the remembered side
- **Key Gate pair** — visible prerequisite object must be acquired before a
  locked barrier can be mutated and traversed

Current evidence stack includes:

- qualified SameBoy 1.0.3 GB/GBC runtime
- deterministic Replay Ledger
- governed Φ-Bot seat
- provider-neutral Agent Driver Protocol
- loopback-only Ollama vision adapter
- bounded Autodrive
- native adaptive observation cadence
- action-aware settle + empty-turn backoff
- governed UTF-8 agent working-memory capsules
- framebuffer + memory hash-bound Agent Driver Protocol v2
- balanced Temporal Cue LEFT/RIGHT memory probes
- identical later decision framebuffer across opposite cue variants
- task-scoped benchmark control grants
- carry-through refusal + neutral re-arm qualification
- Benchmark Suite v3 with exact 5-task membership
- balanced three-room Relay Rooms LEFT/RIGHT objective probes
- identical post-briefing corridor + terminal scenes across relay variants
- direct-route negative control + wrong-terminal negative control
- Benchmark Suite v4 with exact 7-task membership
- balanced Key Gate LEFT/RIGHT stateful-object probes
- explicit A-to-pickup + A-to-unlock world-state transitions
- direct-gate and empty-side negative controls
- post-pickup and open-gate framebuffer convergence controls
- Benchmark Suite v5 with exact 9-task membership
- memory limits frozen into Autodrive policy v1
- memory revision/update/refusal evidence in Autodrive receipts
- migration-safe legacy cadence policy deserialization
- digest-bound model qualification
- exact-digest gameplay receipts
- repeated task-local benchmark campaigns
- provenance-checked Comparison Lab
- immutable source-first benchmark task registry
- explicit versioned suite registry
- Suite v1 preserved as the original two-task population
- Suite v2 with Wall Detour obstacle navigation
- arbitrary-length frozen oracle paths
- suite-scoped cross-task cohort discovery
- provenance-checked Benchmark Suite reports
- provenance-checked Suite Comparison Lab
- task-paired per-task Welch/Hedges comparisons
- descriptive cross-suite A−B statistics
- persistent evidence IDs across app sessions

## Run

```bash
npm install
npm run desktop
```

## Stateful key-gate dependency benchmark

Suite v5 adds two source-first Game Boy tasks:

- `key-gate-left-v1`
- `key-gate-right-v1`

The player starts below a locked barrier with the final X visible above it. A
key appears on one side. The controller must overlap the real key and press A,
return to the central gate, press A again to unlock it, then move through to the
target.

The joint qualifier separately proves that A at the gate without a key fails,
that A on the empty side does not create key state, and that correct pickup
causes both variants to converge to the same post-pickup and opened-gate world.

See `docs/STATEFUL_KEY_GATE_BENCHMARK.md` and `docs/BENCHMARK_SUITE_V5.md`.

## Multi-room relay objective benchmark

Suite v4 adds two source-first Game Boy tasks:

- `relay-rooms-left-v1`
- `relay-rooms-right-v1`

Each starts in a briefing room with a visible LEFT/RIGHT cue. A removes the cue,
then both variants enter the same wall-detour corridor. After navigating
DOWN → RIGHT → UP, both reach the same terminal scene, where the remembered
briefing side determines the correct final choice.

The joint qualifier freezes corridor and terminal states, proves a direct RIGHT
shortcut is blocked, verifies the wrong final terminal fails, and verifies the
remembered terminal succeeds.

See `docs/MULTI_ROOM_RELAY_BENCHMARK.md` and `docs/BENCHMARK_SUITE_V4.md`.

## Temporal Cue memory benchmark

Suite v3 adds two source-first Game Boy tasks:

- `temporal-cue-left-v1`
- `temporal-cue-right-v1`

Both show an initial arrow, require A to dismiss it, enforce a 90-frame lockout,
then present the same two-door decision screen. The correct later choice is
opposite across the pair.

A neutral-arm rule prevents a direction scheduled during the cue turn from being
carried through the delay. CI jointly verifies that the cue frames differ while
the later decision frames are pixel-identical.

See `docs/TEMPORAL_CUE_BENCHMARK.md` and `docs/BENCHMARK_SUITE_V3.md`.

## Governed agent working memory

Agent Driver Protocol v2 carries an explicit memory capsule with every turn.

- the model receives only the current rendered framebuffer plus the explicit capsule,
- the response must echo the exact pending memory SHA-256,
- memory updates replace the capsule rather than append hidden history,
- native PhiCade enforces total and per-turn byte budgets,
- Autodrive starts from an empty capsule,
- autonomous-run receipts seal initial/final hashes, final content, revision count,
  updates, bytes written, and refused memory proposals,
- old policy JSON without cadence fields deserializes to legacy cadence semantics
  instead of inheriting current defaults.

The default autonomous policy allows a 4096-byte capsule and a 1024-byte
replacement proposal per turn.

See `docs/GOVERNED_AGENT_MEMORY.md`.

## Adaptive observation cadence

Autodrive no longer requests the next model observation merely because the prior
action queue is empty.

The native runtime now freezes a cadence policy with:

- 2-frame minimum observation spacing,
- 2-frame post-action settle,
- 8-frame initial empty-turn backoff,
- exponential empty backoff capped at 60 frames.

The desktop displays the next eligible observation frame, but native PhiCade remains
authoritative and refuses early autonomous turns.

See `docs/ADAPTIVE_OBSERVATION_CADENCE.md`.

## Versioned Benchmark Suites

Suite membership is separate from frozen task identity.

`benchmarks/suite-v1.json` remains the original two-task population. Rung 19 adds
`benchmarks/suite-v2.json`, which reuses those exact task hashes and adds
`wall-detour-v1`.

The desktop **SUITE REPORT** lane exposes an explicit suite selector. READY coverage,
report IDs, report storage, and Suite Comparison are all scoped by suite ID.

See `docs/BENCHMARK_SUITE_V2.md`.

## Benchmark Suite v1

Suite manifest:

`benchmarks/suite-v1.json`

Registered tasks:

- `move-block-to-x-v1`
- `move-block-to-x-mirror-v1`

Both tasks are independently assembled, hash-pinned, rendered-pixel scored,
oracle-qualified, and replay-qualified.

## Build task evidence

With a registered benchmark ROM loaded and a qualified model handed off:

- **BENCH TASK** creates one scored gameplay receipt,
- **CAMPAIGN 5×** creates repeated evidence for that task,
- **COMPARISON LAB** compares compatible campaigns from the same task.

## Build a Suite Report

Once the same model cohort has one COMPLETE fully scoreable campaign for every
task in the selected suite:

1. open **SUITE REPORT**,
2. select **Suite v1**, **Suite v2**, **Suite v3**, **Suite v4**, or **Suite v5**,
3. select a READY cohort,
4. press **BUILD REPORT**.

A cohort pins:

- provider/model/exact digest,
- model qualification receipt hash,
- SameBoy binary/identity,
- Autodrive policy,
- trials per task.

Native PhiCade re-verifies every campaign and every underlying trial receipt before
aggregation.

The report contains:

- one campaign receipt hash per task,
- each task's original campaign statistics,
- macro mean score across task means,
- overall success rate across all trials,
- min/max task mean,
- population standard deviation across task means.

Incomplete cohorts remain visible but cannot be built.

## Compare Suite Reports

Once at least two complete reports exist for the same selected suite:

1. select the suite version,
2. open **SUITE COMPARE**,
3. select report A and report B,
4. press **COMPARE SUITES**.

Cross-version reports are never compared as though they covered the same task
population.

PhiCade re-opens both suite reports, re-hashes every referenced campaign, re-validates
every underlying gameplay trial receipt, and recomputes the suite aggregates before
comparison.

Each frozen task is compared A−B with the existing campaign Welch 95% CI, Hedges' g,
and success-rate delta machinery. The suite layer then reports the macro mean-score
delta, overall success-rate delta, task-delta min/max, and task-delta population
standard deviation.

There is deliberately no winner field.

## Evidence semantics

**Comparison Lab stays like-for-like.**

It never directly compares campaigns from different benchmark tasks.

**Suite Report is the explicit cross-task layer.**

Macro task weighting and trial-weighted success rate are reported separately rather
than collapsed into one mystery score.

## Qualification

```bash
bash ./scripts/qualify-sameboy.sh
```

CI assembles and qualifies all nine registered benchmark ROMs from source,
jointly qualifies the Temporal Cue, Relay Rooms, and Key Gate pairs, tests
prior-suite preservation, Suite v5 7/9 → 8/9 → 9/9 coverage, digest splitting,
and trial-tamper refusal.

See:

- `docs/STATEFUL_KEY_GATE_BENCHMARK.md`
- `docs/BENCHMARK_SUITE_V5.md`
- `docs/MULTI_ROOM_RELAY_BENCHMARK.md`
- `docs/BENCHMARK_SUITE_V4.md`
- `docs/TEMPORAL_CUE_BENCHMARK.md`
- `docs/BENCHMARK_SUITE_V3.md`
- `docs/GOVERNED_AGENT_MEMORY.md`
- `docs/ADAPTIVE_OBSERVATION_CADENCE.md`
- `docs/BENCHMARK_SUITE_V1.md`
- `docs/BENCHMARK_SUITE_V2.md`
- `docs/BENCHMARK_SUITE_REPORTS.md`
- `docs/BENCHMARK_SUITE_COMPARISON.md`
- `docs/BENCHMARK_CAMPAIGNS.md`
- `docs/COMPARISON_LAB.md`
- `docs/ARCHITECTURE.md`

## Content policy

PhiCade does **not** distribute commercial ROMs, proprietary BIOS/firmware,
decryption keys, or copyrighted game assets.

## License

PhiCade's own code, including the benchmark task sources, is MIT. Third-party
emulator cores and model runtimes retain their own licenses and notices.
