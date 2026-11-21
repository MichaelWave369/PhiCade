# PC Classics: DOS, ScummVM and Steam

**Rung 1: user-owned content routing and optional installed-client handoff.**

## Existing ScummVM is not duplicated

PhiCade already contains a qualified **ScummVM v2026.3.0 libretro launcher**
(see [ScummVM qualification](SCUMMVM_QUALIFICATION.md)).
This change only recognizes `*.scummvm` hook files in the user-selected
nonrecursive library scan and adds a `SCUMMVM` tab/core preference.
It does not bundle ScummVM engines, copyrighted game data or a new executable.
ScummVM's actual game detection and supported engine set remain upstream
responsibilities; existing feature/qualification limitations remain intact.

## DOS: DOSBox Pure (candidate)

Official libretro docs:
https://docs.libretro.com/library/dosbox_pure/

The scanner recognizes **`.dosz` only**, a DOSBox Pure-specific DOS archive
extension. It deliberately rejects generic `.zip`, `.exe`, `.com`,
and `.bat` because unrelated files or hostile executables should never
be mistaken for games. The user supplies lawful DOS content and registers
their chosen DOSBox Pure libretro core using PhiCade's existing runtime
manager. Existing core SHA-256 checks, identity verification and **explicit
per-session approval** apply without special privileges.

Library route:
`SELECT ROM DIRECTORY → DOS tab → select .dosz → PLAY GAME`
(after registering a compatible core).

**Unverified:** Full DOSBox Pure compatibility, graphics, MIDI, audio,
keyboard, mouse, save and lifecycle behavior with PhiCade's generic
libretro frontend. These require actual self-authored DOS test program
qualification, not just extension matching. Do not advertise DOS games
as playable before those tests.

**DOSBox Staging** is a separately maintained standalone emulator:
https://www.dosbox-staging.org/
It is a future user-selected external runner candidate, but **is not
integrated or launched by this PR**. No shell-command execution
is delegated to a DOS game.

## Steam: client handoff, not Steam emulation

Steam games on Windows are already Windows applications. Steam is a
distribution platform/client, not a console CPU that PhiCade needs to
emulate. Steam's own documented protocol supports `steam://run/<AppID>`:
https://partner.steamgames.com/doc/sdk/api

PhiCade's optional Windows desktop panel accepts one numeric `u32`
App ID and requires a direct user click. The backend independently
checks `operatorApproved`, builds the URI itself, invokes the system
Explorer via an **argv array**, and returns a **handoff-requested**
receipt. It doesn't claim that a game was installed, owned, launched,
rendered, or closed. Windows may prompt for protocol handling; the
installed Steam client controls ownership, licensing, updates, and
launch. Other operating systems explicitly return unsupported.

**No Steam API emulation, DLL replacement, fake licensing,
DRM bypass, unauthorized downloads, Steam credentials, appmanifest
scraping, or automatic game execution.**

## Capability claims

| Capability | Rung 1 status |
| --- | --- |
| Indexed DOS `.dosz` | Implemented; no gameplay qualification |
| Registered DOSBox Pure core route | Existing generic libretro route; pending DOS tests |
| Indexed `.scummvm` hooks | Implemented, uses prior ScummVM runtime |
| ScummVM launcher | Pre-existing qualified integration |
| DOSBox Staging external runtime | Not implemented |
| Steam client App ID handoff | Implemented on Windows, explicit approval |
| Steam game compatibility or ownership detection | Not implemented |
| Steamworks API emulation | Out of scope |

## Rung 2: original DOS VGA fixture qualification

An original 16-bit DOS .COM program lives in fixtures/dos/phitest.asm.
CI assembles it, packages a single-file DOSZ archive, builds a pinned
DOSBox Pure core from upstream source, and runs it through PhiCade's
actual libretro host. The receipt records observed video and audio
callbacks, core and fixture hashes, and retropad host-state changes.

**Still not qualified:** guest keyboard/mouse, DOS-side exit, or real
game compatibility. See [Rung 2 evidence](DOSBOX_PURE_QUALIFICATION.md).

## CI acceptance

- [ ] Classification accepts `.dosz` / `.scummvm` and excludes
      generic archives and executable content.
- [ ] Web/UI typecheck and build.
- [ ] Rust-native tests for approval and numeric Steam URI.
- [ ] No regression to existing ScummVM/SameBoy/libretro paths.
- [ ] Real Windows Steam client handoff is manually observed.
- [ ] Real original DOS fixture plays through DOSBox Pure.
- [ ] No claims of real game compatibility before those validations.

All code work is GitHub-first. Actual game execution necessarily requires
a user's machine and legally possessed game data; CI cannot simulate
a local Steam installation or infer an account's entitlements.
