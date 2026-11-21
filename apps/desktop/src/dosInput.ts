/** Human-only DOS control mapping. Browser capture is opt-in per session. */
export function dosKeyFromBrowser(key: string): string | null {
  const mapped: Record<string, string> = {
    Escape: "Escape", Enter: "Enter", " ": "Space", Backspace: "Backspace",
    ArrowUp: "ArrowUp", ArrowDown: "ArrowDown",
    ArrowLeft: "ArrowLeft", ArrowRight: "ArrowRight",
    w: "w", a: "a", s: "s", d: "d", q: "q",
  };
  return Object.prototype.hasOwnProperty.call(mapped, key) ? mapped[key] : null;
}

export function dosKeyFromGamepad(button: string): string | null {
  const mapped: Record<string, string> = {
    UP: "ArrowUp", DOWN: "ArrowDown", LEFT: "ArrowLeft", RIGHT: "ArrowRight",
    A: "Space", B: "Escape", START: "Enter", SELECT: "q",
  };
  return Object.prototype.hasOwnProperty.call(mapped, button) ? mapped[button] : null;
}

export function boundedDosDelta(value: number): number {
  if (!Number.isFinite(value)) return 0;
  return Math.max(-64, Math.min(64, Math.trunc(value)));
}
