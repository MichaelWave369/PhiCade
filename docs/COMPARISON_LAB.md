# Rung 14 Comparison Lab

Rung 14 compares two completed Rung 13 benchmark campaigns only after native
PhiCade verifies that they represent the same benchmark environment.

The lab reports measured differences and uncertainty.

It does not emit a winner field, rank, tier, or universal model verdict.

## Entry requirements

Comparison requires two distinct campaign IDs.

Both campaign receipts must be:

- schema `phicade.benchmark-campaign.v1`,
- `recordStatus = COMPLETE`,
- complete for every configured trial,
- free of scoring-error trials,
- numeric for every trial score,
- at least two scored trials.

Native PhiCade then re-hashes every individual Rung 12 gameplay receipt referenced
by both campaign summaries.

If any referenced trial receipt is missing or its SHA-256 no longer matches the
campaign summary, comparison is refused.

## Compatibility gate

The two campaigns must match on:

- benchmark ID,
- provider,
- Φ-Agent Gym source SHA-256,
- Φ-Agent Gym ROM SHA-256,
- SameBoy binary SHA-256,
- SameBoy name/version,
- exact Autodrive policy,
- configured trial count.

The model name, model digest, and model qualification receipt are intentionally
allowed to differ. Those are the subjects being compared.

A campaign may also be compared against another campaign from the same model
digest to study repeatability.

## Persistent evidence IDs

Rung 14 also fixes evidence allocation across emulator sessions.

At session start, native PhiCade scans existing evidence directories and starts
the next IDs at the highest existing receipt plus one for:

- Autodrive receipts,
- model gameplay benchmark receipts,
- benchmark campaign receipts,
- campaign comparison receipts.

A new session therefore does not restart at `run-000001.json` or
`campaign-000001.json` and overwrite historical evidence.

## Comparison statistics

Schema:

`phicade.campaign-comparison.v1`

Let:

- A = the selected campaign A score sample,
- B = the selected campaign B score sample.

All signed statistics use the convention:

`A - B`

### Mean score difference

`mean(A) - mean(B)`

Positive values mean A's observed mean score was higher in these campaigns.
Negative values mean B's observed mean score was higher.

### Welch 95% confidence interval

The lab uses the Welch unequal-variance standard error and
Welch-Satterthwaite degrees of freedom.

Because current campaigns contain at most 20 trials per side, effective degrees
of freedom remain within the frozen lookup range.

For fractional degrees of freedom, PhiCade uses the lower integer Student-t
critical value. This is conservative because the critical value decreases as
degrees of freedom increase.

The receipt records:

- Welch standard error,
- Welch degrees of freedom when defined,
- lower 95% bound,
- upper 95% bound.

If both samples have zero variance, standard error is zero and the interval is the
exact observed mean difference.

### Hedges' g

PhiCade reports small-sample corrected standardized mean difference:

`Hedges g(A - B)`

The sign follows A minus B.

If both samples have zero pooled variance and equal means, g is 0.

If pooled variance is zero but the means differ, finite standardized effect size
is undefined and the receipt stores null.

### Success-rate difference

`successRate(A) - successRate(B)`

The UI renders this as percentage-point difference.

## Interpreting uncertainty

A confidence interval that crosses zero means the observed campaign data do not
exclude zero mean-score difference at the stated interval construction.

A confidence interval that does not cross zero describes these matched campaign
samples. It does not establish that one model is universally superior across
other games, prompts, providers, policies, hardware, versions, or future digests.

Comparison Lab deliberately reports the measurements instead of converting them
into a winner badge.

## Comparison receipt

Each comparison receipt binds:

- comparison ID,
- campaign A ID,
- campaign A receipt SHA-256,
- campaign A model/digest/qualification hash,
- campaign B ID,
- campaign B receipt SHA-256,
- campaign B model/digest/qualification hash,
- matched benchmark/environment/core/policy pins,
- configured trial count,
- full campaign-level source statistics,
- mean-score difference,
- Welch standard error,
- Welch degrees of freedom,
- 95% confidence interval,
- Hedges' g,
- success-rate difference.

Comparison receipts live under:

`campaign-comparisons/<gym-rom-sha256>/comparison-000001.json`

## Desktop

The desktop discovers persistent campaign receipts whenever the Gym session starts.

When at least two COMPLETE campaigns exist, it preselects the two newest complete
campaigns.

The operator can choose any A/B pair and press:

`COMPARE`

The UI sends only the campaign IDs.

Native PhiCade reloads the receipts from disk, walks the trial hashes, re-runs the
compatibility gate, computes statistics, and writes the comparison receipt.

## CI

CI proves Comparison Lab through:

- shared Welch/effect-size math tests,
- same-environment/different-model compatibility test,
- same-campaign refusal,
- policy-drift refusal,
- PARTIAL refusal,
- scoring-error refusal,
- mutated-trial hash refusal,
- persistent receipt-ID allocation test,
- full native governance tests,
- existing SameBoy / Φ-Agent Gym qualification,
- web/type checks.

CI does not fabricate real model campaign comparisons. Real comparison receipts
are created from real local campaign evidence.
