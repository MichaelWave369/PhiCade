# PhiCade

> **Old worlds. New players.**

PhiCade is an open-source retro game runtime and emulator front end built around
one rule: controllers may propose actions; the runtime owns authority.

## Current status — Rung 38

PhiCade now has an indirect context-routing benchmark. The briefing exposes
two competing memory banks plus a STAR/MOON pointer map. After both the bank
relations and the pointer map are erased, the later scene reveals only a pointer
token and query symbol. Suite v13 preserves all 61 Suite v12 tasks and adds the
full 16-way layout × pointer-map × pointer-token × query factorial for 77 frozen
tasks.

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
- **Suite v12** — the exact v11 tasks + the sixteen **Selective Context Routing** 2×2×2×2 factorial variants
- **Suite v13** — the exact v12 tasks + the sixteen **Indirect Context Routing** 2×2×2×2 factorial variants
- **Runtime Capability Manifest v1** — adapters declare execution model and per-capability UNSUPPORTED / SUPPORTED / QUALIFIED status without inflating weaker runtimes into emulator-shaped interfaces
- **Content Descriptor v1** — FILE / DIRECTORY / LAUNCH_TARGET content locators with optional system and runtime hints, while legacy GameImage behavior remains intact
- **Generic libretro Host v1** — generic FILE loading, full 16-button RetroPad, analog axes, and runtime-observed optional capability claims
- **ScummVM 2026.3.0 qualification** — pinned upstream no-engine launcher build, real framebuffer/input qualification, explicit no-snapshot/no-exact-replay receipt
- **Runtime Registration v1** — user-supplied libretro cores are fingerprinted, identified, capability-described, and persisted without implicitly receiving qualification or launch authority
- **Registered Session Routing v1** — registered core SHA routing with stale-binary checks, explicit operator launch approval, FILE/no-content loading, and capability-aware timeline features
- **Desktop Runtime Manager v1** — native UI for registering user-supplied libretro cores and inspecting SHA, identity, capability, qualification-profile, evidence-binding, and authority state
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
- **Selective Context Routing 16-way set** — two competing erased TRIANGLE/SQUARE
  memory banks require later bank-addressed retrieval before MATCH/FLIP commitment
- **Indirect Context Routing 16-way set** — an erased STAR/MOON pointer map must
  first resolve the erased bank before the queried symbol can resolve the side

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
- sixteen-way layout × bank × query × operator selective-context factorial
- same-condition choice-frame convergence across opposite erased layouts
- fixed-side, operator-ignoring, bank-ignoring, and query-ignoring controls capped at 8/16
- explicit selective-context provenance path uniqueness gate
- Benchmark Suite v12 with exact 61-task membership
- sixteen-way layout × pointer-map × pointer-token × query indirect-context factorial
- four-history choice-frame convergence for fixed pointer + query
- map/bank/query shortcut families capped at exactly 8/16
- Benchmark Suite v13 with exact 77-task membership
- Runtime Capability Manifest v1 with conservative core defaults
- pinned SameBoy 1.0.3 capability profile with source revision + binary-evidence requirement
- capability manifest embedded in the SameBoy qualification receipt
- Content Descriptor v1 with fail-closed legacy EmulatorCore compatibility bridge
- GameImage ↔ FILE+system descriptor compatibility with no SameBoy behavior change
- generic libretro FILE loading without a SystemId allowlist
- full 16-button RetroPad mapping plus left/right analog axes
- state-snapshot/save-data capability probes for unqualified libretro cores
- generic exact replay remains unsupported unless separately qualified
- pinned ScummVM v2026.3.0 / fed42f2068dcafc6aafa1c28c77e4c88def74b66
- no-content ScummVM launcher qualification with no game assets
- ScummVM binary SHA-256 bound into qualification receipt
- governed RetroPad + analog callback-path evidence
- explicit zero-byte serialization and zero libretro save-RAM evidence
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

## Desktop runtime manager

Rung 38 exposes Runtime Registration v1 in the native desktop sidebar.

Operators can select a local libretro core, have PhiCade inspect and fingerprint
it through the governed backend, and review its runtime identity, capability
counts, qualification profile, binary-evidence state, and authority state.

Registration remains inventory/provenance only and does not launch the runtime
or grant authority.

See `docs/DESKTOP_RUNTIME_MANAGER.md`.

## Registered session routing

Rung 37 adds a second session entrypoint backed by Runtime Registration v1.

A registered core is selected by SHA-256, re-hashed before launch, identity
checked against its stored receipt, and requires explicit per-session operator
approval. FILE content and libretro no-content launcher sessions are supported.

Session features are derived from the live capability manifest. SameBoy keeps
rewind/save-state/Replay v1 behavior; ScummVM gets governed frame/audio/input
sessions without timeline features it does not support.

See `docs/REGISTERED_SESSION_ROUTING.md`.

## Runtime registration

Rung 36 adds `phicade.runtime-registration.v1`.

The native shell can now inspect and persist a user-supplied libretro core by
canonical path and SHA-256. The receipt includes the core's identity and
capability manifest, but explicitly records `binaryEvidenceBound=false` and
`authorityGranted=false` until later evidence-binding/session-routing steps
prove more.

See `docs/RUNTIME_REGISTRATION.md`.

## ScummVM qualification

Rung 35 qualifies the official ScummVM `v2026.3.0` libretro port from pinned
source commit `fed42f2068dcafc6aafa1c28c77e4c88def74b66`.

