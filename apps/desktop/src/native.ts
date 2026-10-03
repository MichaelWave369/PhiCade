import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { ActionEnvelope } from "./actionBus";

export interface AppSettings {
  romDirectory: string | null;
  autoScan: boolean;
  controllerDeadzone: number;
  sameboyCorePath: string | null;
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
}

export const defaultSettings: AppSettings = {
  romDirectory: null,
  autoScan: false,
  controllerDeadzone: 0.18,
  sameboyCorePath: null,
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
