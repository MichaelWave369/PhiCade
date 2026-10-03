# Rung 18 Adaptive Observation Cadence

Rung 18 makes model observation timing a native PhiCade governance rule.

Before this rung, Autodrive already refused to issue a new turn while a previous
Agent Driver action queue was still active. Once that queue drained, however, the
desktop could request the next observation immediately.

That is safe, but it is not always useful.

A game may need a few frames to visually settle after a delayed input sequence, and
a model that repeatedly returns no actions should not be shown nearly identical
frames as fast as the host can ask.

Rung 18 therefore adds a bounded adaptive cadence between autonomous observations.

## Default cadence policy

The default Autodrive policy now freezes:

- minimum observation interval: 2 frames
- post-action settle: 2 frames
- first empty-turn backoff: 8 frames
- maximum observation interval: 60 frames

The existing execution limits remain:

- max turns: 32
- max total actions: 128
- max consecutive empty turns: 4
- max emulated frames: 3,600

All cadence values are part of the same native `AutodrivePolicy` that is already
bound into gameplay, campaign, Suite Report, and Suite Comparison evidence.

## Action-aware settle

For a turn that returns one or more actions, PhiCade computes:

`max accepted delayFrames + postActionSettleFrames`

The result is clamped between the minimum and maximum observation intervals.

Example with the default policy:

- press at delay 0
- release at delay 5
- post-action settle 2

The next observation is not eligible until at least 7 emulated frames after the
response was accepted.

The Agent Driver queue still has to be empty as well.

## Empty-turn backoff

An empty model response is evidence that another nearly identical observation may
not be useful immediately.

Consecutive empty turns therefore use exponential spacing:

- empty turn 1: 8 frames
- empty turn 2: 16 frames
- empty turn 3: 32 frames
- empty turn 4: 60 frames
- later turns: still capped at 60 frames

The existing `maxConsecutiveEmptyTurns` stop rule remains authoritative. Cadence
does not weaken that bound.

A non-empty turn resets the empty-turn streak.

## Native authority

The desktop is not the cadence authority.

Native `issue_agent_turn` checks the current emulated frame against
`nextObservationFrame`.

An early autonomous request is refused before a framebuffer observation is created.

The desktop uses the same status field to avoid asking early, but that is only a
convenience. Native enforcement remains the security and evidence boundary.

## THINK PAUSE remains unchanged

Cadence applies between autonomous turns.

Once an observation is issued and the model begins inference, the existing THINK
PAUSE behavior still freezes emulation so the observed framebuffer cannot age while
the model is deciding.

The sequence is therefore:

```text
previous response accepted
        |
        v
scheduled Agent Driver actions
        |
        v
queue drains
        |
        v
native cadence / settle window
        |
        v
nextObservationFrame eligible
        |
        v
framebuffer observation
        |
        v
THINK PAUSE
        |
        v
model response
```

## Status evidence

`phicade.autodrive-status.v2` adds:

- `nextObservationFrame`
- `lastObservationFrame`
- `totalScheduledCadenceWaitFrames`
- `maxScheduledCadenceWaitFrames`

The desktop displays:

- next eligible observation frame
- READY / remaining wait
- accumulated cadence wait
- maximum single cadence wait
- consecutive empty-turn streak

## Receipt evidence

`phicade.autodrive-receipt.v2` records:

- total scheduled cadence wait frames
- maximum scheduled cadence wait frames
- last observation frame
- complete cadence policy

Because Autodrive policy is already pinned into model benchmark campaigns, Suite
Reports, and Suite Comparisons, cadence differences cannot silently disappear from
model-to-model evidence.

## Qualification

The SameBoy Autodrive qualification is upgraded to:

`phicade.autodrive-qualification.v2`

In addition to the existing action/turn/frame/empty-stop probes, CI proves:

- delayed actions include the post-action settle interval
- consecutive empty turns back off 8 → 16 → 32 → 60
- the maximum interval cap is enforced
- a turn is not observation-eligible before its scheduled frame
- it becomes eligible exactly at the scheduled frame

## Principle

**Do not confuse faster polling with better agency. Let actions land, let the world
move, back off when the model has nothing to do, and make the runtime prove when the
next observation is allowed.**
