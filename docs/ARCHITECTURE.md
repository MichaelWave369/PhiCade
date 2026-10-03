# PhiCade Architecture

## Goal

PhiCade is a desktop emulator front end and game runtime. The runtime is designed so
a keyboard, gamepad, replay file, script, network peer, or future Phi-Bot can all
enter through one governed input seam instead of each subsystem inventing its own
back door.

## Rung 1 boundaries

```text
Human / Replay / Script / Phi-Bot
              |
              v
        +-------------+
        | Action Bus  |
        +-------------+
              |
              v
       Authority layer
              |
              v
       Core Adapter API
              |
      +-------+-------+
      |               |
   libretro       native core
   adapter         adapter
      |               |
      +-------+-------+
              |
        video / audio
```

The Action Bus is implemented now. Authority policy and real core adapters are
deliberately separate rungs.

## Core rules

1. Core adapters receive resolved frame inputs. They do not read controllers or
   agent messages directly.
2. Game images and firmware remain user-supplied local resources.
3. Each integrated core must have a recorded source, version, license, and binary
   hash before it is enabled in a release build.
4. Save states are namespaced by core identity + core version + game fingerprint.
5. Replays record normalized Action Bus events, not host keyboard scan codes.
6. Phi-Bot never gains extra emulator privileges merely because it is an AI seat.

## Determinism direction

A replay receipt should eventually bind:

- PhiCade version
- core adapter ID and version
- core binary hash
- game image hash
- firmware hash(es), when required
- runtime configuration
- initial state hash
- frame-stamped normalized actions
- periodic state hashes

This makes "the bot beat the game" a testable claim instead of folklore.
