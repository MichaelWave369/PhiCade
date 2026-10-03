# Multi-Room Relay Objective Benchmark

Rung 22 extends PhiCade's governed-memory benchmark from a delayed binary cue
into a small three-room objective.

The benchmark remains intentionally narrow. It tests whether earlier visible
evidence can survive while the controller performs an intervening navigation
task and crosses multiple rendered room states before that evidence becomes
relevant again.

## Balanced pair

Two independently assembled Game Boy ROMs form the probe:

- `relay-rooms-left-v1`
- `relay-rooms-right-v1`

They use the same provider instruction, controls, corridor geometry, terminal
geometry, player movement, wall collision, and room-transition rules.

Only the briefing arrow and the correct final terminal differ.

## Three-room sequence

### 1. Briefing

The player begins at screen position `(72, 24)`.

A visible LEFT or RIGHT arrow provides the only task-specific clue.

Pressing A accepts the objective and removes the briefing cue.

### 2. Corridor

Both variants enter the same corridor at `(16, 24)`.

A visible wall blocks the direct route:

- wall x: 72..79
- wall y: 0..95
- gap begins at y: 96

The deterministic positive-control route is:

`DOWN 40 → RIGHT 60 → UP 40`

A direct RIGHT-only shortcut stops immediately left of the wall.

The corridor constrains the player away from the terminal target Y, preventing
the generic rendered-pixel scorer from accidentally declaring success in the
wrong room.

### 3. Terminal

After traversing the corridor, both variants enter the same terminal scene at
`(72, 112)`.

Two visually identical terminals appear at:

- left: `(24, 112)`
- right: `(120, 112)`

The terminal room does not repeat the briefing cue.

A neutral D-pad frame is required after room entry before LEFT or RIGHT can move
the player.

The correct terminal is determined only by the earlier briefing evidence.

## Why this is stronger than Temporal Cue

Rung 21 separated perception from a later choice with time and a visual-state
change.

Rung 22 adds an intervening task that consumes observations and actions.

The controller must retain the briefing clue while solving visible navigation:

`briefing evidence → corridor navigation → terminal decision`

This creates a controlled analogue of game objectives such as remembering an NPC
instruction, carrying a quest target across rooms, or retaining a clue while
handling unrelated traversal.

## Task authority

Relay tasks receive exactly:

- A
- UP
- DOWN
- LEFT
- RIGHT

The full D-pad is required for the corridor detour.

Earlier benchmark tasks retain their existing narrower scopes.

## Rendered-pixel scoring

The player remains the unique solid 8x8 dark patch.

The benchmark's frozen task geometry uses:

- briefing start: `(72, 24)`
- LEFT final target: `(24, 112)`
- RIGHT final target: `(120, 112)`
- initial Manhattan distance: 136 pixels
- success distance: ≤4 pixels

The final target Y is deliberately outside the corridor's reachable scoring
band.

## Generic qualification

Each relay ROM receives the standard Phi-Agent Gym controls:

- NO-INPUT negative control
- frozen oracle positive control
- deterministic replay
- exact source SHA-256
- exact ROM SHA-256
- pinned SameBoy core evidence

The frozen oracle is:

`A 1 → WAIT 2 → DOWN 40 → RIGHT 60 → UP 40 → WAIT 2 → final side 24`

## Pair qualification

`relay_rooms_qualify` jointly tests both ROMs.

The pair passes only if:

- briefing framebuffer hashes differ,
- corridor framebuffer hashes are identical,
- terminal framebuffer hashes are identical,
- both corridor start positions are identical,
- both terminal start positions are identical,
- direct RIGHT is blocked in both corridors,
- the wrong terminal choice fails in both variants,
- the remembered correct terminal succeeds in both variants.

The pair harness freezes and restores corridor and terminal states so wrong and
correct choices are evaluated from the same deterministic state.

## Governed-memory interpretation

The provider receives the current framebuffer plus the explicit Rung 20 memory
capsule.

It does not receive the benchmark title, correct side, or an instruction that
distinguishes LEFT from RIGHT.

A successful balanced campaign therefore supports a descriptive claim: under the
frozen task/model/provider/core/policy conditions, earlier briefing information
survived an intervening navigation task well enough to guide a later terminal
choice.

It does not establish a general theory of memory or identify the model's
internal mechanism.
