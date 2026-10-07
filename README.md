# PhiCade

> **Old worlds. New players.**

PhiCade is an open-source game runtime where humans, scripts, replays, and AI
agents play through the same governed control seam.

**Controllers propose actions. The runtime owns authority.**

PhiCade keeps runtime capabilities, model identity, benchmark conditions, and
available replay evidence explicit instead of assuming every controller or core
deserves the same powers.

## What you can do

- **Play GB/GBC games** through a pinned, qualified SameBoy 1.0.3 path.
- **Register user-supplied libretro cores** by exact binary SHA-256.
- **Launch registered runtimes from the desktop** with explicit session approval.
- **Run ScummVM** as a governed registered runtime without bundling the GPL core
  or game data.
- **Seat an AI controller** through the Agent Driver Protocol.
- **Use a local Ollama vision model** over loopback-only provider access.
- **Run bounded Autodrive** with governed memory, cadence, and action receipts.
- **Benchmark agent behavior** across a frozen 77-task Suite v13 ladder.
- **Build campaign, suite, comparison, public-result, and portable evidence**
  without collapsing unlike tasks into one mystery score.

## Why PhiCade is different

Most emulator front ends ask one question:

> Can this controller make the game move?

PhiCade asks several:

```text
What did the controller observe?
What actions was it allowed to propose?
Which runtime actually executed them?
Which capabilities has that runtime proved?
Which exact model/core binary was involved?
What evidence survives afterward?
```

Registration, authority, capability, and qualification are deliberately separate.

A registered runtime does not automatically become trusted:

```text
REGISTERED
!= BINARY EVIDENCE BOUND
!= AUTHORITY GRANTED
```

And a runtime is not given features it cannot support. ScummVM, for example,
can participate in governed play while exact Replay v1 and state rewind remain
disabled because its pinned libretro profile does not expose serialization.

## Runtime support

| Runtime path | Play | Governed input | State snapshots | Exact Replay v1 | Persistent libretro save data |
|---|---|---|---|---|---|
| Qualified SameBoy 1.0.3 | Yes | Yes | Qualified | Qualified | Qualified |
| Pinned ScummVM v2026.3.0 profile | Yes | Qualified frame/input path | Unsupported | Unsupported | Unsupported |
| Other registered libretro core | Capability-derived | Capability-derived | Probed / profile-dependent | Unsupported unless separately qualified | Probed / profile-dependent |

Registered core identity is checked again at launch, and the local binary is
re-hashed before a registered session starts.

## Desktop runtime flow

```text
local libretro core
       |
       v
REGISTER
  SHA-256 + identity + capabilities
       |
       v
SELECT RUNTIME
       |
       v
APPROVE THIS SESSION
       |
       +------ RUN LAUNCHER
       |
       +------ RUN FILE...
       |
       v
native re-hash + identity check
       |
       v
capability-aware governed session
```

The legacy qualified SameBoy GB/GBC path remains available alongside registered
runtime launch.

## AI player path

```text
Human / Replay / Script / Phi-Bot
                |
                v
            Action Bus
                |
                v
         Authority Policy
                |
                v
          Runtime Adapter
                |
                v
              Game
```

The controller never receives a privileged emulator handle merely because it is
an AI controller.

The local Ollama path supports model discovery, qualification, governed turns,
bounded working memory, adaptive observation cadence, and receipt-bound
benchmark campaigns.

## Benchmarks

PhiCade currently freezes **Suite v13 with 77 tasks**.

The ladder progresses from navigation into delayed memory, stateful causal
dependencies, erased relational memory, rule composition, selective context
routing, and indirect context routing.

The newest 16-task Indirect Context Routing slice forces:

```text
pointer token
  -> erased pointer map
  -> erased bank
  -> erased symbol relation
  -> final side
```

Known shortcut families are deliberately balanced on that 16-task slice:

| Shortcut family | Score |
|---|---:|
| Always LEFT | 8 / 16 |
| Always RIGHT | 8 / 16 |
| Assume NORMAL pointer map | 8 / 16 |
| Assume SWAPPED pointer map | 8 / 16 |
| Always resolve CIRCLE | 8 / 16 |
| Always resolve CROSS | 8 / 16 |
| Ignore query as fixed TRIANGLE/SQUARE | 8 / 16 |

Those are **slice-specific structural controls**, not a claim that one shortcut
scores 50% across the full heterogeneous 77-task suite.

See [docs/BENCHMARKS.md](docs/BENCHMARKS.md).

## Results

PhiCade can turn a verified Suite Report into a deterministic **Public Suite
Result v1**.

The desktop **PUBLIC RESULT** lane revalidates the selected Suite Report through
its campaigns, model-gameplay receipts, Autodrive receipts, and exact
model-qualification receipt, then exports both JSON and Markdown containing:

