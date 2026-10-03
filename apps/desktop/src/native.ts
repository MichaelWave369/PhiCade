import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { ActionEnvelope, GameAction } from "./actionBus";

export interface AppSettings {
  romDirectory: string | null;
  autoScan: boolean;
  controllerDeadzone: number;
  sameboyCorePath: string | null;
  ollamaBaseUrl: string;
  ollamaModel: string | null;
}

export interface RomEntry {
  path: string;
  displayName: string;
  system: string;
  extension: string;
}

export interface CoreIdentity {
  libraryName: string;
  libraryVersion: string;
  validExtensions: string;
  needFullpath: boolean;
  fps: number;
  sampleRateHz: number;
}

export interface GameProfile {
  fastForward: 1 | 2 | 4;
  rewindSeconds: number;
  rewindIntervalFrames: number;
  saveSlot: number;
}

export interface SessionInfo {
  gamePath: string;
  gameKey: string;
  corePath: string;
  core: CoreIdentity;
  profile: GameProfile;
}

export interface ReplayCheckpoint {
  frame: number;
  stateSha256: string;
  frameSha256: string;
  inputMask: number;
}

export type ReplayVerificationResult = "pass" | "diverged";

export interface ReplayVerification {
  result: ReplayVerificationResult;
  checkedCheckpoints: number;
  firstDivergenceFrame: number | null;
  expectedStateSha256: string | null;
  actualStateSha256: string | null;
  expectedFrameSha256: string | null;
  actualFrameSha256: string | null;
}

export interface ReplayReceipt {
  schema: string;
  replaySha256: string;
  gameSha256: string;
  coreName: string;
  coreVersion: string;
  coreSha256: string;
  startFrame: number;
  endFrame: number;
  actionCount: number;
  checkpointCount: number;
  verification: ReplayVerification | null;
}

export interface ReplayStatus {
  recording: boolean;
  actionCount: number;
  checkpointCount: number;
  lastReplayPath: string | null;
  lastReceiptPath: string | null;
  lastReplaySha256: string | null;
}

export interface ReplayArtifact {
  replayPath: string;
  receiptPath: string;
  replaySha256: string;
  receipt: ReplayReceipt;
}

export type ControlMode = "human" | "phi-bot" | "coop" | "versus";

export interface AuthorityStatus {
  mode: ControlMode;
  playablePorts: number;
  agentId: string | null;
  agentSeat: number | null;
  allowedButtons: string[];
  allowedAxes: string[];
  expiresAtFrame: number | null;
  rejectedActions: number;
  lastReason: string | null;
}

export interface PhiBotObservation {
  schema: string;
  frame: number;
  width: number;
  height: number;
  rgbaBase64: string;
  frameSha256: string;
  inputMask: number;
  gameSha256: string;
  coreName: string;
  coreVersion: string;
  agentId: string;
  seat: number;
  controlMode: string;
  allowedButtons: string[];
  allowedAxes: string[];
  expiresAtFrame: number | null;
}


export interface AgentTurnAction {
  delayFrames: number;
  action: GameAction;
}

export interface AgentTurnRequest {
  schema: string;
  turnId: number;
  observation: PhiBotObservation;
  maxActions: number;
  maxDelayFrames: number;
  validUntilFrame: number;
}

export interface AgentTurnResponse {
  schema: string;
  turnId: number;
  agentId: string;
  seat: number;
  observationFrame: number;
  observationSha256: string;
  actions: AgentTurnAction[];
}

export interface DriverStatus {
  pendingTurnId: number | null;
  queuedActions: number;
  nextTurnId: number;
}

export interface AutodrivePolicy {
  maxTurns: number;
  maxTotalActions: number;
  maxConsecutiveEmptyTurns: number;
  maxEmulatedFrames: number;
}

export type AutodriveStopReason =
  | "operator-stop"
  | "human-takeover"
  | "turn-budget"
  | "action-budget"
  | "frame-budget"
  | "empty-turn-limit"
  | "task-success"
  | "provider-failure"
  | "grant-expired"
  | "core-shutdown";

