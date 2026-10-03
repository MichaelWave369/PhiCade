# Rung 19 Benchmark Suite v2

Rung 19 introduces explicit benchmark-suite versioning without changing the frozen
meaning of Suite v1.

Suite v2 ID:

`phicade-agent-gym-suite-v2`

Manifest:

`benchmarks/suite-v2.json`

## Versioning rule

A benchmark **task** has one immutable identity:

- task ID
- source SHA-256
- ROM SHA-256
- start/target geometry
- scorer
- oracle
- prompt
- qualification contract

A benchmark **suite** owns membership separately.

This means an already-frozen task may appear in multiple suite versions without
being copied, renamed, re-hashed, or reinterpreted.

Suite v1 therefore remains exactly:

1. Move the Block to the X
2. Mirror Dash

Suite v2 contains those exact same two task specs plus:

3. Wall Detour

No historical Suite v1 receipt is upgraded to three tasks.

## Task C — Wall Detour

Task ID:

`wall-detour-v1`

Source:

`benchmarks/agent-gym-wall-detour/main.asm`

Frozen source SHA-256:

`02fc444e7ffc6f9de819f7462685448268b41a641592332918f972e17fd0fc96`

Frozen ROM SHA-256:

`c8bfbe95b368635370a61abf004d0efd4135b96c8fa9e4fb510582e25534c4f8`

Start:

- screen x: 16
- screen y: 24

Target:

- screen x: 136
- screen y: 24

The target is directly to the player's right.

A visible striped wall occupies screen x 72..79 and y 0..95. The only opening begins
at y 96.

The deterministic positive-control oracle is:

1. DOWN for 40 frames
2. RIGHT for 60 frames
3. UP for 40 frames

Movement remains 2 pixels per emulated frame.

## Why this task is materially different

Suite v1 tests directional generalization under the same basic open-field control
problem.

Wall Detour changes the problem class.

At the start, Manhattan distance to the target is 120 pixels. Moving DOWN toward
the only valid route **increases** Manhattan distance before the agent can make
progress again.

A controller that greedily chooses actions only when they immediately reduce
distance can become trapped on the left side of the wall.

Success therefore requires using the visible obstacle geometry rather than merely
following the apparent target direction.

This still does not make Suite v2 a comprehensive measure of game-playing
intelligence. It introduces one controlled visual-navigation capability that v1
does not contain.

## Rendered-pixel observation

The benchmark remains framebuffer-only.

The wall uses a striped tile with 32 dark pixels per 8×8 patch.

The player remains a solid tile with 64 dark pixels.

That preserves the existing rendered-framebuffer player locator: the wall cannot
masquerade as the player by presenting an equally dark 8×8 patch.

Collision is implemented by the benchmark ROM itself.

The scorer still does not inspect emulator RAM.

## Scoring

Wall Detour preserves the existing task-local score:

- Manhattan-distance progress
- 0..1000
- success within 4 pixels of the target

Because the required detour initially increases target distance, the progress score
can remain at zero during useful early movement.

That is intentional.

Suite-level reporting continues to report task-local evidence rather than treating
intermediate score as an oracle for the correct path.

## Arbitrary oracle legs

Rung 19 removes the old assumption that every benchmark task has exactly two oracle
legs.

`BenchmarkTaskSpec.oracle` is now an immutable slice.

The qualification harness can execute any number of frozen D-pad legs while retaining
the same controls:

- NO-INPUT negative control
- deterministic oracle positive control
- serialized-start-state replay
- exact final framebuffer replay
- source SHA-256 check
- ROM SHA-256 check
- SameBoy provenance

Tasks A and B remain two-leg tasks.

Wall Detour is three-leg.

## Qualification result

The pinned RGBDS v1.0.3 + pinned SameBoy qualification established:

- exact start geometry: PASS
- NO-INPUT control: PASS
- Wall Detour oracle: PASS
- oracle final distance: 0
- deterministic replay: PASS
- source hash match: PASS
- ROM hash match: PASS after the canonical hash was frozen

Qualification receipt:

`artifacts/agent-gym-wall-detour-qualification.json`

## Versioned Suite Reports

Suite Report candidate discovery now requires an explicit suite ID.

For Suite v1, READY still means 2/2 registered v1 tasks.

For Suite v2, READY means 3/3 registered v2 tasks.

The same model/core/policy/trial-count cohort rules still apply.

Evidence is stored under separate namespaces:

`suite-reports/phicade-agent-gym-suite-v1/`

`suite-reports/phicade-agent-gym-suite-v2/`

Report IDs are persistent inside each suite namespace.

## Versioned Suite Comparison

Suite Comparison also requires an explicit suite ID.

Reports from different suite versions are never compared as though they covered the
same task population.

Comparison evidence is namespaced likewise:

`suite-comparisons/phicade-agent-gym-suite-v1/`

`suite-comparisons/phicade-agent-gym-suite-v2/`

Per-task Welch confidence intervals, Hedges' g, and success-rate deltas remain
task-paired and provenance-checked.

## Desktop

The SUITE REPORT lane now exposes an explicit suite selector.

Changing suite version re-scopes:

- cohort discovery
- READY/INCOMPLETE status
- report building
- report ledger
- Suite Comparison selectors

The desktop defaults to the newest registered suite for convenience, but Suite v1
remains selectable and fully supported.

## Principle

**A new benchmark version adds membership. It does not rewrite old evidence.
Reuse frozen tasks exactly, add genuinely new capability probes, and make the suite
identity explicit everywhere evidence crosses a boundary.**
