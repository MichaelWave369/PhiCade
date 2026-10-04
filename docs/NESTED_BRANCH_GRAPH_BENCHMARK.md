# Nested Branch Graph Benchmark

Rung 26 extends PhiCade from one conditional branch to a two-level decision
graph with delayed condition revelation.

## Factorial task family

Suite v8 adds four source-first Game Boy tasks:

- `nested-branch-triangle-circle-v1`
- `nested-branch-triangle-cross-v1`
- `nested-branch-square-circle-v1`
- `nested-branch-square-cross-v1`

The quartet forms a 2×2 factorial:

```text
Stage 1 family      Stage 2 submodule
----------------    -----------------
TRIANGLE             CIRCLE
TRIANGLE             CROSS
SQUARE               CIRCLE
SQUARE               CROSS
```

Stage 2 is intentionally not rendered before a correct Stage 1 commitment.

## Stage 1

Every task initially renders:

- TRIANGLE candidate on the left,
- SQUARE candidate on the right,
- one Stage 1 selector in the center.

The controller must match the selector and press A on the corresponding family.

A wrong Stage 1 commitment is irreversible.

## Stage 2

Only after Stage 1 succeeds does the world render:

- CIRCLE submodule on the left,
- CROSS submodule on the right,
- one newly revealed Stage 2 selector.

The controller must make a second independent choice from the new framebuffer.

A wrong Stage 2 commitment is also irreversible.

## Why four tasks are required

Two variants would allow Stage 2 to be correlated with Stage 1.

The 2×2 design explicitly breaks that shortcut:

- TRIANGLE can lead to CIRCLE or CROSS,
- SQUARE can lead to CIRCLE or CROSS.

A controller therefore has to observe the second selector after Stage 1 rather
than infer it from the first answer.

## Causal graph

```text
                   Stage 1 selector
                    /            \
              TRIANGLE          SQUARE
                /  \             /  \
          CIRCLE  CROSS     CIRCLE  CROSS
                \   |        |   /
                 accepted Stage 2
                        |
                    generator
                        |
                       gate
                        |
                      target
```

Wrong commitment at either decision depth enters a terminal FAIL world.

## Factorial qualification

`nested_branch_qualify` runs all four ROMs under the same pinned SameBoy core.

The verifier requires:

1. TRIANGLE→CIRCLE and TRIANGLE→CROSS initial frames are identical.
2. SQUARE→CIRCLE and SQUARE→CROSS initial frames are identical.
3. TRIANGLE-family initial frames differ from SQUARE-family initial frames.
4. After correct Stage 1, the two CIRCLE worlds are pixel-identical regardless
   of Stage 1 history.
5. After correct Stage 1, the two CROSS worlds are pixel-identical regardless
   of Stage 1 history.
6. CIRCLE and CROSS Stage 2 worlds remain visibly distinct.
7. Stage 1 wrong-branch FAIL states converge across all four tasks.
8. Stage 2 wrong-branch FAIL states converge across all four tasks.
9. Generator activation is refused after Stage 2 failure.
10. Correct Stage 2 acceptance converges all four tasks onto one framebuffer.
11. Powered-generator worlds converge.
12. Open-gate worlds converge.
13. Every correct path reaches the same target.

## Oracle geometry

Frozen player/target geometry:

- start: `(72,112)`
- Stage 2 center: `(72,80)`
- closed-gate stop: y=56
- target: `(72,24)`
- initial Manhattan distance: 88
- success threshold: ≤4 pixels

The Stage 1 and Stage 2 side choices are mirrored around x=72.

## Provider semantics

All four tasks receive the same provider instruction.

The prompt describes the fixed candidate layout and the two-stage rule but does
not disclose which of the four variants is currently loaded.

The framebuffer remains authoritative.

## Claim boundary

A passing result supports a narrow claim under the frozen task/core/policy
conditions: the controller can make one evidence-conditioned commitment, accept
a newly revealed independent condition, make a second commitment, avoid
terminal wrong branches at both depths, and continue after graph convergence.

It is not a claim of general planning, causal reasoning, or game-playing
intelligence.
