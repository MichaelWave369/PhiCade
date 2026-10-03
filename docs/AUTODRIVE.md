# Rung 9 Governed Autodrive

Rung 9 turns one-shot Agent Driver turns into bounded autonomous gameplay.

The model still does not own the runtime. PhiCade owns the run budget, timing,
turn issuance, action scheduling, authority, stop reasons, and receipt.

```text
AUTO DRIVE
    |
    v
native AutodrivePolicy
    |
    v
AgentTurnRequest
    |
    v
local Ollama vision model
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
SameBoy
    |
    +--> queue drains
    |
    +--> next bounded turn
```

## Default policy

The desktop starts with:

- max turns: 32
- max total actions: 128
- max consecutive empty turns: 4
- max emulated frames: 3,600
- minimum observation interval: 2 frames
- post-action settle: 2 frames
- empty-turn backoff: 8 frames
- maximum observation interval: 60 frames

The frame budget matches the default Phi-Bot grant lifetime.

Rung 18 makes observation cadence native and adaptive. After an action-bearing turn,
the next observation waits for the maximum accepted delay plus the post-action
settle window. Consecutive empty turns back off exponentially, capped by the maximum
observation interval. Native turn issuance refuses observations before the scheduled
frame.

All limits are validated natively. The UI cannot extend the run merely by
continuing to request turns.

## Run lifecycle

AUTO DRIVE requires:

- a running qualified game
- 1x speed
- PHI-BOT handoff mode
- idle Agent Driver state
- a selected local Ollama model
- no active replay recording

A run receives a monotonically increasing runId.

For each turn:

1. native PhiCade checks the run budget,
2. native PhiCade issues an AgentTurnRequest,
3. emulation enters THINK PAUSE,
4. Ollama returns one structured AgentTurnResponse,
5. native PhiCade validates the response,
6. native PhiCade charges the action budget,
7. actions enter the scheduled driver inbox,
8. emulation resumes,
9. the next turn is not issued until that inbox is empty,
10. native cadence must also mark the current frame observation-eligible.

This queue-aware barrier prevents a model from reasoning over a frame while its
previous delayed press/release sequence is only half executed.

## Stop reasons

Schema values are explicit:

- operator-stop
- human-takeover
- turn-budget
- action-budget
- frame-budget
- empty-turn-limit
- provider-failure
- grant-expired
- core-shutdown

A stop clears:

- pending AgentTurnRequest
- queued driver actions
- held frontend input mask

The active gameplay authority is preserved unless the stop itself is a human
takeover or grant expiry.

## Human takeover

HUMAN remains enabled during an autonomous run.

Selecting HUMAN:

1. ends the run with human-takeover,
2. persists the receipt,
3. clears pending/queued bot work,
4. neutralizes held bot input,
5. revokes the Phi-Bot grant,
6. returns seat 1 to the human.

No model response can override this path.

## System/timeline actions

During autonomous driving, live Action Bus system commands are refused.

That includes save-state, load-state, rewind, and reset. These commands would
otherwise mutate the run timeline outside the model-action budget.

Screenshots and battery-RAM persistence remain host observations/persistence and
do not become bot controls.

## Provider failure

If the Ollama request fails while AUTO DRIVE is active:

- the run stops with provider-failure,
- pending/queued driver work is cleared,
- no provider action is submitted,
- the receipt is persisted,
- emulation resumes in a neutral input state.

## Receipt

Schema:

`phicade.autodrive-receipt.v2`

Receipts are stored under the ROM fingerprint namespace:

```text
autodrive/<rom-sha256>/run-000001.json
```

A receipt records:

- runId
- provider
- model
- ROM SHA-256
- core name/version
- start/end emulated frame
- turns issued/completed
- total accepted driver actions
- total cadence wait frames
- maximum cadence wait frames
- last observation frame
- stop reason
- final framebuffer SHA-256
- complete run policy

The receipt describes bounded execution. It is not a claim that the selected
model played well.

## Qualification

`phicade.autodrive-qualification.v2` runs against the frozen SameBoy core.

The positive control executes three deterministic Agent Driver turns with two
actions each.

The qualification then requires the fourth turn to be refused by the turn
budget.

Additional controls prove:

- action-budget stop detection
- frame-budget stop detection
- consecutive-empty-turn stop detection
- delayed-action settle timing
- 8 → 16 → 32 → 60 empty-turn backoff
- hard maximum cadence cap
- observation refusal before the eligible frame

CI fails if the loop can run past those frozen policy boundaries.
