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

## Rung 3: opt-in, CI-only guest keyboard and mouse-device probe

The libretro host now retains the actual keyboard callback registered by
DOSBox Pure. An explicit Cargo feature, `dos-lab-input` (off by default),
enables a tightly scoped Escape/Q injection and bounded synthetic mouse
state. These are **not** Tauri commands, app controls, or agent capabilities.

Source-compiled `PHITEST.COM` paints magenta, then only paints **green**
after the BIOS `int 16h` key-read branch observes Escape. CI injects Escape
through the registered upstream callback and requires multiple green
frames. This is end-to-end guest keyboard evidence, not just input-mask
verification. The test sends `Q` as a second guest key to attempt normal
DOS shutdown. A subsequent video transition is measured but does **not**
alone establish DOS-side exit, so `guestCleanExitQualified` remains false.

Mouse support is intentionally staged: the lab feature lets the core
poll a bounded relative mouse snapshot. Receipt counts record actual
mouse-device polls, but no guest DOS mouse-driver acknowledgment is
demonstrated, so `mouseGuestDeliveryQualified` remains false.

The core's normal `ActionBus`, Tauri commands, runtime registration,
controllers and Phi-Bot permissions are unchanged. No mouse/keyboard
events are synthesized from unapproved user or agent input. Outside the
opt-in CI feature, no lab injection methods compile into the host.

### Qualification matrix

| Evidence | After green guest transition |
| --- | --- |
| Source-built DOSZ + VGA magenta | Already proven in Rung 2 |
| Registered keyboard callback | Must be present |
| Guest BIOS read of ESC | Require multiple VGA green frames |
| Synthetic mouse sampled by core | Record poll count, not guest claim |
| Q-triggered visual return | Observe separately, not conclusive exit |
| Guest DOS-side exit | Still unqualified |
| Retail DOS game support | Still unqualified |

Next: implement a tiny guest-generated exit receipt or instrument a
source-built fixture to prove `int 21h / 4Ch` completion; then design
separate properly authorized desktop keyboard/mouse interfaces.

## Rung 4: independent guest mouse button and guest exit evidence

**Target:** Go beyond host-side input polls and frame changes.

1. Build the same independently authored 16-bit DOS program, now with
   an explicit INT 33h function 3 polling branch.
2. The DOS program first receives Escape through BIOS INT 16h and paints
   VGA green. It then checks the *guest-side* DOS mouse-driver button mask.
3. Only if the DOS mouse driver reports a left click does the guest paint
   VGA cyan, which the host must observe in at least three frames.
4. When the guest later reads Q, after both previous milestones, it
   restores text mode and runs DOS INT 21h AH=4Ch.
5. In this controlled CI-only test the host requests DOSBox Pure's
   documented menu_time=0 behavior. Only a post-Q
   RETRO_ENVIRONMENT_SHUTDOWN callback counts as completed guest exit.

**Lab isolation:** The new option setter is compiled only with
`dos-lab-input`; it requires a specific core identity, prior user-provided
test archive, and an unloaded core. It cannot be used by the normal Tauri
commands, Action Bus, or Phi-Bot. No retail game binaries are supplied.

**Meaning of a passing receipt:** It qualifies the original fixture's
BIOS ESC path, DOS mouse-driver *left click* and guest exit as observed
by DOSBox Pure's own shutdown notification. It **does not** qualify
mouse motion in general, arbitrary keyboard combinations, retail DOS
game compatibility, persistence, Windows hardware input, or a universal
clean-exit lifecycle contract for other cores.

Until the GitHub checks actually pass these are proposed acceptance tests
and not completed claims.
