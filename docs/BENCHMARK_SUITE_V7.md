# Benchmark Suite v7

Suite v7 adds balanced conditional-branch tasks without mutating any earlier
task or suite.

Suite ID:

`phicade-agent-gym-suite-v7`

## Membership

Suite v7 contains thirteen tasks:

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
12. `branch-selector-triangle-v1`
13. `branch-selector-square-v1`

The first eleven entries are the exact frozen Suite v6 population.

## Coverage semantics

- Suite v6 remains READY at 11/11.
- Suite v7 is INCOMPLETE at 11/13.
- Suite v7 remains INCOMPLETE at 12/13.
- Suite v7 becomes READY only at 13/13.

## Evidence namespaces

Suite v7 reports and comparisons remain suite-scoped:

- `suite-reports/phicade-agent-gym-suite-v7/`
- `suite-comparisons/phicade-agent-gym-suite-v7/`

Earlier suite namespaces remain unchanged.

## Versioning rule

Suite v7 adds membership only.

It does not mutate prior task IDs, source hashes, ROM hashes, campaign
receipts, Suite Reports, or Suite Comparisons.

See `docs/CONDITIONAL_BRANCH_SELECTOR_BENCHMARK.md`.
