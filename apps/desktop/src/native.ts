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
