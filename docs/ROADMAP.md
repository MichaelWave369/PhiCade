# PhiCade Roadmap

## Rung 1 — Foundation
- [x] Repository and MIT license
- [x] React/Vite desktop UI prototype
- [x] Rust runtime crate
- [x] Shared Action Bus concept in UI + runtime
- [x] Emulator-core trait boundary
- [x] ROM/firmware repository policy
- [x] CI for TypeScript build and Rust tests

## Rung 2 — Native shell
- [x] Tauri desktop shell
- [x] Local directory access
- [x] Persistent settings
- [x] Controller discovery
- [x] Canonical ActionEnvelope IPC
- [x] Native compile gate

## Rung 3 — First qualified core
- [x] SameBoy 1.0.3 / GB+GBC provenance freeze
- [x] MIT-licensed dmg-acid2 fixture
- [x] Video/audio bridge
- [x] Human gamepad through Action Bus
- [x] SHA-256 qualification receipt

## Rung 4 — Session machinery
- [x] Save RAM
- [x] Save/load state
- [x] Fast-forward
- [x] Rewind buffer
- [x] Screenshots
- [x] Per-game profiles
- [x] State round-trip qualification

## Rung 5 — Replay Ledger
- [x] Deterministic input recording
- [x] Replay verification
- [x] State/frame/input checkpoints
- [x] Content-addressed receipts
- [x] Divergence detector
- [x] Positive + mutated-replay controls

## Rung 6 — Phi-Bot seat
- [x] Frame observation API
- [x] Scoped input authority
- [x] Human handoff/takeover
- [x] Same-seat co-op
- [x] Explicit one-port versus refusal
- [x] No privileged side-channel input
- [x] Human/agent parity receipt

## Rung 7 — Agent Driver Protocol
- [x] Transport-neutral turn schemas
- [x] Observation hash + turn binding
- [x] Action/delay budgets
- [x] Turn expiry
- [x] Native driver inbox
- [x] Host-canonical sequencing
- [x] Human-last same-frame precedence
- [x] Reference driver
- [x] Driver/direct parity receipt

## Rung 8 — Local Ollama provider
- [x] Loopback-only Ollama REST adapter
- [x] Local model discovery
- [x] RGBA framebuffer → PNG vision payload
- [x] JSON-schema-constrained button output
- [x] AgentTurnResponse conversion
- [x] THINK PAUSE frame consistency
- [x] Provider-failure turn cancellation
- [x] Persistent local model selection
- [x] Loopback mock-server CI tests

## Rung 9 — Governed Autodrive
- [x] Native bounded autonomous-run policy
- [x] Turn/action/frame/empty-turn budgets
- [x] Queue-aware multi-turn loop
- [x] Persistent stop-reason receipts
- [x] Immediate HUMAN takeover
- [x] Grant-expiry stop + neutralization
- [x] Provider-failure stop
- [x] System/timeline-command refusal during autonomous runs
- [x] SameBoy bounded-loop qualification receipt

## Rung 10 — Model Qualification Registry
- [x] Exact Ollama model digest discovery
- [x] /api/show capability inspection
- [x] Vision capability gate
- [x] Synthetic image visual probe
- [x] Structured-output qualification probe
- [x] Digest-bound persistent receipts
- [x] One-shot unqualified experimentation
- [x] Native AUTO DRIVE exact-digest gate
- [x] Changed-digest negative control
- [x] Mock capability/qualification CI tests

## Rung 11 — Φ-Agent Gym
- [x] Source-first Game Boy benchmark ROM
- [x] Machine-readable task manifest
- [x] Rendered-pixel player detection
- [x] Manhattan-distance progress score
- [x] NO-INPUT negative control
- [x] Oracle D-pad positive control
- [x] Deterministic replay control
- [x] Source/ROM/core hash-bound receipt
- [x] SameBoy CI integration

