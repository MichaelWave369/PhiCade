# PhiCade

> **Old worlds. New players.**

PhiCade is an open-source retro game runtime and emulator front end built around
one rule: controllers may propose actions; the runtime owns authority.

## Current status — Rung 16

PhiCade can now aggregate one exact model cohort across every registered task in
Benchmark Suite v1 without weakening task-local Comparison Lab semantics.

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
- digest-bound model qualification
- exact-digest gameplay receipts
- repeated task-local benchmark campaigns
- provenance-checked Comparison Lab
- two-task source-first benchmark registry
- cross-task cohort discovery
- provenance-checked Benchmark Suite reports
- persistent evidence IDs across app sessions

## Run

```bash
npm install
npm run desktop
```

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

- `docs/BENCHMARK_SUITE_V1.md`
- `docs/BENCHMARK_SUITE_REPORTS.md`
- `docs/BENCHMARK_CAMPAIGNS.md`
- `docs/COMPARISON_LAB.md`
- `docs/ARCHITECTURE.md`

## Content policy

PhiCade does **not** distribute commercial ROMs, proprietary BIOS/firmware,
decryption keys, or copyrighted game assets.

## License

PhiCade's own code, including Benchmark Suite v1 task sources, is MIT. Third-party
emulator cores and model runtimes retain their own licenses and notices.
