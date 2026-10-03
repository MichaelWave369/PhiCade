# PhiCade

> **Old worlds. New players.**

PhiCade is an open-source retro game runtime and emulator front end built around
one rule: controllers may propose actions; the runtime owns authority.

## Current status — Rung 9

PhiCade can now run a local Ollama vision model through a **bounded autonomous
multi-turn gameplay loop**.

Current stack:

- Tauri 2 + React/TypeScript desktop shell
- qualified SameBoy 1.0.3 GB/GBC runtime
- save RAM, states, rewind, screenshots, profiles
- deterministic Replay Ledger
- governed Phi-Bot seat
- provider-neutral Agent Driver Protocol
- loopback-only local Ollama vision adapter
- native bounded Autodrive policy
- queue-aware repeated model turns
- immediate human takeover
- explicit provider/grant/budget stop reasons
- persistent autonomous-run receipts
- SameBoy bounded-loop CI qualification

## Run

```bash
npm install
npm run desktop
```

Choose a ROM directory, select a compatible SameBoy libretro core, and load a
`.gb` or `.gbc` game.

## Local autonomous play

With Ollama running locally:

1. **SCAN MODELS**
2. select a vision-capable local model
3. load a game
4. choose **HANDOFF**
5. press **AUTO DRIVE**

Default autonomous limits are:

- 32 turns
- 128 total proposed actions
- 4 consecutive empty turns
- 3,600 emulated frames

PhiCade pauses the emulator during each vision inference, resumes to execute the
bounded action sequence, waits for the native queue to drain, then issues the next
turn.

**HUMAN remains live at all times** and immediately terminates the run and revokes
the bot grant.

AUTO DRIVE can also stop because of provider failure, grant expiry, turn/action/
frame budgets, repeated empty turns, or core shutdown.

Each completed/stopped run writes a receipt under the local game fingerprint.

## Qualification

```bash
bash ./scripts/qualify-sameboy.sh
```

Produces:

- `artifacts/sameboy-qualification.json`
- `artifacts/replay-qualification.json`
- `artifacts/phibot-qualification.json`
- `artifacts/agent-driver-qualification.json`
- `artifacts/autodrive-qualification.json`

Native CI also runs the local mock-Ollama adapter tests.

See:

- `docs/cores/SAMEBOY.md`
- `docs/SESSION_MACHINERY.md`
- `docs/REPLAY_LEDGER.md`
- `docs/PHIBOT_SEAT.md`
- `docs/AGENT_DRIVER_PROTOCOL.md`
- `docs/OLLAMA_PROVIDER.md`
- `docs/AUTODRIVE.md`
- `docs/ARCHITECTURE.md`

## Content policy

PhiCade does **not** distribute commercial ROMs, proprietary BIOS/firmware,
decryption keys, or copyrighted game assets. Use homebrew/public-domain test
software or software you are authorized to use.

## License

PhiCade's own code is MIT. Third-party emulator cores and model runtimes retain
their own licenses and notices.
