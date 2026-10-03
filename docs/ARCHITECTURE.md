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
