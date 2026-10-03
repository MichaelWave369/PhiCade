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
- [x] Wrap the UI in Tauri
- [x] Add user-selected local directory access
- [x] Persist settings without ROM content
- [x] Add controller enumeration
- [x] Define canonical JSON/IPC representation of ActionEnvelope
- [x] Add native-shell CI compile gate

## Rung 3 — First qualified core
- [x] SameBoy 1.0.3 / GB+GBC provenance freeze
- [x] MIT-licensed dmg-acid2 smoke fixture
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
- [x] CI state round-trip qualification

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
- [x] Local model discovery via /api/tags
- [x] RGBA framebuffer → PNG vision payload
- [x] JSON-schema-constrained button output
- [x] AgentTurnResponse conversion
- [x] THINK PAUSE frame consistency
- [x] Explicit pending-turn cancellation on provider failure
- [x] Persistent local model selection
- [x] Loopback mock-server CI tests

## Later

Pinned model-specific qualification, provider capability discovery, autonomous
multi-turn driving, cloud-provider adapters, metadata/cover art, additional
systems, true multi-port versus play, netplay, spectator mode, achievements,
tournaments, CommonLine rooms, and a shared Phi Game Runtime with Night Circuit.
