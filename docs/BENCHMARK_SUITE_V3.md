# Benchmark Suite v3

Suite v3 adds temporal evidence-retention probes without changing any earlier
suite or task artifact.

Suite ID:

`phicade-agent-gym-suite-v3`

## Membership

Suite v3 contains five frozen tasks:

1. `move-block-to-x-v1`
2. `move-block-to-x-mirror-v1`
3. `wall-detour-v1`
4. `temporal-cue-left-v1`
5. `temporal-cue-right-v1`

The first three entries are the exact Suite v2 task specifications.

The two temporal tasks are introduced in Suite v3.

## Coverage semantics

Suite readiness remains exact-membership based.

- Suite v1 is READY at 2/2.
- Suite v2 is READY at 3/3.
- Suite v3 is INCOMPLETE at 3/5 when only the Suite v2 population exists.
- Suite v3 is still INCOMPLETE at 4/5 with only one temporal cue.
- Suite v3 becomes READY only at 5/5.

This prevents a single directional cue from standing in for a balanced
retention probe.

## Temporal pair

The two new tasks deliberately require opposite later choices while presenting
an identical post-cue decision framebuffer.

Their provider instruction is identical and does not reveal which side is
correct.

See `docs/TEMPORAL_CUE_BENCHMARK.md`.

## Evidence namespaces

Suite reports and suite comparisons continue to use suite-scoped storage and
IDs:

- `suite-reports/phicade-agent-gym-suite-v3/`
- `suite-comparisons/phicade-agent-gym-suite-v3/`

Earlier v1/v2 namespaces are unchanged.

## Versioning rule

A new suite adds membership. It does not rewrite old evidence.

Suite v3 therefore extends the benchmark population while preserving all
existing task identities, hashes, campaign receipts, suite reports, and suite
comparison semantics.
