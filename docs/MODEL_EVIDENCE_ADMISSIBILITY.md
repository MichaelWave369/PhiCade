# PhiCade Rung 52 — Model Evidence Admissibility

Rung 52 turns governed model-trial evidence into a routing **admission decision**.

It does not pick a winner.

That distinction matters.

A model may be:

- eligible for routing;
- quarantined because its identity drifted;
- quarantined because identity could not be resolved;
- excluded because its identity was stable but the governed playtest failed;
- rejected because the evidence itself is contradictory.

## Input classification

The current source classification is the Rung 51 isolated model matrix:

`spark.isolated-model-identity-matrix.v1`

with per-model states:

```text
PASS_STABLE
IDENTITY_DRIFT
IDENTITY_UNRESOLVED
PLAYTEST_FAIL_STABLE_IDENTITY
```

## PhiCade admission result

Each model is converted into:

`phicade.model-admission.v1`

with one disposition:

```text
ELIGIBLE
QUARANTINED_IDENTITY_DRIFT
QUARANTINED_IDENTITY_UNRESOLVED
EXCLUDED_BEHAVIOR_FAILURE
INVALID_EVIDENCE
```

## Fail-closed rules

`PASS_STABLE` is not accepted merely because a label says so.

PhiCade additionally requires:

- stable before/after model digest;
- governed playtest PASS;
- at least one executed action;
- SPARK runtime hash change.

Contradictory evidence becomes `INVALID_EVIDENCE`.

Identity drift is never silently converted into a gameplay failure.

## Current Rung 51 interpretation

The evidence currently observed under isolated hosted trials is:

```text
qwen2.5:0.5b-instruct -> ELIGIBLE
llama3.2:1b           -> ELIGIBLE
gemma3:1b             -> QUARANTINED_IDENTITY_DRIFT
```

This is an admissibility result only.

It is **not** evidence that Qwen is universally better than Llama, or that Gemma
is poor at gameplay.

## Transformer

PhiCade includes a small evidence transformer:

```bash
cargo run -p phicade-runtime --example qualify_model_admission -- \
  --input rung51-model-identity-matrix.json \
  --out model-admissions.json
```

The transformer consumes the exact Rung 51 matrix shape and emits a machine-readable
admission set.

That makes the output suitable for a later routing layer such as BrainC without
forcing BrainC to reinterpret raw CI logs.

## Next step

The next routing rung should compare only **eligible** models across repeated
governed trials and add task-quality evidence before any automatic model selection
policy is allowed to choose between them.
