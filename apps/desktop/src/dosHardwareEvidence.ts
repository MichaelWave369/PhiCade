/** Manual, local observations only. No keys, device IDs, game paths, telemetry. */
export const DOS_HARDWARE_CHECKS = [
  { id: "keyboard", label: "Physical keyboard", instruction: "With CRT focus and explicit approval, verify arrow keys or Escape visibly affect the DOS program." },
  { id: "mouseClick", label: "Physical mouse click", instruction: "Verify left click reaches a DOS program that handles mouse buttons." },
  { id: "mouseMotion", label: "Physical mouse motion", instruction: "Verify movement inside the DOS program, not merely movement of the desktop cursor." },
  { id: "gamepad", label: "Physical gamepad", instruction: "Test D-pad, A=Space, B=Escape, Start=Enter; verify guest responses." },
  { id: "focusLoss", label: "Alt+Tab / focus loss", instruction: "Approve, hold a mapped key, Alt+Tab away and return. Verify input is disabled and no key remains held." },
  { id: "sessionReplacement", label: "Game stop and replacement", instruction: "Eject and launch another DOS session. Verify fresh approval is required before input works." },
] as const;
export type DosHardwareCheckId = typeof DOS_HARDWARE_CHECKS[number]["id"];
export type DosHardwareOutcome = "not-tested" | "pass" | "fail";
export type DosHardwareSelections = Record<DosHardwareCheckId, DosHardwareOutcome>;
export function emptyDosHardwareSelections(): DosHardwareSelections {
  return Object.fromEntries(DOS_HARDWARE_CHECKS.map((c) => [c.id, "not-tested"])) as DosHardwareSelections;
}
export function buildDosHardwareReport(selections: DosHardwareSelections, generatedAt: string) {
  if (!Number.isFinite(Date.parse(generatedAt))) throw new Error("Valid timestamp required");
  const checks = DOS_HARDWARE_CHECKS.map(({ id, label }) => {
    const outcome = selections[id];
    if (outcome !== "not-tested" && outcome !== "pass" && outcome !== "fail") {
      throw new Error("Invalid manual test status");
    }
    return { id, label, outcome };
  });
  return {
    schema: "phicade.dos.windows.manual-hardware.v1",
    evidenceBasis: "operator-self-reported",
    generatedAt,
    checks,
    passCount: checks.filter((c) => c.outcome === "pass").length,
    failCount: checks.filter((c) => c.outcome === "fail").length,
    untestedCount: checks.filter((c) => c.outcome === "not-tested").length,
    automatedHardwareTest: false,
    physicalHardwareVerifiedByCI: false,
    retailDosGameCompatibilityQualified: false,
    containsKeystrokesOrDeviceIdentifiers: false,
  };
}
