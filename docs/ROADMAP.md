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

- [x] Select SameBoy 1.0.3 / GB+GBC with compatible redistribution terms
- [x] Freeze source revision, version, license, and provenance
- [x] Load the MIT-licensed dmg-acid2 smoke fixture in CI
- [x] Present libretro video frames in the CRT canvas
- [x] Produce libretro audio through Web Audio
- [x] Translate standard human gamepad state into Action Bus events
- [x] Produce an exportable SHA-256 smoke-test receipt

## Rung 4 — Session machinery

- [x] Save RAM
- [x] Save/load state
- [x] Fast-forward
- [x] Rewind buffer
- [x] Screenshots
- [x] Per-game profiles
- [x] CI state round-trip qualification receipt

## Rung 5 — Replay Ledger

- [x] Deterministic input recording
- [x] Replay verification
- [x] State/frame/input checkpoints
- [x] Exportable content-addressed session receipt
- [x] Divergence detector
- [x] CI positive + mutated-replay negative controls

## Rung 6 — Phi-Bot seat

- [x] Frame observation API
- [x] Scoped input authority
- [x] Human handoff / takeover
- [x] Same-seat human + agent co-op mode
- [x] Two-seat versus topology with explicit single-port SameBoy refusal
- [x] No privileged side-channel input
- [x] CI human/agent parity + authority qualification receipt

## Rung 7 — Agent Driver Protocol

- [x] Transport-neutral request/response schemas
- [x] Observation-bound turn IDs + hashes
- [x] Bounded action count and delay budgets
- [x] Turn expiry / stale-response rejection
- [x] Native scheduled driver inbox
- [x] Host-canonical live action sequencing
- [x] Deterministic human-last co-op precedence
- [x] Desktop reference driver
- [x] CI driver/direct behavioral parity receipt

## Later

Provider adapters (Ollama/local HTTP, cloud APIs), metadata/cover art, additional
systems, true multi-port versus play, netplay, spectator mode, achievements,
tournaments, CommonLine session rooms, and a shared Phi Game Runtime with
Night Circuit.
