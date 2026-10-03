# Conditional Branch Selector Benchmark

Rung 25 extends PhiCade from a linear prerequisite chain to a conditional
branch that must be selected from current rendered evidence.

## Balanced pair

Two source-first Game Boy ROMs form the pair:

- `branch-selector-triangle-v1`
- `branch-selector-square-v1`

Both variants render the same two modules in the same fixed positions:

- TRIANGLE module on the left,
- SQUARE module on the right.

The only initial task condition that changes is the central selector symbol.

The TRIANGLE variant requires the triangle module. The SQUARE variant requires
the square module.

## Why this is different from Power Chain

Power Chain proves an ordered dependency:

`fuse → generator → gate → target`

Branch Selector adds a decision node before the shared downstream chain:

```text
                 selector
                /        \
        triangle          square
            \              /
             accepted module
                    |
                generator
                    |
                  gate
                    |
                 target
```

A fixed LEFT or RIGHT policy therefore cannot solve both variants.

## Irreversible wrong branch

Pressing A while overlapping the non-matching module enters an explicit FAIL
state.

The transition is intentionally irreversible for the run:

- both side modules disappear,
- the selector becomes a shared FAIL marker,
- no accepted module exists,
- the generator refuses activation,
- the gate remains closed.

This prevents blind trial-and-error from recovering after a wrong commitment.

## Correct branch convergence

Pressing A on the module matching the selector:

- removes both side modules,
- removes the selector,
- creates one shared carried-module badge.

After the player returns to center, TRIANGLE and SQUARE variants are expected
to be pixel-identical.

The shared downstream sequence is then:

1. install the accepted module in the generator,
2. observe the powered generator state,
3. move to the gate switch,
4. open the powered gate,
5. reach the visible X target.

## Pair qualification

`branch_selector_qualify` freezes the same start state and tests both a wrong
commitment and a correct commitment for each variant.

Required negative-control evidence:

- wrong module visibly changes the world into FAIL,
- failed worlds converge to the same center framebuffer,
- A at the generator after failure changes nothing,
- the failed run remains blocked at gate y=80,
- target success remains false.

Required positive-control evidence:

- correct selection changes the rendered world,
- post-selection center frames converge exactly,
- powered-generator frames converge exactly,
- opened-gate frames converge exactly,
- both variants reach the shared target.

## Frozen oracle cadence

TRIANGLE:

`LEFT 28 → A → RIGHT 28 → WAIT 2 → A → WAIT 2 → UP 20 → A → WAIT 2 → UP 32`

SQUARE mirrors the first and return legs.

The three A interactions are:

1. commit to the selected module,
2. install the accepted module,
3. open the powered gate.

## Scoring

The benchmark reuses the rendered-pixel player detector and Manhattan progress
score.

Frozen geometry:

- player start: `(72,112)`,
- target: `(72,24)`,
- initial distance: 88 pixels,
- success distance: ≤4 pixels.

## Provider semantics

Both variants receive the exact same provider instruction.

The prompt identifies the two visible module shapes and explains that the
central selector determines which one is valid. It does not reveal which
selector variant is currently loaded.

The framebuffer is authoritative.

## Interpretation

A successful balanced campaign supports a narrow descriptive claim: under the
frozen model/provider/core/policy conditions, the controller can observe a
conditional task signal, commit to the matching branch, avoid an irreversible
wrong branch, and continue through a shared downstream dependency chain.

This structure is directly relevant to:

- conditional quests,
- dialogue consequence branches,
- alternate puzzle solutions,
- route selection,
- state-dependent tool choice,
- prerequisite graphs that reconverge.

It is not a claim of general causal or planning intelligence.
