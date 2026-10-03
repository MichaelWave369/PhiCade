# PhiCade

> **Old worlds. New players.**

PhiCade is an open-source retro game runtime and emulator front end built around
one core idea: every player enters through the same Action Bus.

Human controls, deterministic replays, scripts, network input, and a future
Phi-Bot seat can therefore share the same authority and evidence machinery.

## Current status — Rung 2

PhiCade now has a native Tauri shell around the React interface.

- React + TypeScript + Vite terminal UI
- Tauri 2 + Rust desktop shell
- user-selected local ROM directory picker
- local metadata-only ROM scanner
- persistent app settings
- controller discovery seam
- canonical Rust/TypeScript ActionEnvelope IPC
- core-neutral Rust runtime
- ROM/firmware repository policy
- CI for web, runtime, and native-shell compilation

No emulator core binary is integrated yet. Rung 3 is the first qualified core.

## Run the web preview

```bash
npm install
npm run dev
```

The preview keeps native filesystem features disabled.

## Run the desktop shell

Install the platform prerequisites for Tauri, then:

```bash
npm install
npm run desktop
```

## Tests

```bash
cargo test -p phicade-runtime --all-targets
```

## Repository map

```text
apps/desktop/              React/Vite PhiCade UI
crates/phicade-runtime/    Core-neutral runtime + canonical actions
src-tauri/                 Native Tauri host and local scanner
docs/ARCHITECTURE.md       Input/core trust boundaries
docs/NATIVE_BOUNDARY.md    Native filesystem/settings/IPC rules
docs/ROM_POLICY.md         Game image + firmware repository policy
docs/ROADMAP.md            Implementation rungs
```

## Content policy

PhiCade does **not** distribute commercial ROMs, proprietary BIOS/firmware,
decryption keys, or copyrighted game assets. Use homebrew/public-domain test
software or software you are legally entitled to use.

## License

MIT. See [LICENSE](LICENSE).
