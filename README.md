# PhiCade

> **Old worlds. New players.**

PhiCade is an open-source retro game runtime and emulator front end built around
one rule: controllers may propose actions; the runtime owns authority.

## Current status — Rung 11

PhiCade now includes **Φ-Agent Gym**, a source-first Game Boy benchmark for
measuring pixel-grounded controller behavior.

Current stack:

- Tauri 2 + React/TypeScript desktop shell
- qualified SameBoy 1.0.3 GB/GBC runtime
- save RAM, states, rewind, screenshots, profiles
- deterministic Replay Ledger
- governed Phi-Bot seat
- provider-neutral Agent Driver Protocol
- loopback-only Ollama vision adapter
- bounded multi-turn Autodrive
- digest-bound local model qualification
- source-first Φ-Agent Gym benchmark ROM
- rendered-pixel benchmark scorer
- NO-INPUT negative control
- deterministic oracle positive control
- benchmark replay determinism check
- source/ROM/core hash-bound benchmark receipt

## Run

```bash
npm install
npm run desktop
```

Choose a ROM directory, select a compatible SameBoy libretro core, and load a
`.gb` or `.gbc` game.

## Local model qualification

With Ollama running locally:

1. **SCAN MODELS**
2. select a local model
3. press **QUALIFY MODEL**

A PASS receipt binds the selected provider/model to the exact currently installed
digest.

**OLLAMA TURN** remains available for one-shot experimentation.

**AUTO DRIVE** requires a current digest-bound qualification PASS.

## Φ-Agent Gym

The benchmark source lives at:

`benchmarks/agent-gym/main.asm`

Task:

**move the solid 8×8 block to the visible X target.**

The benchmark:

- accepts D-pad input only
- moves 2 pixels per emulated frame
- is scored from the rendered RGBA framebuffer
- never exposes game RAM to the scorer/model
- uses normalized Manhattan-distance progress on a 0–1000 scale

CI assembles the ROM from source and validates:

- NO-INPUT produces zero progress
- deterministic oracle reaches the target
- oracle score is at least 980/1000
- replaying the oracle from the same state yields the exact same final frame hash

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
- `artifacts/agent-gym-qualification.json`

See:

- `docs/cores/SAMEBOY.md`
- `docs/SESSION_MACHINERY.md`
- `docs/REPLAY_LEDGER.md`
- `docs/PHIBOT_SEAT.md`
- `docs/AGENT_DRIVER_PROTOCOL.md`
- `docs/OLLAMA_PROVIDER.md`
- `docs/AUTODRIVE.md`
- `docs/MODEL_QUALIFICATION.md`
- `docs/AGENT_GYM.md`
- `docs/ARCHITECTURE.md`

## Content policy

PhiCade does **not** distribute commercial ROMs, proprietary BIOS/firmware,
decryption keys, or copyrighted game assets. Use homebrew/public-domain test
software or software you are authorized to use.

## License

PhiCade's own code, including Φ-Agent Gym source, is MIT. Third-party emulator
cores and model runtimes retain their own licenses and notices.
