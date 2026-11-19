import { describe, it, expect } from "vitest";
import { WiiMotionAdapter } from "../motion/WiiMotionAdapter";
import { questGamepad, neutralInput } from "../interaction/gamepad";
import { mapVideoContent } from "../lightgun/PhiLightGun";
import { isPose, type Pose } from "../protocol";
import { parsePeerMessage } from "../protocol";
import centerHit from "../fixtures/input-center-hit.json";
const pose = (x = 0): Pose => ({
  position: { x, y: 0, z: 0 },
  orientation: { x: 0, y: 0, z: 0, w: 1 },
});
describe("motion and controller adapters", () => {
  it("uses the same wire fixture as the real native ABI qualifier", () => {
    expect(parsePeerMessage(JSON.stringify(centerHit))).toEqual(centerHit);
  });
  it("maps forward orientation to sensor-bar center and clamps Nunchuk analogs", () => {
    const m = new WiiMotionAdapter().sample(pose(), 0, [2, -2], true, false);
    expect(m.sensorBar).toEqual({ x: 0, y: -0, visible: true });
    expect(m.nunchuk).toEqual({ x: 1, y: -1, c: true, z: false });
  });
  it("detects bounded swing/shake pulses with cooldown and ignores tracking jumps", () => {
    const m = new WiiMotionAdapter();
    m.sample(pose(0), 0);
    expect(m.sample(pose(0.06), 20).swing).toBe(true);
    expect(m.sample(pose(0.12), 40).swing).toBe(false);
    expect(m.sample(pose(30), 1000).swing).toBe(false);
    expect(() =>
      m.sample({ ...pose(), position: { x: NaN, y: 0, z: 0 } }, 1100),
    ).toThrow();
  });
  it("maps Quest trigger, face buttons, sticks and handedness to normal Action Bus names", () => {
    const buttons = Array.from({ length: 6 }, (_, i) => ({
      pressed: i === 0 || i === 4,
      touched: true,
      value: 1,
    }));
    const a = questGamepad({ buttons, axes: [0, 0, 0.5, -1] }, "right");
    expect(a).toContainEqual({ kind: "button", button: "R2", pressed: true });
    expect(a).toContainEqual({ kind: "button", button: "A", pressed: true });
    expect(a).toContainEqual({ kind: "axis", axis: "RIGHT_Y", value: -32767 });
    expect(
      neutralInput(false).some((a) =>
        (a.kind === "button" ? a.button : a.axis).startsWith("GUN_"),
      ),
    ).toBe(false);
    expect(neutralInput(true)).toContainEqual({
      kind: "axis",
      axis: "GUN_X",
      value: -32768,
    });
  });
  it("maps video letterboxing before emitting emulator coordinates", () => {
    const a = { u: 0.5, v: 0.5, x: 0, y: 0, offscreen: false };
    expect(mapVideoContent(a, 1920, 1080)).toMatchObject({
      x: 0,
      y: 0,
      offscreen: false,
    });
    expect(mapVideoContent({ ...a, v: 0.05 }, 1920, 1080).offscreen).toBe(true);
    expect(mapVideoContent({ ...a, u: 0.05 }, 256, 240).offscreen).toBe(true);
    expect(mapVideoContent({ ...a, u: 0.125 }, 256, 240).x).toBeCloseTo(
      -30719,
      0,
    );
  });
  it("rejects invalid poses rather than broadcasting corrupt avatar transforms", () => {
    expect(isPose(pose())).toBe(true);
    expect(isPose({ ...pose(), orientation: { x: 0, y: 0, z: 0, w: 4 } })).toBe(
      false,
    );
  });
});
