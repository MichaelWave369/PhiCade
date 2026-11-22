import { describe, expect, it } from "vitest";
import { buildDosHardwareReport, DOS_HARDWARE_CHECKS, emptyDosHardwareSelections } from "./dosHardwareEvidence";
describe("DOS physical hardware manual receipt", () => {
  it("defaults to untested and never claims automatic hardware verification", () => {
    const r = buildDosHardwareReport(emptyDosHardwareSelections(), "2026-11-21T00:00:00Z");
    expect(r.checks).toHaveLength(6);
    expect(r.untestedCount).toBe(6);
    expect(r.physicalHardwareVerifiedByCI).toBe(false);
    expect(r.retailDosGameCompatibilityQualified).toBe(false);
  });
  it("records operator pass, failure, untested separately", () => {
    const v = emptyDosHardwareSelections();
    v.keyboard = "pass";
    v.focusLoss = "fail";
    const r = buildDosHardwareReport(v, "2026-11-21T12:00:00Z");
    expect([r.passCount, r.failCount, r.untestedCount]).toEqual([1, 1, 4]);
    expect(r.evidenceBasis).toBe("operator-self-reported");
    expect(r.containsKeystrokesOrDeviceIdentifiers).toBe(false);
    expect(r.checks.map((c) => c.id)).toEqual(DOS_HARDWARE_CHECKS.map((c) => c.id));
  });
  it("refuses fabricated outcomes and timestamps", () => {
    const v = emptyDosHardwareSelections();
    v.keyboard = "verified" as never;
    expect(() => buildDosHardwareReport(v, "2026-11-21T12:00:00Z")).toThrow();
    expect(() => buildDosHardwareReport(emptyDosHardwareSelections(), "not-a-date")).toThrow();
  });
});
