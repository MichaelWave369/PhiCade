# Benchmark Suite v12

Suite v12 is PhiCade's 61-task frozen benchmark population.

It preserves the exact 45 tasks from Suite v11 and adds sixteen Selective
Context Routing tasks.

No prior task identity, source hash, ROM hash, oracle, prompt, geometry, or suite
origin is mutated.

## New 2×2×2×2 factorial

Independent factors:

- briefing layout: A / B
- selected memory bank: CIRCLE / CROSS
- query: TRIANGLE / SQUARE
- operator: MATCH / FLIP

Both memory banks are visible during briefing and use opposite TRIANGLE/SQUARE
bindings. The briefing is erased before the later bank selector, query, and
operator appear.

## Frozen population

Suite v11: 45 tasks

Suite v12: 61 tasks

The sixteen new source/ROM hash pairs are frozen in:

- `crates/phicade-runtime/src/agent_gym.rs`
- `benchmarks/suite-v12.json`

They were minted from the corrected CIR/CRS provenance build on PR #33 after a
shell-variable collision in the initial PR #32 hash-print stage was detected and
repaired.

## Coverage gate

Native qualification requires:

- Suite v11 remains READY at 45/45;
- Suite v12 is INCOMPLETE at 45/61;
- remains INCOMPLETE at every intermediate coverage through 60/61;
- becomes READY only at 61/61.

## Capability increment

Suite v11:

`single erased relation + sequential hidden-state transformation -> final side`

Suite v12:

`multiple competing erased relations + context selector + query + rule -> final side`

The key new capability is addressed retrieval: the same TRIANGLE or SQUARE fact
has different answers depending on which erased memory bank is selected later.

## Shortcut controls

The factorial is balanced so that:

- always-left succeeds exactly 8/16;
- always-right succeeds exactly 8/16;
- ignoring MATCH/FLIP succeeds exactly 8/16;
- always applying FLIP succeeds exactly 8/16;
- always reading CIRCLE succeeds exactly 8/16;
- always reading CROSS succeeds exactly 8/16;
- always querying TRIANGLE succeeds exactly 8/16;
- always querying SQUARE succeeds exactly 8/16.

See `SELECTIVE_CONTEXT_ROUTING_BENCHMARK.md` for the complete behavioral
qualification contract.
