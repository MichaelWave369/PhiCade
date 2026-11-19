# PhiCade XR architecture · prototype v1

The desktop runtime remains the emulator host. `apps/xr` is a second frontend,
not an emulator replacement. Existing ROM discovery, registration, binary identity,
Phi-Bot, PixelForge, replay and N64 window routes stay in place.

| Boundary | Code | Contract |
|---|---|---|
| Headset presentation | `apps/xr/src/main.ts` | Three.js/WebXR scene; no native filesystem or emulator handle |
| Controller adapters | `xr/lightgun`, `xr/interaction`, `xr/motion` | Bounded normalized button/axis values; explicit profiles |
| Transport | `xr/bridge/peer.ts` | Manual SDP/ICE pairing; DTLS WebRTC video, audio and data; no public STUN/TURN |
| Desktop media host | `apps/desktop/src/XrHostPanel.tsx` | Capture a copy of the existing rendered canvas and Web Audio mix |
| Native pairing grant | `crates/phicade-xr-protocol` + `src-tauri/src/xr_host.rs` | Session-bound token, peer identity, input/device allowlist, monotonic sequence, bounded queue |
| Emulator authority | existing `step_emulation` + `AuthorityPolicy` | XR actions join the existing human seat, are re-sequenced and checked with all live actions |
| Light-gun core ABI | `crates/phicade-libretro/src/lightgun.rs` | Port 0, absolute screen X/Y, offscreen, trigger and reload |
| External VR runtime | `crates/phicade-xr-bridge` + `src-tauri/src/dolphinxr.rs` | Local inspected DolphinXR executable, re-hashed at launch, exact arguments, isolated user profile |
| Room state | `xr/multiplayer/room.ts` | Host creates identities and assigns the single cabinet owner; peers publish transforms only |

## Pairing and authority

The operator launches the game through existing PhiCade approvals, opens XR Host,
chooses spectator or input and explicitly creates an offer. The offer contains a
fresh invitation, room identity, assigned role and WebRTC fingerprint/candidates.
The answer must match that invitation. A controller role additionally creates a
48-hex-character native grant using browser cryptographic randomness, held by the
local host. It is never sent to the peer. Guest messages cannot choose an action
source, seat, local path, command, runtime, or grant. SDP codes are sensitive
session invitations; there is no authentication HTTP endpoint to expose.

Native ingress only queues known `button` and `axis` actions. Each game session
owns its own gateway; stop/relaunch drops the old grant. Control-mode changes revoke
it. It expires after 30 minutes, accepts at most 32 actions/message, 180
messages/second and 256 queued actions, and releases held inputs after 300 ms of
silence. Disconnect/revoke also neutralizes inputs. A Phi-Bot grant remains
independent; a remote controller does not become a bot or bypass handoff policy.

WebRTC uses one ordered data channel and local ICE candidates. Re-pair with new
codes after failure, sleep or revocation; there is no silent persistent
reconnection authority. Cross-network/NAT traversal is unsupported in this rung.
Times in data packets are informational. RTT is measured by returning the same
sender timestamp; clocks are not incorrectly assumed to be synchronized.

## Media and input

The host captures a bounded canvas copy at 640/960/1280 width and 30/60 FPS with a
4/8/16 Mbps encoder target. Capture never resizes the emulator canvas. It adds
the existing 48 kHz Web Audio output stream, with no microphone or desktop-screen
capture permission. The guest fits decoded video inside the virtual display.
Light-gun coordinates account for letterbox/pillarbox margins, affine
calibration, offscreen status and the libretro -32768 sentinel. Gamepad snapshots
and light-gun packets use separately advertised device capabilities.

The native libretro host currently exposes one playable input port. Dual guns are
implemented in the original browser demo, not aliased onto a second native port.
Light-gun selection disables inherited exact Replay v1 claims for that session:
the existing checkpoint format records a joypad mask, not full gun state. A later
qualified replay schema must bind all absolute input state before making stronger
claims. This does not alter ordinary gamepad replay behavior.

The included `xr/fixtures/photon-range-core.c` is an original MIT no-content core.
The qualifier loads its real dynamic-library ABI, configures lightgun device 4,
submits authority-accepted actions, and checks visible score pixels and hit audio.
It provides reproducible proof without proprietary game data.

## WorldWalk, motion and mixed reality

Holler Circuit is an original tile game with a frozen semantic profile,
`phicade.holler-circuit.v1`. Screen, diorama and first-person modes consume the
same map/player/coin state. Reconstruction is deterministic and rejects unknown
profiles. No arbitrary-ROM tile extraction, memory reading or proprietary 3dSen
code is present. First-person camera behavior is experimental; canonical grid
collision remains authoritative.

The browser motion adapter computes orientation, a sensor-bar approximation,
Nunchuk values and cooldown-bounded swing/shake gestures from valid tracked poses.
The arcade can use optional swing/shake reload gestures. These samples are not
injected through a fabricated Dolphin network API. DolphinXR owns native OpenXR
tracking, pointer/motion processing and rumble for its separate process.

Mixed reality requests `immersive-ar` with optional hit tests and DOM overlays.
If unsupported, VR or desktop remains available. The screen can be placed ahead
of the viewer or on a hit surface, resized, world locked or head following.
No spatial-map persistence or upload is implemented. Headset scene/passthrough
behavior requires hardware validation.

## DolphinXR integration

The inspected upstream reference is
`iChris4/dolphinXR@aeb496accb6e2f333e4995b893c01ed04a8d876d`.
It is reference evidence, not a claim that an arbitrary selected binary was built
from that commit. The local executable and installed stock controller profiles
are hashed; stock-profile presence only makes an installation a candidate. The
operator explicitly confirms an OpenXR build before launch. No installation or
binary download is automatic.

The launcher recognizes stock `OpenXR Wii Remote.ini` and
`Quest Touch GameCube.ini` in the portable installation. It reuses the user's
installed configuration locally, records the effective per-game config hash and
preserves later local edits. Wii Source 3 selects upstream's dedicated OpenXR
source. Verified CLI configuration names use the `Graphics.VR` namespace.
Only bounded scale/pitch, spatial-screen and passthrough settings are exposed.
The optional upstream agent server is explicitly disabled. There is no Phi-Bot
control grant or browser-pose injection into DolphinXR in this release.

The launch receipt proves a process was spawned with a bound binary/game/profile
hash and exact argument vector. It does not prove a game initialized, controllers
worked, or a headset entered VR. Close and save from DolphinXR's own UI; no abrupt
process termination is exposed to network peers.
