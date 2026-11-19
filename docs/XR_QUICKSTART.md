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
