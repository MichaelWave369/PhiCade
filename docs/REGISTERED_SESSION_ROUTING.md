# Registered Session Routing v1

Rung 37 routes registered libretro runtimes into normal PhiCade sessions without
re-introducing a SameBoy-shaped assumption at the session layer.

The routing flow is:

```text
runtime registration SHA
        |
        v
load registration receipt
        |
        +-- re-hash local core binary
        +-- reject stale or mutated binary
        +-- verify runtime identity still matches registration
        +-- require explicit per-session operator approval
        |
        v
open registered libretro runtime
        |
        +-- FILE content
        |      or
        +-- no-content launcher
        |
        v
derive live capability manifest
        |
        v
capability-aware PhiCade session
```

## Native command

`start_registered_emulation(coreSha256, contentPath?, operatorApproved)`

The command refuses to start unless `operatorApproved=true`.

Registration remains provenance/inventory. Operator approval is scoped to the
individual session and does not mutate the registration receipt into a blanket
launch grant.

## Stale binary protection

Before launch PhiCade canonicalizes the registered path and recomputes the local
binary SHA-256.

If the observed hash differs from the registration digest, launch fails closed.

PhiCade also re-opens the runtime and checks that its reported library name and
version still match the stored registration identity.

## Content modes

Registered libretro sessions currently support:

- `FILE` content through Content Descriptor v1;
- no-content launcher mode when the core explicitly requested libretro
  no-game support.

Directories and runtime-native launch targets remain outside the libretro
adapter contract for this rung.

For ScummVM this means the registered runtime can start its launcher with no
game data or can later be offered a user-supplied file such as a `.scummvm`
hook or a file inside a valid game folder.

## Capability-aware session features

The live manifest determines whether timeline features are enabled.

### SameBoy

The pinned SameBoy profile keeps:

- state snapshots;
- rewind;
- save/load state;
- exact Replay v1;
- persistent save RAM.

### ScummVM

The pinned ScummVM profile keeps:

- governed frame stepping;
- framebuffer/audio;
- governed RetroPad/analog actions;
- launcher/content execution.

It explicitly does not receive:

- rewind;
- save/load state;
- Replay v1 recording;
- Replay v1 verification;
- libretro save-RAM persistence.

Attempts to use unsupported timeline actions fail with an explicit capability
error instead of crashing later.

## Qualification evidence

A registration may match a named qualification profile while still carrying
`binaryEvidenceBound=false`.

Session routing does not silently upgrade that evidence state.

The session response therefore carries both:

- the live capability manifest;
- route evidence including registration SHA, binary-evidence binding state,
  stored registration authority state, and the explicit session operator
  approval.

Functional use and scientific qualification remain separate claims.

## Legacy compatibility

The existing SameBoy `start_emulation(corePath, gamePath)` command remains
available for the current desktop UI and benchmark path.

It now also derives runtime features from the capability manifest so the same
session machinery is used on both legacy and registered routes.