export interface AutodriveStatus {
  schema: string;
  runId: number;
  active: boolean;
  provider: string;
  model: string;
  startedFrame: number;
  currentFrame: number;
  turnsIssued: number;
  turnsCompleted: number;
  totalActions: number;
  consecutiveEmptyTurns: number;
  policy: AutodrivePolicy;
  stopReason: AutodriveStopReason | null;
}

export interface AutodriveReceipt {
  schema: string;
  runId: number;
  provider: string;
  model: string;
  gameSha256: string;
  coreName: string;
  coreVersion: string;
  startedFrame: number;
  endedFrame: number;
  turnsIssued: number;
  turnsCompleted: number;
  totalActions: number;
  stopReason: AutodriveStopReason;
  finalFrameSha256: string;
  policy: AutodrivePolicy;
}

export interface AutodriveArtifact {
  receiptPath: string;
  receipt: AutodriveReceipt;
}


export interface PixelPoint {
  x: number;
  y: number;
}

export interface ModelGameplayBenchmarkReceipt {
  schema: string;
  recordStatus: "COMPLETE" | "SCORING_ERROR";
  benchmarkId: string;
  benchmarkRunId: number;
  provider: string;
  model: string;
  modelDigest: string;
  modelQualificationSha256: string;
  gymSourceSha256: string;
  gymRomSha256: string;
  coreSha256: string;
  coreName: string;
  coreVersion: string;
  autodriveReceiptSha256: string;
  autodriveRunId: number;
  policy: AutodrivePolicy;
  stopReason: AutodriveStopReason;
  startedFrame: number;
  endedFrame: number;
  startPlayer: PixelPoint;
  finalPlayer: PixelPoint | null;
  target: PixelPoint;
  initialDistance: number;
  finalDistance: number | null;
  progress: number | null;
  score1000: number | null;
  taskSuccess: boolean;
  turnsIssued: number;
  turnsCompleted: number;
  totalActions: number;
  finalFrameSha256: string;
  scoringError: string | null;
}

export interface ModelBenchmarkArtifact {
  receiptPath: string;
  receipt: ModelGameplayBenchmarkReceipt;
}

export interface ModelBenchmarkStart {
  benchmarkRunId: number;
  autodrive: AutodriveStatus;
}


export interface CampaignTrialEvidence {
  benchmarkRunId: number;
  receiptSha256: string;
  recordStatus: string;
  score1000: number | null;
  taskSuccess: boolean;
  stopReason: AutodriveStopReason;
}

export interface BenchmarkCampaignStats {
  observedTrials: number;
  scoredTrials: number;
  scoringErrorTrials: number;
  successfulTrials: number;
  successRate: number;
  meanScore1000: number | null;
  medianScore1000: number | null;
  minScore1000: number | null;
  maxScore1000: number | null;
  populationStddevScore1000: number | null;
}

export interface BenchmarkCampaignReceipt {
  schema: string;
  recordStatus: "COMPLETE" | "PARTIAL";
  campaignId: number;
  benchmarkId: string;
  provider: string;
  model: string;
  modelDigest: string;
  modelQualificationSha256: string;
  gymSourceSha256: string;
  gymRomSha256: string;
  coreSha256: string;
  coreName: string;
  coreVersion: string;
  policy: AutodrivePolicy;
  totalTrials: number;
  completedTrials: number;
  trials: CampaignTrialEvidence[];
  stats: BenchmarkCampaignStats;
}

export interface BenchmarkCampaignArtifact {
  receiptPath: string;
  receipt: BenchmarkCampaignReceipt;
}

export interface BenchmarkCampaignStatus {
  schema: string;
  campaignId: number;
  active: boolean;
  totalTrials: number;
  completedTrials: number;
  provider: string;
  model: string;
  modelDigest: string;
  policy: AutodrivePolicy;
}

export interface BenchmarkCampaignStart {
  status: BenchmarkCampaignStatus;
  benchmark: ModelBenchmarkStart;
}


export interface CampaignListEntry {
  campaignId: number;
  receiptSha256: string;
  recordStatus: string;
  model: string;
  modelDigest: string;
  totalTrials: number;
  completedTrials: number;
  meanScore1000: number | null;
  successRate: number;
}

