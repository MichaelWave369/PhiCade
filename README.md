# PhiCade

> **Old worlds. New players.**

PhiCade is an open-source retro game runtime and emulator front end built around
one core idea: every player enters through the same Action Bus.

Human controls, deterministic replays, scripts, network input, and a future
Phi-Bot seat can therefore share the same authority and evidence machinery.

## Rung 1

The first scaffold is intentionally core-neutral:

- React + TypeScript + Vite terminal UI
- Rust runtime workspace
- normalized Action Bus
- emulator-core adapter trait
- ROM/firmware safety boundary
- architecture + roadmap docs
- GitHub Actions for web build and Rust tests

Real emulation begins after the core ABI, provenance, and licensing gates exist.
That is less flashy than dropping random DLLs into a folder, but considerably
less cursed.

## Run the shell

```bash
npm install
npm run dev
```

## Test the runtime

```bash
cargo test --workspace --all-targets
```

## Repository map

```text
apps/desktop/              React/Vite PhiCade shell
crates/phicade-runtime/    Core-neutral Rust runtime contracts
docs/ARCHITECTURE.md       Input/core trust boundaries
docs/ROM_POLICY.md         Game image + firmware repository policy
docs/ROADMAP.md            Implementation rungs
```

## Content policy

PhiCade does **not** distribute commercial ROMs, proprietary BIOS/firmware,
decryption keys, or copyrighted game assets. Use homebrew/public-domain test
software or software you are legally entitled to use.

## Planned core model

PhiCade is being designed for adapter-based emulation, including
libretro-compatible cores where their licenses and redistribution terms are
compatible with the way PhiCade ships them.

## License

MIT. See [LICENSE](LICENSE).
