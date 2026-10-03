use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use phicade_libretro::{CoreIdentity, LibretroCore};
use phicade_runtime::{ActionEnvelope, AudioBuffer, EmulatorCore, FrameBuffer, GameImage, SystemId};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};
use tauri::{AppHandle, Manager, State};

const MAX_LIBRARY_ENTRIES: usize = 4096;

#[derive(Default)]
struct EmulatorState {
    session: Mutex<Option<EmulatorSession>>,
}

struct EmulatorSession {
    core: LibretroCore,
    game_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct AppSettings {
    rom_directory: Option<String>,
    auto_scan: bool,
    controller_deadzone: f32,
    sameboy_core_path: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            rom_directory: None,
            auto_scan: false,
            controller_deadzone: 0.18,
            sameboy_core_path: None,
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionInfo {
    game_path: String,
    core_path: String,
    core: CoreIdentity,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct FramePacket {
    frame: u64,
    width: u32,
    height: u32,
    rgba_base64: String,
    audio_base64: String,
    sample_rate_hz: u32,
    shutdown_requested: bool,
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let directory = app.path().app_config_dir()
        .map_err(|error| format!("cannot resolve app config directory: {error}"))?;
    Ok(directory.join("settings.json"))
}

#[tauri::command]
fn load_settings(app: AppHandle) -> Result<AppSettings, String> {
    let path = settings_path(&app)?;
    if !path.exists() { return Ok(AppSettings::default()); }
    let json = fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    serde_json::from_str(&json)
        .map_err(|error| format!("cannot parse {}: {error}", path.display()))
}

#[tauri::command]
fn save_settings(app: AppHandle, settings: AppSettings) -> Result<(), String> {
    let path = settings_path(&app)?;
    let parent = path.parent().ok_or_else(|| "settings path has no parent directory".to_owned())?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    let json = serde_json::to_string_pretty(&settings)
        .map_err(|error| format!("cannot serialize settings: {error}"))?;
    fs::write(&path, json).map_err(|error| format!("cannot write {}: {error}", path.display()))
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

fn system_id_from_extension(extension: &str) -> Option<SystemId> {
    match extension {
        "gb" => Some(SystemId::GameBoy),
        "gbc" => Some(SystemId::GameBoyColor),
        _ => None,
    }
}

fn entry_from_path(path: &Path) -> Option<RomEntry> {
    let extension = path.extension()?.to_string_lossy().to_ascii_lowercase();
    let system = classify_extension(&extension)?;
    let display_name = path.file_stem()?.to_string_lossy().to_string();
    Some(RomEntry { path: path.to_string_lossy().to_string(), display_name, system, extension })
}

#[tauri::command]
fn scan_rom_directory(path: String) -> Result<Vec<RomEntry>, String> {
    let root = PathBuf::from(&path).canonicalize()
        .map_err(|error| format!("cannot resolve selected directory: {error}"))?;
    if !root.is_dir() { return Err("selected path is not a directory".to_owned()); }
    let entries = fs::read_dir(&root)
        .map_err(|error| format!("cannot read {}: {error}", root.display()))?;
    let mut games = Vec::new();
    for entry in entries.take(MAX_LIBRARY_ENTRIES) {
        let entry = entry.map_err(|error| format!("cannot read directory entry: {error}"))?;
        let candidate = entry.path();
        if candidate.is_file() {
            if let Some(game) = entry_from_path(&candidate) { games.push(game); }
        }
    }
    games.sort_by(|left, right| left.system.cmp(right.system)
        .then_with(|| left.display_name.to_lowercase().cmp(&right.display_name.to_lowercase())));
    Ok(games)
}

#[tauri::command]
fn validate_action_envelope(envelope: ActionEnvelope) -> Result<ActionEnvelope, String> {
    envelope.validate()?;
    Ok(envelope)
}

fn app_emulator_dirs(app: &AppHandle) -> Result<(PathBuf, PathBuf), String> {
    let root = app.path().app_data_dir()
        .map_err(|error| format!("cannot resolve app data directory: {error}"))?;
    let system = root.join("system");
    let saves = root.join("saves");
    fs::create_dir_all(&system).map_err(|error| format!("cannot create {}: {error}", system.display()))?;
    fs::create_dir_all(&saves).map_err(|error| format!("cannot create {}: {error}", saves.display()))?;
    Ok((system, saves))
}

#[tauri::command]
fn start_emulation(
    app: AppHandle,
    state: State<'_, EmulatorState>,
    core_path: String,
    game_path: String,
) -> Result<SessionInfo, String> {
    let game = PathBuf::from(&game_path).canonicalize()
        .map_err(|error| format!("cannot resolve selected game: {error}"))?;
    let core_file = PathBuf::from(&core_path).canonicalize()
        .map_err(|error| format!("cannot resolve selected core: {error}"))?;

    let extension = game.extension().and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| "selected game has no supported extension".to_owned())?;
    let system = system_id_from_extension(&extension)
        .ok_or_else(|| "Rung 3 SameBoy sessions accept .gb or .gbc images only".to_owned())?;
    let display_name = game.file_stem()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| "Game Boy image".to_owned());

    {
        let mut session = state.session.lock().map_err(|_| "emulator session lock poisoned".to_owned())?;
        *session = None;
    }

    let (system_dir, save_dir) = app_emulator_dirs(&app)?;
    let mut core = LibretroCore::open(&core_file, &system_dir, &save_dir)
        .map_err(|error| format!("cannot open libretro core: {error:?}"))?;
    let identity = core.identity().clone();
    if !identity.library_name.to_ascii_lowercase().contains("sameboy") {
        return Err(format!("Rung 3 qualification expects SameBoy, got {} {}", identity.library_name, identity.library_version));
    }

    core.load_game(&GameImage::new(&game, system, display_name))
        .map_err(|error| format!("cannot load game image: {error:?}"))?;

    let info = SessionInfo {
        game_path: game.to_string_lossy().to_string(),
        core_path: core_file.to_string_lossy().to_string(),
        core: identity,
    };
    let mut session = state.session.lock().map_err(|_| "emulator session lock poisoned".to_owned())?;
    *session = Some(EmulatorSession { core, game_path: info.game_path.clone() });
    Ok(info)
}

#[tauri::command]
fn step_emulation(
    state: State<'_, EmulatorState>,
    actions: Vec<ActionEnvelope>,
) -> Result<FramePacket, String> {
    let mut session = state.session.lock().map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session.as_mut().ok_or_else(|| "no emulator session is running".to_owned())?;
    let mut video = FrameBuffer::default();
    let mut audio = AudioBuffer::default();
    session.core.step_frame(&actions, &mut video, &mut audio)
        .map_err(|error| format!("core frame failed: {error:?}"))?;

    let mut audio_bytes = Vec::with_capacity(audio.interleaved_stereo_f32.len() * 2);
    for sample in audio.interleaved_stereo_f32 {
        let quantized = (sample.clamp(-1.0, 1.0) * 32767.0).round() as i16;
        audio_bytes.extend_from_slice(&quantized.to_le_bytes());
    }

    Ok(FramePacket {
        frame: session.core.frame_count(),
        width: video.width,
        height: video.height,
        rgba_base64: BASE64.encode(video.rgba8),
        audio_base64: BASE64.encode(audio_bytes),
        sample_rate_hz: audio.sample_rate_hz,
        shutdown_requested: session.core.shutdown_requested(),
    })
}

#[tauri::command]
fn stop_emulation(state: State<'_, EmulatorState>) -> Result<(), String> {
    let mut session = state.session.lock().map_err(|_| "emulator session lock poisoned".to_owned())?;
    *session = None;
    Ok(())
}

#[tauri::command]
fn running_game(state: State<'_, EmulatorState>) -> Result<Option<String>, String> {
    let session = state.session.lock().map_err(|_| "emulator session lock poisoned".to_owned())?;
    Ok(session.as_ref().map(|value| value.game_path.clone()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(EmulatorState::default())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            load_settings, save_settings, scan_rom_directory, validate_action_envelope,
            start_emulation, step_emulation, stop_emulation, running_game
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
    #[test]
    fn rung_three_maps_game_boy_extensions() {
        assert_eq!(system_id_from_extension("gb"), Some(SystemId::GameBoy));
        assert_eq!(system_id_from_extension("gbc"), Some(SystemId::GameBoyColor));
        assert_eq!(system_id_from_extension("gba"), None);
    }
}
