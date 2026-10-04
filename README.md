# PhiCade

> **Old worlds. New players.**

PhiCade is an open-source retro game runtime and emulator front end built around
one rule: controllers may propose actions; the runtime owns authority.

## Current status — Rung 29

PhiCade now has a sequential rule-composition benchmark. A briefing places
TRIANGLE and SQUARE on opposite sides, then erases that relation. Stage 1 later
reveals a query plus MATCH/FLIP operator 1; A erases Stage 1 after the controller
must retain the transformed intermediate side. Stage 2 then reveals operator 2
only before a final identical-door commitment. Suite v11 preserves all 29 Suite
v10 tasks and adds the full 16-way arrangement × query × operator1 × operator2
factorial.

Benchmark suites:

- **Suite v1** — Move the Block to the X + Mirror Dash
- **Suite v2** — the exact v1 tasks + **Wall Detour**
- **Suite v3** — the exact v2 tasks + **Temporal Cue: Left** + **Temporal Cue: Right**
- **Suite v4** — the exact v3 tasks + **Relay Rooms: Left** + **Relay Rooms: Right**
- **Suite v5** — the exact v4 tasks + **Key Gate: Left** + **Key Gate: Right**
- **Suite v6** — the exact v5 tasks + **Power Chain: Left** + **Power Chain: Right**
- **Suite v7** — the exact v6 tasks + **Branch Selector: Triangle** + **Branch Selector: Square**
- **Suite v8** — the exact v7 tasks + the four **Nested Branch** 2×2 factorial variants
- **Suite v9** — the exact v8 tasks + the four **Relational Binding Memory** 2×2 factorial variants
- **Suite v10** — the exact v9 tasks + the eight **Compositional Recall** 2×2×2 factorial variants
- **Suite v11** — the exact v10 tasks + the sixteen **Sequential Rule Composition** 2×2×2×2 factorial variants
- **Wall Detour** — target is directly right, but a visible wall forces a
  DOWN → RIGHT → UP route through a lower gap
- **Temporal Cue pair** — opposite initial cues lead to an identical later
  decision screen that requires opposite correct choices
- **Relay Rooms pair** — opposite briefing cues survive a shared wall-detour
  corridor before an identical terminal scene requires the remembered side
- **Key Gate pair** — visible prerequisite object must be acquired before a
  locked barrier can be mutated and traversed
- **Power Chain pair** — visible fuse → powered generator → opened gate must
  occur in order before the target becomes reachable
- **Branch Selector pair** — both branch objects are present, a visible selector
  chooses the valid branch, and a wrong commitment irreversibly dead-ends
- **Nested Branch quartet** — Stage 2 remains hidden until Stage 1 succeeds;
  four factorial variants prevent the second answer from being inferred from the first
- **Relational Binding Memory quartet** — briefing arrangement is erased before
  a later symbol query; identical doors force retrieval of the earlier symbol→position relation
- **Compositional Recall octet** — after erased relational memory, a later MATCH/FLIP
  operator forces preservation or inversion of the recalled side before commitment
- **Sequential Rule Composition 16-way set** — Stage 1 query+operator is erased before
  Stage 2 operator appears, forcing retention of a hidden intermediate result

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
- balanced Power Chain LEFT/RIGHT ordered-causality probes
- explicit fuse-acquire → generator-power → gate-open state sequence
- generator-before-fuse, gate-before-power, and fake-pickup negative controls
- post-fuse, powered-generator, and opened-gate convergence controls
- Benchmark Suite v6 with exact 11-task membership
- balanced TRIANGLE/SQUARE Branch Selector conditional-decision probes
- same two visible modules in both variants with selector-only task condition
- irreversible wrong-branch failure state
- failed-state and accepted-state framebuffer convergence controls
- shared generator → gate → target continuation after correct branch
- Benchmark Suite v7 with exact 13-task membership
- four-way TRIANGLE/SQUARE × CIRCLE/CROSS nested-branch factorial
- future Stage 2 condition hidden before Stage 1 commitment
- Stage 1 history erased before Stage 2 comparison
- irreversible failure controls at both decision depths
- four-way accepted/powered/open-gate convergence controls
- Benchmark Suite v8 with exact 17-task membership
- four-way arrangement × query relational binding-memory factorial
- post-briefing position evidence erased before the query decision
- same-query choice-frame convergence across opposite earlier arrangements
- fixed-left and fixed-right controls each capped at exactly 2/4
- irreversible wrong-door commitment and recovery refusal
- Benchmark Suite v9 with exact 21-task membership
- eight-way arrangement × query × MATCH/FLIP compositional-recall factorial
- same query+operator choice-frame convergence across opposite erased histories
- always-left/right and ignore-operator/always-flip shortcuts capped at 4/8
- Benchmark Suite v10 with exact 29-task membership
- sixteen-way arrangement × query × operator1 × operator2 sequential composition factorial
- Stage 1 future-operator independence and Stage 2 history-erasure convergence controls
- fixed-side and one-operator shortcut families capped at exactly 8/16
- Benchmark Suite v11 with exact 45-task membership
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

## Sequential rule-composition benchmark

Suite v11 adds sixteen source-first Game Boy tasks spanning NORMAL/SWAPPED
arrangement × TRIANGLE/SQUARE query × MATCH/FLIP operator 1 × MATCH/FLIP
operator 2.

