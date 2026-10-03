# Ordered Power Chain Benchmark

Rung 24 extends PhiCade's stateful interaction benchmarks from one prerequisite
to a short ordered causal chain.

The task remains deliberately small enough to audit from rendered pixels and
deterministic receipts.

## Balanced pair

Two independently assembled Game Boy ROMs form the probe:

- `power-chain-left-v1`
- `power-chain-right-v1`

Both variants share the same:

- player start,
- side-pedestal geometry,
- generator,
- gate,
- target,
- controller authority,
- provider instruction,
- success contract.

Only the visible fuse location differs.

## Causal chain

The required order is:

1. find the visible fuse,
2. press A while overlapping it,
3. return to the central generator,
4. press A to install the fuse and power the system,
5. move to the central gate switch,
6. press A to open the powered gate,
7. move to the visible X target.

The two variants therefore test the same dependency graph while preventing a
single hard-coded side route from standing in for perception.

## Rendered world states

The benchmark exposes each accepted state transition visually.

### Fuse acquired

The world fuse disappears and a shared carried-fuse badge appears.

### Generator powered

The carried badge disappears and the center generator changes from OFF to ON.

### Gate opened

The horizontal barrier row disappears.

These rendered changes give the pair qualifier observable checkpoints instead
of relying only on hidden WRAM flags.

## Wrong-order controls

`power_chain_qualify` freezes the same start state and tests invalid sequences
before the positive control.

### Generator before fuse

Pressing A at the generator before acquiring the fuse must leave the rendered
frame unchanged.

### Gate before power

Walking to the gate and pressing A before powering the generator must leave the
player blocked at the gate stop.

### Empty-pedestal fake pickup

The player visits the side without the fuse and presses A.

The before-A and after-A rendered frame hashes must match.

Returning to the generator and gate after that fake pickup must still leave the
gate blocked.

## Positive chain

The valid path is:

LEFT variant:

`LEFT 28 → A → RIGHT 28 → WAIT 2 → A → WAIT 2 → UP 20 → A → WAIT 2 → UP 32`

RIGHT variant:

`RIGHT 28 → A → LEFT 28 → WAIT 2 → A → WAIT 2 → UP 20 → A → WAIT 2 → UP 32`

The three A presses mean:

1. acquire fuse,
2. install fuse / power generator,
3. open powered gate.

The explicit WAIT legs freeze short settle windows between causal handoffs so the
generic oracle observes the same deterministic cadence already exercised by the
joint pair qualifier.

## Pair convergence

Once the variant-specific fuse has been correctly acquired and the player
returns to center, the two worlds should converge.

The pair qualifier therefore requires exact cross-variant framebuffer identity
for:

- post-fuse center state,
- powered-generator state,
- opened-gate state.

The initial LEFT/RIGHT frames must differ because the visible fuse appears on
opposite sides.

## Scoring

The task uses the existing rendered-pixel player detector and Manhattan score.

Frozen geometry:

- start: `(72,112)`
- target: `(72,24)`
- initial distance: 88 pixels
- success distance: ≤4 pixels

The closed gate stops the player at y `80`, far outside the success radius.

## Provider semantics

Both ROMs receive the same provider instruction.

The prompt explains the visible fuse → generator → gate chain but never reveals
which side contains the fuse.

The current rendered framebuffer remains authoritative for fuse location.

## Interpretation

A successful balanced campaign supports a narrow descriptive claim: under the
frozen model/provider/core/policy conditions, the controller can follow a
visible multi-step dependency chain whose later actions are only valid after
earlier state transitions have occurred.

This is directly relevant to game mechanics such as:

- fetch → install → activate,
- item → machine → door,
- prerequisite quest stages,
- power routing,
- multi-step environmental puzzles,
- ordered scripted objectives.

It does not establish general causal reasoning outside the frozen task.
