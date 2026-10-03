import { useEffect, useMemo, useRef, useState } from "react";
import { ActionBus, type GameAction } from "./actionBus";
import {
  diffGameBoyButtons,
  emptyGameBoyButtons,
  listConnectedControllers,
  readGameBoyButtons,
  type ControllerSnapshot,
  type GameBoyButtonState,
} from "./controllers";
import {
  defaultSettings,
  isNativeShell,
  loadSettings,
  saveSettings,
  scanRomDirectory,
  selectRomDirectory,
  selectSameBoyCore,
  startEmulation,
  stepEmulation,
  stopEmulation,
  validateActionEnvelope,
  type AppSettings,
  type FramePacket,
  type RomEntry,
  type SessionInfo,
} from "./native";

const systems = ["ALL", "NES", "SNES", "GB", "GBC", "GBA", "GENESIS", "PS1"] as const;

const milestones = [
  ["NATIVE SHELL", "READY", "Tauri owns native filesystem, settings, and core loading."],
  ["SAMEBOY", "QUALIFIED", "Pinned SameBoy 1.0.3 libretro core with CI provenance gate."],
  ["A/V BRIDGE", "LIVE", "Core video reaches the CRT canvas and audio reaches Web Audio."],
  ["ACTION INPUT", "LIVE", "Standard gamepad changes enter the same governed Action Bus."],
] as const;

function decodeBase64(value: string): Uint8Array {
  if (!value) return new Uint8Array();
  const binary = window.atob(value);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) bytes[i] = binary.charCodeAt(i);
  return bytes;
}

