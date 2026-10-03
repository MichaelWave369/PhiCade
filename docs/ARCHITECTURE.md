# PhiCade Architecture

## Goal

PhiCade is a desktop emulator front end and game runtime. The runtime is designed so
a keyboard, gamepad, replay file, script, network peer, or future Phi-Bot can all
enter through one governed input seam instead of each subsystem inventing its own
back door.

## Rung 1 boundaries

```text
Human / Replay / Script / Phi-Bot
              |
              v
        +-------------+
        | Action Bus  |
        +-------------+
              |
              v
       Authority layer
              |
              v
       Core Adapter API
              |
      +-------+-------+
      |               |
   libretro       native core
   adapter         adapter
      |               |
      +-------+-------+
              |
        video / audio
```

The Action Bus is implemented now. Authority policy and real core adapters are
deliberately separate rungs.

## Core rules

1. Core adapters receive resolved frame inputs. They do not read controllers or
   agent messages directly.
2. Game images and firmware remain user-supplied local resources.
3. Each integrated core must have a recorded source, version, license, and binary
   hash before it is enabled in a release build.
4. Save states are namespaced by core identity + core version + game fingerprint.
5. Replays record normalized Action Bus events, not host keyboard scan codes.
6. Phi-Bot never gains extra emulator privileges merely because it is an AI seat.

## Determinism / Replay Ledger

Rung 5 now implements the first deterministic replay contract.

A replay binds:

- core identity/version and binary SHA-256
- game image SHA-256
- initial serialized core state
- initial frontend input mask
- exact applied-frame normalized actions
- periodic serialized-state SHA-256
- periodic RGBA framebuffer SHA-256
- periodic frontend input-mask state
- final state/frame hashes

The replay file is content-addressed by SHA-256 and paired with a verification
receipt. Verification restores the replay start, reruns the action stream, compares
checkpoints, reports the first observed divergent checkpoint, and restores the live
session afterward.

Firmware hashes remain a required extension when PhiCade qualifies cores that use
external firmware. SameBoy's current qualified desktop configuration does not
require proprietary BIOS distribution.

This makes claims such as "a human/bot produced this run" testable rather than
folklore.


## Phi-Bot authority

Rung 6 makes the controller boundary explicit:

```text
Framebuffer observation
        |
        v
Agent policy / human controller
        |
        v
Action Bus
        |
        v
AuthorityPolicy
        |
        +--> reject + receipt telemetry
        |
        v
Session machinery
        |
        v
Core adapter
```

The native host owns authority enforcement. UI controls do not decide whether an
action is permitted; they merely enqueue ActionEnvelope events.

A Phi-Bot source is identified by both `agentId` and `seat`. Older serialized
Phi-Bot ActionSource objects without a seat deserialize as seat 1 for replay
compatibility.

The observation boundary exposes rendered pixels and public session/control
metadata only. Serialized core state, save RAM, ROM bytes, and emulator memory are
not part of the agent observation contract.

SameBoy currently has one playable input port. The shared policy supports a
two-seat versus topology, but the SameBoy host refuses VERSUS rather than aliasing
seat 2 onto player 1.


## Agent Driver Protocol

Rung 7 separates provider/model execution from game-runtime authority.

```text
PhiBotObservation
       |
       v
AgentTurnRequest
       |
       v
provider / local model / script
       |
       v
AgentTurnResponse
       |
       v
native validation + scheduled inbox
       |
       v
canonical live sequencing
       |
       v
AuthorityPolicy
       |
       v
core
```

The driver protocol is transport-neutral. A provider does not receive a core
handle and does not publish ActionEnvelope values directly. It returns bounded
action intents tied to one observation hash and turn ID.

The native runtime owns:

- turn IDs
- turn expiry
- response validation
- scheduling
- ActionSource identity
- canonical action sequence numbers
- authority filtering

Browser/UI action sequence numbers are not treated as global truth. Live actions
are re-sequenced after authority acceptance at the native boundary.

