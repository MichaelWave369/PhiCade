import { useMemo, useState } from "react";
import { ActionBus } from "./actionBus";

const systems = ["ALL", "NES", "SNES", "GB", "GBA", "GENESIS", "PS1"] as const;

const milestones = [
  ["ACTION BUS", "READY", "Human / replay / script / Phi-Bot share one input contract."],
  ["ROM LIBRARY", "SCAFFOLD", "Local-only scan boundary. No bundled commercial content."],
  ["CORE HOST", "NEXT", "libretro-compatible adapter comes after ABI + license gates."],
  ["REPLAY LEDGER", "PLANNED", "Frame-stamped inputs become reproducible session receipts."],
] as const;

export function App() {
  const bus = useMemo(() => new ActionBus(), []);
  const [activeSystem, setActiveSystem] = useState<(typeof systems)[number]>("ALL");
  const [lastInput, setLastInput] = useState("NO INPUT");

  const pulseInput = (button: string) => {
    const event = bus.publish(0, { kind: "human", seat: 1 }, {
      kind: "button",
      button,
      pressed: true,
    });
    setLastInput(`#${event.sequence.toString().padStart(4, "0")} HUMAN:P1 → ${button}`);
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
          <span><i className="lamp lamp-green" /> SHELL READY</span>
          <span><i className="lamp lamp-amber" /> CORE HOST OFFLINE</span>
        </div>
      </header>

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
          <div className="empty-state">
            <div className="cartridge-glyph">Φ</div>
            <strong>NO ROM DIRECTORY MOUNTED</strong>
            <p>Rung 1 deliberately ships without game images or firmware.</p>
            <button disabled>SCAN DIRECTORY</button>
          </div>
          <div className="rule" />
          <p className="microcopy">
            HOMEbrew / PUBLIC DOMAIN / LEGALLY-OWNED IMAGES ONLY
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
                <small>INSERT A QUALIFIED CORE + GAME IMAGE</small>
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
          <div className="panel-title">RUNTIME // RUNG 1</div>
          <dl>
            <div><dt>FRAME</dt><dd>000000</dd></div>
            <div><dt>INPUT QUEUE</dt><dd>{bus.pending.toString().padStart(6, "0")}</dd></div>
            <div><dt>VIDEO</dt><dd>NO CORE</dd></div>
            <div><dt>AUDIO</dt><dd>NO CORE</dd></div>
            <div><dt>STATE</dt><dd>IDLE</dd></div>
          </dl>
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
        CAPABILITY ≠ AUTHORITY // CORE EXECUTION WILL REQUIRE EXPLICIT QUALIFICATION
      </footer>
    </main>
  );
}
