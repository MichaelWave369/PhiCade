# PhiCade

> **Old worlds. New players.**

PhiCade is an open-source retro game runtime and emulator front end built around
one core idea: every player enters through the same Action Bus.

Human controls, deterministic replays, scripts, network input, and a future
Phi-Bot seat can therefore share the same authority and evidence machinery.

## Current status — Rung 3

PhiCade can now host its first qualified libretro core in the native Tauri shell.

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
- CI smoke qualification against MIT-licensed `dmg-acid2`
- SHA-256 qualification receipt for core, fixture, and final frame

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

## Tests

```bash
cargo test -p phicade-runtime -p phicade-libretro --all-targets
```

On Linux, the complete pinned SameBoy qualification is:

```bash
bash ./scripts/qualify-sameboy.sh
```

See `docs/cores/SAMEBOY.md` for the frozen upstream revision and evidence model.

## Repository map

```text
apps/desktop/              React/Vite PhiCade UI + CRT A/V bridge
crates/phicade-runtime/    Core-neutral runtime + canonical actions
crates/phicade-libretro/   Minimal dynamic libretro host + qualification CLI
src-tauri/                 Native Tauri host, library scanner, core session IPC
docs/ARCHITECTURE.md       Input/core trust boundaries
docs/cores/SAMEBOY.md      First-core provenance and qualification contract
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