CI builds ScummVM's no-engine launcher in software-rendering mode and runs it
through PhiCade with no game content. The qualifier requires real framebuffer
output and governed joypad/analog callback evidence while proving that the
pinned libretro core exposes no serializable state or save RAM.

ScummVM therefore enters PhiCade as an `EMBEDDED_FRAME_CORE` with qualified
frame/render behavior but no state-checkpointed exact replay.

PhiCade does not distribute the built GPL ScummVM core or any game data.

See `docs/SCUMMVM_QUALIFICATION.md`.

## Generic libretro host

Rung 34 removes the Game Boy-specific compatibility gate from the libretro
adapter. A FILE Content Descriptor can now be offered directly to the loaded
core even without a `SystemId`; the core itself decides whether the content is
valid.

The governed input surface now covers all 16 standard RetroPad buttons plus
left/right analog X/Y axes. Generic cores only advertise optional snapshot/save
capabilities after the loaded runtime actually exposes them. Exact replay stays
UNSUPPORTED unless a named qualification profile proves otherwise.

SameBoy keeps its existing pinned qualification and benchmark behavior.

See `docs/GENERIC_LIBRETRO_HOST.md`.

## Runtime-neutral content

Rung 33 adds `phicade.content-descriptor.v1`.

PhiCade can now describe a selected FILE, DIRECTORY, or runtime-native
LAUNCH_TARGET without assuming every game is a console ROM. Existing
`GameImage` callers remain valid: FILE + system descriptors bridge back into
the legacy emulator path, while directories and launch targets fail closed on
cores that do not explicitly support them.

The optional `runtimeHint` is advisory routing metadata only. It grants no
install, launch, or provider authority.

This is the content seam required for future ScummVM directories/targets and
PixelForge cartridges.

See `docs/CONTENT_DESCRIPTOR.md`.

## Runtime capability manifests

Rung 32 adds `phicade.runtime-capability-manifest.v1`.

Runtime adapters now declare an execution model plus explicit capability status:
`UNSUPPORTED`, `SUPPORTED`, or `QUALIFIED`. The core-neutral
`EmulatorCore` default only claims frame stepping, framebuffer/audio output,
governed actions, and reset. SameBoy's stronger state-snapshot and exact-replay
claims are promoted only under its named qualification profile, and exact binary
evidence is still required.

This is the compatibility seam for future external runtimes such as ScummVM and
semantic/browser runtimes such as PixelForge without forcing them to impersonate
SameBoy.

See `docs/RUNTIME_CAPABILITY_MANIFEST.md`.

## Run

```bash
npm install
npm run desktop
```

## Indirect context-routing benchmark

Suite v13 adds sixteen source-first Game Boy tasks spanning Layout A/B ×
NORMAL/SWAPPED pointer map × STAR/MOON pointer token × TRIANGLE/SQUARE query.

The briefing exposes two opposite CIRCLE/CROSS symbol banks and a separate
STAR/MOON pointer map. Pressing A erases both layers. The later choice names
neither CIRCLE nor CROSS: the controller must recover the pointer mapping first,
then retrieve the symbol-to-side relation from the resolved erased bank.

The joint qualifier proves four briefing-history controls, byte-identical
same-pointer+query choice frames across erased histories, terminal wrong
commitments, exact registry hashes, and balanced 8/16 shortcut ceilings.

See `docs/INDIRECT_CONTEXT_ROUTING_BENCHMARK.md` and
`docs/BENCHMARK_SUITE_V13.md`.

## Selective context-routing benchmark

Suite v12 adds sixteen source-first Game Boy tasks spanning Layout A/B ×
CIRCLE/CROSS bank × TRIANGLE/SQUARE query × MATCH/FLIP operator.

During briefing, both memory banks are visible simultaneously and their
TRIANGLE/SQUARE bindings are opposite. Pressing A erases the bank markers and
all position-bearing symbol evidence. The later scene reveals only the bank
selector, query, operator, and two identical doors.

The joint qualifier proves briefing condition-independence, same-condition
choice convergence across opposite erased layouts, visible bank/query/operator
distinctions, exact registry hashes, terminal wrong commitments, and balanced
8/16 shortcut ceilings.

See `docs/SELECTIVE_CONTEXT_ROUTING_BENCHMARK.md` and
`docs/BENCHMARK_SUITE_V12.md`.

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
2. select the desired frozen suite version, including **Suite v13**,
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

CI assembles and qualifies all seventy-seven registered benchmark ROMs from source,
jointly qualifies the Temporal Cue, Relay Rooms, Key Gate, Power Chain, Branch
Selector, Nested Branch Graph, Relational Binding Memory, Compositional Recall,
Sequential Rule Composition, Selective Context Routing, and Indirect Context
Routing controls, tests prior-suite preservation, Suite v13 61/77 through 77/77
coverage, provenance path uniqueness, digest splitting, and trial-tamper refusal.

See:

- `docs/DESKTOP_RUNTIME_MANAGER.md`
- `docs/REGISTERED_SESSION_ROUTING.md`
- `docs/RUNTIME_REGISTRATION.md`
- `docs/SCUMMVM_QUALIFICATION.md`
- `docs/GENERIC_LIBRETRO_HOST.md`
- `docs/CONTENT_DESCRIPTOR.md`
- `docs/RUNTIME_CAPABILITY_MANIFEST.md`
- `docs/INDIRECT_CONTEXT_ROUTING_BENCHMARK.md`
- `docs/BENCHMARK_SUITE_V13.md`
- `docs/SELECTIVE_CONTEXT_ROUTING_BENCHMARK.md`
- `docs/BENCHMARK_SUITE_V12.md`
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
