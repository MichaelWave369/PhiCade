# Rung 58 — SPARK Semantic Action Affordances

## Why this rung exists

The merged [SPARK Rung 57 hosted A/B trial](https://github.com/MichaelWave369/SparkTheSubstrate/pull/11) compared two admitted real Ollama models under identical fixed game source, five-turn budgets, and three frozen objectives.

It found:

| Model | Task | Before | After |
| --- | --- | ---: | ---: |
| Qwen 0.5B | move-east | 0/2 | 0/2 |
| Qwen 0.5B | use-dash | 2/2 | 2/2 |
| Qwen 0.5B | use-pulse | 0/2 | 0/2 |
| Llama 1B | move-east | 2/2 | 2/2 |
| Llama 1B | use-dash | 2/2 | 2/2 |
| Llama 1B | use-pulse | 0/2 | 0/2 |

All 48 previous `DASH_COOLDOWN` rejections dropped to zero.
However, neither model proposed `PULSE` on its new Pulse-objective trials.

The next hypothesis is **missing action semantics**: the original provider said
`PULSE` was allowed and described the vessel power as `Lumen pulse`, but it
did not explicitly say **what pressing the PULSE control does**.

This hypothesis is **not yet confirmed**. Other explanations, including
structured-output biases and the limitations of small local models, remain possible.

## Change: an observation-grounded control dictionary

This rung adds `spark_action_affordance_guide()` to the **SPARK-only** local
Ollama semantic provider. It explains every control that is *currently offered*:

| Offered control | Meaning |
| --- | --- |
| UP / DOWN | Move north / south |
| LEFT / RIGHT | Move west / east |
| DASH_UP / DASH_DOWN / DASH_LEFT / DASH_RIGHT | Short burst in the named direction, consuming dash readiness |
| PULSE | Activate the currently equipped vessel power, named from the actual semantic observation |

For example, when ready, the guide describes:

```text
RIGHT: move east (increase X)
DASH_RIGHT: burst east; consumes dash readiness
PULSE: activate the currently equipped vessel power (Lumen pulse)
```

If DASH is cooling down, the guide does not mention unavailable DASH
controls. If the vessel power is cooling down, it does not offer `PULSE`.

The dictionary does not invent hidden state or expose the saved run.
The guide is deliberately **objective-neutral**: changing the objective does
not change the set or definitions of available buttons.

## Governance remains unchanged

The existing fixed `AgentGrant` and PhiCade `AuthorityPolicy` remain the
final action gate. This is explanation, not a new grant.

The following stay as they were:

- `phicade.spark-agent-turn-request.v1` and the response schema;
- one-action maximum;
- exact runtime hash, observation tick and memory-hash binding;
- cooldown-filtered per-turn available-button menu;
- PixelForge JSONL submission;
- SPARK's canonical engine, powers and cooldowns;
- evidence receipt schema `phicade.spark-ollama-playtest.v2`.

The original framebuffer Ollama prompt remains **unchanged**.

## Qualification

Tests assert:

1. the guide matches the actual supplied button controls and names the
   current power from the bounded observation;
2. unavailable DASH and PULSE are not advertised;
3. changed task objectives do not change the action menu, button meanings,
   or structured response enum;
4. the new guidance is restricted to the semantic SPARK prompt;
5. legacy provider behavior and existing governance tests still compile
   and pass.

## Next experiment

After green CI and merge, run the same admitted models (Qwen and Llama)
through the three frozen SPARK microtasks with this new semantic guide.

Record both task success and **the actual frequency of PULSE proposals**,
rather than claiming improved gameplay from prompt wording alone.

A model that is able to select a documented control under a simple objective
has demonstrated a bounded control-vocabulary response, not general planning.

No automatic BrainC model routing changes are authorized by this rung.
