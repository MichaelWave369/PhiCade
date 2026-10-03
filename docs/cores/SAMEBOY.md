# Qualified Core: SameBoy

PhiCade Rung 3 freezes its first emulator target instead of treating any random
shared library named `*_libretro` as trusted.

## Frozen provenance

| Field | Value |
| --- | --- |
| Core | SameBoy |
| Upstream | `https://github.com/LIJI32/SameBoy` |
| Upstream version | `1.0.3` |
| Source revision | `213a12ce93d66b105a113debd9396306066a7cfc` |
| License for libretro/core code | Expat (MIT-family) |
| Target | `libretro` |
| Qualified systems | Game Boy / Game Boy Color |

SameBoy's repository license applies the Expat license to the core and libretro
code used here. The separate iOS marketplace condition is outside PhiCade's
Rung 3 desktop libretro build path.

PhiCade does not commit a SameBoy binary. CI builds the frozen source revision,
then records the resulting binary SHA-256 in the qualification receipt. Release
packaging can later choose to reproduce and bundle a qualified build while
preserving upstream notices.

## Smoke fixture

Rung 3 uses Matt Currie's `dmg-acid2` v1.0 Game Boy PPU test ROM.

- source: `https://github.com/mattcurrie/dmg-acid2`
- release fixture: `dmg-acid2.gb`
- license: MIT
- purpose: deterministic emulator smoke/PPU output, not gameplay content

The ROM is fetched only during qualification. It is not a commercial game image
and is not used as an excuse to weaken `docs/ROM_POLICY.md`.

## Qualification gate

`scripts/qualify-sameboy.sh`:

1. builds pinned RGBDS tooling,
2. checks out the frozen SameBoy revision,
3. builds `sameboy_libretro.so`,
4. downloads the MIT smoke fixture,
5. loads the core through PhiCade's own libretro host,
6. runs 240 frames,
7. requires 160x144 video and audio callback samples, and
8. emits `artifacts/sameboy-qualification.json` with core/fixture hashes and the
   final RGBA frame hash.

The receipt is evidence of what was actually executed. A filename is not
provenance, because filenames have never once lied to a human being, obviously.

## Runtime boundary

The desktop app still requires an explicitly selected SameBoy libretro binary.
At load time PhiCade verifies the core-reported identity before allowing a GB/GBC
session. Gamepad changes are normalized into `ActionEnvelope` events before the
core can observe them. The core never polls browser/controller APIs directly. SameBoy's
libretro callback exposes its native high-rate audio stream (2,097,152 Hz in the frozen
qualification); the native host resamples rates above the Web Audio nominal range to
48 kHz before PCM crosses the Tauri IPC boundary.
