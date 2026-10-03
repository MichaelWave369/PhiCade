# Benchmark Suite v4

Suite v4 adds balanced multi-room relay objectives without rewriting any earlier
benchmark task or suite.

Suite ID:

`phicade-agent-gym-suite-v4`

## Membership

Suite v4 contains seven tasks:

1. `move-block-to-x-v1`
2. `move-block-to-x-mirror-v1`
3. `wall-detour-v1`
4. `temporal-cue-left-v1`
5. `temporal-cue-right-v1`
6. `relay-rooms-left-v1`
7. `relay-rooms-right-v1`

The first five entries are the exact frozen Suite v3 task specifications.

## Coverage semantics

Suite readiness remains exact-membership based.

- Suite v3 remains READY at 5/5.
- Suite v4 is INCOMPLETE at 5/7 when only the Suite v3 population exists.
- Suite v4 remains INCOMPLETE at 6/7 with only one relay direction.
- Suite v4 becomes READY only at 7/7.

This prevents one remembered-side variant from standing in for a balanced
multi-room result.

## Evidence namespaces

Suite v4 reports and comparisons remain suite-scoped:

- `suite-reports/phicade-agent-gym-suite-v4/`
- `suite-comparisons/phicade-agent-gym-suite-v4/`

Earlier v1/v2/v3 namespaces remain unchanged.

## Versioning rule

Suite v4 adds membership only.

It does not mutate:

- prior task IDs,
- prior source hashes,
- prior ROM hashes,
- prior campaign receipts,
- prior Suite Report coverage,
- prior Suite Comparison semantics.

See `docs/MULTI_ROOM_RELAY_BENCHMARK.md`.
