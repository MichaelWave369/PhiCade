# Benchmark Suite v8

Suite v8 adds the four-task Nested Branch Graph factorial without mutating any
earlier task or suite.

Suite ID:

`phicade-agent-gym-suite-v8`

## Membership

Suite v8 contains the exact thirteen frozen Suite v7 tasks plus:

14. `nested-branch-triangle-circle-v1`
15. `nested-branch-triangle-cross-v1`
16. `nested-branch-square-circle-v1`
17. `nested-branch-square-cross-v1`

## Coverage semantics

- Suite v7 remains READY at 13/13.
- Suite v8 is INCOMPLETE at 13/17.
- Suite v8 remains INCOMPLETE at 14/17.
- Suite v8 remains INCOMPLETE at 15/17.
- Suite v8 remains INCOMPLETE at 16/17.
- Suite v8 becomes READY only at 17/17.

## Versioning

Suite v8 adds membership only. Earlier source hashes, ROM hashes, campaign
receipts, reports, and comparisons remain frozen in their original suite
namespaces.

See `docs/NESTED_BRANCH_GRAPH_BENCHMARK.md`.
