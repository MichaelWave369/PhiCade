# Rung 10 Model Qualification Registry

Rung 10 separates **provider availability** from **model qualification**.

An installed Ollama model is not assumed to be safe or suitable for autonomous
gameplay merely because its name sounds multimodal.

PhiCade now requires evidence for the exact installed model digest before AUTO
DRIVE can start.

## Discovery

The selected model is inspected through Ollama's model-details endpoint.

PhiCade records:

- model name
- exact installed digest
- advertised capabilities
- family
- parameter size
- quantization level

A model that does not advertise the `vision` capability cannot pass Rung 10
qualification.

## Synthetic vision probe

Capability metadata is necessary but not sufficient.

PhiCade generates a 64×64 solid-red RGBA image locally and asks the selected model
to identify the dominant color using a strict structured-output schema whose only
valid values are:

- red
- blue

The probe uses deterministic inference settings where supported by the API
request.

A PASS requires:

1. exact model digest discovered,
2. advertised `vision` capability,
3. valid structured JSON response,
4. observed dominant color = `red`.

The probe contains no game ROM data and no emulator state.

## Qualification receipt

Schema:

`phicade.ollama-model-qualification.v1`

A receipt records:

- provider
- model
- exact digest
- advertised capabilities
- vision-advertised result
- structured-output result
- vision-probe result
- expected/observed probe value
- provider duration/eval metadata when available
- error text on failure

Receipts are persisted under the application data directory:

```text
model-qualifications/ollama/<digest>.json
```

Failed probes may also be persisted. Only a PASS receipt unlocks autonomous mode.

## Digest invalidation

Qualification is tied to the exact installed model digest.

If a model is re-pulled, rebuilt, replaced, or otherwise changes digest, the old
receipt does not qualify the new artifact.

The desktop re-checks the selected model's current digest before AUTO DRIVE.

The native AUTO gate then loads the persisted receipt for that digest and requires:

- provider = ollama
- model name match
- digest match
- result = PASS
- vision advertised
- structured output PASS
- visual probe PASS

A stale receipt therefore cannot silently authorize a changed model.

## One-shot vs autonomous use

Rung 10 deliberately keeps two different trust levels.

### OLLAMA TURN

One-shot turns remain available for experimentation with an unqualified model.

All existing Agent Driver and AuthorityPolicy checks still apply.

### AUTO DRIVE

Autonomous multi-turn execution requires a current qualification PASS.

This distinction lets users test new models without pretending that installation
equals qualification.

## CI coverage

GitHub CI does not run a large local model.

The mock Ollama tests now prove:

- model list parsing
- exact digest lookup
- `/api/show` capability parsing
- advertised vision detection
- synthetic red-frame qualification PASS
- non-vision model qualification FAIL
- structured-output parsing
- native exact-digest gate PASS
- native changed-digest gate FAIL

This qualifies the registry and gate machinery, not the gameplay ability of any
specific real-world model.
