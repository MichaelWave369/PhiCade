# Sequential Rule Composition Benchmark — Rung 29

Rung 29 extends compositional recall into **multi-step hidden-state transformation**.

Suite v10 asks the controller to remember an erased symbol↔position relation and
apply one later MATCH/FLIP operator.

Rung 29 requires the controller to:

1. remember the erased relation;
2. apply operator 1;
3. retain the intermediate side after operator 1 disappears;
4. apply operator 2 later;
5. commit only after both transformations.

The controller must therefore maintain and update an internal result across two
separated reasoning stages.

## Interaction sequence

1. Briefing shows TRIANGLE and SQUARE in opposite positions.
2. A erases all position-bearing briefing evidence.
3. After a lockout, Stage 1 reveals:
   - one query symbol: TRIANGLE or SQUARE;
   - operator 1: MATCH (=) or FLIP (X).
4. A acknowledges Stage 1. The Stage 1 query/operator is erased.
5. A second lockout separates Stage 1 from Stage 2.
6. Stage 2 reveals operator 2 only: MATCH (=) or FLIP (X).
7. The controller must apply operator 2 to the intermediate side produced by
   Stage 1.
8. Two identical doors appear.
9. The controller moves to the final side and presses A to commit.
10. Wrong commitment is terminal.

## Full 2×2×2×2 factorial

Independent factors:

- briefing arrangement: NORMAL / SWAPPED
- query: TRIANGLE / SQUARE
- operator 1: MATCH / FLIP
- operator 2: MATCH / FLIP

Total: **16 frozen task variants**.

The final answer is the parity of the two operators applied to the remembered
query side:

- MATCH + MATCH → remembered side
- MATCH + FLIP → opposite side
- FLIP + MATCH → opposite side
- FLIP + FLIP → remembered side

Because operator 1 disappears before operator 2 appears, the controller cannot
solve Stage 2 from the current framebuffer alone.

## Capability increment

Suite v10:

`remembered relation + query + one current operator -> transformed side`

Rung 29:

`remembered relation + query + operator1 -> hidden intermediate`

then:

`hidden intermediate + operator2 -> final side`

This separates one-step transformation from stateful multi-step composition.

## Required behavioral controls

SameBoy qualification must prove:

- NORMAL briefing is condition-independent across query/op1/op2;
- SWAPPED briefing is condition-independent across query/op1/op2;
- NORMAL and SWAPPED briefings are visibly distinct;
- Stage 1 frames are independent of operator 2;
- operator 2 is completely absent before Stage 1 acknowledgement;
- Stage 1 pixels are erased before Stage 2;
- Stage 2 frames are identical across histories that differ only in erased
  arrangement and operator 1 but share the same operator 2;
- MATCH and FLIP remain visibly distinct at each stage;
- all choice geometry is identical;
- all 16 correct oracles succeed;
- all wrong final commitments are terminal;
- equal-time neutral/recovery futures from FAIL remain identical;
- always-left succeeds exactly 8/16;
- always-right succeeds exactly 8/16;
- ignore-op1 succeeds exactly 8/16;
- ignore-op2 succeeds exactly 8/16;
- use-only-op1 succeeds exactly 8/16;
- use-only-op2 succeeds exactly 8/16;
- all 16 provider instructions and authority scopes are identical;
- source and ROM hashes match the frozen registry exactly;
- deterministic replay reproduces all 16 oracle outcomes.

## Anti-shortcut note

The final output depends only on operator parity, but Stage 2 never receives
operator 1. The benchmark therefore does not claim to measure symbolic
algebraic sophistication in isolation. It measures whether a controller can
carry forward an intermediate transformed state after the evidence that
produced it has disappeared.

That distinction is the point.

## Suite v11

Suite v11 will preserve the exact 29-task Suite v10 population and append the 16
Sequential Rule Composition tasks for **45 total frozen tasks**.

Coverage regression must prove:

- Suite v10 remains READY at 29/29;
- Suite v11 remains INCOMPLETE from 29/45 through 44/45;
- Suite v11 becomes READY only at 45/45.
