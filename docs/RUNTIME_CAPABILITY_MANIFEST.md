# Runtime Capability Manifest v1

PhiCade runtime adapters must describe what they can actually provide instead of
being forced into one fake-common emulator interface.

Schema:

`phicade.runtime-capability-manifest.v1`

## Status levels

Every capability is one of:

- `UNSUPPORTED` — the adapter does not provide the capability.
- `SUPPORTED` — the adapter exposes the capability, but PhiCade does not claim
  that the current runtime/profile has passed the relevant qualification.
- `QUALIFIED` — the capability belongs to a named qualification profile and
  is expected to be backed by matching source/binary evidence.

`QUALIFIED` is not a substitute for receipt verification. A runtime claiming the
same name/version as a qualified profile does not become trusted automatically.
The exact binary/hash evidence remains authoritative.

## Execution models

v1 recognizes four execution models:

- `EMBEDDED_FRAME_CORE`
- `EXTERNAL_PROCESS`
- `WEB_RUNTIME`
- `BRIDGED_RUNTIME`

This lets PhiCade describe emulators, external compatibility runtimes, browser
engines, and semantic bridges without pretending they share identical control
surfaces.

## Capability fields

| Capability | Meaning |
| --- | --- |
| `frameStep` | PhiCade can advance the runtime on an explicit frame boundary. |
| `renderedFramebuffer` | PhiCade receives rendered pixels as an observation surface. |
| `audioStream` | PhiCade receives runtime audio samples. |
| `governedActions` | Inputs can pass through PhiCade's authority-resolved action path. |
| `reset` | The runtime exposes an explicit reset operation. |
| `stateSnapshots` | Runtime state can be serialized and restored. |
| `exactReplay` | PhiCade has a qualification profile for deterministic replay of governed actions. |
| `persistentSaveData` | Persistent game/runtime save data can be read or written. |
| `gameDetection` | The runtime can identify compatible game/content data. |
| `externalProcessLifecycle` | PhiCade can start/stop/observe an external runtime process. |
| `semanticEvents` | The runtime exposes structured game/runtime events beyond pixels/audio. |

## Core-neutral baseline

The `EmulatorCore` trait itself guarantees only:

- frame stepping;
- rendered framebuffer output;
- audio output;
- governed action input;
- reset.

All stronger claims default to `UNSUPPORTED`. Adapters must opt in explicitly.

## Generic libretro probing

For unqualified libretro cores, PhiCade does not infer optional capabilities
from ABI symbol presence alone.

After content is loaded:

- a non-zero `retro_serialize_size()` promotes `stateSnapshots` to SUPPORTED;
- non-zero libretro save RAM promotes `persistentSaveData` to SUPPORTED;
- `exactReplay` remains UNSUPPORTED unless a separate qualification profile
  proves it.

Before those behaviors are observed, the optional capability remains
UNSUPPORTED.

This distinction is important for cores that implement the standard entry
points but intentionally return no serializable state.

## SameBoy / libretro profile

The current pinned SameBoy 1.0.3 profile declares:

| Capability | Status |
| --- | --- |
| frame step | QUALIFIED |
| rendered framebuffer | QUALIFIED |
| audio stream | SUPPORTED |
| governed actions | QUALIFIED |
| reset | SUPPORTED |
| state snapshots | QUALIFIED |
| exact replay | QUALIFIED |
| persistent save data | SUPPORTED |
| game detection | UNSUPPORTED |
| external process lifecycle | UNSUPPORTED |
| semantic events | UNSUPPORTED |

Qualification profile:

- profile: `phicade.sameboy-1.0.3-qualified.v1`
- source revision: `213a12ce93d66b105a113debd9396306066a7cfc`
- exact binary evidence required: **true**

The core qualification receipt embeds the manifest beside the exact core SHA-256.

## Why this exists

Runtime integration now becomes capability negotiation instead of interface
fiction.

A future external ScummVM provider could honestly begin approximately like this:

```text
executionModel            EXTERNAL_PROCESS
frameStep                 UNSUPPORTED
renderedFramebuffer       UNSUPPORTED
audioStream               UNSUPPORTED
governedActions           SUPPORTED
reset                     SUPPORTED
stateSnapshots            UNSUPPORTED
exactReplay               UNSUPPORTED
persistentSaveData        SUPPORTED
gameDetection             SUPPORTED
externalProcessLifecycle  SUPPORTED
semanticEvents            UNSUPPORTED
```

Those values are illustrative until the ScummVM adapter exists and is tested.
The important rule is that ScummVM would not be required to imitate SameBoy.

A future PixelForge bridge can likewise declare semantic events or web/bridge
execution without claiming emulator-specific state semantics.

## Trust rule

**Capability declaration is not authority.**

A manifest says what an adapter exposes. Qualification receipts say what exact
artifacts were tested. PhiCade governance still decides what the runtime is
allowed to do.
