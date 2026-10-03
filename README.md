# PhiCade

> **Old worlds. New players.**

PhiCade is an open-source retro game runtime and emulator front end built around
one rule: controllers may propose actions; the runtime owns authority.

## Current status — Rung 13

PhiCade can now run **repeated exact-digest gameplay benchmark campaigns** against
the frozen Φ-Agent Gym environment.

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
- shared rendered-pixel scorer
- exact-digest single-run gameplay receipts
- 3–20 trial native benchmark campaigns
- live Ollama digest re-check before every trial
- qualification/core/policy drift refusal
- immutable per-trial receipt hashes
- success rate, mean, median, min/max, and population standard deviation
- COMPLETE and PARTIAL campaign receipts

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
digest.

## Score one run

Load the exact Φ-Agent Gym ROM, choose **HANDOFF**, then press **BENCH GYM**.

PhiCade creates one Rung 12 model gameplay receipt.

## Run a repeated campaign

With the same frozen gym and a qualified model selected:

1. choose **HANDOFF**,
2. press **CAMPAIGN 5×**.

The desktop runs five trials by default.

Before every continuation trial, native PhiCade re-queries Ollama and confirms the
installed model digest still matches the campaign pin.

Each trial writes its own Rung 12 receipt. The final campaign receipt references
those trial receipts by SHA-256 and reports:

- observed/scored/error trial counts,
- success rate,
- mean score,
- median score,
- min/max score,
- population standard deviation.

Pressing **END CAMPAIGN** writes a PARTIAL summary instead of discarding completed
evidence.

## Evidence semantics

A COMPLETE campaign means all configured trials were recorded.

It does **not** mean the model succeeded.

Score statistics exclude scoring-error trials rather than inventing numeric values.
Success rate uses all observed trials.

## CI qualification

```bash
bash ./scripts/qualify-sameboy.sh
```

CI qualifies the frozen environment and campaign machinery. Real model campaign
receipts are local runtime evidence and are not fabricated by CI.

See:

- `docs/MODEL_GAMEPLAY_BENCHMARK.md`
- `docs/BENCHMARK_CAMPAIGNS.md`
- `docs/ARCHITECTURE.md`

## Content policy

PhiCade does **not** distribute commercial ROMs, proprietary BIOS/firmware,
decryption keys, or copyrighted game assets.

## License

PhiCade's own code, including Φ-Agent Gym source, is MIT. Third-party emulator
cores and model runtimes retain their own licenses and notices.
