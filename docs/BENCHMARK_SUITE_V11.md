# Benchmark Suite v11

Suite v11 is PhiCade's 45-task frozen benchmark population.

It preserves the exact 29 tasks from Suite v10 and adds sixteen Sequential Rule
Composition tasks.

No prior task identity, source hash, ROM hash, oracle, prompt, geometry, or suite
origin is mutated.

## New 2×2×2×2 factorial

Independent factors:

- arrangement: NORMAL / SWAPPED
- query: TRIANGLE / SQUARE
- operator 1: MATCH / FLIP
- operator 2: MATCH / FLIP

The final side is produced by applying operator 1 to the remembered queried side,
retaining that hidden intermediate after Stage 1 is erased, then applying
operator 2 later.

## Frozen population

Suite v10: 29 tasks

Suite v11: 45 tasks

The sixteen new source/ROM hash pairs are frozen in:

- `crates/phicade-runtime/src/agent_gym.rs`
- `benchmarks/suite-v11.json`

They were observed from PR #30's pinned RGBDS/SameBoy CI run before registry
freeze.

## Coverage gate

Native qualification requires:

- Suite v10 remains READY at 29/29;
- Suite v11 is INCOMPLETE at 29/45;
- remains INCOMPLETE at every intermediate coverage through 44/45;
- becomes READY only at 45/45.

## Capability increment

Suite v10:

`remembered relation + query + one current operator -> transformed side`

Suite v11:

`remembered relation + query + operator1 -> hidden intermediate`

then:

`hidden intermediate + operator2 -> final side`

The Stage 1 query/operator disappears before Stage 2. The final framebuffer
therefore cannot reveal the erased arrangement, query, or operator 1.

## Shortcut controls

The factorial is balanced so that:

- always-left succeeds exactly 8/16;
- always-right succeeds exactly 8/16;
- ignoring operator 1 succeeds exactly 8/16;
- ignoring operator 2 succeeds exactly 8/16;
- using only operator 1 succeeds exactly 8/16;
- using only operator 2 succeeds exactly 8/16.

See `SEQUENTIAL_RULE_COMPOSITION_BENCHMARK.md` for the behavioral
qualification contract.
