# PhiCade Roadmap

## Rung 1 — Foundation
- [x] Repository and MIT license
- [x] React/Vite desktop UI prototype
- [x] Rust runtime crate
- [x] Shared Action Bus concept in UI + runtime
- [x] Emulator-core trait boundary
- [x] ROM/firmware repository policy
- [x] CI for TypeScript build and Rust tests

## Rung 2 — Native shell
- [x] Tauri desktop shell
- [x] Local directory access
- [x] Persistent settings
- [x] Controller discovery
- [x] Canonical ActionEnvelope IPC
- [x] Native compile gate

## Rung 3 — First qualified core
- [x] SameBoy 1.0.3 / GB+GBC provenance freeze
- [x] MIT-licensed dmg-acid2 fixture
- [x] Video/audio bridge
- [x] Human gamepad through Action Bus
- [x] SHA-256 qualification receipt

## Rung 4 — Session machinery
- [x] Save RAM
- [x] Save/load state
- [x] Fast-forward
- [x] Rewind buffer
- [x] Screenshots
- [x] Per-game profiles
- [x] State round-trip qualification

## Rung 5 — Replay Ledger
- [x] Deterministic input recording
- [x] Replay verification
- [x] State/frame/input checkpoints
- [x] Content-addressed receipts
- [x] Divergence detector
- [x] Positive + mutated-replay controls

## Rung 6 — Phi-Bot seat
- [x] Frame observation API
- [x] Scoped input authority
- [x] Human handoff/takeover
- [x] Same-seat co-op
- [x] Explicit one-port versus refusal
- [x] No privileged side-channel input
- [x] Human/agent parity receipt

## Rung 7 — Agent Driver Protocol
- [x] Transport-neutral turn schemas
- [x] Observation hash + turn binding
- [x] Action/delay budgets
- [x] Turn expiry
- [x] Native driver inbox
- [x] Host-canonical sequencing
- [x] Human-last same-frame precedence
- [x] Reference driver
- [x] Driver/direct parity receipt

## Rung 8 — Local Ollama provider
- [x] Loopback-only Ollama REST adapter
- [x] Local model discovery
- [x] RGBA framebuffer → PNG vision payload
- [x] JSON-schema-constrained button output
- [x] AgentTurnResponse conversion
- [x] THINK PAUSE frame consistency
- [x] Provider-failure turn cancellation
- [x] Persistent local model selection
- [x] Loopback mock-server CI tests

## Rung 9 — Governed Autodrive
- [x] Native bounded autonomous-run policy
- [x] Turn/action/frame/empty-turn budgets
- [x] Queue-aware multi-turn loop
- [x] Persistent stop-reason receipts
- [x] Immediate HUMAN takeover
- [x] Grant-expiry stop + neutralization
- [x] Provider-failure stop
- [x] System/timeline-command refusal during autonomous runs
- [x] SameBoy bounded-loop qualification receipt

## Rung 10 — Model Qualification Registry
- [x] Exact Ollama model digest discovery
- [x] /api/show capability inspection
- [x] Vision capability gate
- [x] Synthetic image visual probe
- [x] Structured-output qualification probe
- [x] Digest-bound persistent receipts
- [x] One-shot unqualified experimentation
- [x] Native AUTO DRIVE exact-digest gate
- [x] Changed-digest negative control
- [x] Mock capability/qualification CI tests

## Rung 11 — Φ-Agent Gym
- [x] Source-first Game Boy benchmark ROM
- [x] Machine-readable task manifest
- [x] Rendered-pixel player detection
- [x] Manhattan-distance progress score
- [x] NO-INPUT negative control
- [x] Oracle D-pad positive control
- [x] Deterministic replay control
- [x] Source/ROM/core hash-bound receipt
- [x] SameBoy CI integration

## Later

Qualified-model gameplay runs against Φ-Agent Gym, adaptive observation cadence,
model-specific benchmark comparisons, provider capability profiles beyond Ollama,
cloud-provider adapters, metadata/cover art, additional systems, true multi-port
versus play, netplay, spectator mode, achievements, tournaments, CommonLine rooms,
and a shared Phi Game Runtime with Night Circuit.