export function App() {
  const bus = useMemo(() => new ActionBus(), []);
  const native = isNativeShell();
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const frameRef = useRef(0);
  const runningRef = useRef(false);
  const gamepadRef = useRef<GameBoyButtonState>(emptyGameBoyButtons());
  const audioContextRef = useRef<AudioContext | null>(null);
  const nextAudioTimeRef = useRef(0);

  const [activeSystem, setActiveSystem] = useState<(typeof systems)[number]>("ALL");
  const [lastInput, setLastInput] = useState("NO INPUT");
  const [notice, setNotice] = useState(native ? "NATIVE SHELL ONLINE" : "WEB PREVIEW // NATIVE FEATURES OFFLINE");
  const [settings, setSettings] = useState<AppSettings>(defaultSettings);
  const [games, setGames] = useState<RomEntry[]>([]);
  const [controllers, setControllers] = useState<ControllerSnapshot[]>([]);
  const [selectedGame, setSelectedGame] = useState<RomEntry | null>(null);
  const [session, setSession] = useState<SessionInfo | null>(null);
  const [running, setRunning] = useState(false);
  const [frameNumber, setFrameNumber] = useState(0);
  const [audioRate, setAudioRate] = useState(0);
  const [scanning, setScanning] = useState(false);

  useEffect(() => {
    runningRef.current = running;
  }, [running]);

  useEffect(() => {
    let cancelled = false;
    void loadSettings()
      .then(async (loaded) => {
        if (cancelled) return;
        const merged = { ...defaultSettings, ...loaded };
        setSettings(merged);
        if (native && merged.autoScan && merged.romDirectory) {
          const found = await scanRomDirectory(merged.romDirectory);
          if (!cancelled) {
            setGames(found);
            setNotice(`AUTO-SCAN // ${found.length} IMAGES INDEXED`);
          }
        }
      })
      .catch((error: unknown) => {
        if (!cancelled) setNotice(`SETTINGS ERROR // ${String(error)}`);
      });
    return () => { cancelled = true; };
  }, [native]);

  useEffect(() => {
    const refresh = () => setControllers(listConnectedControllers());
    refresh();
    const timer = window.setInterval(refresh, 750);
    window.addEventListener("gamepadconnected", refresh);
    window.addEventListener("gamepaddisconnected", refresh);
    return () => {
      window.clearInterval(timer);
      window.removeEventListener("gamepadconnected", refresh);
      window.removeEventListener("gamepaddisconnected", refresh);
    };
  }, []);

  const drawFrame = (packet: FramePacket) => {
    if (!packet.rgbaBase64 || packet.width === 0 || packet.height === 0) return;
    const canvas = canvasRef.current;
    const context = canvas?.getContext("2d");
    if (!canvas || !context) return;
    const bytes = decodeBase64(packet.rgbaBase64);
    const expected = packet.width * packet.height * 4;
    if (bytes.length !== expected) return;
    canvas.width = packet.width;
    canvas.height = packet.height;
    context.putImageData(new ImageData(new Uint8ClampedArray(bytes), packet.width, packet.height), 0, 0);
  };

  const playAudio = (packet: FramePacket) => {
    if (!packet.audioBase64 || packet.sampleRateHz <= 0) return;
    const context = audioContextRef.current;
    if (!context) return;
    const bytes = decodeBase64(packet.audioBase64);
    if (bytes.length < 4) return;
    const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    const stereoFrames = Math.floor(bytes.length / 4);
    const buffer = context.createBuffer(2, stereoFrames, packet.sampleRateHz);
    const left = buffer.getChannelData(0);
    const right = buffer.getChannelData(1);
    for (let frame = 0; frame < stereoFrames; frame += 1) {
      left[frame] = view.getInt16(frame * 4, true) / 32768;
      right[frame] = view.getInt16(frame * 4 + 2, true) / 32768;
    }
    const source = context.createBufferSource();
    source.buffer = buffer;
    source.connect(context.destination);
    const startAt = Math.max(context.currentTime + 0.005, nextAudioTimeRef.current);
    source.start(startAt);
    nextAudioTimeRef.current = startAt + buffer.duration;
  };

  useEffect(() => {
    if (!running) return undefined;
    let cancelled = false;
    let inFlight = false;
    let animation = 0;

    const tick = async () => {
      animation = window.requestAnimationFrame(tick);
      if (cancelled || inFlight || !runningRef.current) return;
      inFlight = true;
      try {
        const controllerIndex = controllers[0]?.index ?? 0;
        const nextButtons = readGameBoyButtons(controllerIndex, settings.controllerDeadzone);
        for (const action of diffGameBoyButtons(gamepadRef.current, nextButtons)) {
          const event = bus.publish(frameRef.current, { kind: "human", seat: 1 }, action);
          setLastInput(`#${event.sequence.toString().padStart(4, "0")} GAMEPAD:P1 → ${action.kind === "button" ? `${action.button}:${action.pressed ? "DOWN" : "UP"}` : action.kind}`);
        }
        gamepadRef.current = nextButtons;

        const packet = await stepEmulation(bus.drainThrough(frameRef.current));
        frameRef.current = packet.frame;
        drawFrame(packet);
        playAudio(packet);
        setAudioRate((current) => current === packet.sampleRateHz ? current : packet.sampleRateHz);
        if (packet.frame % 6 === 0) setFrameNumber(packet.frame);
        if (packet.shutdownRequested) {
          setNotice("CORE REQUESTED SHUTDOWN");
          setRunning(false);
        }
      } catch (error) {
        setNotice(`CORE ERROR // ${String(error)}`);
        setRunning(false);
      } finally {
        inFlight = false;
      }
    };

    animation = window.requestAnimationFrame(tick);
    return () => {
      cancelled = true;
      window.cancelAnimationFrame(animation);
    };
  }, [running, controllers, settings.controllerDeadzone, bus]);

  useEffect(() => () => {
    if (native) void stopEmulation();
    void audioContextRef.current?.close();
  }, [native]);

  const visibleGames = games.filter((game) => activeSystem === "ALL" || game.system === activeSystem);

  const chooseDirectory = async () => {
    if (!native) {
      setNotice("DIRECTORY PICKER REQUIRES THE TAURI DESKTOP SHELL");
      return;
    }
    setScanning(true);
    try {
      const selected = await selectRomDirectory();
      if (!selected) {
        setNotice("DIRECTORY SELECTION CANCELLED");
        return;
      }
      const nextSettings = { ...settings, romDirectory: selected };
      const found = await scanRomDirectory(selected);
      await saveSettings(nextSettings);
      setSettings(nextSettings);
      setGames(found);
      setSelectedGame(null);
      setNotice(`SCAN COMPLETE // ${found.length} SUPPORTED IMAGES INDEXED`);
    } catch (error) {
      setNotice(`SCAN ERROR // ${String(error)}`);
    } finally {
      setScanning(false);
    }
  };

  const chooseCore = async () => {
    if (!native) {
      setNotice("CORE PICKER REQUIRES THE TAURI DESKTOP SHELL");
      return null;
    }
    const selected = await selectSameBoyCore();
    if (!selected) return null;
    const nextSettings = { ...settings, sameboyCorePath: selected };
    await saveSettings(nextSettings);
    setSettings(nextSettings);
    setNotice("SAMEBOY CORE PATH SAVED // IDENTITY STILL VERIFIED AT LOAD");
    return selected;
  };

  const toggleAutoScan = async () => {
    const nextSettings = { ...settings, autoScan: !settings.autoScan };
    setSettings(nextSettings);
    try {
      await saveSettings(nextSettings);
      setNotice(`AUTO-SCAN // ${nextSettings.autoScan ? "ENABLED" : "DISABLED"}`);
    } catch (error) {
      setNotice(`SETTINGS ERROR // ${String(error)}`);
    }
  };

  const queueAction = (action: GameAction) => {
    const event = bus.publish(frameRef.current, { kind: "human", seat: 1 }, action);
    const detail = action.kind === "button" ? `${action.button}:${action.pressed ? "DOWN" : "UP"}` : action.kind;
    setLastInput(`#${event.sequence.toString().padStart(4, "0")} HUMAN:P1 → ${detail}`);
    void validateActionEnvelope(event).catch((error: unknown) => setNotice(`ACTION IPC ERROR // ${String(error)}`));
  };

  const launchGame = async () => {
    if (!native) {
      setNotice("EMULATION REQUIRES THE TAURI DESKTOP SHELL");
      return;
    }
    if (!selectedGame) {
      setNotice("SELECT A GB/GBC IMAGE FROM THE LIBRARY FIRST");
      return;
    }
    if (!(["GB", "GBC"] as string[]).includes(selectedGame.system)) {
      setNotice("RUNG 3 QUALIFICATION IS GB/GBC ONLY // OTHER SYSTEMS STAY GATED");
      return;
    }

    try {
      let corePath = settings.sameboyCorePath;
      if (!corePath) corePath = await chooseCore();
      if (!corePath) {
        setNotice("SAMEBOY CORE SELECTION CANCELLED");
        return;
      }

      if (!audioContextRef.current) audioContextRef.current = new AudioContext();
      await audioContextRef.current.resume();
      nextAudioTimeRef.current = audioContextRef.current.currentTime;
      gamepadRef.current = emptyGameBoyButtons();
      frameRef.current = 0;
      setFrameNumber(0);
      setAudioRate(0);

      const info = await startEmulation(corePath, selectedGame.path);
      setSession(info);
      setRunning(true);
      setNotice(`CORE ONLINE // ${info.core.libraryName} ${info.core.libraryVersion} // ${selectedGame.displayName}`);
    } catch (error) {
      setNotice(`LAUNCH ERROR // ${String(error)}`);
      setRunning(false);
    }
  };

  const stopGame = async () => {
    setRunning(false);
    gamepadRef.current = emptyGameBoyButtons();
    try {
      await stopEmulation();
      setSession(null);
      setNotice("CORE SESSION STOPPED");
    } catch (error) {
      setNotice(`STOP ERROR // ${String(error)}`);
    }
  };

  return (
    <main className="shell">
      <header className="masthead">
        <div>
          <p className="eyebrow">Φ GAME RUNTIME / CARTRIDGE TERMINAL</p>
          <h1>PhiCade</h1>
          <p className="tagline">OLD WORLDS. NEW PLAYERS.</p>
        </div>
        <div className="status-cluster" aria-label="runtime status">
          <span><i className={`lamp ${native ? "lamp-green" : "lamp-amber"}`} /> {native ? "TAURI NATIVE" : "WEB PREVIEW"}</span>
          <span><i className="lamp lamp-green" /> ACTION IPC READY</span>
          <span><i className={`lamp ${running ? "lamp-green" : "lamp-amber"}`} /> {running ? "SAMEBOY RUNNING" : "CORE HOST STANDBY"}</span>
        </div>
      </header>

      <div className="notice-bar">{notice}</div>

      <nav className="system-tabs" aria-label="systems">
        {systems.map((system) => (
          <button key={system} className={activeSystem === system ? "active" : ""} onClick={() => setActiveSystem(system)}>
            {system}
          </button>
        ))}
      </nav>

      <section className="console-grid">
        <aside className="panel library-panel">
          <div className="panel-title">LIBRARY // {activeSystem}</div>

          {games.length === 0 ? (
            <div className="empty-state">
              <div className="cartridge-glyph">Φ</div>
              <strong>NO GAME IMAGES INDEXED</strong>
              <p>Select a local folder. PhiCade indexes supported filenames only; no ROM bytes are uploaded.</p>
              <button onClick={chooseDirectory} disabled={scanning}>{scanning ? "SCANNING..." : "SELECT ROM DIRECTORY"}</button>
            </div>
          ) : (
            <div className="game-list">
              {visibleGames.slice(0, 24).map((game) => (
                <button
                  className={`game-row ${selectedGame?.path === game.path ? "selected" : ""}`}
                  key={game.path}
                  title={game.path}
                  onClick={() => setSelectedGame(game)}
                >
                  <span className="system-chip">{game.system}</span>
                  <strong>{game.displayName}</strong>
                  <small>.{game.extension}</small>
                </button>
              ))}
              {visibleGames.length > 24 && <p className="microcopy">+ {visibleGames.length - 24} MORE</p>}
            </div>
          )}

          <div className="rule" />
          <button className="utility-button" onClick={chooseDirectory} disabled={scanning}>CHANGE DIRECTORY</button>
          <button className="utility-button" onClick={chooseCore} disabled={!native}>SAMEBOY CORE: {settings.sameboyCorePath ? "SET" : "SELECT"}</button>
          <button className="utility-button" onClick={toggleAutoScan} disabled={!native}>AUTO-SCAN: {settings.autoScan ? "ON" : "OFF"}</button>
          <p className="microcopy path-copy">{settings.romDirectory ?? "NO DIRECTORY SAVED"}</p>
        </aside>

        <section className="screen-panel" aria-label="emulator display">
          <div className="bezel">
            <div className="crt">
              <canvas ref={canvasRef} className={`game-canvas ${running ? "visible" : ""}`} aria-label="emulator video" />
              <div className="scanlines" />
              {!running && (
                <div className="boot-copy">
                  <div className="phi-mark">Φ</div>
                  <h2>PHICADE</h2>
                  <p>{selectedGame ? `${selectedGame.system} // ${selectedGame.displayName}` : "SELECT CARTRIDGE"}</p>
                  <small>RUNG 3 // SAMEBOY QUALIFIED CORE BAY</small>
                </div>
              )}
            </div>
          </div>

          <div className="session-strip">
            <button onClick={launchGame} disabled={running || !native}>LOAD / RUN</button>
            <button onClick={stopGame} disabled={!running}>EJECT</button>
            <span>{session ? `${session.core.libraryName} ${session.core.libraryVersion}` : "NO CORE LOADED"}</span>
          </div>

          <div className="control-strip">
            {["UP", "DOWN", "LEFT", "RIGHT", "A", "B", "SELECT", "START"].map((button) => (
              <button
                key={button}
                onPointerDown={() => queueAction({ kind: "button", button, pressed: true })}
                onPointerUp={() => queueAction({ kind: "button", button, pressed: false })}
                onPointerCancel={() => queueAction({ kind: "button", button, pressed: false })}
                disabled={!running}
              >
                {button}
              </button>
            ))}
          </div>
          <div className="receipt">{lastInput}</div>
        </section>

        <aside className="panel telemetry-panel">
          <div className="panel-title">RUNTIME // RUNG 3</div>
          <dl>
            <div><dt>FRAME</dt><dd>{frameNumber.toString().padStart(6, "0")}</dd></div>
            <div><dt>INPUT QUEUE</dt><dd>{bus.pending.toString().padStart(6, "0")}</dd></div>
            <div><dt>LIBRARY</dt><dd>{games.length.toString().padStart(6, "0")}</dd></div>
            <div><dt>GAMEPADS</dt><dd>{controllers.length.toString().padStart(6, "0")}</dd></div>
            <div><dt>CORE</dt><dd>{running ? "ONLINE" : "STANDBY"}</dd></div>
            <div><dt>AUDIO</dt><dd>{running ? (audioRate ? `${audioRate} HZ` : "SYNC") : "OFFLINE"}</dd></div>
          </dl>

          <div className="rule" />
          <div className="panel-title">CONTROLLER BAY</div>
          {controllers.length === 0 ? (
            <p className="controller-empty">NO GAMEPADS DETECTED</p>
          ) : (
            controllers.map((controller) => (
              <div className="controller-card" key={controller.index}>
                <span>P{controller.index + 1}</span>
                <strong>{controller.id || "GAMEPAD"}</strong>
                <small>{controller.buttonCount} BTN / {controller.axisCount} AXIS</small>
              </div>
            ))
          )}

          <div className="rule" />
          <div className="seat-card"><span>SEAT 1</span><strong>HUMAN → ACTION BUS</strong></div>
          <div className="seat-card muted"><span>SEAT 2</span><strong>UNASSIGNED</strong></div>
        </aside>
      </section>

      <section className="milestone-grid">
        {milestones.map(([title, state, detail]) => (
          <article className="milestone" key={title}>
            <div><span>{title}</span><strong>{state}</strong></div>
            <p>{detail}</p>
          </article>
        ))}
      </section>

      <footer>CAPABILITY ≠ AUTHORITY // USER-SUPPLIED GAME IMAGES // PINNED CORE PROVENANCE // ACTION BUS ONLY</footer>
    </main>
  );
}
