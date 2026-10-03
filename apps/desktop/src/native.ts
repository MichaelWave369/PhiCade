import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { ActionEnvelope } from "./actionBus";

export interface AppSettings {
  romDirectory: string | null;
  autoScan: boolean;
  controllerDeadzone: number;
}

export interface RomEntry {
  path: string;
  displayName: string;
  system: string;
  extension: string;
}

export const defaultSettings: AppSettings = {
  romDirectory: null,
  autoScan: false,
  controllerDeadzone: 0.18,
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
