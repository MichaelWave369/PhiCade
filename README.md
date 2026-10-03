# PhiCade

> **Old worlds. New players.**

PhiCade is an open-source retro game runtime and emulator front end built around
one rule: controllers may propose actions; the runtime owns authority.

## Current status — Rung 14

PhiCade can now compare two provenance-compatible repeated benchmark campaigns and
persist a statistical comparison receipt.

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
- exact-digest gameplay receipts
- repeated 3–20 trial benchmark campaigns
- persistent evidence IDs across app sessions
- strict campaign compatibility gate
- trial-receipt SHA-256 re-verification before comparison
- Welch 95% mean-difference confidence interval
- Hedges' g small-sample effect size
- success-rate difference
- persistent comparison receipts

## Run

```bash
npm install
npm run desktop
```

## Build campaign evidence

Load the exact Φ-Agent Gym ROM, qualify a local Ollama vision model, choose
**HANDOFF**, then use:

- **BENCH GYM** for one scored run,
- **CAMPAIGN 5×** for a repeated campaign.

Each campaign remains bound to exact model, qualification, core, Gym, and policy
evidence.

## Compare campaigns

When two campaign receipts exist for the loaded Gym:

1. open **COMPARISON LAB**,
2. select campaign A,
3. select campaign B,
4. press **COMPARE**.

Native PhiCade refuses comparison unless both campaigns are COMPLETE, fully
scoreable, have the same configured trial count, and match on provider, Gym,
SameBoy binary/identity, and Autodrive policy.

Before computing statistics, it re-hashes every Rung 12 trial receipt referenced by
both campaign summaries.

The comparison reports:

- mean score difference A−B,
- Welch 95% confidence interval,
- Hedges' g A−B,
- success-rate difference A−B,
- source campaign identities and hashes.

PhiCade does not generate a winner badge from these values.

## Evidence persistence

Receipt numbering is seeded from existing evidence on disk. Restarting PhiCade no
longer resets Autodrive, gameplay-benchmark, campaign, or comparison IDs to 1.

Historical JSON receipts are therefore not silently overwritten by a later session.

## CI qualification

```bash
bash ./scripts/qualify-sameboy.sh
```

CI qualifies the frozen environment and Comparison Lab machinery. Real local model
scores and campaign comparisons are not fabricated by CI.

See:

- `docs/MODEL_GAMEPLAY_BENCHMARK.md`
- `docs/BENCHMARK_CAMPAIGNS.md`
- `docs/COMPARISON_LAB.md`
- `docs/ARCHITECTURE.md`

## Content policy

PhiCade does **not** distribute commercial ROMs, proprietary BIOS/firmware,
decryption keys, or copyrighted game assets.

## License

PhiCade's own code, including Φ-Agent Gym source, is MIT. Third-party emulator
cores and model runtimes retain their own licenses and notices.
