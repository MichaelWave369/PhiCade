# PhiCade

**Old worlds. New players.**

PhiCade is an open-source retro game runtime and emulator front end designed around a shared Action Bus so human input, replay input, scripts, and future governed AI players can use the same deterministic control path.

> PhiCade does not distribute commercial ROMs, BIOS files, encryption keys, or other copyrighted game content. Use homebrew/public-domain test software or software you are legally entitled to use.

## Status

Early architecture scaffold. The first implementation rung establishes the desktop shell, runtime contracts, ROM-library boundary, and CI before emulator cores are integrated.

## Direction

- Native desktop shell: Tauri + Rust
- UI: React + TypeScript
- Emulator integration: core-adapter boundary designed for libretro-compatible cores
- Shared input model: Action Bus
- Deterministic replay and evidence receipts
- Save-state and rewind architecture
- Future Phi-Bot seat through the same authority/input path as humans

## License

MIT. See `LICENSE`.