Same-frame live-source ordering is exported from `phicade-runtime`:

`Replay < Script < Phi-Bot < Human`

Human input therefore has final application precedence on a same-frame CO-OP
conflict.

Control-mode changes and grant expiry neutralize the frontend input mask and clear
pending driver work. Grant expiry falls back to HUMAN authority.


## Local provider adapters

Rung 8 adds the first concrete provider behind Agent Driver Protocol: Ollama on
loopback HTTP.

The provider layer is deliberately outside the authority/core seam:

```text
AgentTurnRequest
      |
      +--> local Ollama adapter
      |        |
      |        +--> PNG vision payload
      |        +--> constrained JSON decision
      |
      v
AgentTurnResponse
      |
      v
Rung 7 validation/scheduling
      |
      v
Rung 6 authority
      |
      v
core
```

The native Ollama adapter accepts only loopback HTTP endpoints. It cannot use an
arbitrary LAN/internet URL.

The desktop uses THINK PAUSE for model turns: emulation stops advancing while the
vision request is in flight, preserving the exact observation frame. A provider
failure cancels the pending turn before emulation resumes.

Provider adapters are not trusted authorities. They translate an observation-bound
request into a proposed AgentTurnResponse. Existing turn validation and gameplay
authority remain downstream and unchanged.


## Governed Autodrive

Rung 9 composes the existing provider, driver, authority, and core seams into a
bounded repeated-turn runtime.

```text
AutodrivePolicy
      |
      v
AgentTurnRequest
      |
      v
provider THINK PAUSE
      |
      v
AgentTurnResponse
      |
      v
native driver inbox
      |
      v
AuthorityPolicy
      |
      v
core
      |
      +--> queue drained?
              |
              +--> yes: next bounded turn
```

The UI may orchestrate when to ask for the next provider turn, but it does not own
the run budget. Turn, action, empty-turn, and emulated-frame limits are evaluated
by shared/native runtime state.

Every autonomous stop path clears pending driver requests, queued agent actions,
and the frontend input mask. The run writes a stop-reason receipt before returning
to an idle state.

HUMAN takeover remains outside the model-action path and stays available during
provider inference. The UI cancels THINK PAUSE immediately; any late provider
response then fails because the pending turn/grant has already been revoked.

Autodrive is currently PHI-BOT handoff only. CO-OP autonomous inference is
deliberately deferred because THINK PAUSE would freeze the human partner while the
model reasons.


## Model Qualification Registry

Rung 10 adds an evidence boundary between an installed provider model and
autonomous runtime authority.

```text
/api/tags -> exact digest
/api/show -> advertised capabilities
synthetic vision probe -> measured vision + structured output
        |
        v
digest-bound qualification receipt
        |
        v
native AUTO DRIVE gate
```

The registry does not infer vision support from model names.

The desktop may use an unqualified model for a one-shot OLLAMA TURN, but native
Autodrive requires a PASS receipt for the exact current digest supplied at run
start.

Qualification receipts are local evidence artifacts and live under the PhiCade
application-data namespace.

A changed model digest invalidates prior qualification without mutating or
deleting the historical receipt.


## Φ-Agent Gym

Rung 11 adds a benchmark environment above the qualified core/runtime stack.

```text
RGBDS source
    |
    v
assembled Game Boy ROM
    |
    v
SameBoy
    |
    v
rendered RGBA framebuffer
    |
    +--> model observation
    |
    +--> pixel-grounded scorer
```

The scorer does not read emulator RAM.

The current task uses a solid 8×8 player sprite and a visible X target. The
benchmark harness locates the player in the rendered framebuffer and scores
normalized Manhattan-distance progress.

Before a real model is compared on the gym, CI self-qualifies the environment
with:

- NO-INPUT negative control
- deterministic oracle positive control
- exact final-frame replay check

The qualification receipt binds source, assembled ROM, and SameBoy hashes so
future model-specific benchmark receipts can reference a frozen environment.