## Rung 12 — Model Gameplay Benchmark
- [x] Exact frozen Φ-Agent Gym ROM gate
- [x] Shared runtime/CI pixel scorer
- [x] Exact model digest + qualification-receipt binding
- [x] SameBoy binary + Autodrive receipt hash binding
- [x] Native reset + 120-frame benchmark warmup
- [x] Frozen start-geometry verification
- [x] D-pad-only benchmark grant
- [x] Visible-task-only provider instruction
- [x] Automatic task-success stop
- [x] Automatic score receipt for every Autodrive stop path
- [x] Full native governance/benchmark CI tests

## Rung 13 — Benchmark Campaigns
- [x] Native 3–20 trial campaign bounds
- [x] Desktop five-trial default campaign
- [x] Shared deterministic campaign statistics
- [x] Live installed-digest re-check before every trial
- [x] Qualification-receipt hash pinning
- [x] SameBoy binary hash pinning
- [x] Exact Autodrive policy pinning
- [x] Immutable SHA-256 references to individual trial receipts
- [x] COMPLETE and PARTIAL campaign summaries
- [x] Operator/end and core-shutdown evidence preservation
- [x] Success rate + mean/median/min/max/population stddev

## Rung 14 — Comparison Lab
- [x] Persistent evidence IDs across app sessions
- [x] Persistent campaign discovery
- [x] Native strict campaign compatibility gate
- [x] COMPLETE-only comparison
- [x] Full-score/no-scoring-error requirement
- [x] Same provider/Gym/core/policy/trial-count requirement
- [x] Re-hash every referenced trial receipt before comparison
- [x] Shared Welch 95% mean-difference confidence interval
- [x] Shared Hedges' g effect size
- [x] Success-rate difference
- [x] Campaign receipt SHA-256 binding in comparison receipt
- [x] Comparison Lab desktop selectors + telemetry
- [x] Tamper/refusal/persistent-ID CI controls

## Rung 15 — Benchmark Suite v1
- [x] Two independently source-first Game Boy benchmark tasks
- [x] Original diagonal + Mirror Dash reversed geometry
- [x] Frozen suite manifest
- [x] Shared runtime task registry
- [x] Exact ROM-hash task lookup
- [x] Generic rendered-pixel task scorer
- [x] Generic task-driven qualification harness
- [x] Per-task no-input/oracle/replay qualification receipts
- [x] Exact source + ROM registry hash enforcement
- [x] Native benchmark/campaign task pinning
- [x] Registry-derived Ollama task prompts
- [x] Native SessionInfo benchmark task metadata
- [x] Desktop BENCH/CAMPAIGN gates for any registered task
- [x] Comparison Lab remains like-for-like across task identity

## Rung 16 — Benchmark Suite Reports
- [x] Cross-task campaign cohort discovery
- [x] Exact model/digest/qualification/core/policy/trial-count cohort pin
- [x] READY only with every registered Suite v1 task
- [x] Deterministic highest-campaign selection per task
- [x] Re-verify every campaign and underlying trial receipt
- [x] Persistent suite-report receipt IDs
- [x] Campaign receipt SHA-256 binding per task
- [x] Shared macro task-mean aggregation
- [x] Trial-weighted overall success rate
- [x] Task mean min/max + population stddev
- [x] Desktop READY/INCOMPLETE cohort lane
- [x] Digest-split, incomplete, and tamper CI controls
- [x] Comparison Lab remains task-local

## Rung 17 — Suite Comparison Lab
- [x] Persistent Suite Report discovery
- [x] Distinct report A/B selection
- [x] Same suite/core/policy/trial-count compatibility gate
- [x] Different provider/model identities allowed by design
- [x] Exact Suite Report receipt SHA-256 binding
- [x] Full report → campaign → trial provenance replay
- [x] Exact task/source/ROM registry re-verification
- [x] Per-task campaign Welch 95% CI reuse
- [x] Per-task Hedges' g + success-rate delta reuse
- [x] Macro paired task-mean A−B summary
- [x] Trial-weighted overall success-rate A−B summary
- [x] Task-delta min/max + population stddev
- [x] Persistent suite-comparison receipts
- [x] Desktop SUITE COMPARE evidence lane
- [x] No winner/ranking field

