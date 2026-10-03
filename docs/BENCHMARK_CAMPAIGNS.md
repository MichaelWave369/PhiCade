# Rung 13 Benchmark Campaigns

Rung 13 turns a single Rung 12 gameplay benchmark into a repeated-trial evidence
campaign.

One run can show what happened once.

A campaign asks a stronger question:

**How consistently does this exact qualified model artifact perform when the world,
core, policy, and task are held fixed?**

## Trial count

Native PhiCade accepts campaigns from 3 through 20 trials.

The desktop default is:

`5 trials`

Each trial is still a complete Rung 12 model gameplay benchmark with its own
receipt.

The campaign summary does not replace those receipts.

## Frozen campaign pins

At campaign start, native PhiCade pins:

- provider,
- model name,
- exact model digest,
- SHA-256 of the exact Rung 10 model qualification receipt,
- frozen Φ-Agent Gym source SHA-256,
- frozen Φ-Agent Gym ROM SHA-256,
- SameBoy binary SHA-256,
- exact Autodrive policy,
- configured trial count.

A trial cannot be absorbed into a campaign unless its receipt matches the pinned
model digest, qualification receipt hash, core hash, gym hashes, and policy.

## Live digest re-check

Before the first trial and before every continuation trial, native PhiCade queries
Ollama again.

It requires the currently installed model digest to still equal the campaign pin.

If the model is pulled, replaced, or updated mid-campaign, continuation is refused.

The campaign also re-hashes the local model qualification receipt and the SameBoy
binary before continuation.

## Trial lifecycle

Every campaign trial reuses the exact Rung 12 benchmark launch path:

1. exact gym hash gate,
2. model qualification gate,
3. SameBoy reset,
4. 120-frame warmup,
5. pixel-grounded frozen start verification,
6. D-pad-only authority,
7. governed Autodrive,
8. pixel scoring,
9. individual gameplay receipt.

The desktop can automatically continue from trial N to trial N+1, but native
PhiCade owns all campaign pins and rejects drift.

## Campaign receipt

Schema:

`phicade.benchmark-campaign.v1`

A campaign receipt contains:

- campaign ID,
- benchmark ID,
- record status,
- provider/model/exact digest,
- model qualification receipt SHA-256,
- gym source/ROM SHA-256,
- SameBoy binary SHA-256 and identity,
- exact Autodrive policy,
- configured total trials,
- completed trials,
- one evidence entry per completed trial,
- aggregate statistics.

Each trial evidence entry contains:

- benchmark run ID,
- SHA-256 of the immutable Rung 12 gameplay receipt,
- trial record status,
- score when scoreable,
- task-success boolean,
- Autodrive stop reason.

The campaign receipt therefore points back to the full underlying trial evidence
rather than flattening it into averages.

## Statistics

The shared `phicade-runtime` campaign scorer computes:

- observed trial count,
- scored trial count,
- scoring-error trial count,
- successful trial count,
- success rate,
- mean score,
- median score,
- minimum score,
- maximum score,
- population standard deviation.

### Score statistics

Mean, median, min, max, and population standard deviation use only trials with a
valid numeric score.

A `SCORING_ERROR` trial is never silently converted into zero.

### Success rate

Success rate uses all observed trials:

`successful trials / observed trials`

A scoring-error trial therefore does not count as a success.

## Complete versus partial

`recordStatus = COMPLETE` means all configured trials were sealed.

`recordStatus = PARTIAL` means the operator, core shutdown, or another campaign
termination ended the campaign before all configured trials were completed.

A partial receipt still includes every completed trial and valid statistic that can
be computed from those observations.

## Operator stop

`END CAMPAIGN` remains available while the model is thinking.

If a trial is active, native PhiCade first seals it through the ordinary governed
Autodrive stop path.

Then it writes a PARTIAL campaign summary if trials remain.

Eject/core shutdown follows the same evidence-preserving rule.

## Storage

Campaign receipts live under the local application-data namespace:

`benchmark-campaigns/<gym-rom-sha256>/campaign-000001.json`

Individual trial receipts remain under the Rung 12 model benchmark namespace.

## CI

CI does not fabricate a campaign for a real local model.

CI proves the campaign machinery through:

- shared deterministic statistics tests,
- trial-count bound tests,
- campaign receipt serialization tests,
- full native governance tests,
- existing frozen SameBoy / Φ-Agent Gym qualification,
- existing web/type checks.

A real campaign summary exists only after a real exact-digest model runs the
configured trials locally.
