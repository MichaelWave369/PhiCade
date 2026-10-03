import { useEffect, useMemo, useRef, useState } from "react";
import { ActionBus, type ActionSource, type GameAction } from "./actionBus";
import {
  diffGameBoyButtons,
  emptyGameBoyButtons,
  listConnectedControllers,
  readGameBoyButtons,
  type ControllerSnapshot,
  type GameBoyButtonState,
} from "./controllers";
import {
  defaultAutodrivePolicy,
  defaultSettings,
  isNativeShell,
  captureScreenshot,
  cancelAgentTurn,
  completeOllamaTurn,
  failAutodriveProvider,
  flushGameSave,
  getAuthorityStatus,
  getAutodriveStatus,
  getDriverStatus,
  getLastAutodriveReceipt,
  issueAgentTurn,
  listOllamaModels,
  loadSettings,
  setControlMode,
  startAutodrive,
  stopAutodrive,
  submitAgentTurn,
  startReplayRecording,
  stopReplayRecording,
  verifyLastReplay,
  saveSettings,
  scanRomDirectory,
  selectRomDirectory,
  selectSameBoyCore,
  setGameProfile,
  startEmulation,
  stepEmulation,
  stopEmulation,
  validateActionEnvelope,
  type AgentTurnResponse,
  type AppSettings,
  type AutodriveArtifact,
  type AutodriveStatus,
  type AuthorityStatus,
  type ControlMode,
  type FramePacket,
  type GameProfile,
  type OllamaModel,
  type ReplayArtifact,
  type RomEntry,
  type SessionInfo,
} from "./native";

const systems = ["ALL", "NES", "SNES", "GB", "GBC", "GBA", "GENESIS", "PS1"] as const;
const PHIBOT_AGENT_ID = "phi-local";