## Rung 18 — Adaptive Observation Cadence
- [x] Native next-observation eligibility frame
- [x] Minimum observation interval
- [x] Action-delay-aware post-action settle
- [x] Exponential consecutive-empty-turn backoff
- [x] Hard maximum observation interval cap
- [x] Native refusal of early autonomous observations
- [x] Desktop waits on native cadence status
- [x] Status exposes next/last observation frames
- [x] Status records total + maximum cadence wait
- [x] Autodrive receipt v2 freezes cadence evidence
- [x] Cadence policy remains part of campaign/suite compatibility
- [x] SameBoy cadence positive/control qualification
- [x] Desktop cadence telemetry
- [x] Existing THINK PAUSE semantics preserved

## Rung 19 — Versioned Benchmark Suites
- [x] Task identity separated from suite membership
- [x] Suite v1 remains the exact original two-task population
- [x] Suite v2 reuses the exact frozen v1 tasks
- [x] New source-first Wall Detour obstacle-navigation task
- [x] Visible wall forces non-greedy DOWN → RIGHT → UP route
- [x] Arbitrary-length frozen oracle paths
- [x] Canonical Wall Detour source + ROM SHA-256 freeze
- [x] NO-INPUT/oracle/deterministic replay qualification
- [x] Native benchmark-suite registry + discovery API
- [x] Explicit suite-scoped cohort discovery
- [x] Suite-scoped Report persistence + IDs
- [x] Suite-scoped Comparison persistence + IDs
- [x] Provenance replay resolves the report's exact suite task population
- [x] Cross-suite Report comparison refused
- [x] Desktop suite-version selector
- [x] Suite v1 READY at 2/2 while Suite v2 remains INCOMPLETE at 2/3
- [x] Suite v2 becomes READY only at 3/3
- [x] Existing Suite v1 evidence namespace preserved unchanged

## Rung 20 — Governed Agent Working Memory
- [x] Agent Driver Protocol v2
- [x] Explicit UTF-8 memory capsule on every agent turn
- [x] Turn response binds framebuffer hash + pending memory hash
- [x] Native-only memory acceptance
- [x] Replacement-style memory updates with hard byte budgets
- [x] 4096-byte default capsule / 1024-byte per-turn update budget
- [x] Autodrive policyVersion 1 freezes memory limits
- [x] Legacy pre-cadence policy JSON preserves legacy cadence semantics
- [x] Every autonomous run begins from empty memory
- [x] Memory revision/update/bytes-written/refusal counters
- [x] Autodrive receipt v3 seals initial/final memory hashes and final content
- [x] Ollama structured output includes optional memoryUpdate
- [x] Reference driver upgraded to Protocol v2
- [x] Desktop live memory telemetry

## Rung 21 — Temporal Cue Memory Benchmark
- [x] Balanced LEFT/RIGHT source-first Game Boy cue tasks
- [x] Identical provider instruction across cue variants
- [x] Cue disappears before the later decision
- [x] 90-frame post-cue lockout
- [x] Identical two-door decision framebuffer design
- [x] Neutral-arm rule blocks same-turn directional carry-through
- [x] Task-owned exact benchmark button grants
- [x] Generic oracle supports A + explicit WAIT legs
- [x] Dedicated pair qualification harness
- [x] Cue-frame difference control
- [x] Decision-frame identity control
- [x] Carry-through refusal control
- [x] Neutral re-arm recovery control
- [x] Benchmark Suite v3 with exact v2 reuse + both temporal tasks
- [x] v2 READY at 3/3 while v3 remains INCOMPLETE at 3/5
- [x] v3 remains INCOMPLETE at 4/5
- [x] v3 READY only at 5/5
- [ ] Freeze canonical temporal source + ROM SHA-256 after behavioral qualification

## Later

Additional benchmark capabilities/tasks, additional uncertainty methods, provider
capability profiles beyond Ollama, cloud-provider adapters, metadata/cover art,
additional systems, true multi-port versus play, netplay, spectator mode,
achievements, tournaments, CommonLine rooms, and a shared Phi Game Runtime with
Night Circuit.
