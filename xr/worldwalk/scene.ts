/** Original MIT demo. Explicit semantic tile profile, never inferred from an arbitrary ROM. */
export const HOLLER_PROFILE = "phicade.holler-circuit.v1";
export const HOLLER_MAP = [
  "###########",
  "#P..#....o#",
  "#.#.#.##..#",
  "#.#...#...#",
  "#...o...#.#",
  "###.#.#.#.#",
  "#o..#.....#",
  "#.....#..E#",
  "###########",
] as const;
export type Tile = { x: number; z: number; kind: "wall" | "coin" | "exit" };
export function reconstruct(profile: string, rows: readonly string[]): Tile[] {
  if (profile !== HOLLER_PROFILE)
    throw new Error("Unsupported WorldWalk profile; use spatial screen");
  if (
    rows.length < 3 ||
    rows.length > 64 ||
    rows.some(
      (r) => r.length !== rows[0].length || r.length > 64 || /[^#.PoE]/.test(r),
    )
  )
    throw new Error("Invalid tile scene");
  return rows.flatMap((row, z) =>
    [...row].flatMap<Tile>((tile, x) =>
      tile === "#"
        ? [{ x, z, kind: "wall" }]
        : tile === "o"
          ? [{ x, z, kind: "coin" }]
          : tile === "E"
            ? [{ x, z, kind: "exit" }]
            : [],
    ),
  );
}
export class HollerCircuit {
  x = 1;
  z = 1;
  steps = 0;
  collected = new Set<string>();
  won = false;
  move(dx: number, dz: number): boolean {
    if (
      !Number.isInteger(dx) ||
      !Number.isInteger(dz) ||
      Math.abs(dx) + Math.abs(dz) !== 1 ||
      this.won
    )
      return false;
    const tile = HOLLER_MAP[this.z + dz]?.[this.x + dx];
    if (!tile || tile === "#") return false;
    this.x += dx;
    this.z += dz;
    this.steps++;
    if (tile === "o") this.collected.add(`${this.x},${this.z}`);
    this.won = tile === "E" && this.collected.size === 3;
    return true;
  }
  reset() {
    this.x = this.z = 1;
    this.steps = 0;
    this.collected.clear();
    this.won = false;
  }
}
