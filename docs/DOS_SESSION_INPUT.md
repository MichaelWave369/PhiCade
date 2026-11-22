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

## Rung 6: production session API to DOS guest

The same original `PHITEST.COM` is now run twice against the pinned DOSBox Pure
source-built core in CI. Rung 4 uses lab-only keyboard/mouse hooks. **Rung 6 uses
the actual `enable_dos_session_input`, `dos_session_key`, and
`dos_session_mouse` methods enabled in the Tauri desktop build.**

The lab feature is enabled simultaneously only to configure a deterministic
DOSBox Pure shutdown-on-guest-exit policy, not to inject inputs. When both
features are compiled, the host deliberately routes libretro mouse polling
to the **production session state**. The second CI receipt must show:

- Preapproval keyboard/mouse delivery was denied.
- An explicit session approval enabled the bounded API.
- Forbidden F11 and out-of-range mouse deltas were denied.
- Disabling approval revoked key/mouse commands; explicit reapproval restored it.
- The source-built DOS guest painted magenta, then green after ESC via BIOS
  `int 16h`, then cyan after a left-click via DOS `int 33h`.
- Q triggered the guest exit and DOSBox Pure's shutdown callback.
- Physical Windows input and retail DOS game compatibility stay `false`.

**The Tauri UI is not running inside this CI test.** It proves production
libretro input methods and a shared DOS fixture. Tauri's human-only session ID
guard has separate unit tests, and all changes require the existing native
compile and browser tests. Hardware focus, controller and Windows device tests
still require an operator at the desktop.
