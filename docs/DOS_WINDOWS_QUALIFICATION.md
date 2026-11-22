# PhiCade DOS Rung 7 | Windows hardware qualification kit

**Status:** Windows CI regression + operator-facing manual evidence form.
**Not** an automatic physical-device qualification, arbitrary DOS game
compatibility claim, or standalone emulator installer.

## What's new

The desktop DOSBox Pure session displays a collapsed **DOS HARDWARE CHECK**
panel. The operator can mark each observation as Not tested, Observed pass,
or Observed failure, and generate/copy a local JSON report. It does not
capture keystrokes, device IDs, Steam information, game paths, credentials,
machine serials or personal information. No report is sent anywhere.

The report is tagged `phicade.dos.windows.manual-hardware.v1` with
`evidenceBasis: operator-self-reported`,
`automatedHardwareTest: false`, and
`physicalHardwareVerifiedByCI: false`. Even six operator-reported passes
do not automatically qualify all DOS games. Each game needs its own testing.

## Six manually observed checks

1. Keyboard: approve DOS input, focus CRT, verify key action in a DOS
   program with working keyboard handling.
2. Mouse click: verify that a DOS mouse-aware program receives left click.
3. Mouse motion: verify guest-relative motion, not the desktop cursor.
4. Controller: test D-pad, A/Space, B/Escape, Start/Enter in a DOS program.
5. Focus loss: hold a mapped key, Alt+Tab, and return. Ensure approval is
   revoked and no key remains stuck. Fresh approval must be required.
6. Replacement: eject a game, start a new DOS session and verify that
   the old approval does not authorize input to the new game.

Mark any check that can't be meaningfully tested as **Not tested**, rather
than guessing it passed. Preserve observed failures in the report for
reproduction. You can paste the JSON into a future GitHub issue or PR
review if you choose, but it is not uploaded automatically.

## Automated CI (not a physical keyboard test)

`.github/workflows/dos-hardware-windows.yml` runs on
`windows-latest` and executes:

- Pure mapping tests for keyboard/gamepad and bounded mouse deltas.
- Event-source tests for window blur, hidden document, listener cleanup,
  immediate revocation if already hidden, and one-shot revocation.
- Manual-report serialization tests requiring untested defaults, explicit
  failure support and false automated-hardware claims.
- Desktop TypeScript/build verification with the real panel imported.
- Native Tauri Rust `DosInputGuard` policy tests for stale ID, revoked
  input, non-human authority, replay and autonomous controls.

These tests simulate input events and check OS-specific compilation;
they do **not** drive physical peripherals or the user's Windows WebView.
The older source-built DOSZ guest CI continues independently to qualify
the emulator's production input API.

## Safety and lifecycle

The DOS session approval remains an explicit human action. OS window blur
or hidden window revokes the permission once and requires reapproval.
A game switch, core stop and non-human authority handoff remain guarded
by the existing backend session checks.

No new Tauri commands, file-system privileges, external network services,
game distribution, automatic install or agent powers are introduced.

## Next rung

Run a real Windows hardware smoke test on the operator's machine using
a self-owned, compatible DOS program. Collect an actual manually generated
report, assess failures, then build deterministic frontend-to-Tauri
integration tests for any gaps. CI alone is not that report.
