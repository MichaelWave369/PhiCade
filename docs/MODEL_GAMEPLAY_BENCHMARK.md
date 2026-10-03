# Rung 12 Model Gameplay Benchmark

Rung 12 binds an exact qualified local model artifact to a scored Φ-Agent Gym run.

Rung 10 answers:

**Can this exact model digest consume vision input and obey the structured-output contract?**

Rung 11 answers:

**Is the benchmark world, scorer, oracle, and replay behavior trustworthy?**

Rung 12 answers:

**What did this exact qualified model actually accomplish in that frozen world?**

## Entry requirements

A model gameplay benchmark can start only when all of the following are true:

- SameBoy is running.
- The loaded ROM SHA-256 exactly matches the frozen Φ-Agent Gym v1 ROM.
- Control mode is PHI-BOT handoff.
- The selected Ollama model has a current Rung 10 PASS receipt.
- The selected model's current digest exactly matches that receipt.
- No replay recording is active.
- No other Autodrive or benchmark run is active.
- Game speed is 1×.

## Benchmark start

Native PhiCade owns benchmark initialization.

When **BENCH GYM** starts:

1. validate the Autodrive policy,
2. verify the exact model qualification receipt,
3. hash that qualification receipt,
4. reset SameBoy,
5. neutralize input,
6. warm exactly 120 emulated frames,
7. score the rendered framebuffer,
8. require the player to be at the frozen start coordinate `(16, 24)`,
9. restrict the model grant to D-pad buttons only,
10. renew the grant for the benchmark budget,
11. enter the same native-qualified Autodrive path used by normal autonomous play.

The browser resynchronizes its frame counter to the native warmed frame before the
next live tick.

## Task instruction

For the exact Φ-Agent Gym ROM hash, the Ollama gameplay prompt adds only:

> move the solid square block onto the visible X target using the D-pad

The prompt does not reveal target coordinates, player coordinates, score internals,
emulator RAM, serialized state, or benchmark answer trajectories.

## Success stop

The model benchmark uses the same shared pixel scorer as CI.

After each emulated frame, native PhiCade scores the rendered framebuffer. If the
player reaches the frozen success radius, the autonomous run ends immediately with:

`task-success`

This prevents a model from solving the task early and then losing credit by
wandering away before a later turn or budget stop.

## Receipt

Schema:

`phicade.model-gameplay-benchmark.v1`

A benchmark receipt binds:

- receipt status,
- benchmark ID and run ID,
- provider and model name,
- exact model digest,
- SHA-256 of the Rung 10 model qualification receipt,
- frozen gym source and ROM SHA-256,
- SameBoy binary SHA-256 and identity,
- SHA-256 of the underlying Autodrive receipt,
- Autodrive run ID and exact policy,
- stop reason,
- start/end emulated frame,
- start/final/target pixel coordinates,
- initial/final distance,
- progress,
- score on a 0–1000 scale,
- task success boolean,
- turn/action counts,
- final framebuffer SHA-256,
- scoring error when pixel scoring fails.

Receipts are stored under the application-data game fingerprint:

`model-benchmarks/<gym-rom-sha256>/run-000001.json`

## Complete evidence is not the same as success

`recordStatus = COMPLETE` means the benchmark evidence was successfully recorded
and scored. It does **not** mean the model solved the task.

A valid model run may record score `0/1000`, `taskSuccess = false`, a budget stop,
provider failure, or human takeover. Those outcomes remain useful evidence.

If the final framebuffer cannot be scored, the receipt uses
`recordStatus = SCORING_ERROR` and preserves the rest of the available evidence.

## Stop paths

Every governed Autodrive stop path finalizes a benchmark receipt:

- task-success,
- operator-stop,
- human-takeover,
- turn-budget,
- action-budget,
- frame-budget,
- empty-turn-limit,
- provider-failure,
- grant-expired,
- core-shutdown.

## CI boundary

GitHub CI does not claim a score for a real local model.

CI proves the machinery by running the frozen Φ-Agent Gym self-qualification,
shared pixel-scorer tests, native receipt serialization tests, exact-digest model
qualification tests, the gym task-prompt regression test, and the full native
governance suite.

A real model score is created only on the machine where that exact qualified model
digest is installed and actually plays the benchmark.
