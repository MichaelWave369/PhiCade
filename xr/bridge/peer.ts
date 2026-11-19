import {
  MAX_WIRE_BYTES,
  XR_PROTOCOL,
  parsePeerMessage,
  type PeerMessage,
  type XrAction,
  type Presence,
} from "../protocol";
export type Role = "controller" | "spectator";
export type Signal = {
  schema: "phicade.xr-pair.v1";
  kind: "offer" | "answer";
  roomId: string;
  invitation: string;
  role: Role;
  sdp: RTCSessionDescriptionInit;
};
const token = () =>
  Array.from(crypto.getRandomValues(new Uint8Array(24)), (b) =>
    b.toString(16).padStart(2, "0"),
  ).join("");
export function encodeSignal(s: Signal): string {
  return btoa(
    String.fromCharCode(...new TextEncoder().encode(JSON.stringify(s))),
  );
}
export function decodeSignal(text: string): Signal {
  if (text.length > 180_000) throw new Error("Pairing code too large");
  const s = JSON.parse(
    new TextDecoder().decode(
      Uint8Array.from(atob(text.trim()), (c) => c.charCodeAt(0)),
    ),
  ) as Signal;
  if (
    s.schema !== "phicade.xr-pair.v1" ||
    !["offer", "answer"].includes(s.kind) ||
    s.sdp?.type !== s.kind ||
    typeof s.sdp.sdp !== "string" ||
    s.sdp.sdp.length > 100_000 ||
    !/^[a-zA-Z0-9_-]{1,64}$/.test(s.roomId) ||
    !/^[a-f0-9]{48}$/.test(s.invitation) ||
    !["controller", "spectator"].includes(s.role)
  )
    throw new Error("Invalid pairing code");
  return s;
}
export class ManualPeer {
  readonly pc: RTCPeerConnection;
  channel: RTCDataChannel | null = null;
  private invitation = "";
  private roomId = "";
  role: Role = "spectator";
  private sequence = 0;
  private pingAt = 0;
  private timer: ReturnType<typeof setInterval> | null = null;
  onMessage: (m: PeerMessage) => void = () => {};
  onStream: (stream: MediaStream) => void = () => {};
  onState: (state: string) => void = () => {};
  onLatency: (rtt: number) => void = () => {};
  constructor(stream?: MediaStream, bitrate = 8_000_000) {
    this.pc = new RTCPeerConnection({
      iceServers: [],
      bundlePolicy: "max-bundle",
    });
    stream?.getTracks().forEach((t) => {
      const s = this.pc.addTrack(t, stream);
      if (t.kind === "video") void s.getParameters();
    });
    this.pc.ontrack = (e) =>
      this.onStream(e.streams[0] ?? new MediaStream([e.track]));
    this.pc.ondatachannel = (e) => this.attach(e.channel);
    this.pc.onconnectionstatechange = () => {
      const state = this.pc.connectionState;
      this.onState(state);
      if (state === "connected") {
        for (const s of this.pc.getSenders()) {
          if (s.track?.kind === "video") {
            const p = s.getParameters();
            p.encodings = p.encodings?.length ? p.encodings : [{}];
            p.encodings[0].maxBitrate = bitrate;
            void s.setParameters(p).catch(() => {});
          }
        }
      }
      if (state === "failed" || state === "closed") this.clearTimer();
    };
  }
  private attach(channel: RTCDataChannel) {
    if (this.channel && this.channel !== channel) {
      channel.close();
      return;
    }
    this.channel = channel;
    channel.onopen = () => {
      this.onState("paired");
      this.timer = setInterval(() => {
        this.pingAt = performance.now();
        this.send({ kind: "ping", protocol: XR_PROTOCOL, sentAt: this.pingAt });
      }, 1500);
    };
    channel.onclose = () => {
      this.clearTimer();
      this.onState("disconnected");
    };
    channel.onmessage = (e) => {
      try {
        const m = parsePeerMessage(e.data);
        if (m.kind === "ping") {
          this.send({ ...m, kind: "pong" });
          return;
        }
        if (m.kind === "pong") {
          if (m.sentAt === this.pingAt)
            this.onLatency(Math.max(0, performance.now() - m.sentAt));
          return;
        }
        this.onMessage(m);
      } catch {
        this.close();
        this.onState("rejected invalid message");
      }
    };
  }
  private async gathered(): Promise<void> {
    if (this.pc.iceGatheringState === "complete") return;
    await new Promise<void>((resolve, reject) => {
      const timeout = setTimeout(() => {
        cleanup();
        reject(
          new Error(
            "Local ICE gathering timed out; create a fresh pairing code.",
          ),
        );
      }, 12_000);
      const done = () => {
        if (this.pc.iceGatheringState === "complete") {
          cleanup();
          resolve();
        }
      };
      const cleanup = () => {
        clearTimeout(timeout);
        this.pc.removeEventListener("icegatheringstatechange", done);
      };
      this.pc.addEventListener("icegatheringstatechange", done);
    });
  }
  async offer(role: Role = "controller", roomId = token()): Promise<string> {
    this.role = role;
    this.roomId = roomId;
    this.invitation = token();
    this.attach(this.pc.createDataChannel("phicade-xr-v1", { ordered: true }));
    await this.pc.setLocalDescription(await this.pc.createOffer());
    await this.gathered();
    return encodeSignal({
      schema: "phicade.xr-pair.v1",
      kind: "offer",
      roomId: this.roomId,
      invitation: this.invitation,
      role,
      sdp: this.pc.localDescription!.toJSON(),
    });
  }
  async answer(text: string): Promise<string> {
    const offer = decodeSignal(text);
    if (offer.kind !== "offer") throw new Error("Expected a host offer");
    this.role = offer.role;
    this.roomId = offer.roomId;
    this.invitation = offer.invitation;
    await this.pc.setRemoteDescription(offer.sdp);
    await this.pc.setLocalDescription(await this.pc.createAnswer());
    await this.gathered();
    return encodeSignal({
      ...offer,
      kind: "answer",
      sdp: this.pc.localDescription!.toJSON(),
    });
  }
  async accept(text: string): Promise<void> {
    const answer = decodeSignal(text);
    if (
      answer.kind !== "answer" ||
      answer.roomId !== this.roomId ||
      answer.invitation !== this.invitation ||
      answer.role !== this.role
    )
      throw new Error("Answer does not match this invitation");
    await this.pc.setRemoteDescription(answer.sdp);
    this.invitation = "";
  }
  send(message: PeerMessage): boolean {
    if (this.channel?.readyState !== "open") return false;
    if (this.channel.bufferedAmount > 64_000) {
      if (message.kind === "input") {
        this.close();
        throw new Error(
          "Input channel congested; disconnected to release controls",
        );
      }
      return false;
    }
    const wire = JSON.stringify(message);
    if (new TextEncoder().encode(wire).length > MAX_WIRE_BYTES)
      throw new Error("XR message too large");
    this.channel.send(wire);
    return true;
  }
  input(actions: XrAction[]) {
    if (this.role === "controller" && actions.length)
      this.send({
        kind: "input",
        protocol: XR_PROTOCOL,
        sequence: ++this.sequence,
        sentAt: performance.now(),
        actions,
      });
  }
  presence(pose: Presence) {
    this.send({ kind: "presence", protocol: XR_PROTOCOL, pose });
  }
  async stats() {
    const report = await this.pc.getStats();
    const result: {
      rttMs?: number;
      frames?: number;
      bytes?: number;
      decodeMs?: number;
    } = {};
    report.forEach((s) => {
      if (s.type === "candidate-pair" && s.state === "succeeded")
        result.rttMs = s.currentRoundTripTime * 1000;
      if (s.type === "inbound-rtp" && s.kind === "video") {
        result.frames = s.framesDecoded;
        result.bytes = s.bytesReceived;
        result.decodeMs = s.framesDecoded
          ? (s.totalDecodeTime / s.framesDecoded) * 1000
          : 0;
      }
    });
    return result;
  }
  private clearTimer() {
    if (this.timer) clearInterval(this.timer);
    this.timer = null;
  }
  close() {
    this.clearTimer();
    this.channel?.close();
    this.pc.close();
  }
}
