import type { Vec3, XrAction } from "../protocol";
export type Calibration = readonly [
  number,
  number,
  number,
  number,
  number,
  number,
];
export const IDENTITY_CALIBRATION: Calibration = [1, 0, 0, 0, 1, 0];
export type Aim = {
  u: number;
  v: number;
  x: number;
  y: number;
  offscreen: boolean;
};
export function mapVideoContent(
  aim: Aim,
  videoWidth: number,
  videoHeight: number,
): Aim {
  if (aim.offscreen || videoWidth <= 0 || videoHeight <= 0) return aim;
  const aspect = videoWidth / videoHeight,
    screenAspect = 4 / 3;
  const contentWidth = Math.min(1, aspect / screenAspect),
    contentHeight = Math.min(1, screenAspect / aspect);
  const u = (aim.u - (1 - contentWidth) / 2) / contentWidth,
    v = (aim.v - (1 - contentHeight) / 2) / contentHeight;
  if (u < 0 || u > 1 || v < 0 || v > 1)
    return { u, v, x: -32768, y: -32768, offscreen: true };
  return {
    u,
    v,
    x: Math.round(u * 65534 - 32767),
    y: Math.round(v * 65534 - 32767),
    offscreen: false,
  };
}
export type GunProfile = {
  id: string;
  hand: "left" | "right";
  seat: 1 | 2;
  offscreenReload: boolean;
  reloadButton: boolean;
  calibration: Calibration;
};
export const RANGE_PROFILE: GunProfile = {
  id: "photon-range-v1",
  hand: "right",
  seat: 1,
  offscreenReload: true,
  reloadButton: true,
  calibration: IDENTITY_CALIBRATION,
};

/** Ray is transformed into the screen's local space; screen faces local +Z. */
export function mapScreenRay(
  origin: Vec3,
  direction: Vec3,
  width: number,
  height: number,
  c: Calibration = IDENTITY_CALIBRATION,
): Aim {
  const off = { u: -1, v: -1, x: -32768, y: -32768, offscreen: true };
  if (
    ![
      ...Object.values(origin),
      ...Object.values(direction),
      width,
      height,
      ...c,
    ].every(Number.isFinite) ||
    width <= 0 ||
    height <= 0 ||
    direction.z >= -1e-7 ||
    origin.z < 0
  )
    return off;
  const t = -origin.z / direction.z;
  if (t < 0 || t > 100) return off;
  const rawU = (origin.x + t * direction.x) / width + 0.5;
  const rawV = 0.5 - (origin.y + t * direction.y) / height;
  const u = c[0] * rawU + c[1] * rawV + c[2];
  const v = c[3] * rawU + c[4] * rawV + c[5];
  if (u < 0 || u > 1 || v < 0 || v > 1) return { ...off, u, v };
  return {
    u,
    v,
    x: Math.round(u * 65534 - 32767),
    y: Math.round(v * 65534 - 32767),
    offscreen: false,
  };
}

/** Fit an affine transform to three measured/desired screen points. */
export function calibrate(
  measured: readonly [number, number][],
  desired: readonly [number, number][],
): Calibration {
  if (
    measured.length !== 3 ||
    desired.length !== 3 ||
    ![...measured.flat(), ...desired.flat()].every(Number.isFinite)
  )
    throw new Error("Calibration requires three finite points");
  const [[a, b], [d, e], [g, h]] = measured;
  const det = a * (e - h) - b * (d - g) + d * h - e * g;
  if (Math.abs(det) < 0.001)
    throw new Error("Calibration points must form a triangle");
  const fit = (j: number) => {
    const [x, y, z] = desired.map((p) => p[j]);
    return [
      (x * (e - h) + y * (h - b) + z * (b - e)) / det,
      (x * (g - d) + y * (a - g) + z * (d - a)) / det,
      (x * (d * h - e * g) + y * (b * g - a * h) + z * (a * e - b * d)) / det,
    ];
  };
  const c = [...fit(0), ...fit(1)] as unknown as Calibration;
  if (c.some((v) => Math.abs(v) > 10))
    throw new Error("Calibration outside supported range");
  return c;
}

export class PhiLightGun {
  private previousTrigger = false;
  private previousReload = false;
  constructor(public profile: GunProfile = { ...RANGE_PROFILE }) {}
  sample(
    aim: Aim,
    trigger: boolean,
    reload: boolean,
  ): { actions: XrAction[]; fired: boolean; reloaded: boolean } {
    const rising = trigger && !this.previousTrigger;
    const reloaded =
      (rising && aim.offscreen && this.profile.offscreenReload) ||
      (reload && !this.previousReload && this.profile.reloadButton);
    const fired = rising && !aim.offscreen;
    const actions: XrAction[] = [
      { kind: "axis", axis: "GUN_X", value: aim.x },
      { kind: "axis", axis: "GUN_Y", value: aim.y },
      { kind: "button", button: "GUN_OFFSCREEN", pressed: aim.offscreen },
      {
        kind: "button",
        button: "GUN_TRIGGER",
        pressed: trigger && !aim.offscreen,
      },
      { kind: "button", button: "GUN_RELOAD", pressed: reloaded },
    ];
    this.previousTrigger = trigger;
    this.previousReload = reload;
    return { actions, fired, reloaded };
  }
  neutral(): XrAction[] {
    this.previousTrigger = this.previousReload = false;
    return this.sample(
      { u: -1, v: -1, x: -32768, y: -32768, offscreen: true },
      false,
      false,
    ).actions;
  }
}
