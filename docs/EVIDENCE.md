# PhiCade Evidence and Qualification

PhiCade separates **functionality**, **registration**, **authority**, and
**qualification**.

A component does not become trusted merely because it loads successfully or
reports a familiar name.

## Core rule

```text
controller proposes
       |
       v
 governed action seam
       |
       v
runtime owns authority
       |
       v
observable execution + receipts
```

Human input, scripts, replay, and agent controllers enter through governed
action paths. Runtime state transitions remain runtime-owned.

## Runtime capability semantics

Runtime adapters publish a Runtime Capability Manifest with per-capability
status:

- `UNSUPPORTED`
- `SUPPORTED`
- `QUALIFIED`

A weaker runtime is not inflated into a SameBoy-shaped interface merely because
the desktop would prefer convenient buttons.

The active desktop session uses the live manifest to decide whether replay,
snapshots, rewind, and persistent save controls should even be available.

See `RUNTIME_CAPABILITY_MANIFEST.md`.

## Registered does not mean qualified

Runtime Registration v1 fingerprints a user-supplied libretro core and records:

- canonical local path;
- exact SHA-256;
- reported runtime identity;
- capability manifest;
- matching qualification profile, when any;
- binary-evidence binding state;
- authority state.

A new registration begins with:

```text
binaryEvidenceBound = false
authorityGranted     = false
```

Matching a known runtime name/version is not proof that the selected binary is
the binary that generated qualification evidence.

See `RUNTIME_REGISTRATION.md`.

## Registered launch authority

The desktop Runtime Manager can select a registered core, but launching it
requires explicit per-session operator approval.

That produces a different scope of evidence:

```text
registration.authorityGranted = false
session.sessionOperatorApproved = true
```

The native route then re-hashes the binary and re-verifies its stored identity
before starting FILE content or a supported no-content launcher.

See:

- `REGISTERED_SESSION_ROUTING.md`
- `DESKTOP_RUNTIME_MANAGER.md`
- `REGISTERED_LAUNCH_UI.md`

## SameBoy

PhiCade has a pinned qualified SameBoy 1.0.3 GB/GBC runtime profile.

For the qualified SameBoy path, PhiCade can support:

- frame stepping;
- rendered framebuffer;
- governed inputs;
- state snapshots;
- rewind;
- save/load state;
- exact Replay v1;
- persistent save RAM.

Replay evidence is tied to exact core identity and binary evidence rather than a
generic claim that all libretro cores are deterministic or snapshot-capable.

## ScummVM

PhiCade qualifies the official ScummVM v2026.3.0 libretro port from pinned
source revision:

`fed42f2068dcafc6aafa1c28c77e4c88def74b66`

CI builds the upstream no-engine launcher and runs it with no game data.

The qualification proves governed framebuffer and input behavior while also
recording the absence of libretro serialization and save RAM in the pinned
profile.

Therefore ScummVM can have a governed PhiCade desktop session, but it does not
receive fictional SameBoy capabilities.

For the pinned ScummVM profile:

- frame/render behavior is qualified;
- governed RetroPad/analog input is available;
- exact Replay v1 is unsupported;
- state snapshots/rewind are unsupported;
- libretro persistent save RAM is unsupported.

PhiCade does not distribute the ScummVM binary or commercial game data.

See `SCUMMVM_QUALIFICATION.md`.

## Agent evidence

The agent path includes:

- governed Phi-Bot seat;
- provider-neutral Agent Driver Protocol;
- loopback-only Ollama vision adapter;
- bounded Autodrive;
- adaptive observation cadence;
- action-aware settle/backoff;
- bounded UTF-8 working-memory capsules;
- framebuffer + memory hash binding;
- task-scoped benchmark control grants;
- refusal and re-arm evidence;
- model digest qualification;
- digest-bound gameplay receipts.

The runtime does not hand an agent a privileged core handle merely because the
agent is the active controller.

See:

- `AGENT_DRIVER_PROTOCOL.md`
- `PHIBOT_SEAT.md`
- `OLLAMA_PROVIDER.md`
- `AUTODRIVE.md`
- `GOVERNED_AGENT_MEMORY.md`
- `MODEL_QUALIFICATION.md`

## Benchmark evidence

The benchmark stack adds:

- source-first frozen task identity;
- exact source/ROM hashes;
- qualified SameBoy execution;
- deterministic oracles;
- wrong-path negative controls;
- convergence controls after erased history;
- balanced shortcut controls;
- versioned suite membership;
- coverage gates;
- provenance-checked campaigns;
- suite reports;
- like-for-like comparisons.

Suite v13 contains 77 frozen tasks. See `BENCHMARKS.md`.

## Evidence closure

Suite-level claims now close transitively through the benchmark evidence graph.

Before Public Suite Result export or Suite Report comparison, PhiCade re-opens
and verifies:

```text
Suite Report
  -> Campaign receipt
  -> Model gameplay receipt
  -> Autodrive receipt

Suite Report / Campaign
  -> exact model qualification receipt
```

Hash equality is necessary but not sufficient. Parsed child receipts must also
match the parent evidence on task identity, model/digest, core identity, policy,
score/success, stop reason, and the relevant run IDs.

Model qualification receipts are now archived by immutable receipt SHA-256 so a
later re-qualification of the same model digest cannot silently replace the
bytes pinned by older benchmark evidence.

See `EVIDENCE_CLOSURE.md`.

## Comparison semantics

Comparison Lab remains like-for-like at the task level.

Suite Report is the explicit cross-task aggregation layer.

Cross-version suite reports are not compared as though they covered the same
population, and the comparison layer deliberately does not emit a synthetic
"winner" field.

## Content and license boundary

PhiCade does not distribute:

- commercial ROMs;
- proprietary BIOS or firmware;
- decryption keys;
- copyrighted game assets;
- the ScummVM GPL core binary.

PhiCade's own code and benchmark task sources are MIT. Third-party cores and
model runtimes retain their own licenses and notices.
