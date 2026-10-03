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

- [ ] Select the first core/target with compatible redistribution terms
- [ ] Freeze source, version, license, and provenance
- [ ] Load a public-domain/homebrew fixture
- [ ] Present video frames in the CRT surface
- [ ] Produce audio through the host output
- [ ] Translate human gamepad state into Action Bus events
- [ ] Produce a golden smoke-test receipt

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
