# Relational Binding Memory Benchmark — Rung 27

Rung 27 extends PhiCade's memory benchmarks from retaining a direction to retaining a **relationship**.

## Factorial design

The briefing always contains TRIANGLE and SQUARE, one on each side.

Two independent factors define the four ROMs:

| Arrangement | Query | Correct later door |
| --- | --- | --- |
| NORMAL: TRIANGLE left, SQUARE right | TRIANGLE | left |
| NORMAL: TRIANGLE left, SQUARE right | SQUARE | right |
| SWAPPED: SQUARE left, TRIANGLE right | TRIANGLE | right |
| SWAPPED: SQUARE left, TRIANGLE right | SQUARE | left |

Proposed task IDs:

- `binding-memory-normal-triangle-v1`
- `binding-memory-normal-square-v1`
- `binding-memory-swapped-triangle-v1`
- `binding-memory-swapped-square-v1`

## Information boundary

1. Briefing renders both symbols and their positions.
2. The controller presses A.
3. Both briefing symbols are erased.
4. A 90-frame lockout prevents immediate carry-through.
5. The choice scene renders only:
   - the queried symbol;
   - two visually identical doors;
   - the player.
6. A neutral D-pad frame is required before movement.
7. The controller moves to a door and presses A.
8. Correct commitment is terminal success; wrong commitment is terminal failure.

The current choice framebuffer therefore contains the query identity but not the answer. The answer requires combining current query identity with the earlier erased arrangement.

## Why this is not Temporal Cue v1 again

Temporal Cue stores one bit of directional information: LEFT versus RIGHT.

Relational Binding Memory stores a mapping and later asks a query against that mapping:

`earlier symbol-position relation + later symbol query -> action`

Swapping the arrangement while keeping the same query reverses the required action. Changing the query while keeping the same arrangement also reverses the required action. No global TRIANGLE=LEFT or SQUARE=RIGHT shortcut survives the full 2x2 set.

## Frozen acceptance contract

Before Suite v9 is allowed to freeze, SameBoy qualification must prove:

- all four source files assemble under pinned RGBDS v1.0.3;
- briefing frames distinguish NORMAL from SWAPPED;
- briefing frames visibly contain both TRIANGLE and SQUARE;
- post-dismiss position evidence is fully erased;
- NORMAL-TRIANGLE and SWAPPED-TRIANGLE choice frames converge exactly;
- NORMAL-SQUARE and SWAPPED-SQUARE choice frames converge exactly;
- TRIANGLE and SQUARE query choice frames remain visibly distinct;
- fixed-left policy succeeds on exactly two of four variants;
- fixed-right policy succeeds on exactly two of four variants;
- wrong commitment is irreversible;
- the correct oracle succeeds on all four variants;
- deterministic replay reproduces the oracle result;
- all four provider instructions are byte-identical and do not reveal arrangement or answer;
- source and ROM SHA-256 values exactly match the frozen registry.

## Suite v9

Suite v9 will preserve the exact 17-task Suite v8 population and add these four tasks for 21 total.

Coverage regression must prove:

- Suite v8 remains READY at 17/17;
- Suite v9 is INCOMPLETE at 17/21;
- remains INCOMPLETE at 18/21, 19/21, and 20/21;
- becomes READY only at 21/21.

No Suite v8 task, hash, oracle, prompt, or membership may be mutated.
