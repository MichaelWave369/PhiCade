# PhiCade XR prototype

Old worlds. New players. New dimensions.

## Run the arcade

Use Node 22+ (24 recommended):

```sh
npm ci
npm run xr
```

Open http://localhost:1430 on the development PC. Photon Range and Holler Circuit
are original MIT games included in source. No ROMs or BIOS are needed. Pick a
cabinet, use the mouse to aim, click to fire, R to reload, WASD to walk, and drag
the scene to look. Holler Circuit uses arrow keys to collect three lights before
the violet exit. WorldWalk selects its spatial screen, diorama or experimental
first-person presentation; all use the same collision and game state.

## Quest

Serve the production `apps/xr/dist` folder over **HTTPS with a certificate trusted
by the Quest browser**. A LAN HTTP address does not qualify as a secure WebXR
context. Do not disable certificate verification. Build with `npm run xr:build`.
Use Enter VR from the headset browser. Quest execution is pending hardware QA.

Trigger aims/fires; offscreen trigger or the primary controller face button
reloads. Squeeze, point at a floor location and release to teleport. Right stick
snap-turns. Smooth movement is optional. The in-world menu selects cabinets,
theater, reload, WorldWalk modes and exit without relying on DOM overlays.
Handedness, dual demo guns, render scale, aim guides and CRT appearance are in
the browser controls. Aim calibration uses three displayed crosses and is saved
locally; recalibrate after moving/resizing the screen.

Mixed reality requests `immersive-ar`, optionally hit-test and DOM overlay. If AR
is unavailable but VR exists, it falls back to VR. The Place screen control places
the panel ahead of the viewer; a supported surface hit plus squeeze-release places
it in the room. Screen scale and head-follow are optional. Stay aware of furniture
and the headset's boundary. No room scan is uploaded or persisted.

## Shared demo room

Open Connect a PC or share this room. Host demo room creates an invitation for
one guest. On the guest browser paste that offer and Create answer, then paste the
answer back on the host and Accept guest. The host spectates and assigns its
Photon Range input to the guest. Head/controller transforms are synchronized;
this is a two-browser room, not emulator netplay.

Pairing codes carry session access and local network candidates. Exchange them
directly with your intended guest. There is no discovery server, public STUN/TURN
server or account service. Reconnect by creating a fresh offer and answer.

## Checks

```sh
npm run typecheck
npm run test:xr
npm run build
npx playwright install chromium
npm run test:xr:browser
cargo test -p phicade-runtime -p phicade-libretro -p phicade-xr-protocol --all-targets
```

See XR_ARCHITECTURE.md, XR_COMPATIBILITY.md and XR_TEST_REPORT.md for the native
bridge, external runtime integration and evidence limits as those phases land.

## Play a native PC game on the paired screen

1. Build/run the desktop application as usual (`npm run desktop`). Register a
   trusted libretro core and launch your own game through PhiCade's existing Play
   approval. GPU-only external N64 windows are not part of this canvas stream.
2. Open **XR HOST** above the desktop CRT. Choose controller or spectator. Only
   enable the light-gun device for a core/profile you are deliberately testing.
3. Select capture width, FPS and bitrate. Click **APPROVE XR SESSION + CREATE
   OFFER**. This grants input to this exact running session; registration is not
   a permanent input grant.
4. In the headset's HTTPS arcade, open the connection panel, paste the offer and
   click Create answer. Paste the answer into the PC and Accept paired headset.
5. Select **YOUR PHICADE**, then its theater or cabinet screen. Enable stream audio
   if the browser pauses playback. For gamepad mode: right face buttons A/B, left
   face buttons X/Y, left stick as D-pad and analog, right stick as analog,
   right stick click Start, left stick click Select, triggers L2/R2 and grips L/R.
6. Revoke from the PC to stop input and release held controls. A stalled input
   stream releases after 300 ms. Failure/sleep/relaunch needs a fresh offer and
   answer. A control-mode change revokes the native XR grant.

Keep the headset and PC on the same reachable LAN; guest-network client isolation
can prevent connection. Use the PC firewall's normal application permissions; do
not expose a privileged port or forward emulator controls to the internet. The
prototype has no public signaling/STUN/TURN service. An HTTPS arcade page can
carry a paired WebRTC stream without exposing Tauri commands as network routes.

Windows WebView2 canvas capture, audio and Quest browser interoperability still
need field qualification. The completed browser tests cover Linux Chromium peers,
not that Windows/headset combination. Only the current locally launched game is
shared; the peer cannot browse local paths or launch arbitrary content.

## Native original light-gun demo

On Linux/macOS with a C compiler and Rust:

```sh
bash scripts/qualify-xr.sh
```

This compiles an original no-content core, runs the native qualifier and writes
`artifacts/xr-lightgun-qualification.json`. On macOS change the output library
extension/flags if required by your compiler. On Windows, use a Visual Studio
Developer Command Prompt:

```bat
mkdir artifacts
cl /nologo /std:c11 /LD xr\fixtures\photon-range-core.c /Fe:artifacts\photon_range_libretro.dll
cargo run -p phicade-libretro --bin xr_lightgun_qualify -- artifacts/photon_range_libretro.dll
```

The Windows XR workflow also builds this original DLL and publishes its SHA-256
as an artifact. Register the DLL in PhiCade, approve **RUN LAUNCHER** (no game file),
then pair XR Host with light-gun enabled. Aim at the center ring. Offscreen trigger
or reload should restore the magazine; the rendered score bar grows on a hit.
The browser demo has optional swing/shake reload gestures and two independent
magazines. Native host port 2 and commercial gun-game profiles remain unqualified.

## DolphinXR Wii/GameCube path

Obtain a portable OpenXR-enabled DolphinXR installation yourself from its official
project, retaining its notices and source information. In the desktop:

1. Open **DOLPHINXR** and select its executable. Keep its `Sys` directory beside
   it, including `Profiles/Wiimote/OpenXR Wii Remote.ini` and/or
   `Profiles/GCPad/Quest Touch GameCube.ini`.
2. Review the executable hash and available profile. Pick your own Wii/GameCube
   disc dump or original homebrew `.dol`/`.elf`.
3. Choose Wii Remote + Nunchuk or GameCube Touch, handedness, bounded scale/motion
   pitch and optional spatial screen/passthrough. Confirm that this is an
   OpenXR-enabled build, then approve this specific launch.
4. Exit browser VR before starting native OpenXR. Ensure your normal Meta Link,
   SteamVR or other supported OpenXR runtime is active and the Quest is connected.
5. Verify the actual native session, pointer, motion, sticks and buttons. Adjust
   game-specific swing/shake/pointer bindings in DolphinXR's own controls UI. Each
   game/profile has an isolated PhiCade user directory shown in the launch notice;
   later manual edits are preserved. Close and save through DolphinXR itself.

An installation/profile hash check is not hardware or game qualification. If a
profile is absent, the binary changes, or a setting is invalid, launch is refused.
No game is listed as tested until real runtime evidence exists. The browser motion
adapter is separately tested and does not inject poses into DolphinXR.

## Production packaging

`npm run xr:build` outputs a standalone static site in `apps/xr/dist`, with relative
asset URLs so it can live in a subdirectory. The existing GitHub Pages workflow
also packages it under `xr/` alongside the Xbox gateway **after these changes are
merged to main by the owner**. No main-branch merge or production deployment is
performed by these PRs. The XR CI artifact includes the built site, browser report
and screenshots for pre-merge review.