export interface CampaignSampleSummary {
  scoredTrials: number;
  successfulTrials: number;
  observedTrials: number;
  meanScore1000: number;
  sampleVarianceScore1000: number;
  successRate: number;
}

export interface CampaignComparisonStats {
  sampleA: CampaignSampleSummary;
  sampleB: CampaignSampleSummary;
  meanScoreDifferenceAMinusB: number;
  welchStandardError: number;
  welchDegreesOfFreedom: number | null;
  meanDifferenceCi95Low: number;
  meanDifferenceCi95High: number;
  hedgesGAMinusB: number | null;
  successRateDifferenceAMinusB: number;
}

export interface ComparisonCampaignRef {
  campaignId: number;
  receiptSha256: string;
  provider: string;
  model: string;
  modelDigest: string;
  modelQualificationSha256: string;
  completedTrials: number;
  stats: BenchmarkCampaignStats;
}

export interface CampaignComparisonReceipt {
  schema: string;
  recordStatus: "COMPLETE";
  comparisonId: number;
  benchmarkId: string;
  gymSourceSha256: string;
  gymRomSha256: string;
  coreSha256: string;
  coreName: string;
  coreVersion: string;
  policy: AutodrivePolicy;
  totalTrials: number;
  campaignA: ComparisonCampaignRef;
  campaignB: ComparisonCampaignRef;
  stats: CampaignComparisonStats;
}

export interface CampaignComparisonArtifact {
  receiptPath: string;
  receipt: CampaignComparisonReceipt;
}

export const defaultAutodrivePolicy: AutodrivePolicy = {
  maxTurns: 32,
  maxTotalActions: 128,
  maxConsecutiveEmptyTurns: 4,
  maxEmulatedFrames: 3600,
};

export interface OllamaModel {
  name: string;
  model: string;
  size: number;
  digest: string;
}

export interface OllamaModelDetails {
  name: string;
  digest: string;
  capabilities: string[];
  family: string | null;
  parameterSize: string | null;
  quantizationLevel: string | null;
}

export interface OllamaQualificationReceipt {
  schema: string;
  result: "PASS" | "FAIL";
  provider: "ollama";
  model: string;
  digest: string;
  capabilities: string[];
  visionAdvertised: boolean;
  structuredOutputPass: boolean;
  visionProbePass: boolean;
  probeExpected: string;
  probeObserved: string | null;
  totalDurationNs: number | null;
  evalCount: number | null;
  error: string | null;
}

export interface OllamaQualificationArtifact {
  receiptPath: string;
  receipt: OllamaQualificationReceipt;
}

export interface OllamaQualificationStatus {
  details: OllamaModelDetails;
  qualified: boolean;
  receipt: OllamaQualificationReceipt | null;
  receiptPath: string | null;
}

export interface OllamaTurnResult {
  provider: "ollama";
  model: string;
  response: AgentTurnResponse;
  totalDurationNs: number | null;
  evalCount: number | null;
}

export interface FramePacket {
  frame: number;
  width: number;
  height: number;
  rgbaBase64: string;
  audioBase64: string;
  sampleRateHz: number;
  shutdownRequested: boolean;
  rewindSnapshots: number;
  fastForward: number;
  replayRecording: boolean;
  replayActions: number;
  replayCheckpoints: number;
  controlMode: ControlMode;
  authorityRejections: number;
  lastAuthorityReason: string | null;
  driverPendingTurnId: number | null;
  driverQueuedActions: number;
  autodrive: AutodriveStatus | null;
}

export const defaultSettings: AppSettings = {
  romDirectory: null,
  autoScan: false,
  controllerDeadzone: 0.18,
  sameboyCorePath: null,
  ollamaBaseUrl: "http://127.0.0.1:11434",
  ollamaModel: null,
};

export function isNativeShell(): boolean {
  return "__TAURI_INTERNALS__" in window;
}

export async function selectRomDirectory(): Promise<string | null> {
  if (!isNativeShell()) return null;
  const selected = await open({
    directory: true,
    multiple: false,
    title: "Select a PhiCade ROM directory",
  });
  return typeof selected === "string" ? selected : null;
}

