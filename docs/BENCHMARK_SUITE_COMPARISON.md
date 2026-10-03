# Benchmark Suite Comparison

## Purpose

Rung 17 adds the explicit comparison layer above Benchmark Suite Reports.

A Suite Comparison answers a narrow question:

> Given two complete reports from the same explicit suite version, produced under
> the same emulator core, Autodrive policy, task population, and trials-per-task
> configuration, what differences are present between report A and report B?

PhiCade does not turn those differences into a winner label.

## Inputs

A comparison requires two distinct
`phicade.benchmark-suite-report.v1` receipts.

The two reports may describe different providers or model identities. That is the
point of this layer.

They must still agree on:

- exact suite ID and therefore exact task population,
- SameBoy binary SHA-256,
- SameBoy identity/version,
- exact Autodrive policy,
- configured trials per task,
- the current registered task set,
- each task's frozen source and ROM hashes.

## Provenance replay

PhiCade does not trust a Suite Report merely because its JSON still exists.

Before comparison it re-opens both reports and, for every registered task:

1. verifies the report schema and COMPLETE status,
2. resolves the exact referenced campaign receipt,
3. re-hashes that campaign receipt,
4. verifies task/source/ROM identity,
5. verifies provider/model/digest/qualification/core/policy pins against the report,
6. re-runs the campaign comparison validator,
7. re-hashes every referenced gameplay trial receipt,
8. recomputes the Suite Report aggregate statistics and requires exact replay.

Any missing or mutated campaign/trial evidence refuses the comparison.

## Task-paired comparison

For each registered task, report A is compared with report B using the existing
campaign comparison machinery:

- mean score difference A−B,
- conservative Welch 95% confidence interval,
- Hedges' g,
- success-rate difference A−B.

This preserves like-for-like task semantics.

PhiCade does not pool trials from different tasks into one synthetic sample.

## Suite-level descriptive statistics

The suite layer reports:

- macro mean score difference A−B across paired task means,
- overall trial-weighted success-rate difference A−B,
- minimum paired task mean difference,
- maximum paired task mean difference,
- population standard deviation of paired task mean differences.

These are descriptive suite-level summaries. They are not a significance test and
do not produce a winner.

## Receipt

A completed comparison is stored as:

`phicade.benchmark-suite-comparison.v1`

under a suite-scoped namespace:

`suite-comparisons/<suite-id>/comparison-000001.json`

For example:

`suite-comparisons/phicade-agent-gym-suite-v1/comparison-000001.json`

`suite-comparisons/phicade-agent-gym-suite-v2/comparison-000001.json`

The receipt binds:

- comparison ID,
- suite ID,
- SameBoy binary and identity,
- exact Autodrive policy,
- trials per task,
- report A identity and exact receipt SHA-256,
- report B identity and exact receipt SHA-256,
- task-paired campaign comparison statistics,
- suite-level descriptive A−B statistics.

Comparison IDs persist across sessions.

## Desktop

The **SUITE COMPARE** lane lists complete reports only from the currently selected
suite version.

Select the suite, choose report A and report B, then press **COMPARE SUITES**.
Cross-version comparisons are refused rather than silently aligning different task
populations.

Native validation remains authoritative. A report can appear in the selector and
still be refused if its underlying evidence has been altered since creation.

## Principle

**Compare complete suites explicitly. Pair the same tasks. Re-verify the whole
evidence chain. Report deltas without manufacturing a verdict.**
