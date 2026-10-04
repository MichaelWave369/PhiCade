# Indirect Context Routing Benchmark — Rung 31

Rung 31 extends PhiCade from direct context selection into **two-hop remembered
indirection**.

Suite v12 proves that a controller can retain two competing erased memory banks
and later retrieve from the bank named directly by a selector.

Rung 31 removes the direct bank selector. Instead, the briefing also shows a
pointer map that binds **STAR** and **MOON** tokens to the CIRCLE and CROSS
memory banks. After erasure, the later query shows only a pointer token and a
symbol. The controller must resolve:

`pointer token -> memory bank -> queried symbol -> remembered side`

before committing.

## Briefing

Two memory banks are visible:

- **CIRCLE**
- **CROSS**

Each contains TRIANGLE and SQUARE on opposite sides.

The banks always use opposite arrangements.

Two pointer tokens are also visible:

- **STAR**
- **MOON**

The pointer map has two variants.

### Pointer map NORMAL

- STAR -> CIRCLE
- MOON -> CROSS

### Pointer map SWAPPED

- STAR -> CROSS
- MOON -> CIRCLE

The bank layout has two variants.

### Layout A

- CIRCLE: TRIANGLE LEFT / SQUARE RIGHT
- CROSS: TRIANGLE RIGHT / SQUARE LEFT

### Layout B

- CIRCLE: TRIANGLE RIGHT / SQUARE LEFT
- CROSS: TRIANGLE LEFT / SQUARE RIGHT

Pressing A erases all bank markers, pointer-map evidence, and position-bearing
symbol pixels.

## Choice scene

After the lockout, the screen reveals only:

1. pointer token: STAR or MOON;
2. query symbol: TRIANGLE or SQUARE;
3. two visually identical doors.

The controller must:

1. recall which bank the pointer token mapped to;
2. retrieve the queried symbol's remembered side from that erased bank;
3. move to the corresponding door;
4. press A to commit.

Wrong commitment is irreversible.

## Full factorial

Independent factors:

- bank layout: A / B
- pointer map: NORMAL / SWAPPED
- pointer token: STAR / MOON
- query: TRIANGLE / SQUARE

Total: **16 frozen task variants**.

## Capability increment

Rung 30:

`direct bank selector + erased bank memory -> side`

Rung 31:

`erased pointer map + erased bank memory + pointer token + query -> side`

The key new requirement is **two-hop addressed retrieval**.

A controller that remembers only the bank bindings cannot solve the task,
because the later frame never names CIRCLE or CROSS.

A controller that remembers only the pointer map also cannot solve the task,
because it still lacks the symbol-to-side relation inside the resolved bank.

## Anti-shortcut balance

The full factorial is balanced so that:

- always-left succeeds exactly 8/16;
- always-right succeeds exactly 8/16;
- assume NORMAL pointer map succeeds exactly 8/16;
- assume SWAPPED pointer map succeeds exactly 8/16;
- always resolve to CIRCLE succeeds exactly 8/16;
- always resolve to CROSS succeeds exactly 8/16;
- ignore query and always retrieve TRIANGLE succeeds exactly 8/16;
- ignore query and always retrieve SQUARE succeeds exactly 8/16.

## Required SameBoy controls

The joint qualifier must prove:

- Layout A + pointer-map NORMAL briefing is condition-independent across future
  pointer/query choice;
- Layout A + pointer-map SWAPPED briefing is condition-independent across future
  pointer/query choice;
- Layout B + pointer-map NORMAL briefing is condition-independent across future
  pointer/query choice;
- Layout B + pointer-map SWAPPED briefing is condition-independent across future
  pointer/query choice;
- all four briefing conditions are visibly distinct where expected;
- all bank markers, pointer-map evidence, and position-bearing symbol pixels are
  erased before choice;
- for a fixed pointer token + query, all four erased-history choice frames are
  byte-identical;
- STAR and MOON choice tokens are visibly distinct;
- TRIANGLE and SQUARE queries are visibly distinct;
- all choice scenes use identical player and door geometry;
- all 16 correct paths succeed;
- every wrong commitment is terminal;
- neutral and attempted-recovery FAIL futures are identical;
- source and ROM hashes exactly match the frozen registry;
- deterministic replay reproduces all 16 oracle outcomes;
- all 16 tasks expose identical provider instructions and identical
  A + LEFT + RIGHT authority scope.

## Provider non-leak

All 16 tasks use one instruction:

> Memorize the TRIANGLE/SQUARE positions in both CIRCLE and CROSS banks and
> also memorize which bank STAR and MOON point to. Press A to erase the
> briefing. Later, use the pointer token to recover the correct bank, then use
> the query symbol to recover the remembered side from that bank. Choose the
> matching identical door and press A to commit. A wrong commitment is
> irreversible.

No layout, pointer mapping, pointer token outcome, selected bank, or answer side
may appear in provider text.

## Suite v13 target

Suite v13 will preserve the exact 61-task Suite v12 population and append the
16 Indirect Context Routing tasks for **77 total frozen tasks**.

Coverage regression must prove:

- Suite v12 remains READY at 61/61;
- Suite v13 remains INCOMPLETE from 61/77 through 76/77;
- Suite v13 becomes READY only at 77/77.
