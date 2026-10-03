# PhiCade

> **Old worlds. New players.**

PhiCade is an open-source retro game runtime and emulator front end built around
one rule: controllers may propose actions; the runtime owns authority.

## Current status — Rung 19

PhiCade now supports explicit, immutable benchmark-suite versioning. Suite v1
remains frozen at two tasks, while Suite v2 reuses those exact tasks and adds a
third obstacle-navigation task without rewriting historical evidence.

Benchmark suites:

- **Suite v1** — Move the Block to the X + Mirror Dash
- **Suite v2** — the exact v1 tasks + **Wall Detour**
- **Wall Detour** — target is directly right, but a visible wall forces a
  DOWN → RIGHT → UP route through a lower gap

Current evidence stack includes:

- qualified SameBoy 1.0.3 GB/GBC runtime
- deterministic Replay Ledger
- governed Φ-Bot seat
- provider-neutral Agent Driver Protocol
- loopback-only Ollama vision adapter
- bounded Autodrive
- native adaptive observation cadence
- action-aware settle + empty-turn backoff
- cadence evidence in Autodrive receipts
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
2. select **Suite v1** or **Suite v2**,
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

CI assembles and qualifies all three registered benchmark ROMs from source and
tests Suite v1 preservation, Suite v2 2/3 → 3/3 coverage, digest splitting, and
trial-tamper refusal.

See:

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