export async function selectSameBoyCore(): Promise<string | null> {
  if (!isNativeShell()) return null;
  const selected = await open({
    directory: false,
    multiple: false,
    title: "Select SameBoy libretro core",
    filters: [{ name: "Libretro core", extensions: ["dll", "so", "dylib"] }],
  });
  return typeof selected === "string" ? selected : null;
}

export async function scanRomDirectory(path: string): Promise<RomEntry[]> {
  return invoke<RomEntry[]>("scan_rom_directory", { path });
}

export async function loadSettings(): Promise<AppSettings> {
  if (!isNativeShell()) return defaultSettings;
  return invoke<AppSettings>("load_settings");
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  if (!isNativeShell()) return;
  await invoke("save_settings", { settings });
}

export async function validateActionEnvelope(
  envelope: ActionEnvelope,
): Promise<ActionEnvelope> {
  if (!isNativeShell()) return envelope;
  return invoke<ActionEnvelope>("validate_action_envelope", { envelope });
}

export async function startEmulation(
  corePath: string,
  gamePath: string,
): Promise<SessionInfo> {
  return invoke<SessionInfo>("start_emulation", { corePath, gamePath });
}

export async function stepEmulation(
  actions: ActionEnvelope[],
): Promise<FramePacket> {
  return invoke<FramePacket>("step_emulation", { actions });
}

export async function setGameProfile(
  profile: GameProfile,
): Promise<GameProfile> {
  return invoke<GameProfile>("set_game_profile", { profile });
}

export async function captureScreenshot(): Promise<string> {
  return invoke<string>("capture_screenshot");
}

export async function flushGameSave(): Promise<void> {
  return invoke("flush_game_save");
}

export async function stopEmulation(): Promise<void> {
  if (!isNativeShell()) return;
  await invoke("stop_emulation");
}


export async function getReplayStatus(): Promise<ReplayStatus> {
  return invoke<ReplayStatus>("replay_status");
}

export async function startReplayRecording(): Promise<ReplayStatus> {
  return invoke<ReplayStatus>("start_replay_recording");
}

export async function stopReplayRecording(): Promise<ReplayArtifact> {
  return invoke<ReplayArtifact>("stop_replay_recording");
}

export async function verifyLastReplay(): Promise<ReplayArtifact> {
  return invoke<ReplayArtifact>("verify_last_replay");
}


export async function getAuthorityStatus(): Promise<AuthorityStatus> {
  return invoke<AuthorityStatus>("authority_status");
}

export async function setControlMode(
  mode: ControlMode,
  agentId: string | null = null,
  allowedButtons: string[] | null = null,
  grantFrames: number | null = null,
): Promise<AuthorityStatus> {
  return invoke<AuthorityStatus>("set_control_mode", {
    mode,
    agentId,
    allowedButtons,
    grantFrames,
  });
}

export async function observePhiBot(
  agentId: string,
  seat = 1,
): Promise<PhiBotObservation> {
  return invoke<PhiBotObservation>("phi_bot_observation", { agentId, seat });
}


export async function getDriverStatus(): Promise<DriverStatus> {
  return invoke<DriverStatus>("driver_status");
}

export async function issueAgentTurn(
  agentId: string,
  seat = 1,
): Promise<AgentTurnRequest> {
  return invoke<AgentTurnRequest>("issue_agent_turn", { agentId, seat });
}

export async function submitAgentTurn(
  response: AgentTurnResponse,
): Promise<DriverStatus> {
  return invoke<DriverStatus>("submit_agent_turn", { response });
}


export async function cancelAgentTurn(): Promise<DriverStatus> {
  return invoke<DriverStatus>("cancel_agent_turn");
}

export async function listOllamaModels(
  baseUrl: string,
): Promise<OllamaModel[]> {
  return invoke<OllamaModel[]>("list_ollama_models", { baseUrl });
}

export async function inspectOllamaModel(
  baseUrl: string,
  model: string,
): Promise<OllamaModelDetails> {
  return invoke<OllamaModelDetails>("inspect_ollama_model", { baseUrl, model });
}

