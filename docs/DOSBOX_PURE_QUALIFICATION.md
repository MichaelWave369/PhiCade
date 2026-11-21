# DOSBox Pure Rung 2: original DOS program qualification

## Scope

This is a real libretro runtime smoke test using an original 16-bit DOS
program with reproducible source and a pinned upstream source revision.
It is not a claim that commercial games or all DOS devices work in PhiCade.

## Upstream, source and format

- Core: https://github.com/libretro/dosbox-pure
- Pinned source SHA: e39497e7c2a32c67c26e6598dd14f6d58c384bfd
- Source at that SHA declares version 1.0-preview8, GPL-2.0-or-later.
- Format: .dosz, a DOSBox Pure-recognized ZIP archive extension.
- Original fixture source: fixtures/dos/phitest.asm
- Build: NASM assembles PHITEST.COM; Python creates deterministic PHITEST.dosz.
- No retail game, firmware, downloaded PC program, or external core binary is committed.

Official reference: https://docs.libretro.com/library/dosbox_pure/

## Original DOS fixture program

1. Set BIOS VGA mode 13h, 320x200.
2. Fill VGA memory with palette index 0Dh (bright magenta).
3. Configure PIT channel 2 and PC speaker for an approximate 440 Hz note.
4. Poll BIOS keyboard for Escape.
5. On Escape, disable the speaker, restore text mode and DOS-exit.

## What GitHub CI attempts to verify

- The source-built core is exactly the pinned revision.
- Core identifies as DOSBox-pure and advertises the .dosz extension.
- The original archive loads through PhiCade's actual libretro host.
- Several rendered frames contain >1/3 magenta pixels.
- Governed retropad press and release update and clear host input state.
- Audio callback counts and nonzero audio sample observations are reported separately.
- The receipt includes core/fixture SHA256 and final framebuffer hash.

**Explicitly unqualified, even after a passing video test:**

- Keyboard input delivered to the DOS BIOS.
- Mouse input delivered to DOS.
- DOS-side clean exit. Dropping the core on the host is NOT guest exit.
- Playability or input/audio compatibility of commercial DOS games.
- Audible PC speaker, MIDI, and save handling unless independently observed.

The receipt sets four explicit false fields for keyboard, mouse, guest exit,
and retail-game compatibility. Audio callback presence is not confused
with nonzero audio samples or user-audible PC speaker sound.

## CI and licensing

The isolated .github/workflows/dosbox-pure.yml job builds its core from
pinned GPL source and assembles the independent PhiCade test program.
It uploads JSON evidence only. There is no redistribution of the GPL
core or third-party game binaries through the repository.

## Next rung

Add controlled keyboard and mouse device delivery, inject Escape into
the original fixture, and demonstrate a guest-originated clean exit.
Do not promote DOSBox Pure to general-game qualification prematurely.
