# Rung 6 Phi-Bot Seat

Rung 6 gives an agent a real gameplay seat without giving it emulator-internal
privileges.

The governing rule is simple:

```text
Observation -> Controller decision -> Action Bus -> Authority policy -> Core
```

The agent never receives a memory pointer, save-state blob, cartridge bytes,
battery RAM, or a private input API.

## Observation v1

Schema: `phicade.phibot-observation.v1`

A Phi-Bot observation contains:

- emulated frame number
- rendered framebuffer dimensions
- rendered RGBA pixels as base64
- framebuffer SHA-256
- current frontend input mask
- ROM SHA-256
- core name/version
- agentId and seat
- active control mode
- allowed buttons and axes
- grant expiry frame

That is deliberately less privileged than the native session machinery.

## Authority modes

### human

Seat 1 gameplay input is human-controlled. No Phi-Bot grant is active.

### phi-bot

Seat 1 gameplay input is assigned to one explicitly granted Phi-Bot identity.
Human gameplay buttons are rejected, while human operator system controls remain
available so the operator can reset/save/take over.

### coop

Human and Phi-Bot gameplay actions are both admitted on seat 1 through the same
Action Bus.

### versus

The shared runtime models human seat 1 versus Phi-Bot seat 2. SameBoy currently
exposes one playable input port, so the native PhiCade host refuses this mode for
the qualified Game Boy core.

This refusal is intentional acceptance behavior, not a missing hidden second
controller.

## Agent grant

A grant binds:

- agentId
- seat
- allowed button names
- allowed axes
- allowed system commands
- maximum actions per frame
- optional expiry frame

The default Game Boy grant exposes A, B, SELECT, START and the four directions.
It exposes no system commands.

The desktop operator grants 3,600 emulated frames by default. The native command
supports explicit grant lifetimes up to 216,000 frames.

## Enforcement

Every live ActionEnvelope entering `step_emulation` is evaluated by
`AuthorityPolicy` before:

- replay recording
- session system-command processing
- libretro input translation
- core execution

Rejected actions therefore cannot affect the emulator and are counted in runtime
telemetry.

Replay verification has its own `Replay` source and remains separate from live
seat authority.

## Human takeover

Switching to HUMAN mode revokes the active agent grant immediately.

The operator control that changes authority is host governance, not in-game
controller input. This is intentional: a player must not need the currently
controlled game character to successfully press a fictional "give control back"
button.

## Same Action Bus

Phi-Bot actions use:

```json
{
  "kind": "phi-bot",
  "agentId": "phi-local",
  "seat": 1
}
```

as their ActionSource. Human and Phi-Bot button actions otherwise use the same
ActionKind representation and reach the same core adapter.

## Qualification

`phicade.phibot-qualification.v1` runs against the frozen SameBoy build.

It proves:

1. a human controller tape runs from a frozen core snapshot,
2. the same tape is rerun with a Phi-Bot source under an explicit grant,
3. final serialized-state and framebuffer SHA-256 hashes match exactly,
4. a Phi-Bot RESET attempt is rejected while a granted A press is accepted,
5. HUMAN takeover revokes the bot and admits human input,
6. CO-OP admits human and bot inputs together,
7. VERSUS is refused on the one-port SameBoy topology,
8. a framebuffer-only Phi-Bot observation satisfies the frozen observation schema.

This does not claim an LLM can play a game well. It proves that an agent can be
connected to the game through the same governed runtime seam without hidden
emulator authority.
