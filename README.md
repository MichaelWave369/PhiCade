# PhiCade

> **Old worlds. New players.**

PhiCade is an open-source retro game runtime and emulator front end built around
one core idea: every player enters through the same Action Bus.

Human controls, deterministic replays, scripts, network input, and Phi-Bot seats
can therefore share the same authority and evidence machinery.

## Current status — Rung 6

PhiCade now has a governed Phi-Bot gameplay seat on top of the qualified SameBoy
runtime, persistent session machinery, and Replay Ledger.

Highlights:

- React + TypeScript + Vite terminal UI
- Tauri 2 + Rust desktop shell
- user-selected local ROM directory
- qualified SameBoy 1.0.3 libretro host
- GB/GBC video + audio
- battery RAM, save states, rewind, screenshots, per-game profiles
- deterministic content-addressed Replay Ledger
- frame/state/input verification receipts
- framebuffer-only Phi-Bot observation API
- explicit agentId + seat authority grants
- scoped allowed-button/axis/system-command permissions
- grant expiry and per-frame action limits
- HUMAN, PHI-BOT and CO-OP control modes
- immediate human takeover / grant revocation
- explicit VERSUS refusal on the one-port SameBoy core
- authority rejection telemetry
- CI human-vs-agent exact-state parity qualification
- CI privileged-action rejection and takeover qualification

No commercial ROMs, proprietary console BIOS files, or third-party core binaries
are committed to this repository.

## Run

```bash
npm install
npm run desktop
```

Choose a ROM directory, select a compatible SameBoy libretro binary, choose a
`.gb` or `.gbc` image, and use **LOAD / RUN**.

## Phi-Bot seat

The operator deck provides:

- **HUMAN** — revoke the bot grant and return gameplay to the human
- **HANDOFF** — grant seat 1 to `phi-local`
- **CO-OP** — human and Phi-Bot share seat 1
- **VERSUS** — demonstrates the current SameBoy single-port refusal
- **OBSERVE + ACT** — fetch a real framebuffer observation and enqueue a
  deterministic demo action through the same Action Bus

The demo control is deliberately simple. Rung 6 builds the governed seat, not a
claim that a particular AI policy is good at games.

## Qualification

```bash
bash ./scripts/qualify-sameboy.sh
```

Produces:

- `artifacts/sameboy-qualification.json`
- `artifacts/replay-qualification.json`
- `artifacts/phibot-qualification.json`

See:

- `docs/cores/SAMEBOY.md`
- `docs/SESSION_MACHINERY.md`
- `docs/REPLAY_LEDGER.md`
- `docs/PHIBOT_SEAT.md`
- `docs/ARCHITECTURE.md`

## Content policy

PhiCade does **not** distribute commercial ROMs, proprietary BIOS/firmware,
decryption keys, or copyrighted game assets. Use homebrew/public-domain test
software or software you are authorized to use.

## License

PhiCade's own code is MIT. Third-party emulator cores retain their own licenses
and notices.
