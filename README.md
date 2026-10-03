# PhiCade

> **Old worlds. New players.**

PhiCade is an open-source retro game runtime and emulator front end built around
one rule: controllers may propose actions; the runtime owns authority.

## Current status — Rung 15

PhiCade now has a **two-task, source-first benchmark suite registry** rather than a
single hardcoded Φ-Agent Gym ROM.

Benchmark Suite v1 contains:

- **Move the Block to the X** — top-left → bottom-right
- **Mirror Dash** — bottom-right → top-left

Both tasks are independently assembled, hash-pinned, pixel-scored, oracle-qualified,
and replay-qualified.

Current stack also retains:

- Tauri 2 + React/TypeScript desktop shell
- qualified SameBoy 1.0.3 GB/GBC runtime
- deterministic Replay Ledger
- governed Φ-Bot seat
- provider-neutral Agent Driver Protocol
- loopback-only Ollama vision adapter
- bounded multi-turn Autodrive
- digest-bound local model qualification
- exact-digest gameplay receipts
- repeated benchmark campaigns
- provenance-checked Comparison Lab
- persistent evidence IDs across app sessions

## Run

\`\`\`bash
npm install
npm run desktop
\`\`\`

## Benchmark Suite v1

Suite manifest:

\`benchmarks/suite-v1.json\`

### Task A

\`move-block-to-x-v1\`

ROM SHA-256:

\`353e69e859f50f5ef14f0221e386b18b8194f771cc603696530a59617593c59e\`

### Task B — Mirror Dash

\`move-block-to-x-mirror-v1\`

ROM SHA-256:

\`278a8106343fe1688a1370c0575578417744c96ae52568ab1e97f446dc222bfb\`

Native PhiCade recognizes benchmark tasks by exact ROM SHA-256 and exposes the
loaded task through \`SessionInfo.benchmarkTask\`.

With a registered task loaded and a qualified model handed off:

- **BENCH TASK** runs one scored model benchmark,
- **CAMPAIGN 5×** runs repeated trials for that task,
- **COMPARISON LAB** compares compatible campaigns from that same frozen task.

## Qualification

\`\`\`bash
bash ./scripts/qualify-sameboy.sh
\`\`\`

The qualification script now assembles and qualifies both suite ROMs.

It produces:

- \`artifacts/agent-gym-qualification.json\`
- \`artifacts/agent-gym-mirror-qualification.json\`

alongside the existing SameBoy, replay, Φ-Bot, driver, and Autodrive receipts.

Every suite task must pass:

- exact registry source/ROM hash match,
- frozen start geometry,
- NO-INPUT zero-progress control,
- deterministic oracle success,
- >= 980/1000 oracle score,
- exact final-frame replay.

## Evidence semantics

A benchmark task's identity is part of its evidence.

Campaigns pin the task ID plus source/ROM hashes.

Comparison Lab remains like-for-like and refuses campaigns from different tasks.

Cross-task aggregation is intentionally a later layer rather than a relaxation of
the existing comparison gate.

See:

- \`docs/BENCHMARK_SUITE_V1.md\`
- \`docs/MODEL_GAMEPLAY_BENCHMARK.md\`
- \`docs/BENCHMARK_CAMPAIGNS.md\`
- \`docs/COMPARISON_LAB.md\`
- \`docs/ARCHITECTURE.md\`

## Content policy

PhiCade does **not** distribute commercial ROMs, proprietary BIOS/firmware,
decryption keys, or copyrighted game assets.

## License

PhiCade's own code, including both Benchmark Suite v1 task sources, is MIT.
Third-party emulator cores and model runtimes retain their own licenses and notices.
