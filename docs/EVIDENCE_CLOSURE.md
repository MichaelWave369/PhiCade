# Evidence Closure v1

Rung 42 closes PhiCade's benchmark evidence graph through the receipts that
previous Suite Report verification previously referenced only by hash.

The closed chain is:

```text
Suite Report
    |
    v
Campaign receipt
    |
    v
Model gameplay receipt
    |
    v
Autodrive receipt

Suite Report / Campaign
    |
    v
Model qualification receipt
```

## Why this rung exists

Before Rung 42, Suite Report provenance validation already re-opened every
campaign and verified each referenced model gameplay receipt by exact SHA-256.

That proved that the campaign still referenced the same trial bytes.

It did not yet parse those trial receipts and prove that their semantic identity
still matched the campaign, nor did it walk the trial's referenced Autodrive
receipt or reopen the exact model qualification receipt pinned by the cohort.

Rung 42 makes those final edges explicit.

## Model gameplay receipt checks

For each campaign trial, PhiCade now requires the parsed model gameplay receipt
to match the campaign/trial on:

- receipt schema;
- benchmark run ID;
- record status;
- benchmark task ID;
- provider;
- model;
- exact model digest;
- model qualification receipt SHA-256;
- benchmark source SHA-256;
- benchmark ROM SHA-256;
- core SHA-256;
- core name/version;
- Autodrive policy;
- stop reason;
- score;
- task success flag.

A hash match with semantically mismatched contents is refused.

## Autodrive receipt checks

The model gameplay receipt's `autodriveReceiptSha256` is resolved under the
task's ROM namespace and re-hashed.

The parsed Autodrive receipt must match the model gameplay receipt on:

- schema;
- Autodrive run ID;
- provider;
- exact `model@digest` identity;
- game/ROM SHA-256;
- core name/version;
- policy;
- stop reason;
- start/end frames;
- turns issued/completed;
- total actions;
- final framebuffer SHA-256.

PhiCade also checks basic receipt self-consistency:

- policy validates;
- end frame is not before start frame;
- completed turns do not exceed issued turns;
- final memory byte count matches the stored UTF-8 content;
- final memory SHA-256 matches the stored memory content.

## Immutable model qualification receipts

Qualification receipts were historically addressed by model digest:

`model-qualifications/ollama/<digest>.json`

That remains as the convenient latest alias.

Rung 42 additionally writes every qualification receipt to an immutable,
SHA-addressed archive:

`model-qualifications/ollama/receipts/<receipt-sha256>.json`

New benchmark evidence can therefore keep referring to the exact qualification
receipt bytes even if the same model digest is qualified again later.

For pre-Rung-42 evidence, verification falls back to the digest alias only when
its current SHA-256 still exactly matches the hash pinned by the benchmark
cohort.

If neither the immutable archive nor an exact matching alias is available,
evidence closure fails rather than substituting a newer qualification receipt.

## Where closure is enforced

Rung 42 requires full evidence closure before:

- Public Suite Result export;
- Suite Report comparison.

The existing top-level Suite Report and campaign checks still run first.

## Non-goals

Evidence Closure v1 does not:

- re-run the model;
- re-run the emulator gameplay;
- reconstruct historical framebuffer pixels not already committed by receipts;
- promote a model or runtime beyond its existing qualification status;
- repair missing historical evidence.

Missing or mutated evidence is a refusal condition.