## Model Gameplay Benchmark

Rung 12 binds the Rung 10 model identity and Rung 11 frozen environment into one
scored execution receipt.

```text
qualified Ollama digest
        |
        +--> qualification receipt SHA-256
        |
        v
exact Phi-Agent Gym ROM
        |
        v
native reset + frozen warmup
        |
        v
D-pad-only governed Autodrive
        |
        v
rendered framebuffer
        |
        +--> shared pixel scorer
        |
        v
model gameplay benchmark receipt
```

Benchmark start is a native operation, not a UI convention. Native PhiCade
requires the exact frozen gym ROM SHA-256, exact qualified model digest, PHI-BOT
handoff, 1× speed, idle driver state, and valid Autodrive policy.

The same `score_agent_gym_frame` implementation is used by CI qualification and
live benchmark finalization.

The benchmark grant is reduced to UP/DOWN/LEFT/RIGHT.

The model receives the visible task objective but no coordinates, memory, state,
or scoring internals.

When the pixel scorer observes task success, native Autodrive stops immediately
with `task-success`. Every other governed Autodrive stop also finalizes a model
benchmark receipt.

A model benchmark receipt hashes both the model qualification receipt and the
underlying Autodrive receipt, binding capability evidence to gameplay execution
without treating either artifact as self-authenticating folklore.


## Benchmark Campaigns

Rung 13 composes repeated Rung 12 benchmark receipts into a pinned campaign.

```text
qualified model digest
        |
        v
campaign pins
(model + qualification hash + core hash + policy)
        |
        v
trial 1 -> gameplay receipt -> SHA-256
        |
        +--> live Ollama digest re-check
        |
        v
trial 2 -> gameplay receipt -> SHA-256
        |
       ...
        |
        v
shared campaign statistics
        |
        v
campaign summary receipt
```

The campaign summary never substitutes for trial evidence. It stores the SHA-256
of each individual Rung 12 receipt plus only the minimal trial outcome fields needed
for aggregate inspection.

Before every continuation trial, native PhiCade re-queries Ollama and requires the
currently installed model digest to match the campaign pin. It also re-hashes the
model qualification receipt and SameBoy binary.

The shared runtime computes campaign statistics. Score statistics use only
scoreable trials. Scoring errors remain explicit and are not coerced to zero.
Success rate uses all observed trials.

A campaign is COMPLETE only when all configured trials have been sealed. Operator
termination or core shutdown writes a PARTIAL summary so completed evidence is not
discarded.


## Comparison Lab

Rung 14 compares two completed campaign receipts only after native PhiCade proves
their evidence is still intact and their environments are compatible.

```text
campaign A summary ----> re-hash every trial receipt
        |                         |
        |                         v
        |                  provenance PASS
        |
        +---- compatibility gate ----+
                                     |
campaign B summary ----> re-hash every trial receipt
                                     |
                                     v
                         shared comparison statistics
                                     |
                                     v
                         comparison receipt
```

Compatibility is intentionally stricter than simple schema equality. Both campaigns
must be COMPLETE, fully scoreable, have the same configured trial count, and match
on benchmark ID, provider, Gym hashes, SameBoy binary/identity, and Autodrive
policy.

Model identity is allowed to differ because it is the subject of comparison.

The shared runtime reports signed A-minus-B statistics:

- mean-score difference,
- Welch unequal-variance standard error,
- Welch-Satterthwaite degrees of freedom,
- conservative two-sided 95% Student-t interval,
- Hedges' g small-sample standardized effect size,
- success-rate difference.

The comparison receipt hashes both source campaign receipts and retains their
model/digest/qualification identities.

Rung 14 also seeds evidence IDs from existing receipt filenames when a session
starts. New sessions therefore append rather than overwrite Autodrive, benchmark,
campaign, and comparison evidence.

The UI chooses campaign IDs only. Native PhiCade reloads and verifies the evidence;
React never owns compatibility or statistical authority.
