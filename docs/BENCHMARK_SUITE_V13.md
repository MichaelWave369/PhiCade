# Benchmark Suite v13 — Indirect Context Routing

Suite v13 freezes the exact **61-task Suite v12 population** and appends the
sixteen source-first Indirect Context Routing tasks from Rung 31.

## Frozen population

- Suite v12 population: **61 tasks**, unchanged.
- Indirect Context Routing additions: **16 tasks**.
- Suite v13 total: **77 tasks**.
- System: Game Boy.
- Qualified core: pinned SameBoy 1.0.3.
- Observation boundary: rendered framebuffer only.

The new factorial is:

- bank layout: A / B;
- pointer map: NORMAL / SWAPPED;
- pointer token: STAR / MOON;
- query: TRIANGLE / SQUARE.

The controller must resolve:

`pointer token -> erased bank -> erased symbol relation -> side`

before committing to one of two visually identical doors.

## Provenance freeze

The sixteen source/ROM hash pairs are the exact hashes minted by the green
Rung 31 source CI run before registry admission. The build script also requires
sixteen unique source paths and sixteen unique ROM paths.

No new task is admitted by filename, title, or intent alone. Registry source
SHA-256 and ROM SHA-256 must match the assembled artifacts.

## Required qualification

The pinned SameBoy joint qualifier must prove:

- each of the four layout × pointer-map briefings is condition-independent across
  future pointer/query choices;
- the four briefing histories are visibly distinguishable;
- for fixed pointer + query, the choice frame converges byte-for-byte across all
  four erased histories;
- STAR/MOON and TRIANGLE/SQUARE choice cues remain visibly distinct;
- choice geometry is identical across all sixteen tasks;
- all registry hashes match;
- every correct path succeeds;
- every wrong commitment is terminal and cannot recover;
- always LEFT and always RIGHT each solve exactly 8/16;
- assuming NORMAL or SWAPPED pointer mapping each solves exactly 8/16;
- always resolving to CIRCLE or CROSS each solves exactly 8/16;
- ignoring the query in favor of TRIANGLE or SQUARE each solves exactly 8/16;
- all sixteen tasks expose the same non-leaking provider instruction and the same
  A + LEFT + RIGHT action scope.

## Coverage gate

Native suite reporting must preserve the previous frozen line:

- Suite v12 remains READY at **61/61**.
- Suite v13 is INCOMPLETE at **61/77** through **76/77**.
- Suite v13 becomes READY only at **77/77**.

This prevents partial evidence from being presented as a complete v13 cohort.

## Files

- `benchmarks/suite-v13.json` — frozen 77-task manifest.
- `docs/INDIRECT_CONTEXT_ROUTING_BENCHMARK.md` — task semantics and controls.
- `artifacts/indirect-context-routing-qualification.json` — generated joint receipt.
