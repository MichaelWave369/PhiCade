# PhiCade

> **Old worlds. New players.**

PhiCade is an open-source retro game runtime and emulator front end built around
one rule: controllers may propose actions; the runtime owns authority.

## Current status — Rung 12

PhiCade can now produce **scored gameplay receipts for an exact qualified local
model digest** in the frozen Φ-Agent Gym world.

Current stack:

- Tauri 2 + React/TypeScript desktop shell
- qualified SameBoy 1.0.3 GB/GBC runtime
- deterministic Replay Ledger
- governed Phi-Bot seat
- provider-neutral Agent Driver Protocol
- loopback-only Ollama vision adapter
- bounded multi-turn Autodrive
- digest-bound local model qualification
- source-first Φ-Agent Gym benchmark ROM
- shared rendered-pixel benchmark scorer
- exact-gym-hash model benchmark gate
- D-pad-only benchmark authority
- automatic task-success stop
- exact model/qualification/core/autodrive evidence binding
- persistent model gameplay score receipts

## Run

```bash
npm install
npm run desktop
```

## Qualify a local model

With Ollama running locally:

1. **SCAN MODELS**
2. select a local model
3. press **QUALIFY MODEL**

A PASS receipt binds the selected provider/model to the exact currently installed
digest. **OLLAMA TURN** remains available for one-shot experimentation.
**AUTO DRIVE** requires a current digest-bound qualification PASS.

## Score a qualified model

Load the exact Φ-Agent Gym ROM whose SHA-256 is:

`353e69e859f50f5ef14f0221e386b18b8194f771cc603696530a59617593c59e`

Then:

1. select a Rung 10-qualified model,
2. choose **HANDOFF**,
3. press **BENCH GYM**.

PhiCade resets and warms the gym to the frozen start, narrows the grant to D-pad
only, starts governed Autodrive, and automatically scores the rendered frame when
the run ends.

If the model reaches the target, the run stops immediately with `task-success`.

The persisted gameplay receipt binds the exact model digest, model qualification
receipt hash, gym source/ROM hashes, SameBoy binary hash, Autodrive receipt hash,
policy, stop reason, score, and final frame hash.

A complete receipt does not imply success. A model can honestly score 0/1000.

## CI qualification

```bash
bash ./scripts/qualify-sameboy.sh
```

CI qualifies the frozen environment and scoring machinery. Real model gameplay
receipts are local runtime evidence and are not fabricated by CI.

See `docs/MODEL_GAMEPLAY_BENCHMARK.md` for the full receipt contract.

## Content policy

PhiCade does **not** distribute commercial ROMs, proprietary BIOS/firmware,
decryption keys, or copyrighted game assets.

## License

PhiCade's own code, including Φ-Agent Gym source, is MIT. Third-party emulator
cores and model runtimes retain their own licenses and notices.
