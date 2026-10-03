# PhiCade

> **Old worlds. New players.**

PhiCade is an open-source retro game runtime and emulator front end built around
one core idea: every player enters through the same Action Bus.

Human controls, deterministic replays, scripts, network input, and a future
Phi-Bot seat can therefore share the same authority and evidence machinery.

## Current status — Rung 4

PhiCade now has persistent session machinery on top of its qualified SameBoy host.

- React + TypeScript + Vite terminal UI
- Tauri 2 + Rust desktop shell
- user-selected local ROM directory picker
- persistent settings and SameBoy core path
- controller discovery and GB/GBC gamepad normalization
- canonical Rust/TypeScript ActionEnvelope IPC
- core-neutral Rust runtime
- minimal governed libretro host
- SameBoy 1.0.3 provenance freeze
- GB/GBC video rendered into the CRT canvas
- high-rate libretro audio resampled to 48 kHz and streamed into Web Audio
- ROM-SHA-256-namespaced battery RAM
- core/version/game-namespaced save-state slots
- governed rewind snapshots
- 1x / 2x / 4x per-game fast-forward profile
- PNG screenshots from the live core framebuffer
- CI state serialize/restore determinism qualification
- SHA-256 qualification receipts for core, fixture, and rendered frames

No commercial ROMs, proprietary console BIOS files, or third-party core binaries
are committed to this repository.

## Run the web preview

```bash
npm install
npm run dev
```

The preview keeps native filesystem and core execution features disabled.

## Run the desktop shell

Install the platform prerequisites for Tauri, then:

```bash
npm install
npm run desktop
```

Select a ROM directory, select a compatible SameBoy libretro binary, choose a
`.gb` or `.gbc` image, and use **LOAD / RUN**. Other indexed systems remain gated
until they receive their own qualification rung.

While a game is running, the Rung 4 session deck exposes save/load state, governed
rewind, fast-forward profile controls, screenshots, and explicit battery-RAM flush.

## Tests

```bash
cargo test -p phicade-runtime -p phicade-libretro --all-targets
```

On Linux, the complete pinned SameBoy + session qualification is:

```bash
bash ./scripts/qualify-sameboy.sh
```

See:

- `docs/cores/SAMEBOY.md` for frozen core provenance
- `docs/SESSION_MACHINERY.md` for save/state/rewind/profile boundaries

## Repository map

```text
apps/desktop/              React/Vite PhiCade UI + CRT/session deck
crates/phicade-runtime/    Core-neutral runtime + canonical actions
crates/phicade-libretro/   Dynamic libretro host + qualification CLI
src-tauri/                 Native shell, persistence, session IPC
docs/ARCHITECTURE.md       Input/core trust boundaries
docs/cores/SAMEBOY.md      First-core provenance and qualification contract
docs/SESSION_MACHINERY.md  Rung 4 persistence and state machinery
docs/NATIVE_BOUNDARY.md    Native filesystem/settings/IPC rules
docs/ROM_POLICY.md         Game image + firmware repository policy
docs/ROADMAP.md            Implementation rungs
```

## Content policy

PhiCade does **not** distribute commercial ROMs, proprietary BIOS/firmware,
decryption keys, or copyrighted game assets. Use homebrew/public-domain test
software or software you are authorized to use.

## License

PhiCade's own code is MIT. Third-party emulator cores retain their own licenses
and notices. See [LICENSE](LICENSE) and the per-core provenance documents.
