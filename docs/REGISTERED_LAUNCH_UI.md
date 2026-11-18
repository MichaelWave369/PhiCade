# Registered Launch UI v1

Rung 39 connects the desktop Runtime Manager to the registered-session route
introduced in Rung 37.

The operator can now select a registered libretro runtime and launch either:

- its no-content launcher, when the core supports libretro no-game mode; or
- a user-selected FILE through Content Descriptor v1.

## Operator approval

Registered runtime launch is deliberately not automatic.

The desktop requires the operator to:

1. select a registered runtime;
2. explicitly enable **APPROVE THIS SESSION LAUNCH**;
3. choose **RUN LAUNCHER** or **RUN FILE…**.

The desktop then calls:

`start_registered_emulation(coreSha256, contentPath?, operatorApproved=true)`

Approval is cleared after a successful start.

This does not mutate the durable registration receipt. A registration can still
carry:

`authorityGranted=false`

while the individual session records:

`sessionOperatorApproved=true`

Those are different scopes.

## Runtime integrity

The desktop selects by registration SHA-256, not by a free-form path.

The native backend still performs the authoritative checks:

- load the stored registration receipt;
- canonicalize the stored core path;
- re-hash the binary;
- fail closed if the hash changed;
- reopen the core;
- verify runtime name/version against registration;
- derive the live capability manifest.

The UI does not bypass or duplicate those trust checks.

## Capability-aware controls

The active SessionInfo now drives visible control availability.

If the runtime does not support state snapshots:

- save-state is disabled;
- load-state is disabled;
- rewind is disabled;
- rewind telemetry shows N/A.

If the runtime does not support exact replay:

- Replay v1 recording is disabled;
- replay verification is disabled;
- telemetry shows UNSUPPORTED/N/A.

If the runtime does not expose libretro persistent save data:

- FLUSH SRAM is disabled.

For the pinned ScummVM profile this means normal governed frame/audio/input
play remains available while snapshot/replay/save-RAM controls are visibly
unavailable.

## Legacy SameBoy path

The existing GB/GBC **LOAD / RUN** route remains intact.

This rung adds a second desktop launch path instead of silently replacing the
qualified SameBoy workflow.

## Scope

Rung 39 does not:

- auto-download runtimes;
- bundle ScummVM;
- bundle game content;
- grant registration-wide authority;
- claim exact replay for ScummVM;
- add directory or runtime-native launch-target routing to libretro.

Those remain separate capability/evidence decisions.


## Arcade Play (desktop, user-approved per session)

The game list now has a direct **PLAY GAME** button, or the operator can
double-click a game row. This explicit operator gesture selects a registered
libretro core whose live-advertised `validExtensions` contains the game's file
extension, using a stable preference order for familiar consoles and honoring
an already-selected matching runtime. A GB/GBC game can alternatively use the
existing legacy SameBoy path when no matching runtime is registered.

The operator's PLAY gesture approves only that one selected game's session.
It does not grant durable registration authority, skip native SHA-256 / identity
verification, install a core, download content, or grant an agent authority.
A failed or missing runtime produces an actionable notice.

`F11` or `Alt+Enter` toggles native-window arcade fullscreen while a game
is running. The dashboard is hidden, video stays at 4:3, and F11, Alt+Enter,
Escape, or the small exit button returns to the dashboard.

Arcade Play does not guarantee every Libretro core works: advertised extensions
are routing hints and native loading/capability checks remain authoritative.
