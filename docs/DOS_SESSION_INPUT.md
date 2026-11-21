# PhiCade Rung 5 | DOS desktop input

**Status:** experimental session-scoped desktop feature. Original DOS fixture
qualification through Rung 4 is established; hardware gameplay with commercial
titles is NOT yet qualified.

## Design
- Use a registered DOSBox Pure core and select user-owned \`.dosz\` content.
- Click **PLAY GAME** (existing runtime approval), then click **ENABLE DOS
  INPUT (HUMAN)**. This is a second, distinct per-session approval.
- PhiCade issues a monotonically increasing session ID. Every input command
  must carry this exact ID. Old input cannot affect a new game session.
- Only when the focused CRT receives keyboard/mouse input will the browser
  call its Tauri backend. No global keyboard hook or OS-level input injection.
- Keys: arrows, Escape, Enter, Space, Backspace, W/A/S/D/Q. No shell commands,
  arbitrary virtual key codes, Meta/Alt/Ctrl, or text injection.
- Gamepad mapping while the bridge is active: D-pad → arrows; A → Space;
  B → Escape; START → Enter; SELECT → Q. Prevent duplicate retropad delivery.
- Mouse: bounded relative movement \`[-64,64]\` per request, left button only;
  movement pulse consumed once per emulated frame.
- Mouse leave, CRT blur, disable, session end or core drop neutralizes input.
- Changing authority away from HUMAN disables the backend bridge.
- No direct path from Phi-Bot, replay, XR host, autodrive, or source catalog
  to DOS input commands. API requires active HUMAN mode, no replay recording,
  and no active autodrive.

## Implementation
- \`crates/phicade-libretro/src/lib.rs\` feature \`dos-session-input\` is
  enabled by the Tauri app. Separate \`dos-lab-input\` feature remains for CI.
- \`src-tauri/src/lib.rs\` verifies registered DOSBox Pure, actual DOSZ content,
  session epoch and operator approval.
- \`apps/desktop/src/App.tsx\` exposes opt-in controls only during an active
  DOSBox Pure session; CRT-focus events and controller bindings are visible.
- \`apps/desktop/src/dosInput.ts\` holds pure mappings with Vitest coverage.

## Acceptance
- [ ] Rust library unit tests pass with \`dos-session-input\`.
- [ ] Desktop TypeScript build and native Tauri compile green.
- [ ] Existing DOS lab, ScummVM, SameBoy, XR and PixelForge CI remain green.
- [ ] Manual Windows hardware proves no stuck keys on focus loss.
- [ ] Real mouse movement and controller mapping tested with original DOS fixture.
- [ ] Input revoked on replacement, stop, human-to-agent handoff and game exit.

**Do not promote DOS gameplay generally until physical keyboard/mouse and
replay/authority tests have been observed on Windows.** No private assets,
games or binaries are included in this feature.
