# Benchmark Suite v6

Suite v6 adds balanced ordered-causality tasks without rewriting any earlier
task or suite.

Suite ID:

`phicade-agent-gym-suite-v6`

## Membership

Suite v6 contains eleven tasks:

1. `move-block-to-x-v1`
2. `move-block-to-x-mirror-v1`
3. `wall-detour-v1`
4. `temporal-cue-left-v1`
5. `temporal-cue-right-v1`
6. `relay-rooms-left-v1`
7. `relay-rooms-right-v1`
8. `key-gate-left-v1`
9. `key-gate-right-v1`
10. `power-chain-left-v1`
11. `power-chain-right-v1`

The first nine entries are the exact frozen Suite v5 population.

## Coverage semantics

- Suite v5 remains READY at 9/9.
- Suite v6 is INCOMPLETE at 9/11 with only the Suite v5 population.
- Suite v6 remains INCOMPLETE at 10/11 with only one Power Chain variant.
- Suite v6 becomes READY only at 11/11.

## Evidence namespaces

Suite v6 reports and comparisons remain suite-scoped:

- `suite-reports/phicade-agent-gym-suite-v6/`
- `suite-comparisons/phicade-agent-gym-suite-v6/`

Earlier suite namespaces remain unchanged.

## Versioning rule

Suite v6 adds membership only.

It does not mutate prior task IDs, source hashes, ROM hashes, campaigns, Suite
Reports, or Suite Comparisons.

See `docs/ORDERED_POWER_CHAIN_BENCHMARK.md`.
