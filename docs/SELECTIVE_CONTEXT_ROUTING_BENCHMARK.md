# Selective Context Routing Benchmark — Rung 30

Rung 30 extends PhiCade from sequential hidden-state transformation into
**context-addressed relational memory**.

Suite v11 proves that a controller can carry an intermediate result across two
separated rule stages.

Rung 30 instead asks whether the controller can retain **two competing
TRIANGLE/SQUARE bindings at once**, then later route a query to the correct
memory bank before applying a rule.

## Briefing

Two independent memory banks are visible simultaneously:

- **CIRCLE bank**
- **CROSS bank**

Each bank contains TRIANGLE and SQUARE on opposite sides.

The two banks always use opposite arrangements.

### Layout A

- CIRCLE: TRIANGLE LEFT / SQUARE RIGHT
- CROSS: SQUARE LEFT / TRIANGLE RIGHT

### Layout B

- CIRCLE: SQUARE LEFT / TRIANGLE RIGHT
- CROSS: TRIANGLE LEFT / SQUARE RIGHT

Pressing A erases every bank marker and every position-bearing TRIANGLE/SQUARE
pixel.

## Choice scene

After the lockout, the screen reveals exactly three pieces of information:

1. bank selector: CIRCLE or CROSS;
2. query symbol: TRIANGLE or SQUARE;
3. operator: MATCH (=) or FLIP (X).

Two visually identical doors are present.

The controller must:

1. select the correct erased memory bank;
2. retrieve the queried symbol's remembered side from that bank;
3. apply MATCH or FLIP;
4. move to the resulting door;
5. press A to commit.

Wrong commitment is irreversible.

## Full factorial

Independent factors:

- briefing layout: A / B
- selected bank: CIRCLE / CROSS
- query: TRIANGLE / SQUARE
- operator: MATCH / FLIP

Total: **16 frozen task variants**.

## Capability increment

Rung 29:

`single remembered relation + sequential operators -> final side`

Rung 30:

`multiple competing remembered relations + later context selector + query + rule -> final side`

The key new requirement is **addressed retrieval**.

A controller that stores only one global TRIANGLE/SQUARE relation cannot solve
the benchmark, because the same query symbol has opposite remembered sides in
the two banks.

## Anti-shortcut balance

Because the two banks are always opposite and the factorial is balanced:

- always-left succeeds exactly 8/16;
- always-right succeeds exactly 8/16;
- ignore MATCH/FLIP succeeds exactly 8/16;
- always FLIP succeeds exactly 8/16;
- ignore bank selector and always read CIRCLE succeeds exactly 8/16;
- ignore bank selector and always read CROSS succeeds exactly 8/16;
- ignore query and always read TRIANGLE succeeds exactly 8/16;
- ignore query and always read SQUARE succeeds exactly 8/16.

These are truth-table controls. They do not claim to model every possible
learned heuristic.

## Required SameBoy controls

The joint qualifier must prove:

- Layout A briefing is condition-independent across future bank/query/operator;
- Layout B briefing is condition-independent across future bank/query/operator;
- Layout A and Layout B briefings are visibly distinct;
- all briefing bank markers and binding pixels are erased before choice;
- for each bank+query+operator condition, Layout A and Layout B choice frames
  are byte-identical;
- CIRCLE and CROSS bank selectors are visibly distinct;
- TRIANGLE and SQUARE queries are visibly distinct;
- MATCH and FLIP operators are visibly distinct;
- all choice scenes use identical player and door geometry;
- all 16 correct paths succeed;
- every wrong commitment is terminal;
- equal-duration neutral and attempted-recovery FAIL futures are identical;
- source and ROM hashes exactly match the frozen registry;
- deterministic replay reproduces all 16 oracle outcomes;
- every variant exposes identical provider instructions and identical
  A + LEFT + RIGHT authority scope.

## Provider non-leak

All 16 tasks use one instruction:

> Memorize the TRIANGLE/SQUARE positions in both the CIRCLE and CROSS banks,
> erase the briefing, then use the later bank selector and query symbol to
> retrieve the correct remembered side. MATCH preserves that side and FLIP
> inverts it. Choose between the identical doors and press A to commit. A wrong
> commitment is irreversible.

No task-specific layout, selected bank, answer side, or operator outcome may
appear in provider text.

## Suite v12 target

Suite v12 will preserve the exact 45-task Suite v11 population and append the
16 Selective Context Routing tasks for **61 total frozen tasks**.

Coverage regression must prove:

- Suite v11 remains READY at 45/45;
- Suite v12 remains INCOMPLETE from 45/61 through 60/61;
- Suite v12 becomes READY only at 61/61.
