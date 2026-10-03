# Native Boundary

Rung 2 makes PhiCade a native desktop application while keeping the webview deliberately narrow.

## Filesystem rule

The frontend does not receive general filesystem permissions.

1. The user explicitly opens the native directory picker.
2. The picker returns one selected path.
3. The path is handed to a Rust command.
4. Rust performs a non-recursive, extension-only inventory.
5. The UI receives metadata: local path, display name, extension, and inferred system.
6. ROM bytes are not copied into settings, uploaded, or returned through IPC.

The first scanner intentionally ignores archives and ambiguous formats. Expansion belongs in later qualified work.

## Settings

PhiCade stores only application settings in the platform app-config directory:

- selected ROM directory path,
- auto-scan preference,
- controller deadzone.

It does not store ROM content in the settings file.

## Controller discovery

Rung 2 enumerates controllers with the webview Gamepad API. Enumeration is separate from action authority. A connected controller does not automatically gain a seat; later mapping code must translate its state into normalized ActionEnvelope events.

## Action IPC

Rust and TypeScript now share the same serialized envelope:

```json
{
  "sequence": 12,
  "frame": 34,
  "source": {
    "kind": "phi-bot",
    "agentId": "phi-7"
  },
  "action": {
    "kind": "system",
    "command": "save-state",
    "slot": 3
  }
}
```

The desktop UI can round-trip an event through Rust validation. This is the seam that future replay, networking, and Phi-Bot control will reuse.
