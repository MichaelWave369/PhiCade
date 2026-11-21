import { describe, expect, it } from "vitest";
import { boundedDosDelta, dosKeyFromBrowser, dosKeyFromGamepad } from "./dosInput";

describe("DOS session input surface", () => {
  it("accepts only mapped human keys", () => {
    expect(dosKeyFromBrowser("ArrowUp")).toBe("ArrowUp");
    expect(dosKeyFromBrowser("Escape")).toBe("Escape");
    expect(dosKeyFromBrowser("F11")).toBeNull();
    expect(dosKeyFromBrowser("Control")).toBeNull();
    expect(dosKeyFromBrowser("Meta")).toBeNull();
    expect(dosKeyFromBrowser("")).toBeNull();
  });
  it("maps a bounded gamepad control set", () => {
    expect(dosKeyFromGamepad("A")).toBe("Space");
    expect(dosKeyFromGamepad("UP")).toBe("ArrowUp");
    expect(dosKeyFromGamepad("SELECT")).toBe("q");
    expect(dosKeyFromGamepad("L3")).toBeNull();
    expect(dosKeyFromGamepad("__proto__")).toBeNull();
  });
  it("bounds every mouse delta", () => {
    expect(boundedDosDelta(999)).toBe(64);
    expect(boundedDosDelta(-999)).toBe(-64);
    expect(boundedDosDelta(Number.NaN)).toBe(0);
    expect(boundedDosDelta(4.9)).toBe(4);
  });
});
