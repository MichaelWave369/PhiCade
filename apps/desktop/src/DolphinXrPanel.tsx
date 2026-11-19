import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
type Install = {
  executable: string;
  binarySha256: string;
  wiiProfileSha256: string | null;
  gamecubeProfileSha256: string | null;
  identityEvidence: string;
};
export function DolphinXrPanel() {
  const [install, setInstall] = useState<Install | null>(null),
    [game, setGame] = useState<string | null>(null),
    [profile, setProfile] = useState("wii-remote-nunchuk"),
    [flat, setFlat] = useState(false),
    [passthrough, setPassthrough] = useState(false),
    [left, setLeft] = useState(false),
    [pitch, setPitch] = useState(20),
    [scale, setScale] = useState(1),
    [confirmed, setConfirmed] = useState(false),
    [notice, setNotice] = useState(
      "Select your own portable OpenXR-enabled DolphinXR installation.",
    ),
    [busy, setBusy] = useState(false);
  const selectInstall = async () => {
    try {
      const path = await open({
        multiple: false,
        directory: false,
        title: "Select installed DolphinXR executable",
      });
      if (typeof path !== "string") return;
      setConfirmed(false);
      setInstall(
        await invoke<Install>("inspect_dolphinxr", { executable: path }),
      );
      setNotice(
        "Installation inspected. Review controller profile and explicitly confirm the OpenXR build.",
      );
    } catch (e) {
      setInstall(null);
      setNotice(String(e));
    }
  };
  const selectGame = async () => {
    const path = await open({
      multiple: false,
      directory: false,
      title: "Select your local Wii/GameCube game or original homebrew",
      filters: [
        {
          name: "Wii / GameCube / homebrew",
          extensions: [
            "iso",
            "gcm",
            "wbfs",
            "rvz",
            "wia",
            "ciso",
            "dol",
            "elf",
          ],
        },
      ],
    });
    if (typeof path === "string") setGame(path);
  };
  const launch = async () => {
    if (!install || !game) return;
    setBusy(true);
    try {
      const result = await invoke<{
        pid: number;
        vrSessionVerified: boolean;
        plan: { userDirectory: string };
      }>("launch_dolphinxr", {
        executable: install.executable,
        expectedHash: install.binarySha256,
        game,
        options: {
          profile,
          flatScreen: flat,
          passthrough,
          leftHanded: left,
          motionPitchDegrees: pitch,
          unitsPerMeter: scale,
          operatorConfirmedOpenxrBuild: confirmed,
        },
        operatorApproved: true,
      });
      setNotice(
        `DolphinXR process ${result.pid} started. Verify the live OpenXR session in DolphinXR. Per-game controls: ${result.plan.userDirectory}`,
      );
    } catch (e) {
      setNotice(String(e));
    } finally {
      setBusy(false);
    }
  };
  return (
    <details className="xr-host-panel">
      <summary>DOLPHINXR · WII / GAMECUBE · EXTERNAL RUNTIME</summary>
      <p>
        Uses your installed DolphinXR and its native OpenXR Quest bindings.
        Close the browser VR session before giving the headset to DolphinXR.
        Compatibility and controller behavior depend on the selected build and
        game.
      </p>
      <button onClick={() => void selectInstall()} disabled={busy}>
        SELECT + INSPECT DOLPHINXR
      </button>
      <button onClick={() => void selectGame()} disabled={busy}>
        SELECT LOCAL GAME / HOMEBREW
      </button>
      {install && (
        <small>
          Executable SHA-256: {install.binarySha256}
          <br />
          {install.identityEvidence}
        </small>
      )}
      {game && <small>Content: {game.split(/[\\/]/).pop()}</small>}
      <div className="xr-host-options">
        <label>
          Quest profile{" "}
          <select value={profile} onChange={(e) => setProfile(e.target.value)}>
            <option
              value="wii-remote-nunchuk"
              disabled={!!install && !install.wiiProfileSha256}
            >
              Wii Remote + Nunchuk
            </option>
            <option
              value="gamecube-touch"
              disabled={!!install && !install.gamecubeProfileSha256}
            >
              GameCube Touch
            </option>
          </select>
        </label>
        <label>
          <input
            type="checkbox"
            checked={flat}
            onChange={(e) => setFlat(e.target.checked)}
          />{" "}
          Spatial screen
        </label>
        <label>
          <input
            type="checkbox"
            checked={passthrough}
            onChange={(e) => setPassthrough(e.target.checked)}
          />{" "}
          Native passthrough if supported
        </label>
        <label>
          <input
            type="checkbox"
            checked={left}
            onChange={(e) => setLeft(e.target.checked)}
          />{" "}
          Left-handed bindings
        </label>
        <label>
          Motion pitch{" "}
          <input
            type="number"
            min={-90}
            max={90}
            value={pitch}
            onChange={(e) => setPitch(Number(e.target.value))}
          />
        </label>
        <label>
          Units / meter{" "}
          <input
            type="number"
            min={0.01}
            max={100}
            step={0.1}
            value={scale}
            onChange={(e) => setScale(Number(e.target.value))}
          />
        </label>
      </div>
      <label>
        <input
          type="checkbox"
          checked={confirmed}
          onChange={(e) => setConfirmed(e.target.checked)}
        />{" "}
        I installed an OpenXR-enabled DolphinXR build and want to launch this
        local session.
      </label>
      <button
        disabled={!install || !game || !confirmed || busy}
        onClick={() => void launch()}
      >
        APPROVE + LAUNCH DOLPHINXR
      </button>
      <output aria-live="polite">{notice}</output>
      <small>
        DolphinXR keeps its own identity and license. No emulator or game files
        are bundled. Stock profile names are checked and hashed; this does not
        prove a selected executable is VR-capable. Runtime/hardware verification
        remains pending. Use DolphinXR's own menu to adjust per-game pointer,
        swing and shake behavior or close/save the session.
      </small>
    </details>
  );
}
