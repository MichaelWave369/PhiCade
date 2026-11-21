# Scarlett Lab | PhiCade Rung 1

**Status: read-only foundation, NOT a Series S/X emulator.**

Scarlett Lab lives inside PhiCade's Rust workspace as `phicade-scarlett-lab`.
It is intentionally disconnected from PhiCade's arcade runtime, AI agent
action bus, controller seats, emulator launchers, and Xbox Edition browser
gateway. No existing game session gains capabilities from its presence.

## Rung 1 deliverables

- Strictly bounded inspection of **generic** x86-64 PE32+ header metadata.
- Synthetic source-built fixture in Rust tests: no retail Xbox binaries.
- Explicit format/CPU checks, checked offsets, section table/section range
  bounds, raw data non-overlap, a 64 MiB file cap, and SHA-256 evidence.
- JSON output with a capability record that states **execution, Xbox game
  support, rendering, arcade integration are all false**.
- Isolated Linux/Windows GitHub Actions test matrix, no local setup needed
  for contributors to see validation.
- Optional CLI for researchers who choose to analyze their own authorized
  and legally obtained samples; the CLI does not execute anything.

## Why this is PE32+, not "Xbox format" detection

Both Windows and console work can involve PE-related formats, but a basic
PE32+ header does not prove Xbox provenance, compatibility, or entitlement.
The parser never tries to decrypt, unpack, boot, or patch any program. It
does not implement Game OS, GDK, Direct3D 12.x, audio, input, accounts or
console security.

## Example (optional)

```bash
cargo test --locked -p phicade-scarlett-lab --all-targets
cargo run --locked -p phicade-scarlett-lab --bin scarlett-inspect -- ./my-authorized-fixture.exe
```

These are reference commands, **not required for the GitHub-first workflow**.
Do not upload copyrighted binaries or personal data into issues or CI.

## Acceptance

- [x] Source-built synthetic fixture tests are present.
- [x] Parser and CLI refuse to execute guest code by design.
- [x] Experimental crate is isolated from normal runtime pathways.
- [ ] GitHub CI passes on both Linux and Windows (check exact PR head).
- [ ] Read-only opt-in desktop lab UI.
- [ ] First independent legal fixture compiled from source and inspected.
- [ ] Safe PixelForge diagnostic report viewer.
- [ ] Native graphics/API compatibility experiments.

## Planned milestones

1. **Rung 1:** source-built generic PE32+ inspection and evidence boundaries.
2. **Rung 2:** documented/import-table and section-layout analysis with fuzzing,
   dedicated opt-in desktop viewer, *still no execution*.
3. **Rung 3:** original test harness with mock OS API calls and deterministic
   capability tests, still no proprietary game loading.
4. **Rung 4:** synthetic D3D12 fixtures and PixelForge read-only visualization.
5. **Later:** reassess feasibility and legal boundaries before any actual
   game runtime adapter. No claim of commercial game booting or preservation
   compatibility before independent verification.

## Security / licensing

Only independently authored or properly authorized test content.
No proprietary SDK, firmware, title files, keys, bypasses, or decrypted retail
images are bundled. No code execution, import resolution, sandbox escape, guest
network or writable filesystem access in the inspection path. Contributions
claiming real Xbox compatibility must provide reproducible evidence, not just
a successful PE header parse.

See [Scarlett-PixelForge boundary](SCARLETT_PIXELFORGE_BRIDGE.md).