The briefing relation is erased before Stage 1. Stage 1 reveals the query and
operator 1, then A erases them before Stage 2 exists. Stage 2 reveals operator 2
only. The controller must therefore carry a transformed intermediate state
across a second delay before choosing between identical doors.

The joint qualifier proves future-operator independence at Stage 1, exact
Stage-2 convergence across erased arrangement/query/operator-1 histories,
operator visibility, balanced shortcut ceilings, and terminal wrong-door
commitment.

See `docs/SEQUENTIAL_RULE_COMPOSITION_BENCHMARK.md` and
`docs/BENCHMARK_SUITE_V11.md`.

## Compositional recall benchmark

Suite v10 adds eight source-first Game Boy tasks spanning NORMAL/SWAPPED
arrangement × TRIANGLE/SQUARE query × MATCH/FLIP operator.

The briefing relation is erased before the later choice. MATCH (=) means choose
the door where the queried symbol appeared earlier; FLIP (X) means choose the
opposite door. The current framebuffer therefore supplies the query and rule,
but not the erased relation the rule must operate on.

The joint qualifier proves same-query+operator convergence across opposite
histories, visible operator/query distinctions, terminal wrong commitments, and
balanced 4/8 shortcut ceilings.

See `docs/COMPOSITIONAL_RECALL_BENCHMARK.md` and
`docs/BENCHMARK_SUITE_V10.md`.

## Relational binding-memory benchmark

Suite v9 adds four source-first Game Boy tasks covering the full
NORMAL/SWAPPED arrangement × TRIANGLE/SQUARE query factorial.

The briefing shows both symbols and their positions. Pressing A removes every
position-bearing briefing pixel, waits through a lockout, then reveals one
query symbol above two identical doors. The controller must remember where that
symbol appeared earlier and press A at the matching door. A wrong commitment is
terminal.

The joint qualifier proves that same-query choice frames converge across
opposite briefing arrangements, that query symbols remain visibly distinct,
that fixed-left and fixed-right policies each solve only two of four variants,
and that wrong commitment cannot recover.

See `docs/RELATIONAL_BINDING_MEMORY_BENCHMARK.md` and
`docs/BENCHMARK_SUITE_V9.md`.

## Nested branch-graph benchmark

Suite v8 adds four source-first Game Boy tasks covering the full
TRIANGLE/SQUARE × CIRCLE/CROSS factorial.

Stage 2 is not visible before Stage 1 succeeds. After the first correct
commitment, the first selector is erased and a new independent CIRCLE/CROSS
selector is revealed. Wrong commitment at either stage is terminal.

The joint qualifier proves delayed information revelation, cross-history
Stage 2 convergence, failure convergence at both depths, and final four-way
world convergence.

See `docs/NESTED_BRANCH_GRAPH_BENCHMARK.md` and `docs/BENCHMARK_SUITE_V8.md`.

## Conditional branch-selector benchmark

Suite v7 adds two source-first Game Boy tasks:

- `branch-selector-triangle-v1`
- `branch-selector-square-v1`

Both render the triangle module on the left and the square module on the right.
A central selector symbol determines which module is valid. Pressing A on the
wrong module enters an irreversible fail state; pressing A on the matching
module removes the variant-specific selector and rejoins the shared
generator → gate → target chain.

The joint qualifier proves wrong-branch dead-end behavior and exact
cross-variant convergence after both failure and correct selection.

See `docs/CONDITIONAL_BRANCH_SELECTOR_BENCHMARK.md` and `docs/BENCHMARK_SUITE_V7.md`.

## Ordered power-chain benchmark

Suite v6 adds two source-first Game Boy tasks:

- `power-chain-left-v1`
- `power-chain-right-v1`

The player must acquire the visible fuse, return to the central generator and
press A to power it, then reach the gate switch and press A again before the
final X can be reached.

The pair qualifier explicitly proves that generator-before-fuse, gate-before-
power, and empty-side fake-pickup sequences do not advance the world.

See `docs/ORDERED_POWER_CHAIN_BENCHMARK.md` and `docs/BENCHMARK_SUITE_V6.md`.

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
2. select the desired frozen suite version, including **Suite v11**,
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

CI assembles and qualifies all forty-five registered benchmark ROMs from source,
jointly qualifies the Temporal Cue, Relay Rooms, Key Gate, Power Chain, Branch
Selector, Nested Branch Graph, Relational Binding Memory, Compositional Recall,
and Sequential Rule Composition controls, tests prior-suite preservation,
Suite v11 29/45 through 45/45 coverage, digest splitting, and trial-tamper
refusal.

See:

- `docs/SEQUENTIAL_RULE_COMPOSITION_BENCHMARK.md`
- `docs/BENCHMARK_SUITE_V11.md`
- `docs/COMPOSITIONAL_RECALL_BENCHMARK.md`
- `docs/BENCHMARK_SUITE_V10.md`
- `docs/RELATIONAL_BINDING_MEMORY_BENCHMARK.md`
- `docs/BENCHMARK_SUITE_V9.md`
- `docs/NESTED_BRANCH_GRAPH_BENCHMARK.md`
- `docs/BENCHMARK_SUITE_V8.md`
- `docs/CONDITIONAL_BRANCH_SELECTOR_BENCHMARK.md`
- `docs/BENCHMARK_SUITE_V7.md`
- `docs/ORDERED_POWER_CHAIN_BENCHMARK.md`
- `docs/BENCHMARK_SUITE_V6.md`
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
