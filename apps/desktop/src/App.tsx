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
  cancelBenchmarkCampaign,
  completeOllamaTurn,
  compareBenchmarkCampaigns,
  compareBenchmarkSuiteReports,
  continueBenchmarkCampaign,
  buildBenchmarkSuiteReport,
  failAutodriveProvider,
  flushGameSave,
  getAuthorityStatus,
  getAutodriveStatus,
  getDriverStatus,
  getBenchmarkCampaignStatus,
  getLastAutodriveReceipt,
  getLastBenchmarkCampaignReceipt,
  getLastModelGameplayBenchmark,
  getOllamaQualificationStatus,
  issueAgentTurn,
  listBenchmarkCampaignReceipts,
  listBenchmarkSuiteReportCandidates,
  listBenchmarkSuiteReports,
  listBenchmarkSuites,
  listOllamaModels,
  listRuntimeRegistrations,
  qualifyOllamaModel,
  registerRuntimeCore,
  loadSettings,
  setControlMode,
  startAutodrive,
  startBenchmarkCampaign,
  startModelGameplayBenchmark,
  stopAutodrive,
  submitAgentTurn,
  startReplayRecording,
  stopReplayRecording,
  verifyLastReplay,
  saveSettings,
  scanRomDirectory,
  selectRomDirectory,
  selectSameBoyCore,
  selectLibretroCore,
  selectRuntimeContent,
  setGameProfile,
  startEmulation,
  startRegisteredEmulation,
  stepEmulation,
  stopEmulation,
  validateActionEnvelope,
  type AgentTurnResponse,
  type AppSettings,
  type AutodriveArtifact,
  type AutodriveStatus,
  type AuthorityStatus,
  type BenchmarkCampaignArtifact,
  type BenchmarkCampaignStatus,
  type CampaignComparisonArtifact,
  type CampaignListEntry,
  type BenchmarkSuiteComparisonArtifact,
  type BenchmarkSuiteListEntry,
  type BenchmarkSuiteReportArtifact,
  type BenchmarkSuiteReportCandidate,
  type BenchmarkSuiteReportListEntry,
  type ControlMode,
  type DriverStatus,
  type FramePacket,
  type GameProfile,
  type ModelBenchmarkArtifact,
  type OllamaModel,
  type OllamaQualificationStatus,
  type ReplayArtifact,
  type RomEntry,
  type RuntimeRegistrationReceipt,
  type SessionInfo,
} from "./native";

const systems = ["ALL", "NES", "SNES", "GB", "GBC", "GBA", "GENESIS", "PS1"] as const;
const PHIBOT_AGENT_ID = "phi-local";

