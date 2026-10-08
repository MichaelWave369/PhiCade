# PhiCade Rung 53 — SPARK Task-Quality Evidence

Rung 53 adds **explicit task objectives** and deterministic task scoring to the
governed SPARK model path.

This is the rung that starts separating:

```text
model is allowed to route
```

from:

```text
model is good at this task
```

Rung 52 handles the first question. Rung 53 begins answering the second.

## Bounded objectives

`phicade.spark-agent-turn-request.v1` now carries an optional bounded
`objective`.

The objective is:

- explicit;
- limited to 256 bytes;
- visible to the model;
- not authority;
- unable to widen the AgentGrant.

The prompt tells the model that the objective is a task instruction only and
that PhiCade still owns authority.

## Playtest evidence v2

Objective-bearing playtests now emit:

`phicade.spark-ollama-playtest.v2`

The v2 receipt adds:

- optional objective;
- initial semantic observation;
- final semantic observation.

That makes quality scoring possible from evidence instead of from inference over
logs.

## Frozen microtasks

Rung 53 defines three deliberately small microtasks.

### move-east

Objective:

```text
Move east as far as you can using only the granted gameplay controls.
```

Evidence metric:

`playerX`

Success means final X is greater than initial X. The raw X delta is retained as
progress. No arbitrary "good enough" threshold is invented.

### use-dash

Objective:

```text
Use a dash successfully at least once using only the granted gameplay controls.
```

Evidence metric:

`dashCount`

Success means the canonical SPARK dash counter increased.

### use-pulse

Objective:

```text
Use Lumen Pulse successfully at least once using only the granted gameplay controls.
```

Evidence metric:

`powerUseCount`

Success means the canonical SPARK power-use counter increased.

## Why these tasks are intentionally small

The goal is not to pretend three toy objectives measure general game intelligence.

The goal is to create the first routing-quality evidence where:

- the task is explicit;
- the source state is bounded;
- the action grant is identical;
- success is scored from canonical SPARK state;
- latency and token evaluation counts can be retained beside task success.

That gives future routing logic evidence it can defend.

## Scoring contract

Task evidence uses:

`phicade.spark-task-quality.v1`

and scored playtests use:

`phicade.spark-task-scored-playtest.v1`.

Example:

```bash
cargo run -p phicade-runtime --example score_spark_playtest -- \
  --input playtest.json \
  --task move-east \
  --out scored.json
```

The scorer refuses:

- non-PASS playtest receipts;
- unsupported playtest schemas;
- an objective that does not exactly match the frozen task objective;
- scoring across different rooms.

## Current routing boundary

Only Rung 52 `ELIGIBLE` models should enter Rung 53 comparisons.

Current isolated evidence admits:

```text
qwen2.5:0.5b-instruct
llama3.2:1b
```

and keeps:

```text
gemma3:1b
```

quarantined for identity drift.

## Next step

Run the admitted models through the same frozen microtask suite on hosted isolated
workers, repeat each task, and aggregate:

- task success rate;
- raw task progress;
- provider latency;
- evaluated-token count;
- identity stability.

Automatic routing should still remain disabled until enough repeated evidence
exists to justify a preference.
