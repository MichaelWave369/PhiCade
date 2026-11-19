/** Original deterministic target game, shared by canvas preview and XR screen. */
export class PhotonRange {
  score = 0; shots = 0; ammo = [6,6]; hits = [0,0]; target = { u: 0.5, v: 0.5 }; round = 0;
  fire(u: number, v: number, seat: 1 | 2 = 1): boolean {
    const p = seat - 1;
    if (!Number.isFinite(u) || !Number.isFinite(v) || this.ammo[p] === 0) return false;
    this.ammo[p]--; this.shots++;
    if (Math.hypot((u-this.target.u)*4/3, v-this.target.v) > 0.095) return false;
    this.score += 100; this.hits[p]++; this.round++;
    this.target = { u: 0.17 + ((this.round*37)%67)/100, v: 0.22 + ((this.round*23)%52)/100 };
    return true;
  }
  reload(seat: 1 | 2 = 1) { this.ammo[seat-1] = 6; }
}
