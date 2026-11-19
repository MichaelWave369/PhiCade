import { isPose, type Pose } from "../protocol";
export type MotionProfile = {
  id: string;
  pointerYawDegrees: number;
  pointerPitchDegrees: number;
  swingMetersPerSecond: number;
  shakeMetersPerSecondSquared: number;
  cooldownMs: number;
};
export const QUEST_WII_PROFILE: MotionProfile = {
  id: "quest-wii-motion-v1",
  pointerYawDegrees: 45,
  pointerPitchDegrees: 35,
  swingMetersPerSecond: 1.8,
  shakeMetersPerSecondSquared: 9,
  cooldownMs: 350,
};
/** Deterministic pose translator. DolphinXR consumes its own native OpenXR data;
 * these browser samples never imply an unsupported emulator pose-injection API. */
export class WiiMotionAdapter {
  private previous: {
    pose: Pose;
    time: number;
    velocity: [number, number, number];
  } | null = null;
  private gestureAt = -Infinity;
  constructor(public profile = { ...QUEST_WII_PROFILE }) {}
  reset() {
    this.previous = null;
    this.gestureAt = -Infinity;
  }
  sample(
    pose: Pose,
    timeMs: number,
    stick: readonly number[] = [0, 0],
    c = false,
    z = false,
  ) {
    if (!isPose(pose) || !Number.isFinite(timeMs))
      throw new Error("Invalid tracked motion pose");
    const q = pose.orientation;
    const pitch = Math.asin(
      Math.max(-1, Math.min(1, 2 * (q.w * q.x - q.y * q.z))),
    );
    const yaw = Math.atan2(
      2 * (q.w * q.y + q.x * q.z),
      1 - 2 * (q.x * q.x + q.y * q.y),
    );
    const roll = Math.atan2(
      2 * (q.w * q.z + q.x * q.y),
      1 - 2 * (q.x * q.x + q.z * q.z),
    );
    const pointerX = yaw / ((this.profile.pointerYawDegrees * Math.PI) / 180),
      pointerY = -pitch / ((this.profile.pointerPitchDegrees * Math.PI) / 180);
    let velocity: [number, number, number] = [0, 0, 0],
      speed = 0,
      acceleration = 0;
    const dt = this.previous ? (timeMs - this.previous.time) / 1000 : 0;
    if (this.previous && dt >= 0.005 && dt <= 0.1) {
      const p = pose.position,
        b = this.previous.pose.position;
      velocity = [(p.x - b.x) / dt, (p.y - b.y) / dt, (p.z - b.z) / dt];
      speed = Math.hypot(...velocity);
      acceleration = Math.hypot(
        ...velocity.map((v, i) => (v - this.previous!.velocity[i]) / dt),
      );
    }
    // Ignore discontinuous tracking jumps, which are not a physical gesture.
    const continuous = dt >= 0.005 && dt <= 0.1 && speed < 8;
    const ready = timeMs - this.gestureAt >= this.profile.cooldownMs;
    const swing =
      continuous && ready && speed >= this.profile.swingMetersPerSecond;
    const shake =
      continuous &&
      ready &&
      acceleration >= this.profile.shakeMetersPerSecondSquared;
    if (swing || shake) this.gestureAt = timeMs;
    this.previous = { pose: structuredClone(pose), time: timeMs, velocity };
    const axis = (v: number) =>
      Number.isFinite(v) ? Math.max(-1, Math.min(1, v)) : 0;
    return {
      orientation: { pitch, yaw, roll },
      sensorBar: {
        x: axis(pointerX),
        y: axis(pointerY),
        visible: Math.abs(pointerX) <= 1 && Math.abs(pointerY) <= 1,
      },
      nunchuk: { x: axis(stick[0] ?? 0), y: axis(stick[1] ?? 0), c, z },
      swing,
      shake,
      speedMetersPerSecond: speed,
    };
  }
}
