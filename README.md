# PhiCade

> **Old worlds. New players.**

PhiCade is an open-source retro game runtime and emulator front end built around
one core idea: every player enters through the same governed runtime seam.

## Current status — Rung 8

PhiCade now has its first concrete AI provider adapter: **local Ollama vision**
feeding the provider-neutral Agent Driver Protocol.

Current stack:

- Tauri 2 + React/TypeScript desktop shell
- qualified SameBoy 1.0.3 GB/GBC runtime
- battery RAM, states, rewind, screenshots, profiles
- deterministic content-addressed Replay Ledger
- framebuffer-only Phi-Bot observation API
- HUMAN / PHI-BOT / CO-OP authority modes
- scoped and expiring agent grants
- provider-neutral AgentTurnRequest / AgentTurnResponse
- native scheduled driver inbox and canonical sequencing
- loopback-only Ollama REST adapter
- local model discovery
- RGBA framebuffer → PNG vision input
- JSON-schema-constrained button decisions
- THINK PAUSE while local inference runs
- explicit turn cancellation on provider failure
- native mock-Ollama CI tests

## Run

```bash
npm install
npm run desktop
```

For ordinary emulation, choose a ROM directory, select a compatible SameBoy
libretro core, choose a `.gb` or `.gbc` image, then use **LOAD / RUN**.

## Local Ollama gameplay

Run Ollama locally on its standard loopback endpoint, then in PhiCade:

1. **SCAN MODELS**
2. select an installed **vision-capable** model
3. load a GB/GBC game
4. select **HANDOFF**
5. press **OLLAMA TURN**

PhiCade freezes emulation on the observed frame while the model evaluates it,
then submits the model's structured button proposal through the same Agent Driver
and AuthorityPolicy used by every other Phi-Bot controller.

The default endpoint is:

`http://127.0.0.1:11434`

Rung 8 is intentionally loopback-only. Remote model transport is not silently
treated as equivalent to a local process.

PhiCade does not guess which installed models support vision. Selecting a text-only
model can fail cleanly without submitting gameplay actions.

## Qualification

```bash
bash ./scripts/qualify-sameboy.sh
```

Produces the frozen core/session/replay/driver receipts.

Native CI additionally runs loopback mock-Ollama tests for:

- model discovery
- RGBA → PNG vision conversion
- structured-output schema budgets
- chat-response → AgentTurnResponse conversion
- remote-host refusal

See:

- `docs/cores/SAMEBOY.md`
- `docs/SESSION_MACHINERY.md`
- `docs/REPLAY_LEDGER.md`
- `docs/PHIBOT_SEAT.md`
- `docs/AGENT_DRIVER_PROTOCOL.md`
- `docs/OLLAMA_PROVIDER.md`
- `docs/ARCHITECTURE.md`

## Content policy

PhiCade does **not** distribute commercial ROMs, proprietary BIOS/firmware,
decryption keys, or copyrighted game assets. Use homebrew/public-domain test
software or software you are authorized to use.

## License

PhiCade's own code is MIT. Third-party emulator cores and model runtimes retain
their own licenses and notices.
