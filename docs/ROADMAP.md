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

- [ ] Wrap the UI in Tauri
- [ ] Add user-selected local directory access
- [ ] Persist settings without ROM content
- [ ] Add controller enumeration
- [ ] Define canonical JSON/IPC representation of ActionEnvelope

## Rung 3 — First qualified core

- [ ] Pick a permissibly redistributable test target/core
- [ ] Record core license + provenance + binary hash
- [ ] Load a public-domain/homebrew fixture
- [ ] Video frame presentation
- [ ] Audio output
- [ ] Human controller input through Action Bus
- [ ] Golden smoke-test receipt

## Rung 4 — Session machinery

- [ ] Save RAM
- [ ] Save/load state
- [ ] Fast-forward
- [ ] Rewind buffer
- [ ] Screenshots
- [ ] Per-game profiles

## Rung 5 — Replay Ledger

- [ ] Deterministic input recording
- [ ] Replay verification
- [ ] State checkpoints
- [ ] Exportable session receipt
- [ ] Divergence detector

## Rung 6 — Phi-Bot seat

- [ ] Frame observation API
- [ ] Scoped input authority
- [ ] Human handoff / takeover
- [ ] Agent-vs-human and co-op modes
- [ ] No privileged side-channel input

## Later

Metadata/cover art, additional systems, netplay, spectator mode, achievements,
tournaments, CommonLine session rooms, and a shared Phi Game Runtime with
Night Circuit.
