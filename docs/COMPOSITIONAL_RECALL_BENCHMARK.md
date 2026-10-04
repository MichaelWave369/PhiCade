# Compositional Recall Benchmark — Rung 28

Rung 28 extends relational binding memory into **memory + rule composition**.

The controller must first remember where TRIANGLE and SQUARE appeared, survive
erasure and delay, then combine that erased relation with a newly revealed rule.

## Core interaction

1. Briefing shows TRIANGLE and SQUARE in opposite positions.
2. A erases all position-bearing briefing evidence.
3. A lockout prevents carry-through.
4. Choice scene reveals:
   - one query symbol: TRIANGLE or SQUARE;
   - one operator: MATCH or FLIP;
   - two visually identical doors.
5. The controller must reconstruct the queried symbol's earlier position.
6. MATCH means choose that remembered side.
7. FLIP means choose the opposite side.
8. A commits; wrong commitment is terminal.

## Full factorial

Three independent binary factors produce eight variants:

- arrangement: NORMAL / SWAPPED
- query: TRIANGLE / SQUARE
- operator: MATCH / FLIP

Correct side is:

| Arrangement | Query | MATCH | FLIP |
| --- | --- | --- | --- |
| NORMAL | TRIANGLE | LEFT | RIGHT |
| NORMAL | SQUARE | RIGHT | LEFT |
| SWAPPED | TRIANGLE | RIGHT | LEFT |
| SWAPPED | SQUARE | LEFT | RIGHT |

Proposed IDs:

- `compositional-recall-normal-triangle-match-v1`
- `compositional-recall-normal-triangle-flip-v1`
- `compositional-recall-normal-square-match-v1`
- `compositional-recall-normal-square-flip-v1`
- `compositional-recall-swapped-triangle-match-v1`
- `compositional-recall-swapped-triangle-flip-v1`
- `compositional-recall-swapped-square-match-v1`
- `compositional-recall-swapped-square-flip-v1`

## Why this is a capability jump

Suite v9 can be solved by retrieving one erased relation:

`query symbol + remembered binding -> side`

Rung 28 requires an additional operation after retrieval:

`query symbol + remembered binding + current operator -> transformed side`

A controller that recalls correctly but fails to apply FLIP will score exactly
half of the factorial. A controller that ignores memory and reacts only to the
current query/operator scene cannot distinguish NORMAL from SWAPPED histories.

## Required controls

Before Suite v10 freezes, SameBoy qualification must prove:

- arrangement evidence is visible during briefing;
- all position-bearing evidence is erased before the choice phase;
- for each query+operator pair, NORMAL and SWAPPED choice frames are byte-identical;
- MATCH and FLIP operators are visibly distinct;
- query symbols are visibly distinct;
- all eight choice scenes have identical player/door geometry;
- all eight correct oracles succeed;
- wrong commitment is terminal;
- equal-time neutral/recovery branches from FAIL remain identical;
- always-left succeeds on exactly 4/8;
- always-right succeeds on exactly 4/8;
- ignore-operator policy succeeds on exactly 4/8;
- always-flip policy succeeds on exactly 4/8;
- all eight provider instructions and authority scopes are identical;
- deterministic replay reproduces all eight oracle outcomes;
- source and ROM hashes match the frozen registry exactly.

## Suite v10

Suite v10 will preserve the exact 21-task Suite v9 population and add these
eight tasks for **29 total**.

Coverage regression must prove Suite v9 remains READY at 21/21 while Suite v10
is INCOMPLETE at 21/29 through 28/29 and becomes READY only at 29/29.
