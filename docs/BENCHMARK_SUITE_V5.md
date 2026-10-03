# Benchmark Suite v5

Suite v5 adds balanced stateful object-dependency tasks without changing any
earlier benchmark task or suite.

Suite ID:

`phicade-agent-gym-suite-v5`

## Membership

Suite v5 contains nine tasks:

1. `move-block-to-x-v1`
2. `move-block-to-x-mirror-v1`
3. `wall-detour-v1`
4. `temporal-cue-left-v1`
5. `temporal-cue-right-v1`
6. `relay-rooms-left-v1`
7. `relay-rooms-right-v1`
8. `key-gate-left-v1`
9. `key-gate-right-v1`

The first seven entries are the exact frozen Suite v4 population.

## Coverage semantics

Suite readiness remains exact-membership based.

- Suite v4 remains READY at 7/7.
- Suite v5 is INCOMPLETE at 7/9 when only the Suite v4 population exists.
- Suite v5 remains INCOMPLETE at 8/9 with only one Key Gate variant.
- Suite v5 becomes READY only at 9/9.

This prevents one fixed key-side strategy from standing in for balanced
state-dependent object interaction.

## Evidence namespaces

Suite v5 reports and comparisons remain suite-scoped:

- `suite-reports/phicade-agent-gym-suite-v5/`
- `suite-comparisons/phicade-agent-gym-suite-v5/`

Earlier suite namespaces are unchanged.

## Versioning rule

Suite v5 adds membership only.

It does not mutate previous task IDs, source hashes, ROM hashes, campaign
receipts, Suite Reports, or Suite Comparisons.

See `docs/STATEFUL_KEY_GATE_BENCHMARK.md`.
