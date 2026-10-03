# Rung 16 Benchmark Suite Reports

Rung 16 adds explicit cross-task aggregation above the task-local campaign layer.

Rung 15 established multiple independently frozen benchmark tasks.

Rung 16 answers:

**What does one exact model cohort look like across every registered task in the suite?**

It does this without weakening Comparison Lab's like-for-like rules.

## Cohort identity

Native PhiCade groups campaigns by the full common pin set:

- provider,
- model name,
- exact model digest,
- SHA-256 of the exact model qualification receipt,
- SameBoy binary SHA-256,
- SameBoy name/version,
- exact Autodrive policy,
- configured trials per task.

That canonical identity is serialized and SHA-256 hashed into a cohort ID.

Campaigns with different digests, qualification receipts, core binaries, policies,
or trial counts therefore land in different cohorts even if the display model name
is the same.

## Task coverage

A cohort is READY only when it contains one verified COMPLETE campaign for every
task in the explicitly selected suite.

Suite v1 requires 2/2:

- move-block-to-x-v1
- move-block-to-x-mirror-v1

Suite v2 requires 3/3:

- the exact two frozen Suite v1 tasks
- wall-detour-v1

Incomplete cohorts remain visible as evidence coverage, for example
`2/3 TASKS // INCOMPLETE`, but native PhiCade refuses to build a report from them.

When more than one campaign exists for the same cohort/task, the highest campaign ID
is selected deterministically.

## Provenance walk

Candidate discovery and report building both use the native campaign validation path.

For every task campaign, PhiCade requires:

- COMPLETE campaign status,
- all configured trials present,
- zero scoring-error trials,
- numeric score for every trial,
- at least two scored trials,
- exact task ID match,
- exact task source SHA-256 match,
- exact task ROM SHA-256 match,
- every referenced Rung 12 trial receipt still present,
- every referenced trial receipt SHA-256 still matching.

A missing or modified underlying trial poisons cohort discovery rather than being
silently skipped.

## Report receipt

Schema:

`phicade.benchmark-suite-report.v1`

A report binds:

- report ID,
- Benchmark Suite ID,
- cohort ID,
- provider/model/exact digest,
- model qualification receipt SHA-256,
- SameBoy binary SHA-256 and identity,
- exact Autodrive policy,
- trials per task,
- one campaign reference per registered task,
- each task campaign receipt SHA-256,
- each task source/ROM SHA-256,
- each task's original campaign statistics,
- suite aggregate statistics.

Reports are namespaced by suite ID:

`suite-reports/<suite-id>/suite-report-000001.json`

For example:

`suite-reports/phicade-agent-gym-suite-v1/suite-report-000001.json`

`suite-reports/phicade-agent-gym-suite-v2/suite-report-000001.json`

Report IDs persist inside each suite namespace. Suite v1 report #1 and Suite v2
report #1 are therefore distinct, unambiguous evidence artifacts.

## Aggregate statistics

The shared runtime computes:

- task count,
- total observed trials,
- total successful trials,
- overall success rate,
- macro mean score,
- minimum task mean,
- maximum task mean,
- population standard deviation of task means.

### Macro mean score

Each registered benchmark task receives equal weight:

`mean(task campaign means)`

This answers:

**How does the model score across benchmark tasks when each task counts equally?**

### Overall success rate

Success rate uses every observed trial across all tasks:

`all successful trials / all observed trials`

This answers:

**How often did the model solve an observed trial across the suite?**

These statistics are intentionally separate.

A benchmark suite can later contain tasks with different trial counts. PhiCade does
not pretend that equal task weighting and equal trial weighting are the same
question.

## Task-spread statistics

The report also preserves:

- minimum task mean,
- maximum task mean,
- population standard deviation across task means.

A strong macro mean therefore cannot hide a large task-to-task spread.

With Suite v1's two mirrored tasks, this can expose directional brittleness such as
strong performance in one geometry and weak performance in the reversed geometry.

It is still a two-task suite, not a universal intelligence measure.

## Suite Report vs Comparison Lab

Comparison Lab remains task-local.

It asks:

**For this exact benchmark task, how do campaign A and campaign B differ?**

Suite Report asks:

**For this exact model cohort, what evidence exists across every registered task?**

Rung 16 does not allow Task A to be directly compared against Mirror Dash using the
Rung 14 A-vs-B statistics.

Cross-task information is combined only through this explicit suite aggregation
artifact.

## Desktop

The SUITE REPORT lane first requires an explicit suite version and then shows
cohorts for that selected task population.

Each cohort displays:

- model,
- short model digest,
- covered/required task count,
- trials per task,
- READY or INCOMPLETE status.

BUILD REPORT is enabled only for a READY cohort.

The UI passes the suite ID and cohort ID.

Native PhiCade rescans the evidence, walks campaign and trial hashes again, computes
the aggregate statistics, and writes the report receipt.

## CI

CI proves Rung 16 through:

- shared suite-statistics tests,
- Suite v1 2/2 READY cohort control,
- Suite v1 incomplete control,
- Suite v2 2/3 INCOMPLETE → 3/3 READY control,
- exact-digest split control,
- mutated underlying trial refusal,
- full native governance tests,
- existing dual-task Benchmark Suite qualification,
- web/type checks.

CI does not fabricate a model's suite result. A suite report is created only from
real local campaign evidence.
