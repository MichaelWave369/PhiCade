import { describe, it, expect } from "vitest";
import { calibrate, mapScreenRay, PhiLightGun } from "../lightgun/PhiLightGun";
import { PhotonRange } from "../lightgun/range";
import {
  HOLLER_MAP,
  HOLLER_PROFILE,
  HollerCircuit,
  reconstruct,
} from "../worldwalk/scene";
import { XR_PROTOCOL, parsePeerMessage } from "../protocol";
import { ArcadeRoom } from "../multiplayer/room";

describe("PhiLightGun screen space", () => {
  it("maps the center and screen corners with libretro Y increasing downward", () => {
    expect(
      mapScreenRay({ x: 0, y: 0, z: 2 }, { x: 0, y: 0, z: -1 }, 4, 3),
    ).toMatchObject({ u: 0.5, v: 0.5, x: 0, y: 0, offscreen: false });
    expect(
      mapScreenRay({ x: -2, y: 1.5, z: 2 }, { x: 0, y: 0, z: -1 }, 4, 3),
    ).toMatchObject({ u: 0, v: 0, x: -32767, y: -32767 });
    expect(
      mapScreenRay({ x: 2, y: -1.5, z: 2 }, { x: 0, y: 0, z: -1 }, 4, 3),
    ).toMatchObject({ x: 32767, y: 32767 });
  });
  it("rejects parallel, backward, behind-screen and nonfinite rays", () => {
    for (const [o, d] of [
      [
        { x: 0, y: 0, z: 2 },
        { x: 0, y: 0, z: 0 },
      ],
      [
        { x: 0, y: 0, z: 2 },
        { x: 0, y: 0, z: 1 },
      ],
      [
        { x: 0, y: 0, z: -2 },
        { x: 0, y: 0, z: -1 },
      ],
      [
        { x: NaN, y: 0, z: 2 },
        { x: 0, y: 0, z: -1 },
      ],
    ])
      expect(mapScreenRay(o, d, 4, 3)).toMatchObject({
        offscreen: true,
        x: -32768,
      });
  });
  it("fits a reproducible skew/offset calibration and rejects collinear points", () => {
    const c = calibrate(
      [
        [0.1, 0.2],
        [0.7, 0.2],
        [0.4, 0.8],
      ],
      [
        [0.2, 0.2],
        [0.8, 0.2],
        [0.5, 0.8],
      ],
    );
    expect(c[0]).toBeCloseTo(1);
    expect(c[2]).toBeCloseTo(0.1);
    expect(c[4]).toBeCloseTo(1);
    expect(() =>
      calibrate(
        [
          [0, 0],
          [0.5, 0.5],
          [1, 1],
        ],
        [
          [0, 0],
          [0.5, 0.5],
          [1, 1],
        ],
      ),
    ).toThrow();
  });
  it("fires on trigger edges, reloads offscreen and releases after tracking loss", () => {
    const g = new PhiLightGun(),
      center = mapScreenRay({ x: 0, y: 0, z: 1 }, { x: 0, y: 0, z: -1 }, 4, 3),
      off = mapScreenRay({ x: 4, y: 0, z: 1 }, { x: 0, y: 0, z: -1 }, 4, 3);
    expect(g.sample(center, true, false).fired).toBe(true);
    expect(g.sample(center, true, false).fired).toBe(false);
    g.sample(center, false, false);
    expect(g.sample(off, true, false)).toMatchObject({
      fired: false,
      reloaded: true,
    });
    expect(g.neutral()).toContainEqual({
      kind: "button",
      button: "GUN_TRIGGER",
      pressed: false,
    });
  });
  it("has independent magazines for both controllers", () => {
    const r = new PhotonRange();
    expect(r.fire(0.5, 0.5, 2)).toBe(true);
    expect(r.ammo).toEqual([6, 5]);
    r.reload(2);
    expect(r.ammo).toEqual([6, 6]);
  });
});
describe("WorldWalk explicit reconstruction", () => {
  it("reconstructs identical geometry deterministically without cartridges", () => {
    expect(reconstruct(HOLLER_PROFILE, HOLLER_MAP)).toEqual(
      reconstruct(HOLLER_PROFILE, HOLLER_MAP),
    );
    expect(
      reconstruct(HOLLER_PROFILE, HOLLER_MAP).filter((t) => t.kind === "coin"),
    ).toHaveLength(3);
    expect(() => reconstruct("arbitrary-rom", HOLLER_MAP)).toThrow(
      /Unsupported/,
    );
  });
  it("preserves canonical 2D collision and win rules", () => {
    const game = new HollerCircuit();
    expect(game.move(-1, 0)).toBe(false);
    expect(game.move(1, 1)).toBe(false);
    expect(game.move(1, 0)).toBe(true);
    expect([game.x, game.z]).toEqual([2, 1]);
    game.reset();
    // Breadth-first oracle collects every light before reaching the exit.
    const queue = [{ x: 1, z: 1, mask: 0, path: [] as [number, number][] }],
      seen = new Set<string>();
    let solution: [number, number][] = [];
    const coins = ["9,1", "4,4", "1,6"];
    while (queue.length) {
      const n = queue.shift()!;
      if (n.x === 9 && n.z === 7 && n.mask === 7) {
        solution = n.path;
        break;
      }
      for (const [dx, dz] of [
        [1, 0],
        [-1, 0],
        [0, 1],
        [0, -1],
      ]) {
        const x = n.x + dx,
          z = n.z + dz,
          t = HOLLER_MAP[z]?.[x];
        if (!t || t === "#") continue;
        const c = coins.indexOf(`${x},${z}`),
          mask = n.mask | (c < 0 ? 0 : 1 << c),
          key = `${x},${z},${mask}`;
        if (!seen.has(key)) {
          seen.add(key);
          queue.push({ x, z, mask, path: [...n.path, [dx, dz]] });
        }
      }
    }
    expect(solution.length).toBeGreaterThan(0);
    for (const [dx, dz] of solution) game.move(dx, dz);
    expect(game.won).toBe(true);
    expect(game.collected.size).toBe(3);
  });
});
describe("network and room boundary", () => {
  it("accepts only bounded known inputs and refuses privileged actions", () => {
    const packet = {
      kind: "input",
      protocol: XR_PROTOCOL,
      sequence: 1,
      sentAt: 0,
      actions: [{ kind: "button", button: "A", pressed: true }],
    };
    expect(parsePeerMessage(JSON.stringify(packet))).toMatchObject(packet);
    for (const actions of [
      [{ kind: "system", command: "load-state" }],
      [{ kind: "axis", axis: "GUN_X", value: 40000 }],
      Array(33).fill(packet.actions[0]),
    ])
      expect(() =>
        parsePeerMessage(JSON.stringify({ ...packet, actions })),
      ).toThrow();
    expect(() => parsePeerMessage("x".repeat(32769))).toThrow();
    expect(() =>
      parsePeerMessage(JSON.stringify({ ...packet, protocol: "v9" })),
    ).toThrow();
  });
  it("keeps a second user a spectator until host grants ownership", () => {
    const room = new ArcadeRoom("test", "host");
    room.join("guest");
    expect(room.canInput("guest")).toBe(false);
    expect(() => room.grant("guest", "guest")).toThrow();
    room.grant("host", "guest");
    expect(room.canInput("host")).toBe(false);
    expect(room.canInput("guest")).toBe(true);
    expect(() => room.join("third")).toThrow();
    room.leave("guest");
    expect(room.canInput("host")).toBe(true);
  });
});
