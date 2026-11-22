import { describe, expect, it, vi } from "vitest";
import { installDosFocusGuard } from "./dosFocusGuard";

class EventSource {
  private listeners = new Map<string, Set<() => void>>();
  visibilityState = "visible";
  addEventListener(type: "blur" | "visibilitychange", listener: () => void) {
    if (!this.listeners.has(type)) this.listeners.set(type, new Set());
    this.listeners.get(type)!.add(listener);
  }
  removeEventListener(type: "blur" | "visibilitychange", listener: () => void) {
    this.listeners.get(type)?.delete(listener);
  }
  fire(type: "blur" | "visibilitychange") {
    for (const fn of this.listeners.get(type) ?? []) fn();
  }
}
describe("DOS approval focus lifecycle", () => {
  it("revokes once on window blur, not visible visibility change", () => {
    const win = new EventSource(), doc = new EventSource();
    const revoke = vi.fn();
    const detach = installDosFocusGuard(win, doc, revoke);
    doc.fire("visibilitychange");
    expect(revoke).not.toHaveBeenCalled();
    win.fire("blur");
    doc.visibilityState = "hidden";
    doc.fire("visibilitychange");
    expect(revoke).toHaveBeenCalledTimes(1);
    detach();
    win.fire("blur");
    expect(revoke).toHaveBeenCalledTimes(1);
  });
  it("revokes immediately when already hidden", () => {
    const win = new EventSource(), doc = new EventSource();
    doc.visibilityState = "hidden";
    const revoke = vi.fn();
    installDosFocusGuard(win, doc, revoke);
    expect(revoke).toHaveBeenCalledOnce();
  });
  it("removes previous session listener on replacement", () => {
    const win = new EventSource(), doc = new EventSource();
    const previous = vi.fn(), current = vi.fn();
    installDosFocusGuard(win, doc, previous)();
    const detach = installDosFocusGuard(win, doc, current);
    win.fire("blur");
    expect(previous).not.toHaveBeenCalled();
    expect(current).toHaveBeenCalledOnce();
    detach();
  });
});
