/** One-shot focus revocation for a HUMAN-approved DOS session.
 * Only the Tauri backend disables the actual session grant.
 */
export interface FocusWindowSource {
  addEventListener(type: "blur", listener: () => void): void;
  removeEventListener(type: "blur", listener: () => void): void;
}
export interface VisibilitySource {
  readonly visibilityState: string;
  addEventListener(type: "visibilitychange", listener: () => void): void;
  removeEventListener(type: "visibilitychange", listener: () => void): void;
}
export function installDosFocusGuard(
  win: FocusWindowSource, doc: VisibilitySource, revoke: () => void,
): () => void {
  let revoked = false;
  const once = () => { if (!revoked) { revoked = true; revoke(); } };
  const onVisibility = () => { if (doc.visibilityState === "hidden") once(); };
  win.addEventListener("blur", once);
  doc.addEventListener("visibilitychange", onVisibility);
  onVisibility();
  return () => {
    win.removeEventListener("blur", once);
    doc.removeEventListener("visibilitychange", onVisibility);
  };
}