const milestones = [
  ["BRIEFING", "BANKS", "CIRCLE and CROSS each hold their own TRIANGLE/SQUARE relation, with opposite bindings visible at the same time."],
  ["ERASURE", "MEMORY", "A removes both bank markers and all position-bearing symbol evidence before the later decision exists."],
  ["ROUTE", "SELECT", "The later scene reveals bank selector + query + MATCH/FLIP, forcing addressed retrieval from the correct erased bank."],
  ["COMMIT", "CONTEXT", "Layout × bank × query × operator spans sixteen variants; fixed and context-ignoring shortcuts cap at 8/16."],
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
  const [driverMemory, setDriverMemory] = useState<DriverStatus | null>(null);
  const [ollamaModels, setOllamaModels] = useState<OllamaModel[]>([]);
  const [ollamaOnline, setOllamaOnline] = useState(false);
  const [ollamaScanning, setOllamaScanning] = useState(false);
  const [modelQualification, setModelQualification] = useState<OllamaQualificationStatus | null>(null);
  const [qualificationBusy, setQualificationBusy] = useState(false);
  const [providerBusy, setProviderBusy] = useState(false);
  const [lastProviderDurationMs, setLastProviderDurationMs] = useState<number | null>(null);
  const [autodrive, setAutodrive] = useState<AutodriveStatus | null>(null);
  const [lastAutodrive, setLastAutodrive] = useState<AutodriveArtifact | null>(null);
  const [benchmarkRunning, setBenchmarkRunning] = useState(false);
  const [benchmarkRunId, setBenchmarkRunId] = useState<number | null>(null);
  const [lastModelBenchmark, setLastModelBenchmark] = useState<ModelBenchmarkArtifact | null>(null);
  const [campaignStatus, setCampaignStatus] = useState<BenchmarkCampaignStatus | null>(null);
  const [lastCampaign, setLastCampaign] = useState<BenchmarkCampaignArtifact | null>(null);
  const [campaignLedger, setCampaignLedger] = useState<CampaignListEntry[]>([]);
  const [comparisonAId, setComparisonAId] = useState<number | null>(null);
  const [comparisonBId, setComparisonBId] = useState<number | null>(null);
  const [lastComparison, setLastComparison] = useState<CampaignComparisonArtifact | null>(null);
  const [comparisonBusy, setComparisonBusy] = useState(false);
  const [suiteRegistry, setSuiteRegistry] = useState<BenchmarkSuiteListEntry[]>([]);
  const [selectedSuiteId, setSelectedSuiteId] = useState("");
  const [suiteCandidates, setSuiteCandidates] = useState<BenchmarkSuiteReportCandidate[]>([]);
  const [selectedSuiteCohortId, setSelectedSuiteCohortId] = useState<string | null>(null);
  const [lastSuiteReport, setLastSuiteReport] = useState<BenchmarkSuiteReportArtifact | null>(null);
  const [suiteReportBusy, setSuiteReportBusy] = useState(false);
  const [suiteReportLedger, setSuiteReportLedger] = useState<BenchmarkSuiteReportListEntry[]>([]);
  const [suiteComparisonAId, setSuiteComparisonAId] = useState<number | null>(null);
  const [suiteComparisonBId, setSuiteComparisonBId] = useState<number | null>(null);
  const [lastSuiteComparison, setLastSuiteComparison] = useState<BenchmarkSuiteComparisonArtifact | null>(null);
  const [suiteComparisonBusy, setSuiteComparisonBusy] = useState(false);
  const [scanning, setScanning] = useState(false);
  const [runtimeRegistrations, setRuntimeRegistrations] = useState<RuntimeRegistrationReceipt[]>([]);
  const [runtimeRegistryBusy, setRuntimeRegistryBusy] = useState(false);
  const [selectedRuntimeSha, setSelectedRuntimeSha] = useState<string | null>(null);
  const [registeredLaunchApproved, setRegisteredLaunchApproved] = useState(false);
  const [registeredLaunchBusy, setRegisteredLaunchBusy] = useState(false);

  const selectedSuite = suiteRegistry.find((suite) => suite.id === selectedSuiteId) ?? null;
  const selectedRuntime = runtimeRegistrations.find((runtime) => runtime.coreSha256 === selectedRuntimeSha) ?? null;

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
        if (native) {
          const registrations = await listRuntimeRegistrations();
          if (!cancelled) setRuntimeRegistrations(registrations);
        }
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

  const registerRuntime = async () => {
    if (!native) {
      setNotice("RUNTIME REGISTRY REQUIRES THE TAURI DESKTOP SHELL");
      return;
    }
    setRuntimeRegistryBusy(true);
    try {
      const selected = await selectLibretroCore();
      if (!selected) {
        setNotice("RUNTIME REGISTRATION CANCELLED");
        return;
      }
      const receipt = await registerRuntimeCore(selected);
      const registrations = await listRuntimeRegistrations();
      setRuntimeRegistrations(registrations);
      setSelectedRuntimeSha(receipt.coreSha256);
      setRegisteredLaunchApproved(false);
      const profile = receipt.qualificationProfileId ?? "UNQUALIFIED PROFILE";
      setNotice(
        `RUNTIME REGISTERED // ${receipt.core.libraryName} ${receipt.core.libraryVersion} // ${profile} // SHA ${receipt.coreSha256.slice(0, 12)}…`,
      );
    } catch (error) {
      setNotice(`RUNTIME REGISTRY ERROR // ${String(error)}`);
    } finally {
      setRuntimeRegistryBusy(false);
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
      if (settings.ollamaModel && models.some((candidate) => candidate.name === settings.ollamaModel || candidate.model === settings.ollamaModel)) {
        try {
          setModelQualification(
            await getOllamaQualificationStatus(settings.ollamaBaseUrl, settings.ollamaModel),
          );
        } catch {
          setModelQualification(null);
        }
      } else {
        setModelQualification(null);
      }
      setNotice(`OLLAMA ONLINE // ${models.length} LOCAL MODELS FOUND`);
    } catch (error) {
      setOllamaModels([]);
      setOllamaOnline(false);
      setModelQualification(null);
      setNotice(`OLLAMA OFFLINE // ${String(error)}`);
    } finally {
      setOllamaScanning(false);
    }
  };

  const chooseOllamaModel = async (model: string) => {
    const nextSettings = { ...settings, ollamaModel: model || null };
    setSettings(nextSettings);
    setModelQualification(null);
    try {
      await saveSettings(nextSettings);
      if (model) {
        const status = await getOllamaQualificationStatus(settings.ollamaBaseUrl, model);
        setModelQualification(status);
        setNotice(
          `OLLAMA MODEL // ${model} // ${status.details.capabilities.join("+") || "NO CAPABILITIES"} // ${status.qualified ? "QUALIFIED" : "UNQUALIFIED"}`,
        );
      } else {
        setNotice("OLLAMA MODEL CLEARED");
      }
    } catch (error) {
      setNotice(`MODEL INSPECTION ERROR // ${String(error)}`);
    }
  };

  const qualifySelectedOllamaModel = async () => {
    if (!settings.ollamaModel) {
      setNotice("SELECT AN OLLAMA MODEL FIRST");
      return;
    }

    providerBusyRef.current = true;
    setProviderBusy(true);
    setQualificationBusy(true);
    setNotice(`MODEL QUALIFICATION // ${settings.ollamaModel} // SYNTHETIC VISION PROBE`);

    try {
      const artifact = await qualifyOllamaModel(settings.ollamaBaseUrl, settings.ollamaModel);
      const status = await getOllamaQualificationStatus(
        settings.ollamaBaseUrl,
        settings.ollamaModel,
      );
      setModelQualification(status);
      const receipt = artifact.receipt;
      setNotice(
        `MODEL QUALIFICATION ${receipt.result} // ${receipt.model} // ${receipt.digest.slice(0, 12)}… // VISION ${receipt.visionProbePass ? "PASS" : "FAIL"} // JSON ${receipt.structuredOutputPass ? "PASS" : "FAIL"}`,
      );
    } catch (error) {
      setModelQualification(null);
      setNotice(`MODEL QUALIFICATION ERROR // ${String(error)}`);
    } finally {
      providerBusyRef.current = false;
      setProviderBusy(false);
      setQualificationBusy(false);
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
      setDriverMemory(status);
      const currentAutodrive = await getAutodriveStatus();
      setAutodrive(currentAutodrive);
      const durationMs = result.totalDurationNs === null
        ? null
        : result.totalDurationNs / 1_000_000;
      setLastProviderDurationMs(durationMs);
      setLastObservation(
        `OLLAMA T${request.turnId} // F${request.observation.frame} // ${result.response.actions.length} ACTIONS // MEM r${status.memoryRevision} ${status.memoryBytes}B`,
      );
      setNotice(
        autonomous && currentAutodrive
          ? `AUTODRIVE TURN ${request.turnId} ACCEPTED // ${result.model} // ${result.response.actions.length} ACTIONS // NEXT OBS F${currentAutodrive.nextObservationFrame}`
          : `OLLAMA TURN ${request.turnId} ACCEPTED // ${result.model} // ${result.response.actions.length} ACTIONS`,
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
          setDriverMemory(status);
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
      setNotice("SELECT A LOCAL OLLAMA MODEL FIRST");
      return;
    }
    try {
      const qualification = await getOllamaQualificationStatus(
        settings.ollamaBaseUrl,
        settings.ollamaModel,
      );
      setModelQualification(qualification);
      if (!qualification.qualified) {
        throw new Error("selected model digest is not qualified for autonomous vision play");
      }

      const status = await startAutodrive(
        "ollama",
        settings.ollamaModel,
        qualification.details.digest,
        defaultAutodrivePolicy,
      );
      setAutodrive(status);
      setLastAutodrive(null);
      setDriverMemory(await getDriverStatus());
      setNotice(
        `AUTODRIVE RUN ${status.runId} // QUALIFIED ${qualification.details.digest.slice(0, 12)}… // ${status.policy.maxTurns}T / ${status.policy.maxTotalActions}A / CADENCE ${status.policy.minObservationIntervalFrames}-${status.policy.maxObservationIntervalFrames}F // MEMORY ${status.policy.maxMemoryBytes}B/${status.policy.maxMemoryUpdateBytes}B`,
      );
    } catch (error) {
      setNotice(`AUTODRIVE START ERROR // ${String(error)}`);
    }
  };

  const refreshSuiteReportCandidates = async (suiteId = selectedSuiteId) => {
    if (!suiteId) {
      setSuiteCandidates([]);
      setSelectedSuiteCohortId(null);
      return;
    }
    try {
      const candidates = await listBenchmarkSuiteReportCandidates(suiteId);
      setSuiteCandidates(candidates);
      const ready = candidates.filter((candidate) => candidate.ready);
      setSelectedSuiteCohortId((current) => {
        if (current && ready.some((candidate) => candidate.cohortId === current)) {
          return current;
        }
        const selectedDigest = modelQualification?.details.digest;
        const sameModel = selectedDigest
          ? ready.find((candidate) => candidate.modelDigest === selectedDigest)
          : null;
        return sameModel?.cohortId ?? ready[0]?.cohortId ?? null;
      });
    } catch (error) {
      setNotice(`SUITE REPORT LEDGER ERROR // ${String(error)}`);
    }
  };

  const refreshSuiteReportLedger = async (suiteId = selectedSuiteId) => {
    if (!suiteId) {
      setSuiteReportLedger([]);
      setSuiteComparisonAId(null);
      setSuiteComparisonBId(null);
      return;
    }
    try {
      const reports = (await listBenchmarkSuiteReports())
        .filter((entry) => entry.suiteId === suiteId);
      setSuiteReportLedger(reports);
      if (reports.length >= 2) {
        const newest = reports[reports.length - 1];
        const previous = reports[reports.length - 2];
        setSuiteComparisonAId((current) =>
          current !== null && reports.some((entry) => entry.reportId === current)
            ? current
            : previous.reportId,
        );
        setSuiteComparisonBId((current) =>
          current !== null && reports.some((entry) => entry.reportId === current)
            ? current
            : newest.reportId,
        );
      } else {
        setSuiteComparisonAId(reports[0]?.reportId ?? null);
        setSuiteComparisonBId(null);
      }
    } catch (error) {
      setNotice(`SUITE COMPARISON LEDGER ERROR // ${String(error)}`);
    }
  };

  const changeSelectedSuite = async (suiteId: string) => {
    setSelectedSuiteId(suiteId);
    setSuiteCandidates([]);
    setSelectedSuiteCohortId(null);
    setLastSuiteReport(null);
    setSuiteReportLedger([]);
    setSuiteComparisonAId(null);
    setSuiteComparisonBId(null);
    setLastSuiteComparison(null);
    await Promise.all([
      refreshSuiteReportCandidates(suiteId),
      refreshSuiteReportLedger(suiteId),
    ]);
  };

  const runSuiteComparison = async () => {
    if (!selectedSuiteId) {
      setNotice("SELECT A BENCHMARK SUITE FIRST");
      return;
    }
    if (suiteComparisonAId === null || suiteComparisonBId === null) {
      setNotice("SUITE COMPARISON REQUIRES TWO REPORTS");
      return;
    }
    if (suiteComparisonAId === suiteComparisonBId) {
      setNotice("SUITE COMPARISON REQUIRES DISTINCT REPORTS");
      return;
    }

    setSuiteComparisonBusy(true);
    try {
      const artifact = await compareBenchmarkSuiteReports(
        selectedSuiteId,
        suiteComparisonAId,
        suiteComparisonBId,
      );
      setLastSuiteComparison(artifact);
      const stats = artifact.receipt.stats;
      setNotice(
        `${artifact.receipt.suiteId.toUpperCase()} // SUITE COMPARISON #${artifact.receipt.comparisonId} // ΔMACRO ${stats.macroMeanScoreDifferenceAMinusB.toFixed(1)} // ΔSUCCESS ${(stats.overallSuccessRateDifferenceAMinusB * 100).toFixed(1)}pp // ${stats.taskCount} TASKS`,
      );
    } catch (error) {
      setLastSuiteComparison(null);
      setNotice(`SUITE COMPARISON REFUSED // ${String(error)}`);
    } finally {
      setSuiteComparisonBusy(false);
    }
  };

  const buildSelectedSuiteReport = async () => {
    if (!selectedSuiteId || !selectedSuiteCohortId) {
      setNotice("SUITE REPORT REQUIRES A SELECTED SUITE AND READY COHORT");
      return;
    }
    setSuiteReportBusy(true);
    try {
      const artifact = await buildBenchmarkSuiteReport(
        selectedSuiteId,
        selectedSuiteCohortId,
      );
      setLastSuiteReport(artifact);
      const stats = artifact.receipt.stats;
      setNotice(
        `${artifact.receipt.suiteId.toUpperCase()} // REPORT #${artifact.receipt.reportId} // MACRO μ ${stats.macroMeanScore1000.toFixed(1)} // SUCCESS ${(stats.overallSuccessRate * 100).toFixed(1)}% // ${stats.taskCount} TASKS`,
      );
      await refreshSuiteReportCandidates(selectedSuiteId);
      await refreshSuiteReportLedger(selectedSuiteId);
    } catch (error) {
      setLastSuiteReport(null);
      setNotice(`SUITE REPORT REFUSED // ${String(error)}`);
    } finally {
      setSuiteReportBusy(false);
    }
  };
  const refreshComparisonCampaigns = async () => {
    try {
      const entries = await listBenchmarkCampaignReceipts();
      setCampaignLedger(entries);
      const complete = entries.filter((entry) => entry.recordStatus === "COMPLETE");
      if (complete.length >= 2) {
        const newest = complete[complete.length - 1];
        const previous = complete[complete.length - 2];
        setComparisonAId((current) =>
          current !== null && complete.some((entry) => entry.campaignId === current)
            ? current
            : previous.campaignId,
        );
        setComparisonBId((current) =>
          current !== null && complete.some((entry) => entry.campaignId === current)
            ? current
            : newest.campaignId,
        );
      } else {
        setComparisonAId(complete[0]?.campaignId ?? null);
        setComparisonBId(null);
      }
    } catch (error) {
      setNotice(`COMPARISON LEDGER ERROR // ${String(error)}`);
    }
  };

  const runCampaignComparison = async () => {
    if (comparisonAId === null || comparisonBId === null) {
      setNotice("COMPARISON REQUIRES TWO CAMPAIGNS");
      return;
    }
    setComparisonBusy(true);
    try {
      const artifact = await compareBenchmarkCampaigns(comparisonAId, comparisonBId);
      setLastComparison(artifact);
      const stats = artifact.receipt.stats;
      setNotice(
        `COMPARISON #${artifact.receipt.comparisonId} // ΔMEAN A−B ${stats.meanScoreDifferenceAMinusB.toFixed(1)} // 95% CI [${stats.meanDifferenceCi95Low.toFixed(1)}, ${stats.meanDifferenceCi95High.toFixed(1)}]`,
      );
    } catch (error) {
      setLastComparison(null);
      setNotice(`COMPARISON REFUSED // ${String(error)}`);
    } finally {
      setComparisonBusy(false);
    }
  };
  const beginModelBenchmark = async () => {
    if (!settings.ollamaModel) {
      setNotice("SELECT A LOCAL OLLAMA MODEL FIRST");
      return;
    }
    if (!session?.benchmarkTask) {
      setNotice("BENCHMARK REQUIRES A REGISTERED SUITE ROM");
      return;
    }

    providerBusyRef.current = true;
    setProviderBusy(true);
    try {
      const qualification = await getOllamaQualificationStatus(
        settings.ollamaBaseUrl,
        settings.ollamaModel,
      );
      setModelQualification(qualification);
      if (!qualification.qualified) {
        throw new Error("selected model digest is not qualified for benchmark play");
      }

      const started = await startModelGameplayBenchmark(
        "ollama",
        settings.ollamaModel,
        qualification.details.digest,
        defaultAutodrivePolicy,
      );
      frameRef.current = started.autodrive.startedFrame;
      setFrameNumber(started.autodrive.startedFrame);
      gamepadRef.current = emptyGameBoyButtons();
      setAutodrive(started.autodrive);
      setAuthority(await getAuthorityStatus());
      setDriverPendingTurnId(null);
      setDriverQueuedActions(0);
      setBenchmarkRunning(true);
      setBenchmarkRunId(started.benchmarkRunId);
      setLastModelBenchmark(null);
      setLastAutodrive(null);
      setNotice(
        `MODEL BENCHMARK #${started.benchmarkRunId} // ${session.benchmarkTask.title} // ${settings.ollamaModel} // RESET + WARMED // SCORE PENDING`,
      );
    } catch (error) {
      setNotice(`MODEL BENCHMARK START ERROR // ${String(error)}`);
    } finally {
      providerBusyRef.current = false;
      setProviderBusy(false);
    }
  };

  const beginBenchmarkCampaign = async () => {
    if (!settings.ollamaModel) {
      setNotice("SELECT A LOCAL OLLAMA MODEL FIRST");
      return;
    }
    if (!session?.benchmarkTask) {
      setNotice("CAMPAIGN REQUIRES A REGISTERED SUITE ROM");
      return;
    }

    providerBusyRef.current = true;
    setProviderBusy(true);
    try {
      const qualification = await getOllamaQualificationStatus(
        settings.ollamaBaseUrl,
        settings.ollamaModel,
      );
      setModelQualification(qualification);
      if (!qualification.qualified) {
        throw new Error("selected model digest is not qualified for campaign play");
      }

      const started = await startBenchmarkCampaign(
        settings.ollamaBaseUrl,
        "ollama",
        settings.ollamaModel,
        qualification.details.digest,
        defaultAutodrivePolicy,
        5,
      );
      frameRef.current = started.benchmark.autodrive.startedFrame;
      setFrameNumber(started.benchmark.autodrive.startedFrame);
      gamepadRef.current = emptyGameBoyButtons();
      setAutodrive(started.benchmark.autodrive);
      setAuthority(await getAuthorityStatus());
      setDriverPendingTurnId(null);
      setDriverQueuedActions(0);
      setBenchmarkRunning(true);
      setBenchmarkRunId(started.benchmark.benchmarkRunId);
      setCampaignStatus(started.status);
      setLastCampaign(null);
      setLastModelBenchmark(null);
      setLastAutodrive(null);
      setNotice(
        `CAMPAIGN #${started.status.campaignId} // TRIAL 1/${started.status.totalTrials} // DIGEST ${started.status.modelDigest.slice(0, 12)}…`,
      );
    } catch (error) {
      setNotice(`CAMPAIGN START ERROR // ${String(error)}`);
    } finally {
      providerBusyRef.current = false;
      setProviderBusy(false);
    }
  };

  const endBenchmarkCampaign = async () => {
    providerBusyRef.current = false;
    setProviderBusy(false);
    try {
      const artifact = await cancelBenchmarkCampaign();
      setLastCampaign(artifact);
      setCampaignStatus(await getBenchmarkCampaignStatus());
      await refreshComparisonCampaigns();
      await refreshSuiteReportCandidates();
      setAutodrive(await getAutodriveStatus());
      setBenchmarkRunning(false);
      setBenchmarkRunId(null);
      setDriverPendingTurnId(null);
      setDriverQueuedActions(0);
      setNotice(
        `CAMPAIGN #${artifact.receipt.campaignId} PARTIAL // ${artifact.receipt.completedTrials}/${artifact.receipt.totalTrials} TRIALS // MEAN ${artifact.receipt.stats.meanScore1000?.toFixed(1) ?? "N/A"}`,
      );
    } catch (error) {
      setNotice(`CAMPAIGN END ERROR // ${String(error)}`);
    }
  };

  const endAutodrive = async () => {
    providerBusyRef.current = false;
    setProviderBusy(false);
    try {
      const artifact = await stopAutodrive();
      setLastAutodrive(artifact);
      setAutodrive(await getAutodriveStatus());
      setDriverPendingTurnId(null);
      setDriverQueuedActions(0);
      setDriverMemory(await getDriverStatus());
      setNotice(
        `AUTODRIVE STOPPED // ${artifact.receipt.stopReason.toUpperCase()} // ${artifact.receipt.turnsCompleted} TURNS // MEM r${artifact.receipt.memoryRevision} ${artifact.receipt.finalMemoryBytes}B`,
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
      || frameNumber < autodrive.nextObservationFrame
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
    frameNumber,
    autodrive?.nextObservationFrame,
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

  useEffect(() => {
    if (
      !benchmarkRunning
      || !autodrive
      || autodrive.active
      || !autodrive.stopReason
    ) {
      return;
    }

    setBenchmarkRunning(false);
    void (async () => {
      try {
        const artifact = await getLastModelGameplayBenchmark();
        if (!artifact) return;
        setLastModelBenchmark(artifact);

        const campaign = await getBenchmarkCampaignStatus();
        setCampaignStatus(campaign);

        if (campaign?.active && campaign.completedTrials < campaign.totalTrials) {
          setNotice(
            `CAMPAIGN #${campaign.campaignId} // TRIAL ${campaign.completedTrials}/${campaign.totalTrials} SEALED // RECHECKING DIGEST`,
          );
          const continued = await continueBenchmarkCampaign(settings.ollamaBaseUrl);
          frameRef.current = continued.benchmark.autodrive.startedFrame;
          setFrameNumber(continued.benchmark.autodrive.startedFrame);
          gamepadRef.current = emptyGameBoyButtons();
          setAutodrive(continued.benchmark.autodrive);
          setAuthority(await getAuthorityStatus());
          setDriverPendingTurnId(null);
          setDriverQueuedActions(0);
          setBenchmarkRunId(continued.benchmark.benchmarkRunId);
          setCampaignStatus(continued.status);
          setBenchmarkRunning(true);
          setNotice(
            `CAMPAIGN #${continued.status.campaignId} // TRIAL ${continued.status.completedTrials + 1}/${continued.status.totalTrials} // DIGEST RECONFIRMED`,
          );
          return;
        }

        if (campaign && !campaign.active) {
          const summary = await getLastBenchmarkCampaignReceipt();
          if (summary) {
            setLastCampaign(summary);
            setBenchmarkRunId(null);
            await refreshComparisonCampaigns();
            await refreshSuiteReportCandidates();
            const stats = summary.receipt.stats;
            setNotice(
              `CAMPAIGN COMPLETE // ${stats.successfulTrials}/${stats.observedTrials} SUCCESS // MEAN ${stats.meanScore1000?.toFixed(1) ?? "N/A"} // σ ${stats.populationStddevScore1000?.toFixed(1) ?? "N/A"}`,
            );
            return;
          }
        }

        setBenchmarkRunId(null);
        const receipt = artifact.receipt;
        const score = receipt.score1000 === null ? "SCORING ERROR" : `${receipt.score1000}/1000`;
        setNotice(
          `MODEL BENCHMARK COMPLETE // ${score} // ${receipt.taskSuccess ? "TARGET REACHED" : "INCOMPLETE"} // ${receipt.stopReason.toUpperCase()}`,
        );
      } catch (error) {
        setBenchmarkRunId(null);
        setNotice(`BENCHMARK / CAMPAIGN CONTINUATION ERROR // ${String(error)}`);
      }
    })();
  }, [benchmarkRunning, autodrive, settings.ollamaBaseUrl]);

  const changeControlMode = async (mode: ControlMode) => {
    if (mode === "human") {
      providerBusyRef.current = false;
      setProviderBusy(false);
    }
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
      setDriverMemory(await getDriverStatus());
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
        schema: "phicade.agent-turn-response.v2",
        turnId: request.turnId,
        agentId: PHIBOT_AGENT_ID,
        seat: 1,
        observationFrame: observation.frame,
        observationSha256: observation.frameSha256,
        memorySha256: request.memorySha256,
        memoryUpdate: null,
        actions: [
          { delayFrames: 0, action: { kind: "button", button, pressed: true } },
          { delayFrames: 2, action: { kind: "button", button, pressed: false } },
        ],
      };

      const status = await submitAgentTurn(response);
      setDriverPendingTurnId(status.pendingTurnId);
      setDriverQueuedActions(status.queuedActions);
      setDriverMemory(status);
      setLastObservation(`T${request.turnId} // F${observation.frame} // ${observation.frameSha256.slice(0, 12)}… // ${button} // MEM r${status.memoryRevision}`);
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

  const prepareSessionLaunch = async () => {
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
    setBenchmarkRunning(false);
    setBenchmarkRunId(null);
    setLastModelBenchmark(null);
    setCampaignStatus(null);
    setLastCampaign(null);
    setCampaignLedger([]);
    setComparisonAId(null);
    setComparisonBId(null);
    setLastComparison(null);
    setSuiteCandidates([]);
    setSelectedSuiteCohortId(null);
    setSelectedSuiteId("");
    setLastSuiteReport(null);
    setSuiteReportBusy(false);
    setSuiteReportLedger([]);
    setSuiteComparisonAId(null);
    setSuiteComparisonBId(null);
    setLastSuiteComparison(null);
    setSuiteComparisonBusy(false);
  };

  const adoptSession = async (info: SessionInfo, label: string) => {
    const [initialAuthority, initialDriver] = await Promise.all([
      getAuthorityStatus(),
      getDriverStatus(),
    ]);
    setSession(info);
    setAuthority(initialAuthority);
    setDriverPendingTurnId(initialDriver.pendingTurnId);
    setDriverQueuedActions(initialDriver.queuedActions);
    setDriverMemory(initialDriver);
    setProfile(info.profile);
    setRunning(true);
    runningRef.current = true;

    const suites = await listBenchmarkSuites();
    setSuiteRegistry(suites);
    const defaultSuiteId = suites[suites.length - 1]?.id ?? "";
    setSelectedSuiteId(defaultSuiteId);
    const [priorCampaigns, priorSuiteCandidates, allSuiteReports] = await Promise.all([
      listBenchmarkCampaignReceipts(),
      defaultSuiteId ? listBenchmarkSuiteReportCandidates(defaultSuiteId) : Promise.resolve([]),
      listBenchmarkSuiteReports(),
    ]);
    const priorSuiteReports = allSuiteReports.filter((entry) => entry.suiteId === defaultSuiteId);
    setCampaignLedger(priorCampaigns);
    setSuiteCandidates(priorSuiteCandidates);
    setSuiteReportLedger(priorSuiteReports);
    if (priorSuiteReports.length >= 2) {
      setSuiteComparisonAId(priorSuiteReports[priorSuiteReports.length - 2].reportId);
      setSuiteComparisonBId(priorSuiteReports[priorSuiteReports.length - 1].reportId);
    } else {
      setSuiteComparisonAId(priorSuiteReports[0]?.reportId ?? null);
      setSuiteComparisonBId(null);
    }

    const readySuiteCandidates = priorSuiteCandidates.filter((candidate) => candidate.ready);
    const selectedDigest = modelQualification?.details.digest;
    const matchingSuiteCandidate = selectedDigest
      ? readySuiteCandidates.find((candidate) => candidate.modelDigest === selectedDigest)
      : null;
    setSelectedSuiteCohortId(matchingSuiteCandidate?.cohortId ?? readySuiteCandidates[0]?.cohortId ?? null);
    const completeCampaigns = priorCampaigns.filter((entry) => entry.recordStatus === "COMPLETE");
    if (completeCampaigns.length >= 2) {
      setComparisonAId(completeCampaigns[completeCampaigns.length - 2].campaignId);
      setComparisonBId(completeCampaigns[completeCampaigns.length - 1].campaignId);
    }

    setNotice(
      `CORE ONLINE // ${info.core.libraryName} ${info.core.libraryVersion} // ${label} // ROUTE ${info.routeEvidence.route.toUpperCase()} // ${priorCampaigns.length} CAMPAIGNS // ${readySuiteCandidates.length} SUITE COHORTS READY`,
    );
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
      setNotice("LEGACY SAMEBOY LAUNCH IS GB/GBC ONLY // USE A REGISTERED RUNTIME FOR OTHER CONTENT");
      return;
    }

    try {
      let corePath = settings.sameboyCorePath;
      if (!corePath) corePath = await chooseCore();
      if (!corePath) {
        setNotice("SAMEBOY CORE SELECTION CANCELLED");
        return;
      }

      await prepareSessionLaunch();
      const info = await startEmulation(corePath, selectedGame.path);
      await adoptSession(info, selectedGame.displayName);
    } catch (error) {
      setNotice(`LAUNCH ERROR // ${String(error)}`);
      setRunning(false);
    }
  };

  const launchRegisteredRuntime = async (mode: "launcher" | "file") => {
    if (!native) {
      setNotice("REGISTERED RUNTIME LAUNCH REQUIRES THE TAURI DESKTOP SHELL");
      return;
    }
    if (!selectedRuntime) {
      setNotice("SELECT A REGISTERED RUNTIME FIRST");
      return;
    }
    if (!registeredLaunchApproved) {
      setNotice("OPERATOR APPROVAL REQUIRED FOR THIS REGISTERED RUNTIME SESSION");
      return;
    }

    setRegisteredLaunchBusy(true);
    try {
      let contentPath: string | null = null;
      if (mode === "file") {
        contentPath = await selectRuntimeContent();
        if (!contentPath) {
          setNotice("RUNTIME CONTENT SELECTION CANCELLED");
          return;
        }
      }

      await prepareSessionLaunch();
      const info = await startRegisteredEmulation(
        selectedRuntime.coreSha256,
        contentPath,
        true,
      );
      const label = contentPath
        ? contentPath.split(/[\\/]/).pop() ?? "FILE CONTENT"
        : "NO-CONTENT LAUNCHER";
      await adoptSession(info, label);
      setRegisteredLaunchApproved(false);
    } catch (error) {
      setNotice(`REGISTERED LAUNCH ERROR // ${String(error)}`);
      setRunning(false);
    } finally {
      setRegisteredLaunchBusy(false);
    }
  };

  const stopGame = async () => {
    setRunning(false);
    gamepadRef.current = emptyGameBoyButtons();
    try {
      await stopEmulation();
      setSession(null);
      setProfile(null);
      setDriverMemory(null);
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
      setBenchmarkRunning(false);
      setBenchmarkRunId(null);
      setLastModelBenchmark(null);
      setCampaignStatus(null);
      setLastCampaign(null);
      setCampaignLedger([]);
      setComparisonAId(null);
      setComparisonBId(null);
      setLastComparison(null);
      setComparisonBusy(false);
      setSuiteCandidates([]);
      setSelectedSuiteCohortId(null);
      setLastSuiteReport(null);
      setSuiteReportBusy(false);
      setSuiteReportLedger([]);
      setSuiteComparisonAId(null);
      setSuiteComparisonBId(null);
      setLastSuiteComparison(null);
      setSuiteComparisonBusy(false);
      providerBusyRef.current = false;
      setProviderBusy(false);
      setNotice("CORE SESSION STOPPED // PERSISTENT DATA FLUSH ATTEMPT COMPLETE");
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
          <span><i className={`lamp ${running ? "lamp-green" : "lamp-amber"}`} /> {running ? `${session?.core.libraryName.toUpperCase() ?? "CORE"} RUNNING` : "CORE HOST STANDBY"}</span>
          <span><i className={`lamp ${replayRecording ? "lamp-amber" : "lamp-green"}`} /> {replayRecording ? "REPLAY RECORDING" : "LEDGER READY"}</span>
          <span><i className={`lamp ${authority?.mode === "phi-bot" || authority?.mode === "coop" ? "lamp-amber" : "lamp-green"}`} /> AUTHORITY {authority?.mode?.toUpperCase() ?? "OFFLINE"}</span>
          <span><i className={`lamp ${driverPendingTurnId !== null || driverQueuedActions > 0 ? "lamp-amber" : "lamp-green"}`} /> DRIVER {driverPendingTurnId !== null ? `TURN ${driverPendingTurnId}` : driverQueuedActions > 0 ? `${driverQueuedActions} QUEUED` : "READY"}</span>
          <span><i className="lamp lamp-green" /> MEMORY {driverMemory ? `r${driverMemory.memoryRevision} / ${driverMemory.memoryBytes}B / ${driverMemory.memorySha256.slice(0, 8)}…` : "STANDBY"}</span>
          <span><i className={`lamp ${ollamaOnline ? "lamp-green" : "lamp-amber"}`} /> OLLAMA {providerBusy ? "THINKING" : ollamaOnline ? "LOCAL" : "UNPROBED"}</span>
          <span><i className={`lamp ${autodrive?.active ? "lamp-amber" : "lamp-green"}`} /> AUTODRIVE {autodrive?.active ? `RUN ${autodrive.runId}` : autodrive?.stopReason?.toUpperCase() ?? "STANDBY"}</span>
          <span><i className={`lamp ${benchmarkRunning ? "lamp-amber" : lastModelBenchmark ? "lamp-green" : "lamp-green"}`} /> BENCH {benchmarkRunning ? `RUN ${benchmarkRunId}` : lastModelBenchmark ? `${lastModelBenchmark.receipt.score1000 ?? "ERR"}/1000` : "STANDBY"}</span>
          <span><i className={`lamp ${campaignStatus?.active ? "lamp-amber" : lastCampaign ? "lamp-green" : "lamp-green"}`} /> CAMPAIGN {campaignStatus?.active ? `${campaignStatus.completedTrials}/${campaignStatus.totalTrials}` : lastCampaign ? `#${lastCampaign.receipt.campaignId}` : "STANDBY"}</span>
          <span><i className={`lamp ${comparisonBusy ? "lamp-amber" : lastComparison ? "lamp-green" : "lamp-green"}`} /> COMPARE {comparisonBusy ? "VERIFYING" : lastComparison ? `#${lastComparison.receipt.comparisonId}` : "STANDBY"}</span>
          <span><i className={`lamp ${suiteReportBusy ? "lamp-amber" : lastSuiteReport ? "lamp-green" : "lamp-green"}`} /> SUITE {suiteReportBusy ? "VERIFYING" : lastSuiteReport ? `#${lastSuiteReport.receipt.reportId}` : suiteCandidates.some((candidate) => candidate.ready) ? "READY" : "INCOMPLETE"}</span>
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
          <button className="utility-button" onClick={chooseCore} disabled={!native}>LEGACY SAMEBOY CORE: {settings.sameboyCorePath ? "SET" : "SELECT"}</button>
          <button className="utility-button" onClick={toggleAutoScan} disabled={!native}>AUTO-SCAN: {settings.autoScan ? "ON" : "OFF"}</button>
          <p className="microcopy path-copy">{settings.romDirectory ?? "NO DIRECTORY SAVED"}</p>

          <div className="rule" />
          <div className="runtime-manager">
            <div className="runtime-manager-heading">
              <span>RUNTIME REGISTRY</span>
              <strong>{runtimeRegistrations.length}</strong>
            </div>
            <button
              className="utility-button"
              onClick={() => void registerRuntime()}
              disabled={!native || runtimeRegistryBusy || running}
            >
              {runtimeRegistryBusy ? "INSPECTING CORE..." : "REGISTER LIBRETRO CORE"}
            </button>
            {runtimeRegistrations.length === 0 ? (
              <p className="microcopy">NO REGISTERED RUNTIMES // REGISTRATION GRANTS NO LAUNCH AUTHORITY</p>
            ) : (
              <div className="runtime-list">
                {runtimeRegistrations.map((runtime) => {
                  const capabilities = runtime.capabilityManifest.capabilities;
                  const qualified = Object.values(capabilities).filter((status) => status === "QUALIFIED").length;
                  const supported = Object.values(capabilities).filter((status) => status !== "UNSUPPORTED").length;
                  return (
                    <div
                      className={`runtime-row ${selectedRuntimeSha === runtime.coreSha256 ? "selected" : ""}`}
                      key={runtime.coreSha256}
                      title={runtime.corePath}
                    >
                      <div className="runtime-row-main">
                        <strong>{runtime.core.libraryName} {runtime.core.libraryVersion}</strong>
                        <span>{runtime.capabilityManifest.executionModel.replaceAll("_", " ")}</span>
                      </div>
                      <small>SHA // {runtime.coreSha256.slice(0, 16)}…</small>
                      <small>CAPS // {qualified} QUALIFIED / {supported} AVAILABLE</small>
                      <small>
                        PROFILE // {runtime.qualificationProfileId ?? "NONE"} // EVIDENCE {runtime.binaryEvidenceBound ? "BOUND" : "UNBOUND"} // AUTHORITY {runtime.authorityGranted ? "GRANTED" : "NONE"}
                      </small>
                      <button
                        className="runtime-select"
                        onClick={() => {
                          setSelectedRuntimeSha(runtime.coreSha256);
                          setRegisteredLaunchApproved(false);
                        }}
                        disabled={running || registeredLaunchBusy}
                      >
                        {selectedRuntimeSha === runtime.coreSha256 ? "SELECTED" : "SELECT"}
                      </button>
                    </div>
                  );
                })}
              </div>
            )}
            {selectedRuntime && (
              <div className="runtime-launch">
                <div className="runtime-launch-heading">
                  <strong>{selectedRuntime.core.libraryName} {selectedRuntime.core.libraryVersion}</strong>
                  <small>{selectedRuntime.coreSha256.slice(0, 16)}…</small>
                </div>
                <label className="runtime-approval">
                  <input
                    type="checkbox"
                    checked={registeredLaunchApproved}
                    onChange={(event) => setRegisteredLaunchApproved(event.target.checked)}
                    disabled={running || registeredLaunchBusy}
                  />
                  <span>APPROVE THIS SESSION LAUNCH</span>
                </label>
                <div className="runtime-launch-actions">
                  <button
                    onClick={() => void launchRegisteredRuntime("launcher")}
                    disabled={!registeredLaunchApproved || running || registeredLaunchBusy}
                  >
                    {registeredLaunchBusy ? "STARTING..." : "RUN LAUNCHER"}
                  </button>
                  <button
                    onClick={() => void launchRegisteredRuntime("file")}
                    disabled={!registeredLaunchApproved || running || registeredLaunchBusy}
                  >
                    RUN FILE…
                  </button>
                </div>
                <small>
                  APPROVAL IS SESSION-SCOPED // REGISTRATION AUTHORITY REMAINS {selectedRuntime.authorityGranted ? "GRANTED" : "NONE"}
                </small>
              </div>
            )}
          </div>
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
                  <p>{selectedGame ? `${selectedGame.system} // ${selectedGame.displayName}` : selectedRuntime ? `RUNTIME // ${selectedRuntime.core.libraryName} ${selectedRuntime.core.libraryVersion}` : "SELECT CARTRIDGE OR RUNTIME"}</p>
                  <small>RUNG 39 // REGISTERED RUNTIME LAUNCH ONLINE</small>
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
            <button className={authority?.mode === "human" ? "active" : ""} onClick={() => changeControlMode("human")} disabled={!running || replayRecording}>HUMAN</button>
            <button className={authority?.mode === "phi-bot" ? "active" : ""} onClick={() => changeControlMode("phi-bot")} disabled={!running || replayRecording || autodrive?.active}>HANDOFF</button>
            <button className={authority?.mode === "coop" ? "active" : ""} onClick={() => changeControlMode("coop")} disabled={!running || replayRecording || autodrive?.active}>CO-OP</button>
            <button onClick={() => changeControlMode("versus")} disabled={!running || replayRecording || autodrive?.active}>VERSUS</button>
            <button onClick={runReferenceDriverTurn} disabled={!running || replayRecording || autodrive?.active || !authority || !["phi-bot", "coop"].includes(authority.mode) || driverPendingTurnId !== null}>DRIVER TURN</button>
            <small>{lastObservation ?? (authority?.agentId ? `${authority.agentId} // P${authority.agentSeat} // GRANT TO F${authority.expiresAtFrame}` : "NO AGENT GRANT")}</small>
            <small>MEMORY // r{driverMemory?.memoryRevision ?? 0} // {driverMemory?.memoryBytes ?? 0}B // {driverMemory?.memoryUpdates ?? 0} UPDATES // {driverMemory?.memoryRefusals ?? 0} REFUSALS // {driverMemory?.memoryContent || "(empty)"}</small>
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
              <option value="">SELECT LOCAL MODEL</option>
              {ollamaModels.map((model) => (
                <option key={model.digest || model.name} value={model.name}>{model.name}</option>
              ))}
            </select>
            <button
              className={modelQualification?.qualified ? "model-qualified" : ""}
              onClick={() => void qualifySelectedOllamaModel()}
              disabled={!settings.ollamaModel || providerBusy || qualificationBusy || autodrive?.active}
            >
              {qualificationBusy ? "QUALIFYING..." : modelQualification?.qualified ? "QUALIFIED ✓" : "QUALIFY MODEL"}
            </button>
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
                || (!autodrive?.active && (!modelQualification?.qualified || !ollamaOnline || providerBusy || driverPendingTurnId !== null || driverQueuedActions !== 0))
              }
            >
              {autodrive?.active ? "STOP AUTO" : "AUTO DRIVE"}
            </button>
            <button
              className={benchmarkRunning && !campaignStatus?.active ? "benchmark-active" : ""}
              onClick={() => void beginModelBenchmark()}
              disabled={
                !running
                || replayRecording
                || autodrive?.active
                || providerBusy
                || campaignStatus?.active
                || authority?.mode !== "phi-bot"
                || !modelQualification?.qualified
                || !session
                || !session.benchmarkTask
              }
            >
              {benchmarkRunning && !campaignStatus?.active ? "BENCH RUNNING" : "BENCH TASK"}
            </button>
            <button
              className={campaignStatus?.active ? "campaign-active" : ""}
              onClick={() => void (campaignStatus?.active ? endBenchmarkCampaign() : beginBenchmarkCampaign())}
              disabled={
                !campaignStatus?.active
                && (
                  !running
                  || replayRecording
                  || autodrive?.active
                  || providerBusy
                  || authority?.mode !== "phi-bot"
                  || !modelQualification?.qualified
                  || !session
                  || !session.benchmarkTask
                )
              }
            >
              {campaignStatus?.active ? "END CAMPAIGN" : "CAMPAIGN 5×"}
            </button>
            <small>
              {campaignStatus?.active && benchmarkRunning && autodrive?.active
                ? `CAMPAIGN #${campaignStatus.campaignId} // TRIAL ${campaignStatus.completedTrials + 1}/${campaignStatus.totalTrials} // AUTO ${autodrive.runId}`
                : benchmarkRunning && autodrive?.active
                  ? `BENCH #${benchmarkRunId} // AUTO ${autodrive.runId} // ${autodrive.turnsCompleted}/${autodrive.policy.maxTurns}T // SCORE PENDING`
                  : autodrive?.active
                  ? `RUN ${autodrive.runId} // ${autodrive.turnsCompleted}/${autodrive.policy.maxTurns}T // ${autodrive.totalActions}/${autodrive.policy.maxTotalActions}A`
                : settings.ollamaModel
                  ? `${settings.ollamaModel} // ${modelQualification?.details.digest.slice(0, 12) ?? "UNINSPECTED"}… // ${modelQualification?.details.capabilities.join("+") || "NO CAPABILITY DATA"} // ${modelQualification?.qualified ? "QUALIFIED" : "UNQUALIFIED"}${lastProviderDurationMs === null ? "" : ` // ${Math.round(lastProviderDurationMs)}MS`}`
                  : `NO MODEL // ${settings.ollamaBaseUrl}`}
            </small>
          </div>

          <div className="comparison-strip">
            <span>COMPARISON LAB</span>
            <select
              value={comparisonAId ?? ""}
              onChange={(event) => {
                setComparisonAId(event.target.value ? Number(event.target.value) : null);
                setLastComparison(null);
              }}
              disabled={campaignLedger.length === 0 || comparisonBusy || autodrive?.active}
              aria-label="Comparison campaign A"
            >
              <option value="">CAMPAIGN A</option>
              {campaignLedger.map((entry) => (
                <option key={`a-${entry.campaignId}`} value={entry.campaignId}>
                  {`#${entry.campaignId} ${entry.model} ${entry.recordStatus} μ${entry.meanScore1000?.toFixed(1) ?? "ERR"}`}
                </option>
              ))}
            </select>
            <select
              value={comparisonBId ?? ""}
              onChange={(event) => {
                setComparisonBId(event.target.value ? Number(event.target.value) : null);
                setLastComparison(null);
              }}
              disabled={campaignLedger.length === 0 || comparisonBusy || autodrive?.active}
              aria-label="Comparison campaign B"
            >
              <option value="">CAMPAIGN B</option>
              {campaignLedger.map((entry) => (
                <option key={`b-${entry.campaignId}`} value={entry.campaignId}>
                  {`#${entry.campaignId} ${entry.model} ${entry.recordStatus} μ${entry.meanScore1000?.toFixed(1) ?? "ERR"}`}
                </option>
              ))}
            </select>
            <button
              onClick={() => void runCampaignComparison()}
              disabled={
                comparisonBusy
                || comparisonAId === null
                || comparisonBId === null
                || comparisonAId === comparisonBId
                || autodrive?.active
                || campaignStatus?.active
              }
            >
              {comparisonBusy ? "VERIFYING..." : "COMPARE"}
            </button>
            <button onClick={() => void refreshComparisonCampaigns()} disabled={comparisonBusy || autodrive?.active}>
              REFRESH
            </button>
            <small>
              {lastComparison
                ? `#${lastComparison.receipt.comparisonId} // Δμ ${lastComparison.receipt.stats.meanScoreDifferenceAMinusB.toFixed(1)} // CI95 [${lastComparison.receipt.stats.meanDifferenceCi95Low.toFixed(1)}, ${lastComparison.receipt.stats.meanDifferenceCi95High.toFixed(1)}] // g ${lastComparison.receipt.stats.hedgesGAMinusB?.toFixed(2) ?? "UNDEFINED"}`
                : `${campaignLedger.length} CAMPAIGN RECEIPTS // A−B, WELCH CI95, HEDGES g`}
            </small>
          </div>
          <div className="suite-strip">
            <span>SUITE REPORT</span>
            <select
              value={selectedSuiteId}
              onChange={(event) => void changeSelectedSuite(event.target.value)}
              disabled={suiteRegistry.length === 0 || suiteReportBusy || suiteComparisonBusy || autodrive?.active}
              aria-label="Benchmark Suite version"
            >
              <option value="">SELECT SUITE</option>
              {suiteRegistry.map((suite) => (
                <option key={suite.id} value={suite.id}>
                  {`V${suite.version} // ${suite.taskCount} TASKS // ${suite.title}`}
                </option>
              ))}
            </select>
            <select
              value={selectedSuiteCohortId ?? ""}
              onChange={(event) => {
                setSelectedSuiteCohortId(event.target.value || null);
                setLastSuiteReport(null);
              }}
              disabled={suiteCandidates.length === 0 || suiteReportBusy || autodrive?.active}
              aria-label="Benchmark Suite cohort"
            >
              <option value="">SELECT COHORT</option>
              {suiteCandidates.map((candidate) => (
                <option key={candidate.cohortId} value={candidate.cohortId}>
                  {`${candidate.model} ${candidate.modelDigest.slice(0, 8)}… // ${candidate.coveredTasks}/${candidate.suiteTaskCount} TASKS // ${candidate.trialsPerTask}× // ${candidate.ready ? "READY" : "INCOMPLETE"}`}
                </option>
              ))}
            </select>
            <button
              onClick={() => void buildSelectedSuiteReport()}
              disabled={
                suiteReportBusy
                || !selectedSuiteCohortId
                || !suiteCandidates.some((candidate) => candidate.cohortId === selectedSuiteCohortId && candidate.ready)
                || autodrive?.active
                || campaignStatus?.active
              }
            >
              {suiteReportBusy ? "VERIFYING..." : "BUILD REPORT"}
            </button>
            <button onClick={() => void refreshSuiteReportCandidates(selectedSuiteId)} disabled={suiteReportBusy || autodrive?.active || !selectedSuiteId}>
              REFRESH
            </button>
            <small>
              {lastSuiteReport
                ? `V${selectedSuite?.version ?? "?"} #${lastSuiteReport.receipt.reportId} // MACRO μ ${lastSuiteReport.receipt.stats.macroMeanScore1000.toFixed(1)} // SUCCESS ${(lastSuiteReport.receipt.stats.overallSuccessRate * 100).toFixed(1)}% // σTASK ${lastSuiteReport.receipt.stats.populationStddevTaskMeanScore1000.toFixed(1)}`
                : `${suiteCandidates.filter((candidate) => candidate.ready).length} READY COHORTS // ${selectedSuite?.taskCount ?? 0} TASKS REQUIRED`}
            </small>
          </div>
          <div className="comparison-strip">
            <span>SUITE COMPARE</span>
            <select
              value={suiteComparisonAId ?? ""}
              onChange={(event) => {
                setSuiteComparisonAId(event.target.value ? Number(event.target.value) : null);
                setLastSuiteComparison(null);
              }}
              disabled={suiteReportLedger.length === 0 || suiteComparisonBusy || autodrive?.active}
              aria-label="Suite Report A"
            >
              <option value="">REPORT A</option>
              {suiteReportLedger.map((entry) => (
                <option key={`suite-a-${entry.reportId}`} value={entry.reportId}>
                  {`#${entry.reportId} ${entry.model} ${entry.modelDigest.slice(0, 8)}… // μ ${entry.macroMeanScore1000.toFixed(1)} // ${(entry.overallSuccessRate * 100).toFixed(1)}%`}
                </option>
              ))}
            </select>
            <select
              value={suiteComparisonBId ?? ""}
              onChange={(event) => {
                setSuiteComparisonBId(event.target.value ? Number(event.target.value) : null);
                setLastSuiteComparison(null);
              }}
              disabled={suiteReportLedger.length === 0 || suiteComparisonBusy || autodrive?.active}
              aria-label="Suite Report B"
            >
              <option value="">REPORT B</option>
              {suiteReportLedger.map((entry) => (
                <option key={`suite-b-${entry.reportId}`} value={entry.reportId}>
                  {`#${entry.reportId} ${entry.model} ${entry.modelDigest.slice(0, 8)}… // μ ${entry.macroMeanScore1000.toFixed(1)} // ${(entry.overallSuccessRate * 100).toFixed(1)}%`}
                </option>
              ))}
            </select>
            <button
              onClick={() => void runSuiteComparison()}
              disabled={
                suiteComparisonBusy
                || suiteComparisonAId === null
                || suiteComparisonBId === null
                || suiteComparisonAId === suiteComparisonBId
                || autodrive?.active
                || campaignStatus?.active
              }
            >
              {suiteComparisonBusy ? "REVERIFYING..." : "COMPARE SUITES"}
            </button>
            <button
              onClick={() => void refreshSuiteReportLedger(selectedSuiteId)}
              disabled={suiteComparisonBusy || autodrive?.active || !selectedSuiteId}
            >
              REFRESH
            </button>
            <small>
              {lastSuiteComparison
                ? `#${lastSuiteComparison.receipt.comparisonId} // ΔMACRO ${lastSuiteComparison.receipt.stats.macroMeanScoreDifferenceAMinusB.toFixed(1)} // ΔSUCCESS ${(lastSuiteComparison.receipt.stats.overallSuccessRateDifferenceAMinusB * 100).toFixed(1)}pp // TASK Δ [${lastSuiteComparison.receipt.stats.minTaskMeanDifferenceAMinusB.toFixed(1)}, ${lastSuiteComparison.receipt.stats.maxTaskMeanDifferenceAMinusB.toFixed(1)}]`
                : `${suiteReportLedger.length} V${selectedSuite?.version ?? "?"} REPORTS // A−B // TASK-PAIRED // NO WINNER BADGE`}
            </small>
          </div>
          <div className="replay-strip">
            <span>REPLAY LEDGER</span>
            <button
              className={replayRecording ? "recording" : ""}
              onClick={replayRecording ? endReplayRecording : beginReplayRecording}
              disabled={!running || autodrive?.active || !session?.runtimeFeatures.exactReplay || (!replayRecording && profile?.fastForward !== 1)}
            >
              {replayRecording ? "STOP + EXPORT" : "REC"}
            </button>
            <button onClick={verifyReplay} disabled={!running || replayRecording || autodrive?.active || !session?.runtimeFeatures.exactReplay || !lastReplay}>VERIFY LAST</button>
            <small>
              {lastReplay
                ? `${lastReplay.receipt.verification?.result?.toUpperCase() ?? "UNVERIFIED"} // ${lastReplay.replaySha256.slice(0, 12)}…`
                : replayRecording
                  ? `${replayActions} ACTIONS / ${replayCheckpoints} CHECKPOINTS`
                  : session && !session.runtimeFeatures.exactReplay
                    ? "EXACT REPLAY UNSUPPORTED BY RUNTIME"
                    : "NO EXPORTED REPLAY"}
            </small>
          </div>

          <div className="session-tools">
            <button onClick={() => queueSystem("save-state", profile?.saveSlot ?? 0)} disabled={!running || replayRecording || autodrive?.active || !session?.runtimeFeatures.stateSnapshots}>SAVE S{profile?.saveSlot ?? 0}</button>
            <button onClick={() => queueSystem("load-state", profile?.saveSlot ?? 0)} disabled={!running || replayRecording || autodrive?.active || !session?.runtimeFeatures.stateSnapshots}>LOAD S{profile?.saveSlot ?? 0}</button>
            <button onClick={() => queueSystem("rewind", 2)} disabled={!running || replayRecording || autodrive?.active || !session?.runtimeFeatures.stateSnapshots}>REWIND 2S</button>
            <button onClick={() => queueSystem("reset")} disabled={!running || replayRecording || autodrive?.active}>RESET</button>
            <button onClick={takeScreenshot} disabled={!running}>SCREENSHOT</button>
            <button onClick={flushBatteryRam} disabled={!running || !session?.runtimeFeatures.persistentSaveData}>FLUSH SRAM</button>
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
            <small>{profile ? (session?.runtimeFeatures.stateSnapshots ? `REWIND ${profile.rewindSeconds}S / EVERY ${profile.rewindIntervalFrames}F` : "REWIND UNSUPPORTED BY RUNTIME") : "PROFILE OFFLINE"}</small>
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
          <div className="panel-title">RUNTIME // RUNG 39</div>
          <dl>
            <div><dt>FRAME</dt><dd>{frameNumber.toString().padStart(6, "0")}</dd></div>
            <div><dt>INPUT QUEUE</dt><dd>{bus.pending.toString().padStart(6, "0")}</dd></div>
            <div><dt>LIBRARY</dt><dd>{games.length.toString().padStart(6, "0")}</dd></div>
            <div><dt>GAMEPADS</dt><dd>{controllers.length.toString().padStart(6, "0")}</dd></div>
            <div><dt>CORE</dt><dd>{running ? "ONLINE" : "STANDBY"}</dd></div>
            <div><dt>ROUTE</dt><dd>{session?.routeEvidence.route.toUpperCase() ?? "NONE"}</dd></div>
            <div><dt>OP APPROVAL</dt><dd>{session?.routeEvidence.sessionOperatorApproved ? "YES" : "NO"}</dd></div>
            <div><dt>STATE SNAP</dt><dd>{session ? (session.runtimeFeatures.stateSnapshots ? "YES" : "NO") : "----"}</dd></div>
            <div><dt>EXACT REPLAY</dt><dd>{session ? (session.runtimeFeatures.exactReplay ? "YES" : "NO") : "----"}</dd></div>
            <div><dt>SAVE DATA</dt><dd>{session ? (session.runtimeFeatures.persistentSaveData ? "YES" : "NO") : "----"}</dd></div>
            <div><dt>AUDIO</dt><dd>{running ? (audioRate ? `${audioRate} HZ` : "SYNC") : "OFFLINE"}</dd></div>
            <div><dt>SPEED</dt><dd>{running ? `${profile?.fastForward ?? 1}×` : "OFFLINE"}</dd></div>
            <div><dt>REWIND</dt><dd>{session?.runtimeFeatures.stateSnapshots ? rewindSnapshots.toString().padStart(6, "0") : "N/A"}</dd></div>
            <div><dt>STATE SLOT</dt><dd>{session?.runtimeFeatures.stateSnapshots ? `S${profile?.saveSlot ?? 0}` : "N/A"}</dd></div>
            <div><dt>GAME HASH</dt><dd>{session ? session.gameKey.slice(0, 8).toUpperCase() : "--------"}</dd></div>
            <div><dt>REG SHA</dt><dd>{session?.routeEvidence.registeredCoreSha256?.slice(0, 8).toUpperCase() ?? "--------"}</dd></div>
            <div><dt>BIN EVIDENCE</dt><dd>{session ? (session.routeEvidence.registrationBinaryEvidenceBound ? "BOUND" : "UNBOUND") : "----"}</dd></div>
            <div><dt>REPLAY</dt><dd>{session && !session.runtimeFeatures.exactReplay ? "UNSUPPORTED" : replayRecording ? "RECORDING" : lastReplay ? "EXPORTED" : "STANDBY"}</dd></div>
            <div><dt>ACTIONS</dt><dd>{replayActions.toString().padStart(6, "0")}</dd></div>
            <div><dt>CHECKPOINTS</dt><dd>{replayCheckpoints.toString().padStart(6, "0")}</dd></div>
            <div><dt>VERIFY</dt><dd>{session && !session.runtimeFeatures.exactReplay ? "N/A" : lastReplay?.receipt.verification?.result.toUpperCase() ?? "UNVERIFIED"}</dd></div>
            <div><dt>AUTHORITY</dt><dd>{authority?.mode.toUpperCase() ?? "OFFLINE"}</dd></div>
            <div><dt>AGENT</dt><dd>{authority?.agentId ?? "NONE"}</dd></div>
            <div><dt>REJECTED</dt><dd>{(authority?.rejectedActions ?? 0).toString().padStart(6, "0")}</dd></div>
            <div><dt>DRIVER TURN</dt><dd>{driverPendingTurnId === null ? "NONE" : driverPendingTurnId}</dd></div>
            <div><dt>DRIVER QUEUE</dt><dd>{driverQueuedActions.toString().padStart(6, "0")}</dd></div>
            <div><dt>OLLAMA</dt><dd>{providerBusy ? "THINK PAUSE" : ollamaOnline ? "ONLINE" : "UNPROBED"}</dd></div>
            <div><dt>MODEL</dt><dd>{settings.ollamaModel ?? "NONE"}</dd></div>
            <div><dt>CAPABILITY</dt><dd>{modelQualification?.details.capabilities.join("+").toUpperCase() || "UNINSPECTED"}</dd></div>
            <div><dt>MODEL DIGEST</dt><dd>{modelQualification?.details.digest.slice(0, 12).toUpperCase() ?? "------------"}</dd></div>
            <div><dt>MODEL QUAL</dt><dd>{modelQualification?.qualified ? "PASS" : modelQualification ? "FAIL / STALE" : "UNTESTED"}</dd></div>
            <div><dt>AUTO RUN</dt><dd>{autodrive?.active ? `#${autodrive.runId} ACTIVE` : autodrive?.stopReason?.toUpperCase() ?? "STANDBY"}</dd></div>
            <div><dt>AUTO TURNS</dt><dd>{autodrive ? `${autodrive.turnsCompleted}/${autodrive.policy.maxTurns}` : "0/0"}</dd></div>
            <div><dt>AUTO ACTIONS</dt><dd>{autodrive ? `${autodrive.totalActions}/${autodrive.policy.maxTotalActions}` : "0/0"}</dd></div>
            <div><dt>NEXT OBS</dt><dd>{autodrive?.active ? `F${autodrive.nextObservationFrame}` : "----"}</dd></div>
            <div><dt>OBS READY</dt><dd>{autodrive?.active ? (frameNumber >= autodrive.nextObservationFrame ? "YES" : `WAIT ${autodrive.nextObservationFrame - frameNumber}F`) : "----"}</dd></div>
            <div><dt>SCHED WAIT</dt><dd>{autodrive ? `${autodrive.totalScheduledCadenceWaitFrames}F` : "0F"}</dd></div>
            <div><dt>SCHED MAX</dt><dd>{autodrive ? `${autodrive.maxScheduledCadenceWaitFrames}F` : "0F"}</dd></div>
            <div><dt>EMPTY STREAK</dt><dd>{autodrive ? autodrive.consecutiveEmptyTurns : 0}</dd></div>
            <div><dt>AUTO RECEIPT</dt><dd>{lastAutodrive ? `RUN ${lastAutodrive.receipt.runId}` : "NONE"}</dd></div>
            <div><dt>BENCH TASK</dt><dd>{session?.benchmarkTask ? session.benchmarkTask.title.toUpperCase() : "NONE"}</dd></div>
            <div><dt>TASK ID</dt><dd>{session?.benchmarkTask?.id ?? "----"}</dd></div>
            <div><dt>SUITES</dt><dd>{session?.benchmarkTask?.suiteIds.join(" + ") ?? "----"}</dd></div>
            <div><dt>TASK CONTROLS</dt><dd>{session?.benchmarkTask?.allowedButtons.join(" + ") ?? "----"}</dd></div>
            <div><dt>BENCH RUN</dt><dd>{benchmarkRunning ? `#${benchmarkRunId} ACTIVE` : lastModelBenchmark ? `#${lastModelBenchmark.receipt.benchmarkRunId}` : "NONE"}</dd></div>
            <div><dt>BENCH SCORE</dt><dd>{lastModelBenchmark?.receipt.score1000 === null || lastModelBenchmark?.receipt.score1000 === undefined ? "----" : `${lastModelBenchmark.receipt.score1000}/1000`}</dd></div>
            <div><dt>BENCH OUTCOME</dt><dd>{lastModelBenchmark ? lastModelBenchmark.receipt.taskSuccess ? "TARGET REACHED" : lastModelBenchmark.receipt.recordStatus : "UNRUN"}</dd></div>
            <div><dt>CAMPAIGN</dt><dd>{campaignStatus?.active ? `#${campaignStatus.campaignId} ${campaignStatus.completedTrials}/${campaignStatus.totalTrials}` : lastCampaign ? `#${lastCampaign.receipt.campaignId} ${lastCampaign.receipt.recordStatus}` : "NONE"}</dd></div>
            <div><dt>SUCCESS RATE</dt><dd>{lastCampaign ? `${(lastCampaign.receipt.stats.successRate * 100).toFixed(1)}%` : "----"}</dd></div>
            <div><dt>MEAN SCORE</dt><dd>{lastCampaign?.receipt.stats.meanScore1000 === null || lastCampaign?.receipt.stats.meanScore1000 === undefined ? "----" : lastCampaign.receipt.stats.meanScore1000.toFixed(1)}</dd></div>
            <div><dt>MEDIAN</dt><dd>{lastCampaign?.receipt.stats.medianScore1000 === null || lastCampaign?.receipt.stats.medianScore1000 === undefined ? "----" : lastCampaign.receipt.stats.medianScore1000.toFixed(1)}</dd></div>
            <div><dt>STDDEV</dt><dd>{lastCampaign?.receipt.stats.populationStddevScore1000 === null || lastCampaign?.receipt.stats.populationStddevScore1000 === undefined ? "----" : lastCampaign.receipt.stats.populationStddevScore1000.toFixed(1)}</dd></div>
            <div><dt>COMPARE</dt><dd>{lastComparison ? `#${lastComparison.receipt.comparisonId} A#${lastComparison.receipt.campaignA.campaignId} / B#${lastComparison.receipt.campaignB.campaignId}` : "NONE"}</dd></div>
            <div><dt>Δ MEAN A−B</dt><dd>{lastComparison ? lastComparison.receipt.stats.meanScoreDifferenceAMinusB.toFixed(1) : "----"}</dd></div>
            <div><dt>CI95 LOW</dt><dd>{lastComparison ? lastComparison.receipt.stats.meanDifferenceCi95Low.toFixed(1) : "----"}</dd></div>
            <div><dt>CI95 HIGH</dt><dd>{lastComparison ? lastComparison.receipt.stats.meanDifferenceCi95High.toFixed(1) : "----"}</dd></div>
            <div><dt>HEDGES g</dt><dd>{lastComparison?.receipt.stats.hedgesGAMinusB === null || lastComparison?.receipt.stats.hedgesGAMinusB === undefined ? "----" : lastComparison.receipt.stats.hedgesGAMinusB.toFixed(2)}</dd></div>
            <div><dt>Δ SUCCESS</dt><dd>{lastComparison ? `${(lastComparison.receipt.stats.successRateDifferenceAMinusB * 100).toFixed(1)}pp` : "----"}</dd></div>
            <div><dt>SUITE COHORTS</dt><dd>{`${suiteCandidates.filter((candidate) => candidate.ready).length}/${suiteCandidates.length}`}</dd></div>
            <div><dt>SUITE REPORT</dt><dd>{lastSuiteReport ? `#${lastSuiteReport.receipt.reportId}` : "NONE"}</dd></div>
            <div><dt>SUITE TASKS</dt><dd>{lastSuiteReport ? `${lastSuiteReport.receipt.stats.taskCount}` : "----"}</dd></div>
            <div><dt>MACRO MEAN</dt><dd>{lastSuiteReport ? lastSuiteReport.receipt.stats.macroMeanScore1000.toFixed(1) : "----"}</dd></div>
            <div><dt>SUITE SUCCESS</dt><dd>{lastSuiteReport ? `${(lastSuiteReport.receipt.stats.overallSuccessRate * 100).toFixed(1)}%` : "----"}</dd></div>
            <div><dt>TASK μ MIN/MAX</dt><dd>{lastSuiteReport ? `${lastSuiteReport.receipt.stats.minTaskMeanScore1000.toFixed(1)} / ${lastSuiteReport.receipt.stats.maxTaskMeanScore1000.toFixed(1)}` : "----"}</dd></div>
            <div><dt>TASK μ σ</dt><dd>{lastSuiteReport ? lastSuiteReport.receipt.stats.populationStddevTaskMeanScore1000.toFixed(1) : "----"}</dd></div>
            <div><dt>SUITE CMP</dt><dd>{lastSuiteComparison ? `#${lastSuiteComparison.receipt.comparisonId}` : "NONE"}</dd></div>
            <div><dt>Δ MACRO</dt><dd>{lastSuiteComparison ? lastSuiteComparison.receipt.stats.macroMeanScoreDifferenceAMinusB.toFixed(1) : "----"}</dd></div>
            <div><dt>Δ SUCCESS</dt><dd>{lastSuiteComparison ? `${(lastSuiteComparison.receipt.stats.overallSuccessRateDifferenceAMinusB * 100).toFixed(1)}pp` : "----"}</dd></div>
            <div><dt>TASK Δ MIN/MAX</dt><dd>{lastSuiteComparison ? `${lastSuiteComparison.receipt.stats.minTaskMeanDifferenceAMinusB.toFixed(1)} / ${lastSuiteComparison.receipt.stats.maxTaskMeanDifferenceAMinusB.toFixed(1)}` : "----"}</dd></div>
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

      <footer>OBSERVE BANKS // ERASE // SELECT CONTEXT // QUERY // TRANSFORM // COMMIT // RECEIPT ROUTING</footer>
    </main>
  );
}
