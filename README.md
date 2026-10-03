# PhiCade

> **Old worlds. New players.**

PhiCade is an open-source retro game runtime and emulator front end built around
one core idea: every player enters through the same governed runtime seam.

## Current status — Rung 7

PhiCade now has a provider-neutral Agent Driver Protocol on top of its governed
Phi-Bot seat, Replay Ledger, persistent session machinery, and qualified SameBoy
runtime.

Highlights:

- Tauri 2 + React/TypeScript desktop shell
- qualified SameBoy 1.0.3 GB/GBC host
- battery RAM, save states, rewind, screenshots, profiles
- content-addressed deterministic Replay Ledger
- framebuffer-only Phi-Bot observation API
- HUMAN / PHI-BOT / CO-OP authority modes
- immediate human takeover
- scoped and expiring agent grants
- transport-neutral AgentTurnRequest / AgentTurnResponse
- observation hash + frame binding
- action-count and delayed-action budgets
- stale/mismatched response rejection
- native scheduled driver inbox
- host-canonical action sequencing
- human-last same-frame co-op precedence
- deterministic desktop reference driver
- CI driver-vs-direct behavioral parity qualification

No model provider is hard-coded into the game runtime.

A local model, cloud model, script, or deterministic controller only needs to turn:

`AgentTurnRequest -> AgentTurnResponse`

Everything after that remains PhiCade's responsibility.

## Run

```bash
npm install
npm run desktop
```

Choose a ROM directory, select a compatible SameBoy libretro core, choose a
`.gb` or `.gbc` image, and use **LOAD / RUN**.

For the agent path:

1. select **HANDOFF** or **CO-OP**
2. press **DRIVER TURN**
3. PhiCade issues a real observation-bound turn request
4. the reference driver produces a bounded response
5. native PhiCade schedules it into the same authority/core path

The reference driver is intentionally simple. Rung 7 proves the integration
contract, not model intelligence.

## Qualification

```bash
bash ./scripts/qualify-sameboy.sh
```

Produces:

- `artifacts/sameboy-qualification.json`
- `artifacts/replay-qualification.json`
- `artifacts/phibot-qualification.json`
- `artifacts/agent-driver-qualification.json`

See:

- `docs/cores/SAMEBOY.md`
- `docs/SESSION_MACHINERY.md`
- `docs/REPLAY_LEDGER.md`
- `docs/PHIBOT_SEAT.md`
- `docs/AGENT_DRIVER_PROTOCOL.md`
- `docs/ARCHITECTURE.md`

## Content policy

PhiCade does **not** distribute commercial ROMs, proprietary BIOS/firmware,
decryption keys, or copyrighted game assets. Use homebrew/public-domain test
software or software you are authorized to use.

## License

PhiCade's own code is MIT. Third-party emulator cores retain their own licenses
and notices.
