# Rung 8 Local Ollama Provider

Rung 8 is PhiCade's first concrete model-provider adapter.

It connects the already-qualified Agent Driver Protocol to a local Ollama server.
The provider adapter does not receive emulator handles, session state, save RAM,
serialized core state, or authority objects.

Its entire job is:

```text
AgentTurnRequest
      |
      v
PNG framebuffer + bounded prompt
      |
      v
local Ollama /api/chat
      |
      v
structured button intents
      |
      v
AgentTurnResponse
```

Everything after AgentTurnResponse remains owned by PhiCade.

## Ollama API boundary

The adapter uses the local REST API:

- `GET /api/tags` to discover installed models
- `POST /api/chat` for one vision turn
- `stream: false`
- a JSON schema supplied through `format`
- a base64-encoded PNG supplied through the message `images` array

The desktop default endpoint is:

`http://127.0.0.1:11434`

Rung 8 intentionally accepts loopback HTTP only:

- `127.0.0.1`
- `localhost`
- `::1`

A LAN or internet hostname is rejected by the native adapter. Remote provider
transport should get its own authentication, TLS, and policy rung instead of
silently turning this local bridge into an SSRF-shaped hole.

## Vision conversion

The Phi-Bot observation stores the exact rendered RGBA framebuffer.

Ollama's REST vision input expects an encoded image, so the adapter:

1. base64-decodes the RGBA bytes,
2. verifies byte length equals width × height × 4,
3. encodes the frame as an 8-bit RGBA PNG,
4. base64-encodes that PNG for Ollama.

The original observation frame and SHA-256 remain inside the AgentTurnRequest.
The provider cannot replace them.

## Structured output

The Ollama JSON schema allows only:

- `actions` array
- `delayFrames` within the request budget
- `button` from the request's allowed-button enum
- `pressed` boolean

The adapter converts those fields into ordinary `AgentTurnAction` values and
builds a normal `phicade.agent-turn-response.v1`.

Rung 7 validates that response again, and Rung 6 authority filters the compiled
ActionEnvelope events before they can reach SameBoy.

Provider output therefore remains a proposal.

## THINK PAUSE

A local vision model may take far longer than one 60 Hz game frame to answer.

The Rung 8 desktop path deliberately pauses emulation while Ollama evaluates a
turn. This keeps the game's emulated frame fixed at the observation that was sent
to the model.

Flow:

1. HANDOFF seat 1 to Phi-Bot.
2. Press OLLAMA TURN.
3. Freeze emulation at the observed frame.
4. Issue AgentTurnRequest.
5. Await Ollama.
6. Submit the returned AgentTurnResponse.
7. Resume emulation with the native scheduled inbox.

This is a correctness-first provider mode. Future real-time agent modes may use
asynchronous inference, predictive input, or slower observation cadences.

## Failure behavior

If Ollama is unavailable, returns invalid JSON, violates the schema, or otherwise
fails:

- the pending turn is explicitly cancelled,
- no provider action is submitted,
- the emulator resumes,
- existing gameplay authority remains unchanged.

## Model selection

SCAN MODELS calls Ollama's local model-list endpoint.

PhiCade does not currently guess which installed models support vision. The
operator selects a model. If a text-only model cannot accept the framebuffer,
Ollama returns an error and no action reaches the game.

This keeps capability discovery honest rather than inferring vision support from a
model name.

## CI qualification

GitHub CI does **not** download or run a large language/vision model.

Instead, native tests launch a temporary loopback HTTP server that speaks the
relevant Ollama response shapes. The tests prove:

- non-loopback endpoints are rejected,
- RGBA observations become valid PNG bytes,
- request budgets appear in the structured-output schema,
- `/api/tags` model-list responses are parsed,
- `/api/chat` structured action content becomes a valid AgentTurnResponse.

This qualifies the adapter plumbing, not model intelligence or model-specific
behavior.
