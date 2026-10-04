# Benchmark Suite v9

Suite v9 is PhiCade's 21-task frozen benchmark population.

It preserves the exact 17 tasks from Suite v8 and adds four Relational Binding
Memory tasks. No prior task identity, source hash, ROM hash, oracle, prompt,
geometry, or suite origin is mutated.

## New tasks

| Task | Briefing arrangement | Later query | Correct door |
| --- | --- | --- | --- |
| `binding-memory-normal-triangle-v1` | TRIANGLE left / SQUARE right | TRIANGLE | left |
| `binding-memory-normal-square-v1` | TRIANGLE left / SQUARE right | SQUARE | right |
| `binding-memory-swapped-triangle-v1` | SQUARE left / TRIANGLE right | TRIANGLE | right |
| `binding-memory-swapped-square-v1` | SQUARE left / TRIANGLE right | SQUARE | left |

## Frozen provenance

### Normal / Triangle
- source SHA-256: `d54a7e830f5350494ac6993dea7145d9fafff48d5819e6e6bac8b2f8ea111e25`
- ROM SHA-256: `5fe04089950b0e0d1e0c4d322ff2642bb6ec6588817ea9586dbb9de8929e16ad`

### Normal / Square
- source SHA-256: `869d2352255828154f8e11a375084c249d490118e5b905964456e8b1bddbbed1`
- ROM SHA-256: `b80e22322e12c86a36b476e437ee17e0f2b0d499a9d6b08b6f3942263c70f90f`

### Swapped / Triangle
- source SHA-256: `62d7e26a2347dc60826fc7bddd2e172b358a567d76bf2da85a29f49672633e2f`
- ROM SHA-256: `1e14a0cc826011690d2fe1c22e11490d426eee4c4a2c7a529ef3d3f0da8a84c2`

### Swapped / Square
- source SHA-256: `96a2c22a4294185e3c6066ebceffe76f69939bf6f52d3f33d68d7449bfe687f9`
- ROM SHA-256: `a1f5a1413ba4d19f9f6781613dbe56f35e624b0edc05209e0489ec83454b3add`

These hashes were observed from the pinned RGBDS v1.0.3 CI build before registry
freeze.

## Coverage gate

Native qualification requires:

- Suite v8 remains READY at 17/17 using the exact prior population;
- Suite v9 is INCOMPLETE at 17/21;
- Suite v9 remains INCOMPLETE at 18/21;
- Suite v9 remains INCOMPLETE at 19/21;
- Suite v9 remains INCOMPLETE at 20/21;
- Suite v9 becomes READY only at 21/21.

## Capability increment

Earlier Temporal Cue and Relay Rooms tasks require retaining a direction.

Suite v9 requires retaining a relation:

`symbol ↔ earlier position`

The later framebuffer reveals the queried symbol but intentionally omits the
earlier spatial binding. Because both the arrangement and the query vary, the
full quartet defeats global TRIANGLE→LEFT, SQUARE→RIGHT, always-left, and
always-right shortcuts.

See `RELATIONAL_BINDING_MEMORY_BENCHMARK.md` for the behavioral qualification
contract.