const milestones = [
  ["AUTODRIVE", "BOUNDED", "Native policy caps turns, total actions, empty turns, and emulated frame span."],
  ["TURN LOOP", "QUEUE-AWARE", "A new model turn is issued only after the previous native action queue fully drains."],
  ["STOP PATHS", "RECEIPTED", "Operator stop, takeover, provider failure, grant expiry, and every budget exit persist a reasoned receipt."],
  ["TAKEOVER", "IMMEDIATE", "Human takeover clears pending turns, queued bot actions, and held input before authority changes."],
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
  const providerBusyRef = useRef(false);

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
  const [rewindSnapshots, setRewindSnapshots] = useState(0);
  const [profile, setProfile] = useState<GameProfile | null>(null);
  const [replayRecording, setReplayRecording] = useState(false);
  const [replayActions, setReplayActions] = useState(0);
  const [replayCheckpoints, setReplayCheckpoints] = useState(0);
  const [lastReplay, setLastReplay] = useState<ReplayArtifact | null>(null);
  const [authority, setAuthority] = useState<AuthorityStatus | null>(null);
  const [lastObservation, setLastObservation] = useState<string | null>(null);
  const [driverPendingTurnId, setDriverPendingTurnId] = useState<number | null>(null);
  const [driverQueuedActions, setDriverQueuedActions] = useState(0);
  const [ollamaModels, setOllamaModels] = useState<OllamaModel[]>([]);
  const [ollamaOnline, setOllamaOnline] = useState(false);
  const [ollamaScanning, setOllamaScanning] = useState(false);
  const [providerBusy, setProviderBusy] = useState(false);
  const [lastProviderDurationMs, setLastProviderDurationMs] = useState<number | null>(null);
  const [autodrive, setAutodrive] = useState<AutodriveStatus | null>(null);
  const [lastAutodrive, setLastAutodrive] = useState<AutodriveArtifact | null>(null);
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
    if (packet.fastForward > 1) return;
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
      if (cancelled || inFlight || !runningRef.current || providerBusyRef.current) return;
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
        setRewindSnapshots((current) => current === packet.rewindSnapshots ? current : packet.rewindSnapshots);
        setReplayRecording(packet.replayRecording);
        setReplayActions(packet.replayActions);
        setReplayCheckpoints(packet.replayCheckpoints);
        setAuthority((current) => current ? {
          ...current,
          mode: packet.controlMode,
          rejectedActions: packet.authorityRejections,
          lastReason: packet.lastAuthorityReason,
        } : current);
        setDriverPendingTurnId(packet.driverPendingTurnId);
        setDriverQueuedActions(packet.driverQueuedActions);
        setAutodrive(packet.autodrive);
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

  const queueSourceAction = (
    source: ActionSource,
    action: GameAction,
    frame = frameRef.current,
  ) => {
    const event = bus.publish(frame, source, action);
    const detail = action.kind === "button" ? `${action.button}:${action.pressed ? "DOWN" : "UP"}` : action.kind;
    const sourceLabel = source.kind === "phi-bot"
      ? `PHI-BOT:${source.agentId}:P${source.seat}`
      : source.kind === "human"
        ? `HUMAN:P${source.seat}`
        : source.kind.toUpperCase();
    setLastInput(`#${event.sequence.toString().padStart(4, "0")} ${sourceLabel} → ${detail}`);
    void validateActionEnvelope(event).catch((error: unknown) => setNotice(`ACTION IPC ERROR // ${String(error)}`));
    return event;
  };

  const queueAction = (action: GameAction) =>
    queueSourceAction({ kind: "human", seat: 1 }, action);

  const queueSystem = (
    command: "reset" | "save-state" | "load-state" | "rewind",
    slot?: number,
  ) => {
    queueAction({
      kind: "system",
      command,
      ...(slot === undefined ? {} : { slot }),
    });
    const label = command.toUpperCase().replace("-", " ");
    setNotice(`ACTION BUS // ${label}${slot === undefined ? "" : ` // ${slot}`}`);
  };

  const updateFastForward = async (fastForward: 1 | 2 | 4) => {
    if (!profile) return;
    try {
      const updated = await setGameProfile({ ...profile, fastForward });
      setProfile(updated);
      setSession((current) => current ? { ...current, profile: updated } : current);
      setNotice(`PROFILE SAVED // FAST-FORWARD ${fastForward}×`);
    } catch (error) {
      setNotice(`PROFILE ERROR // ${String(error)}`);
    }
  };

  const takeScreenshot = async () => {
    try {
      const path = await captureScreenshot();
      setNotice(`SCREENSHOT SAVED // ${path}`);
    } catch (error) {
      setNotice(`SCREENSHOT ERROR // ${String(error)}`);
    }
  };

  const flushBatteryRam = async () => {
    try {
      await flushGameSave();
      setNotice("BATTERY RAM FLUSHED");
    } catch (error) {
      setNotice(`SAVE RAM ERROR // ${String(error)}`);
    }
  };

  const refreshOllamaModels = async () => {
    if (!native) {
      setNotice("OLLAMA DISCOVERY REQUIRES THE TAURI DESKTOP SHELL");
      return;
    }
    setOllamaScanning(true);
    try {
      const models = await listOllamaModels(settings.ollamaBaseUrl);
      setOllamaModels(models);
      setOllamaOnline(true);
      setNotice(`OLLAMA ONLINE // ${models.length} LOCAL MODELS FOUND`);
    } catch (error) {
      setOllamaModels([]);
      setOllamaOnline(false);
      setNotice(`OLLAMA OFFLINE // ${String(error)}`);
    } finally {
      setOllamaScanning(false);
    }
  };

  const chooseOllamaModel = async (model: string) => {
    const nextSettings = { ...settings, ollamaModel: model || null };
    setSettings(nextSettings);
    try {
      await saveSettings(nextSettings);
      setNotice(model ? `OLLAMA MODEL // ${model}` : "OLLAMA MODEL CLEARED");
    } catch (error) {
      setNotice(`SETTINGS ERROR // ${String(error)}`);
    }
  };

  const runOllamaDriverTurn = async (autonomous = false) => {
    if (!settings.ollamaModel) {
      setNotice("SELECT A LOCAL OLLAMA VISION MODEL FIRST");
      return;
    }

    providerBusyRef.current = true;
    setProviderBusy(true);
    setNotice(
      autonomous
        ? `AUTODRIVE THINK // ${settings.ollamaModel} // FRAME ${frameRef.current}`
        : `THINK PAUSE // ${settings.ollamaModel} // FRAME ${frameRef.current}`,
    );

    try {
      const request = await issueAgentTurn(PHIBOT_AGENT_ID, 1);
      setDriverPendingTurnId(request.turnId);
      const result = await completeOllamaTurn(
        request,
        settings.ollamaBaseUrl,
        settings.ollamaModel,
      );
      const status = await submitAgentTurn(result.response);
      setDriverPendingTurnId(status.pendingTurnId);
      setDriverQueuedActions(status.queuedActions);
      const currentAutodrive = await getAutodriveStatus();
      setAutodrive(currentAutodrive);
      const durationMs = result.totalDurationNs === null
        ? null
        : result.totalDurationNs / 1_000_000;
      setLastProviderDurationMs(durationMs);
      setLastObservation(
        `OLLAMA T${request.turnId} // F${request.observation.frame} // ${result.response.actions.length} ACTIONS`,
      );
      setNotice(
        `${autonomous ? "AUTODRIVE" : "OLLAMA"} TURN ${request.turnId} ACCEPTED // ${result.model} // ${result.response.actions.length} ACTIONS`,
      );
    } catch (error) {
      try {
        const currentAutodrive = await getAutodriveStatus();
        if (autonomous && currentAutodrive?.active) {
          const artifact = await failAutodriveProvider();
          setLastAutodrive(artifact);
          setAutodrive(await getAutodriveStatus());
          setDriverPendingTurnId(null);
          setDriverQueuedActions(0);
        } else if (!autonomous) {
          const status = await cancelAgentTurn();
          setDriverPendingTurnId(status.pendingTurnId);
          setDriverQueuedActions(status.queuedActions);
        } else {
          setAutodrive(currentAutodrive);
          const artifact = await getLastAutodriveReceipt();
          if (artifact) setLastAutodrive(artifact);
        }
      } catch {
        setDriverPendingTurnId(null);
      }
      setNotice(`${autonomous ? "AUTODRIVE" : "OLLAMA"} TURN ERROR // ${String(error)}`);
    } finally {
      providerBusyRef.current = false;
      setProviderBusy(false);
    }
  };

  const beginAutodrive = async () => {
    if (!settings.ollamaModel) {
      setNotice("SELECT A LOCAL OLLAMA VISION MODEL FIRST");
      return;
    }
    try {
      const status = await startAutodrive("ollama", settings.ollamaModel, defaultAutodrivePolicy);
      setAutodrive(status);
      setLastAutodrive(null);
      setNotice(
        `AUTODRIVE RUN ${status.runId} // ${status.policy.maxTurns}T / ${status.policy.maxTotalActions}A / ${status.policy.maxEmulatedFrames}F`,
      );
    } catch (error) {
      setNotice(`AUTODRIVE START ERROR // ${String(error)}`);
    }
  };

  const endAutodrive = async () => {
    try {
      const artifact = await stopAutodrive();
      setLastAutodrive(artifact);
      setAutodrive(await getAutodriveStatus());
      setDriverPendingTurnId(null);
      setDriverQueuedActions(0);
      setNotice(
        `AUTODRIVE STOPPED // ${artifact.receipt.stopReason.toUpperCase()} // ${artifact.receipt.turnsCompleted} TURNS`,
      );
    } catch (error) {
      setNotice(`AUTODRIVE STOP ERROR // ${String(error)}`);
    }
  };

  useEffect(() => {
    if (
      !running
      || !autodrive?.active
      || providerBusy
      || driverPendingTurnId !== null
      || driverQueuedActions !== 0
      || authority?.mode !== "phi-bot"
      || !settings.ollamaModel
    ) {
      return undefined;
    }

    const timer = window.setTimeout(() => {
      void runOllamaDriverTurn(true);
    }, 25);
    return () => window.clearTimeout(timer);
  }, [
    running,
    autodrive?.active,
    autodrive?.turnsCompleted,
    providerBusy,
    driverPendingTurnId,
    driverQueuedActions,
    authority?.mode,
    settings.ollamaModel,
  ]);

  useEffect(() => {
    if (
      !autodrive
      || autodrive.active
      || !autodrive.stopReason
      || lastAutodrive?.receipt.runId === autodrive.runId
    ) {
      return;
    }

    void getLastAutodriveReceipt()
      .then((artifact) => {
        if (artifact) setLastAutodrive(artifact);
      })
      .catch(() => undefined);
  }, [autodrive, lastAutodrive?.receipt.runId]);

  const changeControlMode = async (mode: ControlMode) => {
    try {
      const next = await setControlMode(
        mode,
        mode === "human" ? null : PHIBOT_AGENT_ID,
        mode === "human" ? null : ["A", "B", "SELECT", "START", "UP", "DOWN", "LEFT", "RIGHT"],
        mode === "human" ? null : 3_600,
      );
      setAuthority(next);
      gamepadRef.current = emptyGameBoyButtons();
      setLastObservation(null);
      setDriverPendingTurnId(null);
      setDriverQueuedActions(0);
      const currentAutodrive = await getAutodriveStatus();
      setAutodrive(currentAutodrive);
      if (currentAutodrive && !currentAutodrive.active) {
        const artifact = await getLastAutodriveReceipt();
        if (artifact) setLastAutodrive(artifact);
      }
      setNotice(
        mode === "human"
          ? "HUMAN TAKEOVER // PHI-BOT GRANT REVOKED"
          : `AUTHORITY // ${mode.toUpperCase()} // ${next.agentId} P${next.agentSeat} // EXPIRES F${next.expiresAtFrame}`,
      );
    } catch (error) {
      setNotice(`AUTHORITY REFUSAL // ${String(error)}`);
    }
  };

  const runReferenceDriverTurn = async () => {
    try {
      const request = await issueAgentTurn(PHIBOT_AGENT_ID, 1);
      setDriverPendingTurnId(request.turnId);

      const observation = request.observation;
      const candidates = ["A", "B", "RIGHT", "LEFT", "UP", "DOWN"]
        .filter((button) => observation.allowedButtons.includes(button));
      if (candidates.length === 0) throw new Error("active grant has no demo-compatible buttons");

      const selector = Number.parseInt(observation.frameSha256.slice(-2), 16);
      const button = candidates[selector % candidates.length];
      const response: AgentTurnResponse = {
        schema: "phicade.agent-turn-response.v1",
        turnId: request.turnId,
        agentId: PHIBOT_AGENT_ID,
        seat: 1,
        observationFrame: observation.frame,
        observationSha256: observation.frameSha256,
        actions: [
          { delayFrames: 0, action: { kind: "button", button, pressed: true } },
          { delayFrames: 2, action: { kind: "button", button, pressed: false } },
        ],
      };

      const status = await submitAgentTurn(response);
      setDriverPendingTurnId(status.pendingTurnId);
      setDriverQueuedActions(status.queuedActions);
      setLastObservation(`T${request.turnId} // F${observation.frame} // ${observation.frameSha256.slice(0, 12)}… // ${button}`);
      setNotice(`DRIVER TURN ${request.turnId} ACCEPTED // ${button} TAP ENTERED NATIVE INBOX`);
    } catch (error) {
      setNotice(`AGENT DRIVER ERROR // ${String(error)}`);
    }
  };

  const beginReplayRecording = async () => {
    try {
      const status = await startReplayRecording();
      setReplayRecording(status.recording);
      setReplayActions(status.actionCount);
      setReplayCheckpoints(status.checkpointCount);
      setLastReplay(null);
      setNotice("REPLAY LEDGER // RECORDING FROM EXACT CORE SNAPSHOT");
    } catch (error) {
      setNotice(`REPLAY START ERROR // ${String(error)}`);
    }
  };

  const endReplayRecording = async () => {
    try {
      const artifact = await stopReplayRecording();
      setReplayRecording(false);
      setLastReplay(artifact);
      setNotice(`REPLAY EXPORTED // ${artifact.replaySha256.slice(0, 16)}… // VERIFY READY`);
    } catch (error) {
      setNotice(`REPLAY STOP ERROR // ${String(error)}`);
    }
  };

  const verifyReplay = async () => {
    try {
      const artifact = await verifyLastReplay();
      setLastReplay(artifact);
      const verification = artifact.receipt.verification;
      if (verification?.result === "pass") {
        setNotice(`REPLAY EXACT // ${verification.checkedCheckpoints} CHECKPOINTS VERIFIED`);
      } else {
        setNotice(
          `REPLAY DIVERGED // FIRST OBSERVED FRAME ${verification?.firstDivergenceFrame ?? "UNKNOWN"}`,
        );
      }
    } catch (error) {
      setNotice(`REPLAY VERIFY ERROR // ${String(error)}`);
    }
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
      setRewindSnapshots(0);
      setReplayRecording(false);
      setReplayActions(0);
      setReplayCheckpoints(0);
      setLastReplay(null);
      setAuthority(null);
      setLastObservation(null);
      setDriverPendingTurnId(null);
      setDriverQueuedActions(0);
      setLastProviderDurationMs(null);
      setAutodrive(null);
      setLastAutodrive(null);

      const info = await startEmulation(corePath, selectedGame.path);
      const [initialAuthority, initialDriver] = await Promise.all([
        getAuthorityStatus(),
        getDriverStatus(),
      ]);
      setSession(info);
      setAuthority(initialAuthority);
      setDriverPendingTurnId(initialDriver.pendingTurnId);
      setDriverQueuedActions(initialDriver.queuedActions);
      setProfile(info.profile);
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
      setProfile(null);
      setRewindSnapshots(0);
      setReplayRecording(false);
      setReplayActions(0);
      setReplayCheckpoints(0);
      setLastReplay(null);
      setAuthority(null);
      setLastObservation(null);
      setDriverPendingTurnId(null);
      setDriverQueuedActions(0);
      setLastProviderDurationMs(null);
      setAutodrive(null);
      setLastAutodrive(null);
      providerBusyRef.current = false;
      setProviderBusy(false);
      setNotice("CORE SESSION STOPPED // BATTERY RAM FLUSHED");
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
          <span><i className={`lamp ${replayRecording ? "lamp-amber" : "lamp-green"}`} /> {replayRecording ? "REPLAY RECORDING" : "LEDGER READY"}</span>
          <span><i className={`lamp ${authority?.mode === "phi-bot" || authority?.mode === "coop" ? "lamp-amber" : "lamp-green"}`} /> AUTHORITY {authority?.mode?.toUpperCase() ?? "OFFLINE"}</span>
          <span><i className={`lamp ${driverPendingTurnId !== null || driverQueuedActions > 0 ? "lamp-amber" : "lamp-green"}`} /> DRIVER {driverPendingTurnId !== null ? `TURN ${driverPendingTurnId}` : driverQueuedActions > 0 ? `${driverQueuedActions} QUEUED` : "READY"}</span>
          <span><i className={`lamp ${ollamaOnline ? "lamp-green" : "lamp-amber"}`} /> OLLAMA {providerBusy ? "THINKING" : ollamaOnline ? "LOCAL" : "UNPROBED"}</span>
          <span><i className={`lamp ${autodrive?.active ? "lamp-amber" : "lamp-green"}`} /> AUTODRIVE {autodrive?.active ? `RUN ${autodrive.runId}` : autodrive?.stopReason?.toUpperCase() ?? "STANDBY"}</span>
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
                  <small>RUNG 9 // GOVERNED AUTODRIVE ONLINE</small>
                </div>
              )}
            </div>
          </div>

          <div className="session-strip">
            <button onClick={launchGame} disabled={running || !native}>LOAD / RUN</button>
            <button onClick={stopGame} disabled={!running || replayRecording || autodrive?.active}>EJECT</button>
            <span>{session ? `${session.core.libraryName} ${session.core.libraryVersion}` : "NO CORE LOADED"}</span>
          </div>

          <div className="agent-strip">
            <span>Φ-BOT SEAT</span>
            <button className={authority?.mode === "human" ? "active" : ""} onClick={() => changeControlMode("human")} disabled={!running || replayRecording || autodrive?.active}>HUMAN</button>
            <button className={authority?.mode === "phi-bot" ? "active" : ""} onClick={() => changeControlMode("phi-bot")} disabled={!running || replayRecording || autodrive?.active}>HANDOFF</button>
            <button className={authority?.mode === "coop" ? "active" : ""} onClick={() => changeControlMode("coop")} disabled={!running || replayRecording || autodrive?.active}>CO-OP</button>
            <button onClick={() => changeControlMode("versus")} disabled={!running || replayRecording || autodrive?.active}>VERSUS</button>
            <button onClick={runReferenceDriverTurn} disabled={!running || replayRecording || autodrive?.active || !authority || !["phi-bot", "coop"].includes(authority.mode) || driverPendingTurnId !== null}>DRIVER TURN</button>
            <small>{lastObservation ?? (authority?.agentId ? `${authority.agentId} // P${authority.agentSeat} // GRANT TO F${authority.expiresAtFrame}` : "NO AGENT GRANT")}</small>
          </div>

          <div className="provider-strip">
            <span>OLLAMA // LOCAL</span>
            <button onClick={refreshOllamaModels} disabled={!native || ollamaScanning || providerBusy || autodrive?.active}>
              {ollamaScanning ? "SCANNING..." : "SCAN MODELS"}
            </button>
            <select
              value={settings.ollamaModel ?? ""}
              onChange={(event) => void chooseOllamaModel(event.target.value)}
              disabled={ollamaModels.length === 0 || providerBusy || autodrive?.active}
              aria-label="Ollama model"
            >
              <option value="">SELECT VISION MODEL</option>
              {ollamaModels.map((model) => (
                <option key={model.digest || model.name} value={model.name}>{model.name}</option>
              ))}
            </select>
            <button
              onClick={() => void runOllamaDriverTurn(false)}
              disabled={!running || replayRecording || autodrive?.active || providerBusy || authority?.mode !== "phi-bot" || !settings.ollamaModel || driverPendingTurnId !== null}
            >
              {providerBusy && !autodrive?.active ? "THINKING..." : "OLLAMA TURN"}
            </button>
            <button
              className={autodrive?.active ? "autodrive-active" : ""}
              onClick={() => void (autodrive?.active ? endAutodrive() : beginAutodrive())}
              disabled={
                !running
                || replayRecording
                || !settings.ollamaModel
                || authority?.mode !== "phi-bot"
                || (!autodrive?.active && (!ollamaOnline || providerBusy || driverPendingTurnId !== null || driverQueuedActions !== 0))
              }
            >
              {autodrive?.active ? "STOP AUTO" : "AUTO DRIVE"}
            </button>
            <small>
              {autodrive?.active
                ? `RUN ${autodrive.runId} // ${autodrive.turnsCompleted}/${autodrive.policy.maxTurns}T // ${autodrive.totalActions}/${autodrive.policy.maxTotalActions}A`
                : settings.ollamaModel
                  ? `${settings.ollamaModel} // ${settings.ollamaBaseUrl}${lastProviderDurationMs === null ? "" : ` // ${Math.round(lastProviderDurationMs)}MS`}`
                  : `NO MODEL // ${settings.ollamaBaseUrl}`}
            </small>
          </div>

          <div className="replay-strip">
            <span>REPLAY LEDGER</span>
            <button
              className={replayRecording ? "recording" : ""}
              onClick={replayRecording ? endReplayRecording : beginReplayRecording}
              disabled={!running || autodrive?.active || (!replayRecording && profile?.fastForward !== 1)}
            >
              {replayRecording ? "STOP + EXPORT" : "REC"}
            </button>
            <button onClick={verifyReplay} disabled={!running || replayRecording || autodrive?.active || !lastReplay}>VERIFY LAST</button>
            <small>
              {lastReplay
                ? `${lastReplay.receipt.verification?.result?.toUpperCase() ?? "UNVERIFIED"} // ${lastReplay.replaySha256.slice(0, 12)}…`
                : replayRecording
                  ? `${replayActions} ACTIONS / ${replayCheckpoints} CHECKPOINTS`
                  : "NO EXPORTED REPLAY"}
            </small>
          </div>

          <div className="session-tools">
            <button onClick={() => queueSystem("save-state", profile?.saveSlot ?? 0)} disabled={!running || replayRecording || autodrive?.active}>SAVE S{profile?.saveSlot ?? 0}</button>
            <button onClick={() => queueSystem("load-state", profile?.saveSlot ?? 0)} disabled={!running || replayRecording || autodrive?.active}>LOAD S{profile?.saveSlot ?? 0}</button>
            <button onClick={() => queueSystem("rewind", 2)} disabled={!running || replayRecording || autodrive?.active}>REWIND 2S</button>
            <button onClick={() => queueSystem("reset")} disabled={!running || replayRecording || autodrive?.active}>RESET</button>
            <button onClick={takeScreenshot} disabled={!running}>SCREENSHOT</button>
            <button onClick={flushBatteryRam} disabled={!running}>FLUSH SRAM</button>
          </div>

          <div className="speed-strip">
            <span>FAST-FORWARD</span>
            {([1, 2, 4] as const).map((speed) => (
              <button
                key={speed}
                className={profile?.fastForward === speed ? "active" : ""}
                onClick={() => updateFastForward(speed)}
                disabled={!running || replayRecording || autodrive?.active}
              >
                {speed}×
              </button>
            ))}
            <small>{profile ? `REWIND ${profile.rewindSeconds}S / EVERY ${profile.rewindIntervalFrames}F` : "PROFILE OFFLINE"}</small>
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
          <div className="panel-title">RUNTIME // RUNG 9</div>
          <dl>
            <div><dt>FRAME</dt><dd>{frameNumber.toString().padStart(6, "0")}</dd></div>
            <div><dt>INPUT QUEUE</dt><dd>{bus.pending.toString().padStart(6, "0")}</dd></div>
            <div><dt>LIBRARY</dt><dd>{games.length.toString().padStart(6, "0")}</dd></div>
            <div><dt>GAMEPADS</dt><dd>{controllers.length.toString().padStart(6, "0")}</dd></div>
            <div><dt>CORE</dt><dd>{running ? "ONLINE" : "STANDBY"}</dd></div>
            <div><dt>AUDIO</dt><dd>{running ? (audioRate ? `${audioRate} HZ` : "SYNC") : "OFFLINE"}</dd></div>
            <div><dt>SPEED</dt><dd>{running ? `${profile?.fastForward ?? 1}×` : "OFFLINE"}</dd></div>
            <div><dt>REWIND</dt><dd>{rewindSnapshots.toString().padStart(6, "0")}</dd></div>
            <div><dt>STATE SLOT</dt><dd>S{profile?.saveSlot ?? 0}</dd></div>
            <div><dt>GAME HASH</dt><dd>{session ? session.gameKey.slice(0, 8).toUpperCase() : "--------"}</dd></div>
            <div><dt>REPLAY</dt><dd>{replayRecording ? "RECORDING" : lastReplay ? "EXPORTED" : "STANDBY"}</dd></div>
            <div><dt>ACTIONS</dt><dd>{replayActions.toString().padStart(6, "0")}</dd></div>
            <div><dt>CHECKPOINTS</dt><dd>{replayCheckpoints.toString().padStart(6, "0")}</dd></div>
            <div><dt>VERIFY</dt><dd>{lastReplay?.receipt.verification?.result.toUpperCase() ?? "UNVERIFIED"}</dd></div>
            <div><dt>AUTHORITY</dt><dd>{authority?.mode.toUpperCase() ?? "OFFLINE"}</dd></div>
            <div><dt>AGENT</dt><dd>{authority?.agentId ?? "NONE"}</dd></div>
            <div><dt>REJECTED</dt><dd>{(authority?.rejectedActions ?? 0).toString().padStart(6, "0")}</dd></div>
            <div><dt>DRIVER TURN</dt><dd>{driverPendingTurnId === null ? "NONE" : driverPendingTurnId}</dd></div>
            <div><dt>DRIVER QUEUE</dt><dd>{driverQueuedActions.toString().padStart(6, "0")}</dd></div>
            <div><dt>OLLAMA</dt><dd>{providerBusy ? "THINK PAUSE" : ollamaOnline ? "ONLINE" : "UNPROBED"}</dd></div>
            <div><dt>MODEL</dt><dd>{settings.ollamaModel ?? "NONE"}</dd></div>
            <div><dt>AUTO RUN</dt><dd>{autodrive?.active ? `#${autodrive.runId} ACTIVE` : autodrive?.stopReason?.toUpperCase() ?? "STANDBY"}</dd></div>
            <div><dt>AUTO TURNS</dt><dd>{autodrive ? `${autodrive.turnsCompleted}/${autodrive.policy.maxTurns}` : "0/0"}</dd></div>
            <div><dt>AUTO ACTIONS</dt><dd>{autodrive ? `${autodrive.totalActions}/${autodrive.policy.maxTotalActions}` : "0/0"}</dd></div>
            <div><dt>AUTO RECEIPT</dt><dd>{lastAutodrive ? `RUN ${lastAutodrive.receipt.runId}` : "NONE"}</dd></div>
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
            <strong>
              {authority?.mode === "phi-bot"
                ? "Φ-BOT → ACTION BUS"
                : authority?.mode === "coop"
                  ? "HUMAN + Φ-BOT → ACTION BUS"
                  : "HUMAN → ACTION BUS"}
            </strong>
          </div>
          <div className="seat-card muted"><span>SEAT 2</span><strong>SAMEBOY: NO SECOND PLAYABLE PORT</strong></div>
          {authority?.lastReason && <p className="authority-reason">LAST AUTHORITY // {authority.lastReason}</p>}
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

      <footer>CAPABILITY ≠ AUTHORITY // AUTONOMY ≠ UNBOUNDED AUTHORITY // HUMAN TAKEOVER // EVERY RUN ENDS WITH A RECEIPT</footer>
    </main>
  );
}
