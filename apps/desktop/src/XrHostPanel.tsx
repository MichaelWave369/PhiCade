import { useEffect, useRef, useState, type RefObject } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ManualPeer } from "../../../xr/bridge/peer";
import { ArcadeRoom } from "../../../xr/multiplayer/room";
import { XR_PROTOCOL, type PeerMessage } from "../../../xr/protocol";

export function XrHostPanel({
  canvasRef,
  audioContextRef,
  audioStreamRef,
  running,
  sessionKey,
}: {
  canvasRef: RefObject<HTMLCanvasElement | null>;
  audioContextRef: RefObject<AudioContext | null>;
  audioStreamRef: RefObject<MediaStreamAudioDestinationNode | null>;
  running: boolean;
  sessionKey: string | null;
}) {
  const peerRef = useRef<ManualPeer | null>(null),
    streamRef = useRef<MediaStream | null>(null);
  const roomRef = useRef<ArcadeRoom | null>(null),
    grantRef = useRef<{ peerId: string; token: string } | null>(null);
  const [offer, setOffer] = useState(""),
    [answer, setAnswer] = useState(""),
    [notice, setNotice] = useState(
      "Launch a game, then approve a paired XR session.",
    ),
    [hosting, setHosting] = useState(false);
  const [spectator, setSpectator] = useState(false),
    [lightgun, setLightgun] = useState(false),
    [fps, setFps] = useState(60),
    [bitrate, setBitrate] = useState(8),
    [resolution, setResolution] = useState(960);
  const cleanup = () => {
    grantRef.current = null;
    peerRef.current?.close();
    peerRef.current = null;
    streamRef.current?.getVideoTracks().forEach((t) => t.stop());
    streamRef.current = null;
    roomRef.current = null;
    const revoked = invoke("xr_revoke_host").catch(() => {});
    setHosting(false);
    setOffer("");
    setAnswer("");
    return revoked;
  };
  useEffect(() => {
    if (!running) void cleanup();
    return () => {
      void cleanup();
    };
  }, [running, sessionKey]);
  async function host() {
    try {
      await cleanup();
      if (!running || !canvasRef.current)
        throw new Error(
          "A live software-frame session is required; external N64 windows cannot be streamed here.",
        );
      setNotice("Creating an approved session and gathering local candidates…");
      setHosting(true);
      const peerId = crypto.randomUUID(),
        token = Array.from(crypto.getRandomValues(new Uint8Array(24)), (b) =>
          b.toString(16).padStart(2, "0"),
        ).join("");
      if (!spectator) {
        await invoke("xr_open_host", {
          peerId,
          token,
          lightgun,
          operatorApproved: true,
        });
        grantRef.current = { peerId, token };
      }
      // Bound capture dimensions without modifying the emulator display canvas.
      const source = canvasRef.current,
        capture = document.createElement("canvas");
      capture.width = resolution;
      capture.height = Math.round(
        (resolution * source.height) / Math.max(1, source.width),
      );
      const ctx = capture.getContext("2d")!;
      let animation = 0,
        last = 0;
      const copy = (time: number) => {
        if (streamRef.current) {
          animation = requestAnimationFrame(copy);
          if (time - last >= 1000 / fps) {
            ctx.drawImage(source, 0, 0, capture.width, capture.height);
            last = time;
          }
        }
      };
      const stream = capture.captureStream(fps);
      streamRef.current = stream;
      const context = audioContextRef.current;
      if (context) {
        await context.resume();
        if (!audioStreamRef.current)
          audioStreamRef.current = context.createMediaStreamDestination();
        audioStreamRef.current.stream
          .getAudioTracks()
          .forEach((t) => stream.addTrack(t));
      }
      const room = new ArcadeRoom(crypto.randomUUID(), "host");
      room.join("guest");
      if (!spectator) room.grant("host", "guest");
      roomRef.current = room;
      const p = new ManualPeer(stream, bitrate * 1_000_000);
      peerRef.current = p;
      stream
        .getVideoTracks()[0]
        .addEventListener("ended", () => cancelAnimationFrame(animation));
      animation = requestAnimationFrame(copy);
      p.onState = (state) => {
        setNotice(`XR bridge: ${state}`);
        if (state === "paired") {
          p.send(room.snapshot());
          p.send({
            kind: "capabilities",
            protocol: XR_PROTOCOL,
            inputDevice: lightgun ? "lightgun" : "gamepad",
            playablePorts: 1,
            nativeLaunch: false,
          });
        }
        if (state === "failed" || state === "disconnected") {
          cleanup();
          setNotice(
            "Connection lost; controls released. Create a fresh offer to reconnect.",
          );
        }
      };
      p.onLatency = (ms) =>
        setNotice(
          `XR paired · ${ms.toFixed(0)} ms round trip · ${spectator ? "SPECTATOR" : "SEAT 1"} · ${fps} FPS / ${bitrate} Mbps target`,
        );
      p.onMessage = (m: PeerMessage) => {
        if (m.kind === "presence") {
          room.update("guest", m.pose);
          p.send(room.snapshot());
          return;
        }
        if (m.kind !== "input" || !room.canInput("guest") || !grantRef.current)
          return;
        void invoke("xr_submit_input", {
          ...grantRef.current,
          packet: m,
        }).catch((e) => {
          cleanup();
          setNotice(`Native XR input rejected: ${String(e)}`);
        });
      };
      setOffer(await p.offer(spectator ? "spectator" : "controller", room.id));
      setNotice(
        "Offer ready. Paste into the Quest browser, create its answer, then paste that answer below.",
      );
    } catch (e) {
      cleanup();
      setNotice(String(e));
    }
  }
  return (
    <details className="xr-host-panel">
      <summary>XR HOST · QUEST / SHARED ARCADE</summary>
      <p>
        Share this live game with one explicitly paired headset. A controller
        peer receives seat 1 gameplay input through native AuthorityPolicy. A
        spectator receives video/audio only.
      </p>
      <div className="xr-host-options">
        <label>
          <input
            type="checkbox"
            checked={spectator}
            onChange={(e) => setSpectator(e.target.checked)}
            disabled={hosting}
          />{" "}
          Spectator only
        </label>
        <label>
          <input
            type="checkbox"
            checked={lightgun}
            onChange={(e) => setLightgun(e.target.checked)}
            disabled={hosting}
          />{" "}
          Lightgun device · core/game profile required
        </label>
        <label>
          Video width{" "}
          <select
            value={resolution}
            onChange={(e) => setResolution(Number(e.target.value))}
            disabled={hosting}
          >
            <option value={640}>640</option>
            <option value={960}>960</option>
            <option value={1280}>1280</option>
          </select>
        </label>
        <label>
          Frame rate{" "}
          <select
            value={fps}
            onChange={(e) => setFps(Number(e.target.value))}
            disabled={hosting}
          >
            <option>30</option>
            <option>60</option>
          </select>
        </label>
        <label>
          Bitrate{" "}
          <select
            value={bitrate}
            onChange={(e) => setBitrate(Number(e.target.value))}
            disabled={hosting}
          >
            <option value={4}>4 Mbps</option>
            <option value={8}>8 Mbps</option>
            <option value={16}>16 Mbps</option>
          </select>
        </label>
      </div>
      <button onClick={() => void host()} disabled={!running || hosting}>
        APPROVE XR SESSION + CREATE OFFER
      </button>
      <label>
        Host offer
        <textarea value={offer} readOnly spellCheck={false} />
      </label>
      <label>
        Headset answer
        <textarea
          value={answer}
          onChange={(e) => setAnswer(e.target.value)}
          spellCheck={false}
        />
      </label>
      <button
        disabled={!offer || !answer}
        onClick={() =>
          void peerRef.current
            ?.accept(answer)
            .catch((e) => setNotice(String(e)))
        }
      >
        ACCEPT PAIRED HEADSET
      </button>
      <button
        onClick={() => {
          cleanup();
          setNotice("XR session revoked; controls released.");
        }}
        disabled={!hosting}
      >
        REVOKE XR SESSION
      </button>
      <output aria-live="polite">{notice}</output>
      <small>
        Pairing codes grant session access. Use the same LAN and trusted HTTPS
        on Quest. No STUN/TURN or privileged HTTP endpoint. Browser-WebRTC /
        Windows-WebView2 interoperability needs field testing.
      </small>
    </details>
  );
}
