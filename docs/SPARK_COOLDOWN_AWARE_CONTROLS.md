# PhiCade Rung 56 — Cooldown-Aware SPARK Control Menu

Rung 55 diagnosed the actual Rung 54 model traces. Across all twelve
five-turn experiments, models repeatedly proposed directional DASH on cooldown.

- Qwen selected `DASH_DOWN` on all five turns, even for the Pulse objective.
- Llama also repeatedly selected DASH buttons.
- All twelve trials had one successful dash and four `DASH_COOLDOWN`
  rejections.
- Neither model proposed `PULSE` in the Pulse trials.

The canonical SPARK engine already has an independent passing Pulse positive
control. This is primarily a **proposal availability** problem, not evidence
that the engine cannot use its power.

## Fix scope

This rung does not change the SPARK engine or PixelForge transport, grant
contents, memory, or authority policy.

Each semantic turn derives a **temporary proposal menu** by intersecting:

1. PhiCade's fixed `AgentGrant`;
2. SPARK's declared supported action kinds;
3. canonical semantic cooldown readiness;
4. the active/playing state of the controlled vessel.

Availability rules:

| Control family | Visible while |
| --- | --- |
| UP / DOWN / LEFT / RIGHT | `MOVE` advertised and vessel is alive/playing |
| DASH_UP / DASH_DOWN / DASH_LEFT / DASH_RIGHT | `DASH` advertised and `dashReady` |
| PULSE | `PULSE` advertised and `powerCooldown == 0` |

The derived menu is **always a subset of the existing grant**.
It cannot add new buttons or authorize actions.

If the observation is malformed, the player is dead/not playing, the
power cooldown is nonfinite/negative, or there are no usable controls,
the turn fails closed instead of inventing a control.

## Actual model-facing effects

- The Ollama structured-output `button` enum is restricted to the menu.
- The prompt explains that absent cooldown-bound abilities cannot currently
  be requested.
- `SparkAgentTurnRequest::validate` also independently rejects a request
  that tries to present an unavailable button.
- `SparkAgentTurnResponse` remains bound to the request's exact tick,
  runtime hash, agent ID, memory hash, and available buttons.
- The **existing AuthorityPolicy and SPARK engine** still make the actual
  admission and execution decisions.

The real local-model playtest adds `availableButtons` to each turn's
receipt so later evidence audits can see what the model was offered.

## No score inflation

This rung does **not** narrow choices according to the benchmark objective.
If the objective is Pulse, it does not force `PULSE` or remove valid movement
buttons. The model still must choose a useful action itself.

This remains a cooldown-aware proposal filter, not automated gameplay.

## Validation

- runtime unit tests for cooldown entry/exit and grant intersection;
- request validation rejects an unavailable DASH;
- Ollama schema test confirms unavailable DASH and PULSE buttons are absent;
- existing original SPARK semantic-provider and engine tests remain in place.

After this rung passes CI and merges, run the same hosted model/task matrix
again with pinned new PhiCade code and **fresh real model inference**. Compare
new results against Rung 54 as an A/B experiment, including objective success
and the rate of `DASH_COOLDOWN` rejections.

Do not automatically change BrainC routing from this change.
