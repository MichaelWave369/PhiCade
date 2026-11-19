# XR implementation and qualification report

Date: 2026-11-19. Release status: **software prototype; not hardware-qualified**.

## Completed evidence

| Check | Result | Scope |
|---|---|---|
| Both frontend TypeScript builds | PASS locally | Existing desktop + separate XR app |
| Both production bundles | PASS locally | Desktop and static XR build |
| XR deterministic adapter tests | PASS locally, 15 tests | Aim mapping, invalid/behind rays, calibration, letterboxing, magazines, canonical tiles/collision/win, room authority, motion, gamepad, shared wire fixture |
| Rust runtime/libretro/XR package tests | PASS locally | Existing runtime regressions + grant, input, profile and launch-plan tests; exact count recorded by CI |
| Rust formatting and scoped strict Clippy | PASS locally | New XR Rust modules/crates; existing unrelated formatting not rewritten |
| Original dynamic libretro core | PASS locally on Linux | Paired grant → Action Bus authority → real core ABI → visible target hit + audio; brief fire edge retained for one frame |
| Npm dependency audit | PASS, zero findings at review time | Locked JS dependency graph |
| JS/Rust XR license closures | PASS locally | Versions/checksums/integrity pinned; approved license expressions |
| Foundation browser smoke | PASS in Linux GitHub Actions | Target shooting/reload, tile-game movement/presentation, unsupported-XR fallback, invalid pairing rejection |
| Foundation two-browser WebRTC | PASS in Linux GitHub Actions | Offer/answer identity, decoded video, room/controller status and disconnect |

Foundation browser evidence:
[XR run 44623161919](https://github.com/MichaelWave369/PhiCade/actions/runs/44623161919),
commit `8f018a8e0279030eda7e1ff97b42d49d7ac357e2`, three browser tests passed.
The artifact `xr-browser-evidence` includes the built frontend, HTML report and
desktop screenshot. The patched-dependency foundation head is
`01eebdc69c023127a95ee26c6f7ff5d2d23bb1bc`; checks are tracked on PR #65.

The integration branch extends the browser test to require an actual guest-fired
target hit over WebRTC and to check host spectator input ownership. Its separate
Windows job compiles the Tauri shell, builds the original homebrew DLL and runs
the same ABI qualifier. The run/commit evidence is tracked on its integration PR;
pending jobs are not counted as passes.

## Milestones and changed areas

| Milestone | Review | Changed areas |
|---|---|---|
| Foundation, phases 1/3/5/6/7 | [PR #65](https://github.com/MichaelWave369/PhiCade/pull/65) · `8f018a8` + dependency fix `01eebdc` | `apps/xr`, shared `xr` protocol/adapters/demo/room code, XR Rust protocol crate, lockfiles, browser/unit CI, quickstart |
| Native integration, phases 2/3/4/8 | Dependent integration PR | Desktop XR Host and DolphinXR panels; native grant ingress; libretro gun state and original C core; external runtime planner; qualification/packaging/docs |

The independent software pieces are implemented through all eight phases. The
acceptance gates that need Windows/Quest execution remain open.

## Pending or limited

- **Quest hardware:** stereo session entry, tracking loss, controller hands,
  calibration accuracy, haptics, teleport/snap-turn comfort, physical latency,
  passthrough/hit-test/tabletop placement and two-headset co-presence.
- **Windows stream:** real Tauri WebView2 canvas/video/audio capture over LAN to
  Quest, firewall behavior, GPU encode/decode, recovery after headset sleep.
- **DolphinXR:** actual installed runtime launch, native OpenXR session,
  Wii/GameCube homebrew/game input and per-title configuration. A process-spawn
  receipt is not evidence of playable VR. No commercial title is qualified.
- **WorldWalk:** original semantic demo only; first-person comfort is experimental.
  Arbitrary-ROM reconstruction, depth extraction and external-game cameras are
  unsupported.
- **Native topology:** one playable port; no commercial gun-core subdevice profiles,
  native two-player guns, lightgun Replay v1, external N64 capture or emulator netplay.
- **Remote launch:** the host shares its current approved game; peers cannot choose
  local files or execute privileged launch/system operations.

## Environment constraints and resolved failures

Local browser automation could not create Chromium's required Unix sockets. The
cloud browser could not reach this workspace's loopback server. Browser checks
were therefore executed by GitHub Actions, not substituted with a simulation.
The local Tauri check stopped at missing GLib/GTK/WebKit development libraries;
system package installation was unavailable to this executor. Linux/Windows
native checks are supplied by repository CI. No local native-shell pass is claimed.

An initial Vite/plugin type conflict was fixed by using one compatible Vite line.
The dependency audit then identified outdated Vite/Vitest packages; the selected
patched versions now audit cleanly. A failed license inventory initially required
reviewing the exact legacy dual-license expressions for memchr/version_check and
unicode-ident's Unicode attribution. Those expressions are now explicit in the
license checker. The checks do not authorize copying proprietary game assets.
