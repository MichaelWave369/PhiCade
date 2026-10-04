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
- source SHA-256: `1a0b386a2cfa4a8e6caf93e58e40620dd8913f068cf6a90bb09159cafbf506da`
- ROM SHA-256: `5f96edb28dbc65ae6f37d2f0bc740ef2ce9e2c2b8b9191a8e1a9882eaffd1a95`

### Normal / Square
- source SHA-256: `d2607daadac805fe27901ff1840639293f5cadb06b40e86ff5fdb477cd148faa`
- ROM SHA-256: `ee5eeac96f5073605014ad2e709331d96c7771a0b076f332f38ec5ec6356270d`

### Swapped / Triangle
- source SHA-256: `41f8b37237832f824275f307beacabbd3731e86c507ac3098ec206b4b4cdbf2e`
- ROM SHA-256: `31cfb105608c2e33c0cf1f43be7c8c037f0a56a4401aa4abfd43ecc6ff694300`

### Swapped / Square
- source SHA-256: `e1773c96b87e92f51e9129ed54f6cb60ca3cd4d03f35d1ed287edba915460c4d`
- ROM SHA-256: `c6f9a0d73187dbec822753c026405199c6d278dee19b4343de524e6fd4568492`

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
