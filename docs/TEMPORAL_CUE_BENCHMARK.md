# Temporal Cue Benchmark

Rung 21 adds a controlled gameplay probe for information that must survive after
it disappears from the rendered framebuffer.

The probe is deliberately narrow. It does not claim to measure general
intelligence or human-like memory. It tests whether a controller can carry
earlier visible evidence through PhiCade's governed working-memory channel and
use it at a later choice point.

## Balanced pair

Two independently assembled Game Boy ROMs form the probe:

- `temporal-cue-left-v1`
- `temporal-cue-right-v1`

They use the same controller prompt, the same player start, the same two-door
decision chamber, the same lockout, and the same allowed buttons.

Only the initial visible arrow cue and the correct endpoint differ.

That pairing matters. A single fixed task can be passed by a constant LEFT or
RIGHT habit. The balanced pair requires opposite answers after an otherwise
identical later observation.

## Sequence

At the frozen benchmark start:

1. the solid player block is at screen position (72, 96),
2. a large LEFT or RIGHT arrow is visible,
3. the model may observe the cue and propose governed memory,
4. A dismisses the cue,
5. a 90-frame lockout begins,
6. the cue is removed and two visually identical doors are shown,
7. after lockout the D-pad must be neutral for one frame,
8. only a later LEFT or RIGHT press can move the player,
9. the correct endpoint is the side named by the earlier cue.

The correct endpoint is never marked differently on the decision screen.

## Why the neutral-arm rule exists

Agent Driver Protocol v2 allows multiple bounded actions in one response.

Without an additional control, a model could see the cue and schedule a
directional input during the same response, carrying that input through the
visual transition. That would test action scheduling, not retained evidence.

The ROM therefore ignores directional movement during lockout and enters the
choice state unarmed. A neutral D-pad frame is required before a new direction
can move the player.

A direction held continuously from the cue turn remains refused after lockout.
The controller must later release to neutral and choose again.

Because PhiCade's Ollama adapter sends no provider chat transcript between turns,
the explicit governed memory capsule is the intended cross-turn state channel.

## Control scope

The temporal tasks receive exactly:

- A
- LEFT
- RIGHT

Existing benchmark tasks keep their original D-pad-only scope.

Task-level control scope is part of the runtime benchmark registry rather than a
global benchmark privilege.

## Rendered-pixel scoring

The harness still scores from the rendered framebuffer.

The player remains the unique solid 8x8 dark patch. Cue and door graphics are
sparser background tiles so they cannot win the player-locator search.

Both temporal variants begin 48 Manhattan pixels from their correct endpoint:

- LEFT endpoint: (24, 96)
- RIGHT endpoint: (120, 96)

A successful oracle performs:

- A for 1 frame
- WAIT for 101 frames
- the correct direction for 24 frames

The extra neutral wait exceeds the 90-frame ROM lockout and arms the later
choice cleanly.

## Pair qualification

In addition to each task's normal NO-INPUT / oracle / deterministic-replay
qualification, `temporal_cue_qualify` jointly checks both ROMs.

The pair qualifies only when:

- the initial cue frame hashes differ,
- the post-cue decision frame hashes are identical,
- both decision player positions are identical,
- carrying the correct direction from the cue turn through lockout causes zero
  movement,
- releasing to neutral and making a later choice reaches each variant's correct
  endpoint.

This makes the critical contrast explicit:

> different earlier evidence, identical later pixels, opposite correct action.

## Governed memory relationship

Rung 20 made memory an explicit, bounded, hash-bound runtime artifact.

Rung 21 supplies a benchmark where that channel can matter behaviorally.

The benchmark itself does not force a provider to write any particular text.
Memory remains model-authored notes, not authoritative game state. The runtime
continues to own authority, timing, memory acceptance, and evidence receipts.

## Interpretation

A successful campaign on both temporal variants supports a descriptive claim:
under the frozen model, provider, core, policy, task, and memory-budget
conditions, the controller retained enough earlier cue information to produce
the correct later side choice.

It does not by itself identify the internal mechanism used by the model and it
does not establish a general memory capacity outside this controlled task.
