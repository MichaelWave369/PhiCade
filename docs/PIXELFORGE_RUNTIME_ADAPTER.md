# PixelForge Runtime Adapter v1

## Purpose

Rung 44 connects PhiCade to PixelForge Runtime Bridge v1 through PixelForge's
bounded JSONL transport.

This is the first concrete proof of the intended split:

\`\`\`text
PixelForge builds / owns cartridge semantics
                |
                v
PixelForge Runtime Bridge v1
                |
          JSONL Transport v1
                |
                v
PhiCade governs the session
                |
     +----------+----------+
     |          |          |
   Human      Phi-Bot    Replay/later
\`\`\`

Neither project absorbs the other.

## Pinned PixelForge source

The qualification target is the exact merged PixelForge v5.32 transport revision:

\`\`\`text
96809c5ef9608e994a14c300bce9032f59041d0a
\`\`\`

PhiCade refuses the qualification if the local PixelForge checkout is at a
different revision.

The pinned proof uses PixelForge's deliberately boring deterministic reference
counter before any real game cartridge is connected.

## Transport

PixelForge supplies:

- \`pixelforge.runtime-transport.request.v1\`
- \`pixelforge.runtime-transport.response.v1\`

PhiCade launches the local PixelForge reference process and communicates only
through stdin/stdout newline-delimited JSON.

The adapter validates:

- response schema,
- request/response ID matching,
- PixelForge Runtime Bridge protocol identity,
- Runtime Bridge version,
- game/runtime identity,
- explicit remote errors.

EOF owns normal process shutdown.

## PhiCade capability projection

The first adapter declares:

- execution model: **BRIDGED_RUNTIME**
- governed actions: **SUPPORTED**
- external process lifecycle: **SUPPORTED**
- semantic events: **SUPPORTED**
- frame step: **SUPPORTED** only when the PixelForge descriptor is externally
  stepped, or when the Runtime Bridge v1 deterministic compatibility rule applies

It deliberately leaves these **UNSUPPORTED** until separately proven:

- rendered framebuffer,
- audio stream,
- reset,
- restorable state snapshots,
- exact Replay v1,
- persistent save data,
- game detection.

A PixelForge \`snapshot()\` response is not treated as a restorable emulator save
state merely because the method exists.

## Authority proof

The qualification does not submit directly merely because a transport exists.

It constructs a normal PhiCade human Action Bus envelope:

\`\`\`text
seat 1 / A pressed
\`\`\`

The existing PhiCade \`AuthorityPolicy\` must accept that action first.

Only then does the qualification-only mapper translate it to the reference
counter's PixelForge intent:

\`\`\`json
{
  "type": "ADD",
  "actorId": "counter",
  "params": { "amount": 1 }
}
\`\`\`

That mapping is intentionally narrow and is not a generic PixelForge gameplay
grammar. SPARK and other cartridges keep ownership of their own actions.

## Qualification chain

The pinned proof requires:

1. exact PixelForge source revision,
2. launch the real PixelForge JSONL reference server,
3. validate \`describe()\`,
4. register the PhiCade controller,
5. observe counter value \`0\`,
6. authorize one PhiCade action through the existing authority layer,
7. submit the translated PixelForge intent,
8. advance the bridge,
9. require \`ACTION_ACCEPTED\`,
10. consume the same semantic event through \`events(0)\`,
11. observe counter value \`1\`,
12. capture the PixelForge runtime hash,
13. write a PASS receipt.

Receipt:

\`\`\`text
artifacts/pixelforge-bridge-qualification.json
\`\`\`

Schema:

\`\`\`text
phicade.pixelforge-bridge-qualification.v1
\`\`\`

## Local qualification

From PhiCade:

\`\`\`bash
bash ./scripts/qualify-pixelforge.sh /path/to/parallax-pixelforge
\`\`\`

Or directly, including on Windows PowerShell:

\`\`\`powershell
cargo run -p phicade-runtime --example qualify_pixelforge -- --pixelforge-root "C:\\path\\to\\parallax-pixelforge"
\`\`\`

The PixelForge checkout must be at the pinned revision above and Node must be
available on PATH.

## What this proves

PASS proves that the exact pinned PixelForge bridge transport can be inhabited by
PhiCade through a local process, that one PhiCade-authorized action crosses the
boundary, that PixelForge remains responsible for accepting the game intent, and
that semantic events plus a runtime hash return across the same seam.

## What it does not prove

This rung does not claim:

- SPARK is connected yet,
- PixelForge framebuffer/audio streaming exists,
- PixelForge snapshots are restorable,
- exact replay exists for bridged cartridges,
- agent Autodrive is enabled for arbitrary PixelForge games,
- Unreal is bridged,
- a browser/WebGL cartridge has production performance.

Those are later capability-specific rungs.

## Next rung

Connect one real PixelForge cartridge through the same adapter without changing
the transport contract. After that seam survives a real game, SPARK can become
the first large multi-incarnation consumer.
