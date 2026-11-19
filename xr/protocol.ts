/** Public, bounded XR messages. Never contains paths, ROMs, memory or shell commands. */
export const XR_PROTOCOL = "phicade.xr.v1" as const;
export const MAX_WIRE_BYTES = 32_768;
export const BUTTONS = ["A", "B", "X", "Y", "START", "SELECT", "UP", "DOWN", "LEFT", "RIGHT", "L", "R", "L2", "R2", "L3", "R3", "GUN_TRIGGER", "GUN_RELOAD", "GUN_OFFSCREEN"] as const;
export const AXES = ["LEFT_X", "LEFT_Y", "RIGHT_X", "RIGHT_Y", "GUN_X", "GUN_Y"] as const;
export type XrAction = { kind: "button"; button: string; pressed: boolean } | { kind: "axis"; axis: string; value: number };
export type Vec3 = { x: number; y: number; z: number };
export type Pose = { position: Vec3; orientation: { x: number; y: number; z: number; w: number } };
export type Presence = { head: Pose; left: Pose | null; right: Pose | null };
export type InputPacket = { kind: "input"; protocol: typeof XR_PROTOCOL; sequence: number; sentAt: number; actions: XrAction[] };
export type PeerMessage = InputPacket
  | { kind: "presence"; protocol: typeof XR_PROTOCOL; pose: Presence }
  | { kind: "ping" | "pong"; protocol: typeof XR_PROTOCOL; sentAt: number }
  | { kind: "room"; protocol: typeof XR_PROTOCOL; roomId: string; owner: string | null; players: { id: string; role: "controller" | "spectator"; pose: Presence | null }[] }
  | { kind: "status"; protocol: typeof XR_PROTOCOL; text: string };

export function isAction(value: unknown): value is XrAction {
  if (!value || typeof value !== "object") return false;
  const a = value as Record<string, unknown>;
  return (a.kind === "button" && BUTTONS.includes(a.button as typeof BUTTONS[number]) && typeof a.pressed === "boolean") ||
    (a.kind === "axis" && AXES.includes(a.axis as typeof AXES[number]) && Number.isInteger(a.value) && Number(a.value) >= -32768 && Number(a.value) <= 32767);
}
export function isPose(value: unknown): value is Pose {
  if (!value || typeof value !== "object") return false;
  const p = value as Pose;
  if (!p.position || !p.orientation) return false;
  const n = Object.values(p.orientation);
  return [p.position.x, p.position.y, p.position.z].every(v => Number.isFinite(v) && Math.abs(v) <= 100) &&
    n.length === 4 && n.every(v => Number.isFinite(v) && Math.abs(v) <= 1.01) &&
    Math.abs(n.reduce((s, v) => s + v * v, 0) - 1) < 0.05;
}
export function isPresence(value: unknown): value is Presence {
  if (!value || typeof value !== "object") return false;
  const p = value as Presence;
  return isPose(p.head) && (p.left === null || isPose(p.left)) && (p.right === null || isPose(p.right));
}
export function parsePeerMessage(data: unknown): PeerMessage {
  if (typeof data !== "string" || new TextEncoder().encode(data).length > MAX_WIRE_BYTES) throw new Error("XR message exceeds limit");
  const m = JSON.parse(data) as Record<string, unknown>;
  if (!m || m.protocol !== XR_PROTOCOL) throw new Error("XR protocol mismatch");
  if (m.kind === "input" && Number.isSafeInteger(m.sequence) && Number(m.sequence) > 0 && Number.isFinite(m.sentAt) &&
      Array.isArray(m.actions) && m.actions.length > 0 && m.actions.length <= 32 && m.actions.every(isAction)) return m as InputPacket;
  if (m.kind === "presence" && isPresence(m.pose)) return m as PeerMessage;
  if ((m.kind === "ping" || m.kind === "pong") && Number.isFinite(m.sentAt)) return m as PeerMessage;
  if (m.kind === "status" && typeof m.text === "string" && m.text.length <= 240) return m as PeerMessage;
  if (m.kind === "room" && typeof m.roomId === "string" && m.roomId.length <= 64 && (m.owner === null || typeof m.owner === "string") &&
      Array.isArray(m.players) && m.players.length <= 2 && m.players.every(p => p && typeof p.id === "string" && p.id.length <= 64 &&
        (p.role === "controller" || p.role === "spectator") && (p.pose === null || isPresence(p.pose)))) return m as PeerMessage;
  throw new Error("Invalid XR message");
}
