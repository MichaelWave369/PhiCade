# Governed Agent Working Memory

Rung 20 adds an explicit, bounded working-memory channel to PhiCade's Agent
Driver Protocol.

## Principle

Memory is evidence carried forward by the controller. It is not authority.

The model may propose a replacement memory capsule. Native PhiCade owns whether
that proposal is accepted, just as it owns action authority and timing.

There is no hidden provider-side gameplay history in the contract.

## Agent Driver Protocol v2

Every `AgentTurnRequest` includes:

- the rendered framebuffer observation,
- `memory`, a UTF-8 capsule,
- `memorySha256`, the exact SHA-256 of those bytes,
- `maxMemoryBytes`,
- `maxMemoryUpdateBytes`.

Every `AgentTurnResponse` must echo the exact pending `memorySha256` and may
return `memoryUpdate`.

A stale response cannot be applied to a newer memory state.

## Replacement semantics

`memoryUpdate` replaces the capsule.

It does not append to an invisible transcript and it does not gain authority
because the model wrote it. A useful capsule might contain concise notes such as
a discovered route, a visible clue, or a remembered objective.

`null` means preserve the existing capsule.

## Native enforcement

The runtime validates:

1. turn identity,
2. observation frame + framebuffer SHA-256,
3. pending memory SHA-256,
4. turn expiry,
5. action count and delay budgets,
6. memory replacement byte budgets,
7. Autodrive action budget and active authority.

Only after those checks may native PhiCade replace the capsule.

The default autonomous memory policy is:

- total capsule: 4096 bytes,
- one-turn replacement proposal: 1024 bytes.

## Lifecycle

A new autonomous run begins with the empty UTF-8 string.

Its initial digest is therefore the SHA-256 of zero bytes. The capsule evolves
only through accepted turn responses.

When the run stops, PhiCade seals memory evidence into the Autodrive receipt and
clears the live autonomous capsule.

Changing control mode also clears live agent memory, preventing accidental
cross-controller or cross-run leakage.

## Autodrive receipt v3

The receipt records:

- initial memory SHA-256,
- final memory SHA-256,
- final memory content,
- final memory byte length,
- final memory revision,
- accepted update count,
- total replacement bytes written,
- memory refusal count,
- the exact Autodrive policy carrying both memory limits.

Benchmark campaigns already compare exact Autodrive policies, so changing memory
budgets creates a different comparison cohort rather than silently mixing
conditions.

## Historical policy migration

Older Autodrive receipts did not contain a policy version. Rung 18 later added
adaptive cadence fields.

Rung 20 removes whole-struct serde defaults from `AutodrivePolicy`. Missing
historical cadence fields now deserialize to legacy immediate cadence semantics:

- minimum observation interval: 1 frame,
- post-action settle: 0 frames,
- empty-turn backoff: 1 frame,
- maximum observation interval: 1 frame,
- governed memory: disabled.

Existing Rung 18/19 receipts retain their explicit cadence fields but deserialize
with historical `policyVersion = 0` and memory disabled.

New runs require `policyVersion = 1`.

This prevents historical evidence from retroactively inheriting current runtime
behavior.

## Provider behavior

The Ollama adapter receives the current capsule explicitly with the framebuffer.
Its structured response includes both `actions` and `memoryUpdate`.

The provider is instructed that memory is model-authored notes, not world truth
or permission. Native validation remains authoritative.

## Why this matters

Present-frame control is enough for reflex tasks. Games with clues, doors,
routes, NPC hints, delayed consequences, multi-room objectives, or other temporal
dependencies require controlled state carried across observations.

Rung 20 establishes that state channel without weakening the Action Bus,
authority boundary, replay evidence model, or benchmark comparability.

A later Temporal Cue benchmark can now test the distinction directly: show
information, remove it from the framebuffer, delay the decision, and verify
whether bounded retained evidence changes the result.
