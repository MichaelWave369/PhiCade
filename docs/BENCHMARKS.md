# PhiCade Benchmarks

PhiCade's benchmark ladder is a frozen, versioned set of source-first Game Boy
tasks used to test governed controllers under increasingly demanding memory,
routing, and causal requirements.

Suite membership is explicit. Older task identities and hashes remain frozen
when a new suite is added.

## Suite ladder

| Suite | Tasks | Increment |
|---|---:|---|
| v1 | 2 | Move the Block to the X + Mirror Dash |
| v2 | 3 | Wall Detour |
| v3 | 5 | Temporal Cue LEFT / RIGHT |
| v4 | 7 | Relay Rooms LEFT / RIGHT |
| v5 | 9 | Key Gate LEFT / RIGHT |
| v6 | 11 | Power Chain LEFT / RIGHT |
| v7 | 13 | Branch Selector TRIANGLE / SQUARE |
| v8 | 17 | Nested Branch 2x2 factorial |
| v9 | 21 | Relational Binding Memory 2x2 factorial |
| v10 | 29 | Compositional Recall 2x2x2 factorial |
| v11 | 45 | Sequential Rule Composition 2x2x2x2 factorial |
| v12 | 61 | Selective Context Routing 2x2x2x2 factorial |
| v13 | 77 | Indirect Context Routing 2x2x2x2 factorial |

Suite manifests live under `benchmarks/`.

The newest population is **Suite v13: 77 frozen tasks**.

## What the ladder is testing

The early tasks establish navigation, obstacle handling, and visible stateful
dependencies. Later tasks progressively remove information from the current
frame and require a controller to preserve and transform earlier context.

The progression is roughly:

```text
navigation
  -> obstacle routing
  -> delayed cue memory
  -> multi-room objective memory
  -> prerequisite state transitions
  -> ordered causal chains
  -> conditional branch selection
  -> nested decisions
  -> erased relational memory
  -> rule-conditioned recall
  -> sequential hidden-state transformation
  -> selective retrieval from competing erased contexts
  -> indirect retrieval through an erased pointer map
```

The benchmark observation boundary is rendered framebuffer data. Controllers do
not receive emulator RAM, save RAM, or ROM bytes as hidden hints.

## Shortcut controls

PhiCade does not treat task success alone as evidence of the intended
capability. Factorial task families include negative controls designed to make
simple shortcut policies visibly insufficient.

Examples:

| Benchmark slice | Tasks | Frozen shortcut control |
|---|---:|---|
| Relational Binding Memory | 4 | fixed LEFT = 2/4; fixed RIGHT = 2/4 |
| Compositional Recall | 8 | fixed-side / ignore-rule families = 4/8 |
| Sequential Rule Composition | 16 | fixed-side / one-operator families = 8/16 |
| Selective Context Routing | 16 | fixed-side / ignore-bank / ignore-query / ignore-operator families = 8/16 |
| Indirect Context Routing | 16 | fixed-side / assume-map / fixed-bank / ignore-query families = 8/16 |

These are controls for their specific benchmark slices. They are **not** a claim
that a single shortcut policy scores exactly 50% across the full heterogeneous
77-task Suite v13 population.

## Indirect Context Routing

Suite v13 adds sixteen tasks spanning:

- bank layout A / B;
- pointer map NORMAL / SWAPPED;
- pointer token STAR / MOON;
- query TRIANGLE / SQUARE.

The briefing exposes two competing symbol banks and a STAR/MOON pointer map.
Both are erased before the final choice. The controller must reconstruct:

```text
pointer token
  -> erased pointer map
  -> erased bank
  -> erased symbol relation
  -> side
```

For fixed pointer + query, the choice frame converges across the four erased
histories. The visible current frame therefore cannot reveal which hidden
history is correct.

See:

- `INDIRECT_CONTEXT_ROUTING_BENCHMARK.md`
- `BENCHMARK_SUITE_V13.md`

## Building evidence

With a registered benchmark ROM loaded and a qualified model handed off:

- **BENCH TASK** creates one scored gameplay receipt;
- **CAMPAIGN 5x** creates repeated evidence for one task;
- **SUITE REPORT** aggregates a complete compatible cohort across a frozen suite;
- **SUITE COMPARE** compares two reports from the same frozen suite.

A Suite Report cohort pins provider/model/exact digest, model qualification,
SameBoy identity, Autodrive policy, trials per task, campaign receipts, and
underlying gameplay evidence.

Incomplete coverage cannot be promoted to a complete Suite Report.

See:

- `BENCHMARK_CAMPAIGNS.md`
- `BENCHMARK_SUITE_REPORTS.md`
- `BENCHMARK_SUITE_COMPARISON.md`
- `COMPARISON_LAB.md`

## Empirical results

PhiCade supports persistent model benchmark campaigns and suite reports, but
this repository does **not currently freeze a public Suite v13 model result as
a canonical README result**.

That is intentional. A public result should name the exact model, digest,
qualification receipt, suite version, policy, trial count, and evidence ID.

Until such a cohort is committed, the project publishes the benchmark design,
qualification controls, and reporting machinery without inventing a leaderboard.

## Deep benchmark documentation

- `BENCHMARK_SUITE_V1.md`
- `BENCHMARK_SUITE_V2.md`
- `BENCHMARK_SUITE_V3.md`
- `BENCHMARK_SUITE_V4.md`
- `BENCHMARK_SUITE_V5.md`
- `BENCHMARK_SUITE_V6.md`
- `BENCHMARK_SUITE_V7.md`
- `BENCHMARK_SUITE_V8.md`
- `BENCHMARK_SUITE_V9.md`
- `BENCHMARK_SUITE_V10.md`
- `BENCHMARK_SUITE_V11.md`
- `BENCHMARK_SUITE_V12.md`
- `BENCHMARK_SUITE_V13.md`
