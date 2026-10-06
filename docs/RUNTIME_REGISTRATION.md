# Runtime Registration v1

Rung 36 adds a durable local runtime registry for user-supplied libretro cores.

Schema:

`phicade.runtime-registration.v1`

A registration receipt records:

- canonical local core path;
- exact binary SHA-256;
- libretro core identity;
- runtime capability manifest;
- any matching named qualification profile;
- whether that exact binary has been bound to qualification evidence;
- whether registration granted launch authority.

## Registration is not qualification

A core may identify itself as a known runtime/version and therefore match a
known capability profile. That does not prove the selected binary is the same
binary that produced a qualification receipt.

Every new local registration therefore starts with:

```text
binaryEvidenceBound = false
authorityGranted     = false
```

This remains true even when the identity matches pinned SameBoy or ScummVM
profiles.

A later evidence-binding step may promote the binary only after exact receipt /
hash verification.

## Persistence

The Tauri shell stores receipts under the application data directory in:

```text
runtime-registry/<core-sha256>.json
```

Registration is idempotent by binary SHA-256.

## Native commands

- `register_runtime_core(corePath)`
- `list_runtime_registrations()`

`register_runtime_core` opens the core through PhiCade's actual libretro host,
captures identity/capabilities, cleanly deinitializes it, fingerprints the
binary, and persists the receipt.

## Authority boundary

Runtime registration is inventory/provenance only.

It does not:

- launch content;
- mutate the current emulator session;
- grant controller or agent authority;
- auto-download a core;
- install a binary;
- mark a binary as qualified merely from its self-reported name/version.

The next session-routing rung can consume this registry instead of taking an
unverified arbitrary core path on faith.
