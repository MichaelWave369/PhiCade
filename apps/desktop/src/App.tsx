import { useEffect, useMemo, useState } from "react";
import { ActionBus } from "./actionBus";
import { listConnectedControllers, type ControllerSnapshot } from "./controllers";
import {
  defaultSettings,
  isNativeShell,
  loadSettings,
  saveSettings,
  scanRomDirectory,
  selectRomDirectory,
  validateActionEnvelope,
  type AppSettings,
  type RomEntry,
} from "./native";

const systems = ["ALL", "NES", "SNES", "GB", "GBC", "GBA", "GENESIS", "PS1"] as const;

const milestones = [
  ["NATIVE SHELL", "READY", "Tauri owns native filesystem and settings operations."],
  ["ROM LIBRARY", "READY", "User-selected folders are scanned locally without uploading ROM bytes."],
  ["CONTROLLERS", "READY", "Connected gamepads are enumerated through the webview Gamepad API."],
  ["ACTION IPC", "READY", "Rust validates the same canonical ActionEnvelope emitted by TypeScript."],
] as const;

export function App() {
  const bus = useMemo(() => new ActionBus(), []);
  const native = isNativeShell();
  const [activeSystem, setActiveSystem] = useState<(typeof systems)[number]>("ALL");
  const [lastInput, setLastInput] = useState("NO INPUT");
  const [notice, setNotice] = useState(native ? "NATIVE SHELL ONLINE" : "WEB PREVIEW // NATIVE FEATURES OFFLINE");
  const [settings, setSettings] = useState<AppSettings>(defaultSettings);
  const [games, setGames] = useState<RomEntry[]>([]);
  const [controllers, setControllers] = useState<ControllerSnapshot[]>([]);
  const [scanning, setScanning] = useState(false);

  useEffect(() => {
    let cancelled = false;

    void loadSettings()
      .then(async (loaded) => {
        if (cancelled) return;
        setSettings(loaded);

        if (native && loaded.autoScan && loaded.romDirectory) {
          const found = await scanRomDirectory(loaded.romDirectory);
          if (!cancelled) {
            setGames(found);
            setNotice(`AUTO-SCAN // ${found.length} IMAGES INDEXED`);
          }
        }
      })
      .catch((error: unknown) => {
        if (!cancelled) setNotice(`SETTINGS ERROR // ${String(error)}`);
      });

    return () => {
      cancelled = true;
    };
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

  const visibleGames = games.filter(
    (game) => activeSystem === "ALL" || game.system === activeSystem,
  );

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
      setNotice(`SCAN COMPLETE // ${found.length} SUPPORTED IMAGES INDEXED`);
    } catch (error) {
      setNotice(`SCAN ERROR // ${String(error)}`);
    } finally {
      setScanning(false);
    }
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

  const pulseInput = (button: string) => {
    const event = bus.publish(0, { kind: "human", seat: 1 }, {
      kind: "button",
      button,
      pressed: true,
    });

    setLastInput(`#${event.sequence.toString().padStart(4, "0")} HUMAN:P1 → ${button}`);

    void validateActionEnvelope(event)
      .then((validated) => {
        if (native) {
          setLastInput(
            `#${validated.sequence.toString().padStart(4, "0")} IPC VERIFIED → ${button}`,
          );
        }
      })
      .catch((error: unknown) => setNotice(`ACTION IPC ERROR // ${String(error)}`));
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
          <span><i className="lamp lamp-amber" /> CORE HOST OFFLINE</span>
        </div>
      </header>

      <div className="notice-bar">{notice}</div>

      <nav className="system-tabs" aria-label="systems">
        {systems.map((system) => (
          <button
            key={system}
            className={activeSystem === system ? "active" : ""}
            onClick={() => setActiveSystem(system)}
          >
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
              <button onClick={chooseDirectory} disabled={scanning}>
                {scanning ? "SCANNING..." : "SELECT ROM DIRECTORY"}
              </button>
            </div>
          ) : (
            <div className="game-list">
              {visibleGames.slice(0, 24).map((game) => (
                <div className="game-row" key={game.path} title={game.path}>
                  <span className="system-chip">{game.system}</span>
                  <strong>{game.displayName}</strong>
                  <small>.{game.extension}</small>
                </div>
              ))}
              {visibleGames.length > 24 && (
                <p className="microcopy">+ {visibleGames.length - 24} MORE</p>
              )}
            </div>
          )}

          <div className="rule" />
          <button className="utility-button" onClick={chooseDirectory} disabled={scanning}>
            CHANGE DIRECTORY
          </button>
          <button className="utility-button" onClick={toggleAutoScan} disabled={!native}>
            AUTO-SCAN: {settings.autoScan ? "ON" : "OFF"}
          </button>
          <p className="microcopy path-copy">
            {settings.romDirectory ?? "NO DIRECTORY SAVED"}
          </p>
        </aside>

        <section className="screen-panel" aria-label="emulator display">
          <div className="bezel">
            <div className="crt">
              <div className="scanlines" />
              <div className="boot-copy">
                <div className="phi-mark">Φ</div>
                <h2>PHICADE</h2>
                <p>CORE BAY EMPTY</p>
                <small>RUNG 2 NATIVE SHELL // READY FOR FIRST QUALIFIED CORE</small>
              </div>
            </div>
          </div>

          <div className="control-strip">
            {["UP", "LEFT", "A", "B", "START"].map((button) => (
              <button key={button} onClick={() => pulseInput(button)}>
                {button}
              </button>
            ))}
          </div>
          <div className="receipt">{lastInput}</div>
        </section>

        <aside className="panel telemetry-panel">
          <div className="panel-title">RUNTIME // RUNG 2</div>
          <dl>
            <div><dt>FRAME</dt><dd>000000</dd></div>
            <div><dt>INPUT QUEUE</dt><dd>{bus.pending.toString().padStart(6, "0")}</dd></div>
            <div><dt>LIBRARY</dt><dd>{games.length.toString().padStart(6, "0")}</dd></div>
            <div><dt>GAMEPADS</dt><dd>{controllers.length.toString().padStart(6, "0")}</dd></div>
            <div><dt>STATE</dt><dd>{native ? "NATIVE" : "PREVIEW"}</dd></div>
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
          <div className="seat-card">
            <span>SEAT 1</span>
            <strong>HUMAN</strong>
          </div>
          <div className="seat-card muted">
            <span>SEAT 2</span>
            <strong>UNASSIGNED</strong>
          </div>
        </aside>
      </section>

      <section className="milestone-grid">
        {milestones.map(([title, state, detail]) => (
          <article className="milestone" key={title}>
            <div>
              <span>{title}</span>
              <strong>{state}</strong>
            </div>
            <p>{detail}</p>
          </article>
        ))}
      </section>

      <footer>
        CAPABILITY ≠ AUTHORITY // USER-SELECTED LOCAL FILES // CORE EXECUTION STILL GATED
      </footer>
    </main>
  );
}
