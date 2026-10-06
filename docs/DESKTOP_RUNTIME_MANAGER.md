# Desktop Runtime Manager v1

Rung 38 brings Runtime Registration v1 into the PhiCade desktop UI.

The native backend already supported durable runtime registration and
registry-backed session routing. This rung makes that registry visible and
operator-usable without requiring direct Tauri command calls.

## Operator flow

From the library sidebar:

1. choose **REGISTER LIBRETRO CORE**;
2. select a local `.dll`, `.so`, or `.dylib`;
3. PhiCade sends the selected path to `register_runtime_core`;
4. the backend opens and inspects the core, hashes the binary, records identity
   and capabilities, and persists the registration receipt;
5. the desktop refreshes the registry and renders the evidence.

Registration still grants no launch authority.

## Visible evidence

Each registered runtime row exposes:

- library name and version;
- execution model;
- SHA-256 prefix;
- count of QUALIFIED capabilities;
- count of available SUPPORTED/QUALIFIED capabilities;
- qualification profile ID when present;
- binary-evidence binding state;
- stored authority state.

The UI deliberately distinguishes:

```text
REGISTERED
!= BINARY EVIDENCE BOUND
!= AUTHORITY GRANTED
```

A familiar runtime name is not treated as proof.

## Legacy SameBoy path

The existing SameBoy core picker remains available for the current GB/GBC
desktop launch path.

This is intentional. Rung 38 exposes and verifies the generic runtime inventory
without changing the existing launch workflow in the same patch.

The next rung can select a registered runtime and call
`start_registered_emulation` for ScummVM or other compatible libretro
sessions.

## Web preview

Runtime registration remains native-only. The browser preview does not receive
filesystem access or synthetic registry data.