- model name and exact digest;
- model qualification SHA-256;
- suite version and source Suite Report SHA-256;
- core identity and SHA-256;
- Autodrive policy;
- trials per task;
- aggregate suite statistics;
- every task's campaign receipt SHA-256.

The same verified report can also be exported as a deterministic **Portable
Evidence Bundle v1** containing the closed receipt chain, regenerated public
JSON/Markdown, and a SHA-256 manifest. ROMs, cores, model weights, save data,
and commercial assets are deliberately excluded.

The repository still does **not yet publish a canonical Suite v13 local-model
score**. The export machinery exists so the first published result can come
from verified evidence instead of a manually copied leaderboard.

See [docs/PUBLIC_RESULTS.md](docs/PUBLIC_RESULTS.md) and
[docs/PORTABLE_EVIDENCE_BUNDLE.md](docs/PORTABLE_EVIDENCE_BUNDLE.md).

## Quickstart

### 1. Clone

```bash
git clone https://github.com/MichaelWave369/PhiCade.git
cd PhiCade
```

### 2. Install JavaScript dependencies

```bash
npm install
```

### 3. Run the Tauri desktop app

```bash
npm run desktop
```

You also need a Rust toolchain and the normal platform dependencies required by
Tauri 2.

PhiCade does **not** include commercial ROMs, proprietary BIOS/firmware, game
assets, or third-party runtime binaries.

### Optional: qualify SameBoy + benchmark stack

```bash
bash ./scripts/qualify-sameboy.sh
```

### Optional: qualify pinned ScummVM launcher

```bash
bash ./scripts/qualify-scummvm.sh
```

The ScummVM qualification builds the pinned upstream no-engine launcher for CI
evidence. PhiCade does not publish that GPL binary as part of the MIT project.

## Evidence model

PhiCade's evidence stack includes:

- qualified SameBoy runtime identity;
- capability manifests;
- exact binary fingerprints;
- Runtime Registration v1 receipts;
- session-scoped operator approval;
- deterministic Replay v1 where supported;
- governed Phi-Bot / Agent Driver control;
- exact model digest qualification with immutable receipt-SHA archival;
- bounded Autodrive receipts;
- transitive evidence closure from Suite Report through trial/Autodrive evidence;
- source-first benchmark task hashes;
- negative and shortcut controls;
- repeated task campaigns;
- frozen Suite Reports;
- deterministic verified public result exports;
- deterministic portable evidence ZIPs;
- like-for-like comparison receipts.

See [docs/EVIDENCE.md](docs/EVIDENCE.md).

## Current status

**Rung 44 — PixelForge Runtime Adapter v1**

The current desktop can:

1. scan a local GB/GBC library;
2. launch the qualified SameBoy path;
3. register local libretro cores;
4. inspect their identity and capability state;
5. select a registered runtime;
6. explicitly approve one session;
7. launch FILE content or a supported no-content launcher;
8. expose only the replay/state/save controls supported by that runtime;
9. close an existing Suite Report through campaign, trial, Autodrive, and exact
   model-qualification evidence before comparison or deterministic public export;
10. package that closed evidence graph into a deterministic portable ZIP without
    bundling ROMs, cores, model weights, or commercial assets.

## Documentation

Start here:

- [Benchmarks](docs/BENCHMARKS.md)
- [Evidence and qualification](docs/EVIDENCE.md)
- [Evidence Closure](docs/EVIDENCE_CLOSURE.md)
- [Portable Evidence Bundle](docs/PORTABLE_EVIDENCE_BUNDLE.md)
- [Public Suite Results](docs/PUBLIC_RESULTS.md)
- [Architecture](docs/ARCHITECTURE.md)\n- [PixelForge Runtime Adapter](docs/PIXELFORGE_RUNTIME_ADAPTER.md)
- [Registered Launch UI](docs/REGISTERED_LAUNCH_UI.md)
- [Desktop Runtime Manager](docs/DESKTOP_RUNTIME_MANAGER.md)
- [Registered Session Routing](docs/REGISTERED_SESSION_ROUTING.md)
- [Runtime Registration](docs/RUNTIME_REGISTRATION.md)
- [Runtime Capability Manifest](docs/RUNTIME_CAPABILITY_MANIFEST.md)
- [ScummVM Qualification](docs/SCUMMVM_QUALIFICATION.md)
- [Agent Driver Protocol](docs/AGENT_DRIVER_PROTOCOL.md)
- [Autodrive](docs/AUTODRIVE.md)
- [Replay Ledger](docs/REPLAY_LEDGER.md)
- [Roadmap](docs/ROADMAP.md)

## Content policy

PhiCade does **not** distribute commercial ROMs, proprietary BIOS/firmware,
decryption keys, copyrighted game assets, or the ScummVM GPL binary.

Use game content and third-party runtimes only when you have the right to do so.

## License

PhiCade's own code, including benchmark task sources, is MIT.

Third-party emulator cores and model runtimes retain their own licenses and
notices.