export async function qualifyOllamaModel(
  baseUrl: string,
  model: string,
): Promise<OllamaQualificationArtifact> {
  return invoke<OllamaQualificationArtifact>("qualify_ollama_model", { baseUrl, model });
}

export async function getOllamaQualificationStatus(
  baseUrl: string,
  model: string,
): Promise<OllamaQualificationStatus> {
  return invoke<OllamaQualificationStatus>("ollama_qualification_status", { baseUrl, model });
}

export async function completeOllamaTurn(
  request: AgentTurnRequest,
  baseUrl: string,
  model: string,
): Promise<OllamaTurnResult> {
  return invoke<OllamaTurnResult>("complete_ollama_turn", {
    request,
    baseUrl,
    model,
  });
}


export async function startAutodrive(
  provider: string,
  model: string,
  modelDigest: string,
  policy: AutodrivePolicy = defaultAutodrivePolicy,
): Promise<AutodriveStatus> {
  return invoke<AutodriveStatus>("start_autodrive", {
    provider,
    model,
    modelDigest,
    policy,
  });
}

export async function getAutodriveStatus(): Promise<AutodriveStatus | null> {
  return invoke<AutodriveStatus | null>("autodrive_status");
}

export async function stopAutodrive(): Promise<AutodriveArtifact> {
  return invoke<AutodriveArtifact>("stop_autodrive");
}

export async function failAutodriveProvider(): Promise<AutodriveArtifact> {
  return invoke<AutodriveArtifact>("fail_autodrive_provider");
}

export async function getLastAutodriveReceipt(): Promise<AutodriveArtifact | null> {
  return invoke<AutodriveArtifact | null>("last_autodrive_receipt");
}


export async function startModelGameplayBenchmark(
  provider: string,
  model: string,
  modelDigest: string,
  policy: AutodrivePolicy = defaultAutodrivePolicy,
): Promise<ModelBenchmarkStart> {
  return invoke<ModelBenchmarkStart>("start_model_gameplay_benchmark", {
    provider,
    model,
    modelDigest,
    policy,
  });
}

export async function getLastModelGameplayBenchmark(): Promise<ModelBenchmarkArtifact | null> {
  return invoke<ModelBenchmarkArtifact | null>("last_model_gameplay_benchmark");
}


export async function startBenchmarkCampaign(
  baseUrl: string,
  provider: string,
  model: string,
  modelDigest: string,
  policy: AutodrivePolicy = defaultAutodrivePolicy,
  totalTrials = 5,
): Promise<BenchmarkCampaignStart> {
  return invoke<BenchmarkCampaignStart>("start_benchmark_campaign", {
    baseUrl,
    provider,
    model,
    modelDigest,
    policy,
    totalTrials,
  });
}

export async function continueBenchmarkCampaign(
  baseUrl: string,
): Promise<BenchmarkCampaignStart> {
  return invoke<BenchmarkCampaignStart>("continue_benchmark_campaign", { baseUrl });
}

export async function getBenchmarkCampaignStatus(): Promise<BenchmarkCampaignStatus | null> {
  return invoke<BenchmarkCampaignStatus | null>("benchmark_campaign_status");
}

export async function getLastBenchmarkCampaignReceipt(): Promise<BenchmarkCampaignArtifact | null> {
  return invoke<BenchmarkCampaignArtifact | null>("last_benchmark_campaign_receipt");
}

export async function cancelBenchmarkCampaign(): Promise<BenchmarkCampaignArtifact> {
  return invoke<BenchmarkCampaignArtifact>("cancel_benchmark_campaign");
}


export async function listBenchmarkCampaignReceipts(): Promise<CampaignListEntry[]> {
  return invoke<CampaignListEntry[]>("list_benchmark_campaign_receipts");
}

export async function compareBenchmarkCampaigns(
  campaignAId: number,
  campaignBId: number,
): Promise<CampaignComparisonArtifact> {
  return invoke<CampaignComparisonArtifact>("compare_benchmark_campaigns", {
    campaignAId,
    campaignBId,
  });
}
