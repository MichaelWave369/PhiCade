# PhiCade

> **Old worlds. New players.**

PhiCade is an open-source retro game runtime and emulator front end built around
one rule: controllers may propose actions; the runtime owns authority.

## Current status — Rung 10

PhiCade now has a **digest-bound local model qualification registry** on top of
the governed Ollama Autodrive stack.

Current stack:

- Tauri 2 + React/TypeScript desktop shell
- qualified SameBoy 1.0.3 GB/GBC runtime
- save RAM, states, rewind, screenshots, profiles
- deterministic Replay Ledger
- governed Phi-Bot seat
- provider-neutral Agent Driver Protocol
- loopback-only Ollama vision adapter
- bounded multi-turn Autodrive
- exact installed-model digest discovery
- Ollama capability inspection
- synthetic local vision probe
- structured-output qualification probe
- persistent model qualification receipts
- native exact-digest AUTO DRIVE gate
- automatic stale qualification invalidation

## Run

```bash
npm install
npm run desktop
```

Choose a ROM directory, select a compatible SameBoy libretro core, and load a
`.gb` or `.gbc` game.

## Qualify a local model

With Ollama running locally:

1. **SCAN MODELS**
2. select a local model
3. inspect the displayed capability/digest state
4. press **QUALIFY MODEL**

PhiCade:

- resolves the exact current model digest,
- inspects advertised capabilities,
- requires advertised vision support,
- generates a 64×64 solid-red diagnostic image,
- sends the image using a strict red/blue structured-output schema,
- requires the model to return `red`,
- persists the result under that exact digest.

A PASS receipt unlocks **AUTO DRIVE** for that digest.

If the model is re-pulled or otherwise changes digest, the old receipt no longer
qualifies it.

## One-shot vs autonomous

**OLLAMA TURN** remains available for controlled one-shot experimentation with an
unqualified model.

**AUTO DRIVE** requires a current digest-bound qualification PASS in addition to
all existing Rung 9 authority and run-budget controls.

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

Native CI additionally runs mock-Ollama tests for capability inspection, visual
qualification, and digest invalidation.

See:

- `docs/cores/SAMEBOY.md`
- `docs/SESSION_MACHINERY.md`
- `docs/REPLAY_LEDGER.md`
- `docs/PHIBOT_SEAT.md`
- `docs/AGENT_DRIVER_PROTOCOL.md`
- `docs/OLLAMA_PROVIDER.md`
- `docs/AUTODRIVE.md`
- `docs/MODEL_QUALIFICATION.md`
- `docs/ARCHITECTURE.md`

## Content policy

PhiCade does **not** distribute commercial ROMs, proprietary BIOS/firmware,
decryption keys, or copyrighted game assets. Use homebrew/public-domain test
software or software you are authorized to use.

## License

PhiCade's own code is MIT. Third-party emulator cores and model runtimes retain
their own licenses and notices.
