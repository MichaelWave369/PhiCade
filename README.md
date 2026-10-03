# PhiCade

> **Old worlds. New players.**

PhiCade is an open-source retro game runtime and emulator front end built around
one rule: controllers may propose actions; the runtime owns authority.

## Current status — Rung 18

PhiCade now governs when autonomous models may observe again after each turn, with
action-aware settle timing and bounded empty-turn backoff enforced natively.

Benchmark Suite v1 currently contains:

- **Move the Block to the X** — top-left → bottom-right
- **Mirror Dash** — bottom-right → top-left

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
- two-task source-first benchmark registry
- cross-task cohort discovery
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
registered Suite v1 task:

1. open **SUITE REPORT**,
2. select a READY cohort,
3. press **BUILD REPORT**.

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

Once at least two complete Suite Reports exist:

1. open **SUITE COMPARE**,
2. select report A and report B,
3. press **COMPARE SUITES**.

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

CI continues to assemble and qualify both suite ROMs and now also tests cross-task
cohort coverage, digest splitting, and trial-tamper refusal.

See:

- `docs/ADAPTIVE_OBSERVATION_CADENCE.md`
- `docs/BENCHMARK_SUITE_V1.md`
- `docs/BENCHMARK_SUITE_REPORTS.md`
- `docs/BENCHMARK_SUITE_COMPARISON.md`
- `docs/BENCHMARK_CAMPAIGNS.md`
- `docs/COMPARISON_LAB.md`
- `docs/ARCHITECTURE.md`

## Content policy

PhiCade does **not** distribute commercial ROMs, proprietary BIOS/firmware,
decryption keys, or copyrighted game assets.

## License

PhiCade's own code, including Benchmark Suite v1 task sources, is MIT. Third-party
emulator cores and model runtimes retain their own licenses and notices.
