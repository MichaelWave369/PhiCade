# XR compatibility · evidence first

No physical Quest execution has been performed by this development session.
"Implemented" describes source behavior, not hardware qualification.

| Feature | Implemented | Automated evidence | Field status |
|---|---|---|---|
| Separate 3D arcade + desktop preview | Yes | Type-check/build + browser smoke | Browser verified in Linux CI; Quest pending |
| Interactive cabinets and original games | Yes | Browser target hit, movement; deterministic game tests | Quest controller feel pending |
| WebXR stereo VR / tracked rays | Yes | Build + ray/math fixtures | Quest pending |
| Teleport, snap turn, optional smooth movement | Yes | Compiled | Comfort and boundaries pending |
| Absolute light-gun mapping/calibration | Yes | Screen corners, invalid rays, affine fit, letterboxing, trigger/reload tests | Physical calibration/latency pending |
| Native light-gun emulator input | Yes, port 0/device 4 | Real original libretro fixture scores a hit and emits audio on Linux | Windows DLL/Quest end-to-end pending |
| Dual light guns | Original browser demo | Independent magazines and seats | Native port 2 unsupported |
| NES/SNES/arcade/PSX commercial gun games | Unqualified | None | Core-specific subdevices/options/profiles need verification; no title claim |
| PC → browser video/audio + input return | Prototype | Two-browser video/handshake; native grant tests | Windows Tauri WebView2 ↔ Quest LAN pending |
| Gamepad Quest mapping | Yes | Button/axis/deadzone-boundary fixtures | Native game/controller testing pending |
| Remote game library / ROM launch | Current operator-launched session only | Existing native launch path retained | Guest filesystem browsing/privileged launch unsupported |
| DolphinXR external launcher | Yes, local portable installation | Hash/profile/argument/handedness tests | Actual external runtime and Quest pending |
| Wii Remote/Nunchuk + GameCube Touch | Installed upstream stock profiles | Profile presence/hash gates; config planner tests | Per-game compatibility unverified |
| Browser Wii pose translation | Experimental adapter | Orientation/pointer/analog/gesture fixtures | Not injected into DolphinXR; native OpenXR route used |
| WorldWalk screen/diorama | Original Holler Circuit profile | Deterministic geometry and canonical collision/win oracle | Browser smoke; Quest pending |
| WorldWalk first person | Experimental demo profile | Build + same canonical state | Camera/comfort pending; arbitrary ROMs unsupported |
| Passthrough room/tabletop/head-follow | Capability-gated implementation | Unsupported desktop fallback in browser smoke | Actual AR/hit surfaces pending |
| Two-user shared demo room | Prototype | Room owner/spectator tests + WebRTC browser exchange | Quest co-presence pending; emulator netplay unsupported |
| Phi-Bot participation | Existing Action Bus/AuthorityPolicy preserved | Existing runtime regressions + XR handoff rejection test | External DolphinXR agent input unsupported |
| Exact Replay v1 with lightgun state | Unsupported | Disabled when selecting native gun device | Requires extended input-state checkpoints and qualification |
| GPU-only external N64 video in XR | Unsupported | Existing N64 path untouched | Native window capture is outside this canvas-stream prototype |

## Integration and license research

| Project/source | Finding | Integration choice |
|---|---|---|
| [DolphinXR README](https://github.com/iChris4/dolphinXR/blob/aeb496accb6e2f333e4995b893c01ed04a8d876d/Readme.md) and [COPYING](https://github.com/iChris4/dolphinXR/blob/aeb496accb6e2f333e4995b893c01ed04a8d876d/COPYING) | Native OpenXR on supported desktop/Quest paths; original Dolphin source largely GPL-2.0-or-later; aggregate GPLv3-compatible | Separate user-installed process; preserve its notices and applicable source obligations; no linked/copied implementation |
| [DolphinXR command parser](https://github.com/iChris4/dolphinXR/blob/aeb496accb6e2f333e4995b893c01ed04a8d876d/Source/Core/UICommon/CommandLineParse.cpp), GraphicsSettings, OpenXR controller source and stock profiles at the same revision | Exact `--user`, `--exec`, `--config` syntax, configuration names, device names and profile identities inspected | Typed launch options; locally reuse installed stock profiles; record effective hashes |
| [EmuVR about](https://www.emuvr.net/about/) and [FAQ](https://www.emuvr.net/wiki/FAQ) | A useful existing VR/libretro reference; reviewed pages do not establish a reusable open-source license | No code adoption; no assumption that free download grants source reuse |
| [VirtualBoyGo](https://github.com/CidVonHighwind/VirtualBoyGo) | OpenXR/Vulkan Quest/desktop project; repository identifies GPL-3.0 | Research/reference only; no code linked or bundled |
| [Libretro licenses](https://docs.libretro.com/development/licenses/) and [ABI header](https://github.com/libretro/libretro-common/blob/master/include/libretro.h) | Core licenses vary, including noncommercial restrictions; absolute lightgun device/id contract verified | Existing user-selected core registration stays intact; no universal license or compatibility claim |
| [3dSen developer](https://geodstudio.net/) and [supported profiles](https://geodstudio.net/games/) | Game-specific authored reconstructions, not arbitrary universal conversion | Original explicit semantic demo profile; no proprietary code/assets copied |
| [WebXR hit-test](https://www.w3.org/TR/webxr-hit-test-1/) and [DOM overlays](https://www.w3.org/TR/webxr-dom-overlays-1/) | Optional secure-context browser capabilities | Detect and fall back; do not assume passthrough/placement availability |

Third-party JS packages are locked by version and integrity in `package-lock.json`.
Rust packages are pinned by version/checksum in `Cargo.lock`. New XR dependency
license closures are checked in CI. Three.js is MIT; runtime-owned dependencies
retain their package notices. `@webgpu/types` is CC-BY-4.0; preserve its packaged
copyright/license attribution when distributing dependency sources.
