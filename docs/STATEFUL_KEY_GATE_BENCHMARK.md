# Stateful Key-Gate Dependency Benchmark

Rung 23 adds an explicit world-state prerequisite task to PhiCade's benchmark
stack.

The task is intentionally simple enough to audit from pixels while still
requiring a controller to alter game state before later movement can succeed.

## Balanced pair

Two independently assembled Game Boy ROMs form the probe:

- `key-gate-left-v1`
- `key-gate-right-v1`

Both variants share:

- player start,
- target,
- gate geometry,
- interaction rules,
- provider instruction,
- controller authority,
- success distance.

Only the visible key location differs.

## World state

The player starts at `(72,112)`.

A horizontal locked gate occupies screen y `72..79`. The player is stopped at
screen y `80` while the gate is closed.

The target X is visible at `(72,24)`, above the gate.

The key appears at:

- LEFT variant: `(24,112)`
- RIGHT variant: `(120,112)`

## Key acquisition

The key is not acquired by proximity alone.

The player must overlap the visible key and press A.

A successful pickup:

1. sets the ROM's internal acquired-key state,
2. removes the world key tile,
3. draws the same acquired-key badge in both variants.

Pressing A at the empty side does nothing.

## Gate dependency

The gate requires all of the following:

- acquired-key state is true,
- player x is centered at `72`,
- player y is stopped immediately below the gate at `80`,
- A is pressed.

If any prerequisite is absent, the gate remains closed.

A successful unlock removes the rendered barrier row and allows movement toward
the target.

## Frozen positive-control route

LEFT:

`LEFT 24 → A 1 → RIGHT 24 → UP 16 → A 1 → UP 28`

RIGHT:

`RIGHT 24 → A 1 → LEFT 24 → UP 16 → A 1 → UP 28`

The two A presses have distinct meanings:

- first A acquires the key,
- second A unlocks the gate.

## Rendered-pixel scoring

The task uses the existing rendered-pixel player detector and Manhattan score.

Frozen geometry:

- start: `(72,112)`
- target: `(72,24)`
- initial distance: 88 pixels
- success distance: ≤4 pixels

The closed gate stops the player at y `80`, leaving the player far outside the
success radius.

This prevents a missing key or broken unlock rule from being hidden by the
generic scorer.

## Pair qualification

`key_gate_qualify` freezes the initial emulator state and evaluates three
independent paths.

### Direct gate negative control

The player walks straight into the locked gate, presses A without a key, and
continues pressing UP.

The player must remain at the gate stop and fail the benchmark success predicate.

### Empty-side negative control

The player visits the side that contains no key, presses A, returns to the
central gate, tries to unlock, and probes upward.

The gate must remain locked.

This prevents an implementation from silently granting key state merely because
A was pressed in a plausible pickup region.

### Correct dependency path

The player visits the actual rendered key, presses A, and returns to center.

The pair qualifier then requires:

- each variant's post-pickup frame differs from its own initial frame,
- LEFT and RIGHT post-pickup center frames are identical,
- gate approach geometry is identical,
- successful unlock frames are identical,
- the final target succeeds in both variants.

The post-pickup framebuffer convergence is important. Once the visible
variant-specific object has been correctly acquired, the two task worlds become
equivalent.

## Provider semantics

The model instruction is identical across both variants.

It says to find the visible key and use A to acquire it, then unlock the central
gate and reach the visible target.

It does not reveal which side contains the key.

The current framebuffer remains the authority for key location.

## Interpretation

A successful balanced campaign supports a narrow descriptive claim: under the
frozen model/provider/core/policy conditions, the controller can recognize a
visible prerequisite object, deliberately acquire it, carry the resulting game
state forward, and satisfy a later lock dependency.

This is useful groundwork for inventory, quest prerequisites, switches, keys,
doors, pickups, and other stateful game interactions.

It does not imply that the model has a general-purpose inventory system or
understands arbitrary object semantics outside the frozen task.
