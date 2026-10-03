# Rung 7 Agent Driver Protocol

Rung 7 separates **game authority** from **model/provider integration**.

PhiCade does not let an Ollama model, cloud model, script, or future network agent
write directly into the emulator. Instead, the native host issues a bounded turn
request and accepts a bounded turn response.

```text
Rendered frame
    |
    v
AgentTurnRequest
    |
    v
provider / local model / script / bot
    |
    v
AgentTurnResponse
    |
    v
native validation + scheduling
    |
    v
Phi-Bot ActionEnvelope
    |
    v
AuthorityPolicy
    |
    v
core
```

## Schemas

Request schema:

`phicade.agent-turn-request.v1`

Response schema:

`phicade.agent-turn-response.v1`

## AgentTurnRequest

A request binds the driver to one observation and one action budget.

Fields include:

- turnId
- full `PhiBotObservation`
- maxActions
- maxDelayFrames
- validUntilFrame

The observation already binds:

- framebuffer bytes + SHA-256
- emulated frame
- ROM SHA-256
- core identity
- agentId
- seat
- active grant scope

## AgentTurnResponse

A response must echo:

- turnId
- agentId
- seat
- observationFrame
- observationSha256

It then proposes zero or more actions:

```json
{
  "delayFrames": 2,
  "action": {
    "kind": "button",
    "button": "A",
    "pressed": false
  }
}
```

`delayFrames` is relative to the native host frame at which the response is
accepted, not relative to wall-clock time.

## Validation

A response is rejected when:

- the schema is unknown
- turnId does not match the pending turn
- agentId or seat does not match the observation/grant
- observation frame does not match
- observation SHA-256 does not match
- the response arrives after `validUntilFrame`
- action count exceeds `maxActions`
- any delayed action exceeds `maxDelayFrames`

A validated response still does **not** automatically gain gameplay authority.
Compiled actions continue through `AuthorityPolicy`, so an out-of-scope RESET or
expired grant is rejected later at the same authority seam as any other Phi-Bot
action.

## Native inbox

Validated intents are compiled to ordinary `ActionEnvelope` values with a
`PhiBot { agentId, seat }` source and placed in the native driver inbox.

The inbox releases actions only when their emulated frame becomes due.

Changing control mode clears:

- any pending turn request
- any queued driver actions

This makes HUMAN takeover immediate.

## Canonical action order

Rung 7 makes the native runtime the canonical sequencer for live actions.

Browser/UI sequence numbers are transport-local hints. Before accepted actions are
recorded or delivered to the core, the native host assigns canonical monotonic
sequence numbers.

When multiple live sources land on the same emulated frame, the frozen ordering is:

1. replay
2. script
3. Phi-Bot
4. human

Human input is therefore applied last on same-frame CO-OP conflicts.

This rule is exported from `phicade-runtime` so another host, including a future
Night Circuit adapter, can use the same deterministic ordering.

## Reference driver

The desktop **DRIVER TURN** button is not a model integration.

It is a deterministic reference driver used to exercise the real protocol:

1. request a turn,
2. inspect the bound observation hash,
3. choose one allowed button deterministically,
4. return a schema-valid press/release response,
5. let the native host schedule and authorize it.

A real local/cloud model can later replace only step 3. The game runtime does not
need to know which provider produced the response.

## Qualification

`phicade.agent-driver-qualification.v1` runs against the frozen SameBoy core.

It proves:

- a real framebuffer observation can be wrapped in an AgentTurnRequest
- a valid response compiles into scheduled Phi-Bot ActionEnvelope events
- the driver-compiled action burst produces the exact same final state/frame hashes
  as equivalent direct Phi-Bot envelopes
- stale responses are rejected
- wrong observation hashes are rejected
- over-budget responses are rejected
- wrong turn IDs are rejected

The qualification does not claim any particular AI model makes good decisions. It
proves the controller protocol is transport/provider neutral and behaviorally
equivalent to the already-qualified direct Phi-Bot action path.
