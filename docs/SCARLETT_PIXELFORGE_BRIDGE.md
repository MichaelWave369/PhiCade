# Scarlett Lab → PixelForge diagnostic bridge (proposal)

PhiCade **owns authority**; PixelForge **renders approved summaries**.
The current Scarlett Lab phase is intentionally NOT connected to PixelForge.
Do not send executables, full memory images, import payloads, filesystem paths,
auth tokens, process handles, or unredacted crash logs through Runtime Bridge v1.

A future `scarlett.inspection-summary.v1` transport can carry only:
`schema`, `sourceKind: synthetic/authorized`, an optional source content
SHA-256, section count and bounded geometry/ranges, source format and capability
flags. Label every inspection as "metadata parsed, no guest execution".

Before implementing transport, enforce:
- Explicit operator-approved lab session, with no implicit Arcade/agent grants.
- Schema validation, length caps, immutable payloads and timeouts.
- No new PixelForge command surfaces or runtime privileges.
- Screenshot/log redaction and origin checks for any remote presentation.
- Reproducible original fixtures in CI for both sides.
- A separate acceptance report proving the diagnostic view cannot call a
  PhiCade launch action.

This boundary reuses the philosophy of the existing PixelForge Runtime Bridge
without implying console emulation or an implemented visual integration.
