# Public Suite Results v1

Rung 41 adds a deterministic publication layer on top of verified PhiCade
Benchmark Suite Reports.

A public result is not typed in by hand and is not derived from the visible
desktop numbers alone.

The export path is:

```text
Benchmark campaigns
        |
        v
verified Suite Report
        |
        v
revalidate report + every referenced campaign
        |
        v
Public Suite Result v1
        |
        +-- JSON
        +-- Markdown
```

## Desktop flow

The **PUBLIC RESULT** lane appears beside Suite Report and Suite Compare.

1. Select a benchmark suite.
2. Select one existing verified Suite Report.
3. Press **EXPORT JSON + MD**.
4. PhiCade re-opens the report and revalidates its full provenance.
5. If validation succeeds, PhiCade writes deterministic JSON and Markdown
   artifacts.

The exporter never treats the currently displayed score as sufficient evidence.

## Schema

Public JSON uses:

`phicade.public-suite-result.v1`

with:

`recordStatus = VERIFIED_EXPORT`

The artifact includes:

- suite ID, title, and version;
- source Suite Report ID and SHA-256;
- cohort ID;
- provider;
- exact model name and digest;
- model qualification SHA-256;
- exact core SHA-256, name, and version;
- Autodrive policy;
- configured trials per task;
- aggregate suite statistics;
- total scored and scoring-error trials;
- one row per task;
- each task's campaign ID and campaign receipt SHA-256.

## Aggregate semantics

The export preserves the Suite Report's existing semantics.

- **Macro mean score** weights tasks equally.
- **Overall success rate** is trial-weighted.
- **Task mean min/max** expose the score range across tasks.
- **Task-mean population standard deviation** exposes dispersion across task
  means.

Those quantities are reported separately. PhiCade does not collapse them into a
new synthetic ranking number.

## Per-task table

The Markdown artifact contains one row per frozen task with:

- task title;
- mean score / 1000;
- success rate;
- scored / observed trials;
- campaign ID;
- full campaign receipt SHA-256.

This keeps the headline suite result connected to the evidence underneath it.

## Determinism

The public export contains no current timestamp.

For the same verified Suite Report and the same renderer version, the JSON and
Markdown bytes are deterministic.

The returned desktop artifact includes SHA-256 for both files.

## Interpretation boundary

A public export is one empirical cohort.

It does not:

- turn structural shortcut controls into empirical model baselines;
- claim that one model result generalizes to another digest;
- turn a Suite v13 score into evidence for unsupported runtime capabilities;
- declare a universal winner;
- rewrite or mutate the original Suite Report.

## Current publication status

The repository provides the verified export machinery, but it does not yet
commit a canonical Suite v13 local-model result.

That remains an evidence event: a specific completed cohort must first exist,
be exported, reviewed, and then intentionally published.

See also:

- `BENCHMARKS.md`
- `BENCHMARK_SUITE_REPORTS.md`
- `BENCHMARK_CAMPAIGNS.md`
- `EVIDENCE.md`
