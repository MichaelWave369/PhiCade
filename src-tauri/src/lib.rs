use phicade_runtime::ActionEnvelope;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tauri::{AppHandle, Manager};

const MAX_LIBRARY_ENTRIES: usize = 4096;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppSettings {
    rom_directory: Option<String>,
    auto_scan: bool,
    controller_deadzone: f32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            rom_directory: None,
            auto_scan: false,
            controller_deadzone: 0.18,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RomEntry {
    path: String,
    display_name: String,
    system: &'static str,
    extension: String,
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let directory = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("cannot resolve app config directory: {error}"))?;

    Ok(directory.join("settings.json"))
}

#[tauri::command]
fn load_settings(app: AppHandle) -> Result<AppSettings, String> {
    let path = settings_path(&app)?;
    if !path.exists() {
        return Ok(AppSettings::default());
    }

    let json = fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;

    serde_json::from_str(&json)
        .map_err(|error| format!("cannot parse {}: {error}", path.display()))
}

#[tauri::command]
fn save_settings(app: AppHandle, settings: AppSettings) -> Result<(), String> {
    let path = settings_path(&app)?;
    let parent = path
        .parent()
        .ok_or_else(|| "settings path has no parent directory".to_owned())?;

    fs::create_dir_all(parent)
        .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;

    let json = serde_json::to_string_pretty(&settings)
        .map_err(|error| format!("cannot serialize settings: {error}"))?;

    fs::write(&path, json)
        .map_err(|error| format!("cannot write {}: {error}", path.display()))
}

fn classify_extension(extension: &str) -> Option<&'static str> {
    match extension {
        "nes" => Some("NES"),
        "sfc" | "smc" => Some("SNES"),
        "gb" => Some("GB"),
        "gbc" => Some("GBC"),
        "gba" => Some("GBA"),
        "md" | "gen" | "32x" => Some("GENESIS"),
        "cue" | "chd" | "pbp" => Some("PS1"),
        _ => None,
    }
}

fn entry_from_path(path: &Path) -> Option<RomEntry> {
    let extension = path
        .extension()?
        .to_string_lossy()
        .to_ascii_lowercase();
    let system = classify_extension(&extension)?;
    let display_name = path.file_stem()?.to_string_lossy().to_string();

    Some(RomEntry {
        path: path.to_string_lossy().to_string(),
        display_name,
        system,
        extension,
    })
}

#[tauri::command]
fn scan_rom_directory(path: String) -> Result<Vec<RomEntry>, String> {
    let root = PathBuf::from(&path)
        .canonicalize()
        .map_err(|error| format!("cannot resolve selected directory: {error}"))?;

    if !root.is_dir() {
        return Err("selected path is not a directory".to_owned());
    }

    let entries = fs::read_dir(&root)
        .map_err(|error| format!("cannot read {}: {error}", root.display()))?;

    let mut games = Vec::new();

    for entry in entries.take(MAX_LIBRARY_ENTRIES) {
        let entry = entry.map_err(|error| format!("cannot read directory entry: {error}"))?;
        let candidate = entry.path();

        if candidate.is_file() {
            if let Some(game) = entry_from_path(&candidate) {
                games.push(game);
            }
        }
    }

    games.sort_by(|left, right| {
        left.system
            .cmp(right.system)
            .then_with(|| left.display_name.to_lowercase().cmp(&right.display_name.to_lowercase()))
    });

    Ok(games)
}

#[tauri::command]
fn validate_action_envelope(envelope: ActionEnvelope) -> Result<ActionEnvelope, String> {
    envelope.validate()?;
    Ok(envelope)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            load_settings,
            save_settings,
            scan_rom_directory,
            validate_action_envelope
        ])
        .run(tauri::generate_context!())
        .expect("error while running PhiCade");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifier_accepts_known_cartridge_formats() {
        assert_eq!(classify_extension("nes"), Some("NES"));
        assert_eq!(classify_extension("sfc"), Some("SNES"));
        assert_eq!(classify_extension("gba"), Some("GBA"));
        assert_eq!(classify_extension("md"), Some("GENESIS"));
    }

    #[test]
    fn classifier_rejects_unknown_files() {
        assert_eq!(classify_extension("txt"), None);
        assert_eq!(classify_extension("exe"), None);
        assert_eq!(classify_extension("zip"), None);
    }
}
