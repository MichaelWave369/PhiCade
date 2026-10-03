use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use phicade_libretro::{CoreIdentity, LibretroCore};
use phicade_runtime::{
    ActionEnvelope, ActionKind, ActionSource, AudioBuffer, EmulatorCore, FrameBuffer, GameImage,
    ReplayCheckpoint, ReplayLedger, ReplayReceipt, ReplayVerification, ReplayVerificationResult,
    SystemCommand, SystemId, REPLAY_RECEIPT_SCHEMA, REPLAY_SCHEMA,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::VecDeque,
    fs,
    io::BufWriter,
    path::{Path, PathBuf},
    sync::Mutex,
};
use tauri::{AppHandle, Manager, State};

const MAX_LIBRARY_ENTRIES: usize = 4096;
const WEB_AUDIO_SAMPLE_RATE_HZ: u32 = 48_000;
const SRAM_FLUSH_INTERVAL_FRAMES: u64 = 300;
const REPLAY_CHECKPOINT_INTERVAL_FRAMES: u64 = 60;
const STATE_MAGIC: &[u8] = b"PHICADE_STATE_V1\0";

#[derive(Default)]
struct EmulatorState {
    session: Mutex<Option<EmulatorSession>>,
}

#[derive(Debug, Clone)]
struct StateSnapshot {
    frame: u64,
    bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
struct SessionPaths {
    save_ram: PathBuf,
    state_dir: PathBuf,
    screenshot_dir: PathBuf,
    replay_dir: PathBuf,
    profile: PathBuf,
}

#[derive(Debug, Clone)]
struct ReplayRecording {
    ledger: ReplayLedger,
    next_checkpoint_frame: u64,
}

#[derive(Debug, Clone)]
struct ReplayExport {
    replay_path: PathBuf,
    receipt_path: PathBuf,
    replay_sha256: String,
    receipt: ReplayReceipt,
}

struct EmulatorSession {
    core: LibretroCore,
    game_path: String,
    game_key: String,
    profile: GameProfile,
    paths: SessionPaths,
    rewind: VecDeque<StateSnapshot>,
    next_rewind_frame: u64,
    last_sram_flush_frame: u64,
    last_frame: FrameBuffer,
    recording: Option<ReplayRecording>,
    last_replay: Option<ReplayExport>,
}

impl Drop for EmulatorSession {
    fn drop(&mut self) {
        let _ = flush_save_ram(self);
    }
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
struct GameProfile {
    fast_forward: u8,
    rewind_seconds: u16,
    rewind_interval_frames: u16,
    save_slot: u8,
}

impl Default for GameProfile {
    fn default() -> Self {
        Self {
            fast_forward: 1,
            rewind_seconds: 10,
            rewind_interval_frames: 30,
            save_slot: 0,
        }
    }
}

impl GameProfile {
    fn validate(&self) -> Result<(), String> {
        if !matches!(self.fast_forward, 1 | 2 | 4) {
            return Err("fast-forward multiplier must be 1, 2, or 4".into());
        }
        if !(2..=60).contains(&self.rewind_seconds) {
            return Err("rewindSeconds must be in 2..=60".into());
        }
        if !(10..=120).contains(&self.rewind_interval_frames) {
            return Err("rewindIntervalFrames must be in 10..=120".into());
        }
        if self.save_slot > 9 {
            return Err("saveSlot must be in 0..=9".into());
        }
        Ok(())
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
    game_key: String,
    core_path: String,
    core: CoreIdentity,
    profile: GameProfile,
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
    rewind_snapshots: usize,
    fast_forward: u8,
    replay_recording: bool,
    replay_actions: usize,
    replay_checkpoints: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReplayStatus {
    recording: bool,
    action_count: usize,
    checkpoint_count: usize,
    last_replay_path: Option<String>,
    last_receipt_path: Option<String>,
    last_replay_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReplayArtifact {
    replay_path: String,
    receipt_path: String,
    replay_sha256: String,
    receipt: ReplayReceipt,
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

fn app_runtime_dirs(app: &AppHandle) -> Result<(PathBuf, PathBuf, PathBuf), String> {
    let root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("cannot resolve app data directory: {error}"))?;
    let system = root.join("system");
    let saves = root.join("saves");
    fs::create_dir_all(&system)
        .map_err(|error| format!("cannot create {}: {error}", system.display()))?;
    fs::create_dir_all(&saves)
        .map_err(|error| format!("cannot create {}: {error}", saves.display()))?;
    Ok((root, system, saves))
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path)
        .map_err(|error| format!("cannot hash {}: {error}", path.display()))?;
    Ok(sha256_bytes(&bytes))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn sanitize_component(value: &str) -> String {
    let cleaned: String = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                character
            } else {
                '-'
            }
        })
        .collect();
    let trimmed = cleaned.trim_matches('-');
    if trimmed.is_empty() {
        "unknown".to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn session_paths(
    root: &Path,
    game_key: &str,
    identity: &CoreIdentity,
) -> Result<SessionPaths, String> {
    let core_namespace = format!(
        "{}-{}",
        sanitize_component(&identity.library_name),
        sanitize_component(&identity.library_version)
    );
    let state_dir = root.join("states").join(game_key).join(core_namespace);
    let screenshot_dir = root.join("screenshots").join(game_key);
    let profile_dir = root.join("profiles");
    let replay_dir = root.join("replays").join(game_key);

    fs::create_dir_all(&state_dir)
        .map_err(|error| format!("cannot create {}: {error}", state_dir.display()))?;
    fs::create_dir_all(&screenshot_dir)
        .map_err(|error| format!("cannot create {}: {error}", screenshot_dir.display()))?;
    fs::create_dir_all(&profile_dir)
        .map_err(|error| format!("cannot create {}: {error}", profile_dir.display()))?;
    fs::create_dir_all(&replay_dir)
        .map_err(|error| format!("cannot create {}: {error}", replay_dir.display()))?;

    Ok(SessionPaths {
        save_ram: root.join("saves").join(format!("{game_key}.srm")),
        state_dir,
        screenshot_dir,
        replay_dir,
        profile: profile_dir.join(format!("{game_key}.json")),
    })
}

fn load_game_profile(path: &Path) -> Result<GameProfile, String> {
    if !path.exists() {
        return Ok(GameProfile::default());
    }
    let json = fs::read_to_string(path)
        .map_err(|error| format!("cannot read profile {}: {error}", path.display()))?;
    let profile: GameProfile = serde_json::from_str(&json)
        .map_err(|error| format!("cannot parse profile {}: {error}", path.display()))?;
    profile.validate()?;
    Ok(profile)
}

fn persist_game_profile(path: &Path, profile: &GameProfile) -> Result<(), String> {
    profile.validate()?;
    let json = serde_json::to_string_pretty(profile)
        .map_err(|error| format!("cannot serialize game profile: {error}"))?;
    write_atomic(path, json.as_bytes())
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    }
    let temp = path.with_extension("tmp");
    fs::write(&temp, bytes)
        .map_err(|error| format!("cannot write {}: {error}", temp.display()))?;
    if path.exists() {
        fs::remove_file(path)
            .map_err(|error| format!("cannot replace {}: {error}", path.display()))?;
    }
    fs::rename(&temp, path)
        .map_err(|error| format!("cannot replace {}: {error}", path.display()))
}

fn flush_save_ram(session: &EmulatorSession) -> Result<(), String> {
    let bytes = session
        .core
        .read_save_ram()
        .map_err(|error| format!("read save RAM: {error:?}"))?;
    if bytes.is_empty() {
        return Ok(());
    }
    write_atomic(&session.paths.save_ram, &bytes)
}

fn restore_save_ram(core: &mut LibretroCore, path: &Path) -> Result<(), String> {
    if !path.exists() || core.save_ram_size() == 0 {
        return Ok(());
    }
    let bytes = fs::read(path)
        .map_err(|error| format!("cannot read save RAM {}: {error}", path.display()))?;
    core.write_save_ram(&bytes)
        .map_err(|error| format!("restore save RAM: {error:?}"))
}

fn state_path(session: &EmulatorSession, slot: u8) -> PathBuf {
    session.paths.state_dir.join(format!("slot-{slot}.state"))
}

fn encode_state(frame: u64, state: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(STATE_MAGIC.len() + 8 + state.len());
    bytes.extend_from_slice(STATE_MAGIC);
    bytes.extend_from_slice(&frame.to_le_bytes());
    bytes.extend_from_slice(state);
    bytes
}

fn decode_state(bytes: &[u8]) -> Result<(u64, &[u8]), String> {
    let header_len = STATE_MAGIC.len() + 8;
    if bytes.len() < header_len || !bytes.starts_with(STATE_MAGIC) {
        return Err("invalid PhiCade state header".into());
    }
    let frame_offset = STATE_MAGIC.len();
    let frame = u64::from_le_bytes(
        bytes[frame_offset..frame_offset + 8]
            .try_into()
            .map_err(|_| "invalid PhiCade state frame header")?,
    );
    Ok((frame, &bytes[header_len..]))
}

fn save_state_slot(session: &mut EmulatorSession, slot: u8) -> Result<(), String> {
    if slot > 9 {
        return Err("state slot must be in 0..=9".into());
    }
    let state = session
        .core
        .serialize_state()
        .map_err(|error| format!("serialize slot {slot}: {error:?}"))?;
    let payload = encode_state(session.core.frame_count(), &state);
    write_atomic(&state_path(session, slot), &payload)
}

fn load_state_slot(session: &mut EmulatorSession, slot: u8) -> Result<(), String> {
    if slot > 9 {
        return Err("state slot must be in 0..=9".into());
    }
    let path = state_path(session, slot);
    let payload = fs::read(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let (frame, state) = decode_state(&payload)?;
    session
        .core
        .restore_state(state, frame)
        .map_err(|error| format!("restore slot {slot}: {error:?}"))?;
    session.rewind.clear();
    session.next_rewind_frame =
        session.core.frame_count() + u64::from(session.profile.rewind_interval_frames);
    push_rewind_snapshot(session)?;
    Ok(())
}

fn rewind_capacity(session: &EmulatorSession) -> usize {
    let fps = session.core.identity().fps.max(1.0);
    let interval = f64::from(session.profile.rewind_interval_frames.max(1));
    ((f64::from(session.profile.rewind_seconds) * fps / interval).ceil() as usize).max(2) + 1
}

fn push_rewind_snapshot(session: &mut EmulatorSession) -> Result<(), String> {
    let state = session
        .core
        .serialize_state()
        .map_err(|error| format!("serialize rewind snapshot: {error:?}"))?;
    session.rewind.push_back(StateSnapshot {
        frame: session.core.frame_count(),
        bytes: state,
    });
    let capacity = rewind_capacity(session);
    while session.rewind.len() > capacity {
        session.rewind.pop_front();
    }
    Ok(())
}

fn rewind_session(session: &mut EmulatorSession, seconds: u8) -> Result<(), String> {
    if session.rewind.is_empty() {
        return Err("rewind buffer is empty".into());
    }

    let fps = session.core.identity().fps.max(1.0).round() as u64;
    let target = session
        .core
        .frame_count()
        .saturating_sub(fps.saturating_mul(u64::from(seconds.max(1))));

    while session.rewind.len() > 1 {
        let should_pop = session
            .rewind
            .back()
            .is_some_and(|snapshot| snapshot.frame > target);
        if should_pop {
            session.rewind.pop_back();
        } else {
            break;
        }
    }

    let snapshot = session
        .rewind
        .back()
        .cloned()
        .ok_or_else(|| "rewind buffer unexpectedly empty".to_owned())?;

    session
        .core
        .restore_state(&snapshot.bytes, snapshot.frame)
        .map_err(|error| format!("rewind restore failed: {error:?}"))?;
    session.next_rewind_frame =
        snapshot.frame + u64::from(session.profile.rewind_interval_frames);
    Ok(())
}

fn process_session_actions(
    session: &mut EmulatorSession,
    actions: &[ActionEnvelope],
) -> Result<(), String> {
    for envelope in actions {
        match &envelope.action {
            ActionKind::System {
                command: SystemCommand::SaveState,
                slot,
            } => {
                let selected = slot.unwrap_or(session.profile.save_slot);
                session.profile.save_slot = selected;
                persist_game_profile(&session.paths.profile, &session.profile)?;
                save_state_slot(session, selected)?;
            }
            ActionKind::System {
                command: SystemCommand::LoadState,
                slot,
            } => {
                let selected = slot.unwrap_or(session.profile.save_slot);
                session.profile.save_slot = selected;
                persist_game_profile(&session.paths.profile, &session.profile)?;
                load_state_slot(session, selected)?;
            }
            ActionKind::System {
                command: SystemCommand::Rewind,
                slot,
            } => {
                rewind_session(session, slot.unwrap_or(2).max(1))?;
            }
            _ => {}
        }
    }
    Ok(())
}

#[tauri::command]
fn start_emulation(
    app: AppHandle,
    state: State<'_, EmulatorState>,
    core_path: String,
    game_path: String,
) -> Result<SessionInfo, String> {
    let game = PathBuf::from(&game_path)
        .canonicalize()
        .map_err(|error| format!("cannot resolve selected game: {error}"))?;
    let core_file = PathBuf::from(&core_path)
        .canonicalize()
        .map_err(|error| format!("cannot resolve selected core: {error}"))?;

    let extension = game
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| "selected game has no supported extension".to_owned())?;
    let system = system_id_from_extension(&extension)
        .ok_or_else(|| "Rung 4 SameBoy sessions accept .gb or .gbc images only".to_owned())?;
    let display_name = game
        .file_stem()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| "Game Boy image".to_owned());
    let game_key = sha256_file(&game)?;

    {
        let mut session = state
            .session
            .lock()
            .map_err(|_| "emulator session lock poisoned".to_owned())?;
        *session = None;
    }

    let (root, system_dir, save_dir) = app_runtime_dirs(&app)?;
    let mut core = LibretroCore::open(&core_file, &system_dir, &save_dir)
        .map_err(|error| format!("cannot open libretro core: {error:?}"))?;
    let identity = core.identity().clone();

    if !identity.library_name.to_ascii_lowercase().contains("sameboy") {
        return Err(format!(
            "Rung 4 qualification expects SameBoy, got {} {}",
            identity.library_name, identity.library_version
        ));
    }

    core.load_game(&GameImage::new(&game, system, display_name))
        .map_err(|error| format!("cannot load game image: {error:?}"))?;

    let paths = session_paths(&root, &game_key, &identity)?;
    restore_save_ram(&mut core, &paths.save_ram)?;
    let profile = load_game_profile(&paths.profile)?;
    persist_game_profile(&paths.profile, &profile)?;

    let info = SessionInfo {
        game_path: game.to_string_lossy().to_string(),
        game_key: game_key.clone(),
        core_path: core_file.to_string_lossy().to_string(),
        core: identity,
        profile: profile.clone(),
    };

    let mut emulator_session = EmulatorSession {
        core,
        game_path: info.game_path.clone(),
        game_key,
        profile,
        paths,
        rewind: VecDeque::new(),
        next_rewind_frame: 0,
        last_sram_flush_frame: 0,
        last_frame: FrameBuffer::default(),
        recording: None,
        last_replay: None,
    };

    push_rewind_snapshot(&mut emulator_session)?;
    emulator_session.next_rewind_frame =
        u64::from(emulator_session.profile.rewind_interval_frames);

    let mut session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    *session = Some(emulator_session);

    Ok(info)
}

fn resample_stereo(input: &[f32], source_rate: u32, target_rate: u32) -> Vec<f32> {
    let source_frames = input.len() / 2;
    if source_frames == 0 || source_rate == 0 || target_rate == 0 {
        return Vec::new();
    }
    if source_rate == target_rate {
        return input[..source_frames * 2].to_vec();
    }

    let output_frames =
        ((source_frames as u64 * target_rate as u64 + source_rate as u64 - 1)
            / source_rate as u64)
            .max(1) as usize;
    let ratio = source_rate as f64 / target_rate as f64;
    let mut output = Vec::with_capacity(output_frames * 2);

    for output_frame in 0..output_frames {
        let position = (output_frame as f64 * ratio).min((source_frames - 1) as f64);
        let left_index = position.floor() as usize;
        let right_index = (left_index + 1).min(source_frames - 1);
        let fraction = (position - left_index as f64) as f32;

        for channel in 0..2 {
            let left = input[left_index * 2 + channel];
            let right = input[right_index * 2 + channel];
            output.push(left + (right - left) * fraction);
        }
    }

    output
}

#[tauri::command]
fn step_emulation(
    state: State<'_, EmulatorState>,
    actions: Vec<ActionEnvelope>,
) -> Result<FramePacket, String> {
    let mut session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_mut()
        .ok_or_else(|| "no emulator session is running".to_owned())?;

    process_session_actions(session, &actions)?;

    let speed = session.profile.fast_forward.clamp(1, 4);
    let mut video = FrameBuffer::default();
    let mut audio = AudioBuffer::default();

    for index in 0..speed {
        let frame_actions = if index == 0 { actions.as_slice() } else { &[] };
        session
            .core
            .step_frame(frame_actions, &mut video, &mut audio)
            .map_err(|error| format!("core frame failed: {error:?}"))?;

        session.last_frame = FrameBuffer {
            width: video.width,
            height: video.height,
            rgba8: video.rgba8.clone(),
        };

        let frame = session.core.frame_count();
        if frame >= session.next_rewind_frame {
            push_rewind_snapshot(session)?;
            session.next_rewind_frame =
                frame + u64::from(session.profile.rewind_interval_frames);
        }

        if frame.saturating_sub(session.last_sram_flush_frame) >= SRAM_FLUSH_INTERVAL_FRAMES {
            flush_save_ram(session)?;
            session.last_sram_flush_frame = frame;
        }
    }

    let output_rate = if audio.sample_rate_hz > 96_000 {
        WEB_AUDIO_SAMPLE_RATE_HZ
    } else {
        audio.sample_rate_hz
    };
    let output_samples = resample_stereo(
        &audio.interleaved_stereo_f32,
        audio.sample_rate_hz,
        output_rate,
    );
    let mut audio_bytes = Vec::with_capacity(output_samples.len() * 2);
    for sample in output_samples {
        let quantized = (sample.clamp(-1.0, 1.0) * 32767.0).round() as i16;
        audio_bytes.extend_from_slice(&quantized.to_le_bytes());
    }

    Ok(FramePacket {
        frame: session.core.frame_count(),
        width: video.width,
        height: video.height,
        rgba_base64: BASE64.encode(video.rgba8),
        audio_base64: BASE64.encode(audio_bytes),
        sample_rate_hz: output_rate,
        shutdown_requested: session.core.shutdown_requested(),
        rewind_snapshots: session.rewind.len(),
        fast_forward: speed,
    })
}

#[tauri::command]
fn set_game_profile(
    state: State<'_, EmulatorState>,
    profile: GameProfile,
) -> Result<GameProfile, String> {
    profile.validate()?;
    let mut session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_mut()
        .ok_or_else(|| "no emulator session is running".to_owned())?;

    persist_game_profile(&session.paths.profile, &profile)?;
    session.profile = profile.clone();
    let capacity = rewind_capacity(session);
    while session.rewind.len() > capacity {
        session.rewind.pop_front();
    }
    session.next_rewind_frame =
        session.core.frame_count() + u64::from(session.profile.rewind_interval_frames);
    Ok(profile)
}

#[tauri::command]
fn capture_screenshot(state: State<'_, EmulatorState>) -> Result<String, String> {
    let session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_ref()
        .ok_or_else(|| "no emulator session is running".to_owned())?;

    if session.last_frame.width == 0
        || session.last_frame.height == 0
        || session.last_frame.rgba8.is_empty()
    {
        return Err("no rendered frame is available yet".into());
    }

    let path = session.paths.screenshot_dir.join(format!(
        "frame-{:012}.png",
        session.core.frame_count()
    ));
    let file = fs::File::create(&path)
        .map_err(|error| format!("cannot create {}: {error}", path.display()))?;
    let writer = BufWriter::new(file);
    let mut encoder = png::Encoder::new(
        writer,
        session.last_frame.width,
        session.last_frame.height,
    );
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut png_writer = encoder
        .write_header()
        .map_err(|error| format!("cannot encode PNG header: {error}"))?;
    png_writer
        .write_image_data(&session.last_frame.rgba8)
        .map_err(|error| format!("cannot encode PNG pixels: {error}"))?;

    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
fn flush_game_save(state: State<'_, EmulatorState>) -> Result<(), String> {
    let session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_ref()
        .ok_or_else(|| "no emulator session is running".to_owned())?;
    flush_save_ram(session)
}

#[tauri::command]
fn stop_emulation(state: State<'_, EmulatorState>) -> Result<(), String> {
    let mut session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    if let Some(active) = session.as_ref() {
        flush_save_ram(active)?;
    }
    *session = None;
    Ok(())
}

#[tauri::command]
fn running_game(state: State<'_, EmulatorState>) -> Result<Option<String>, String> {
    let session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    Ok(session.as_ref().map(|value| value.game_path.clone()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(EmulatorState::default())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            load_settings,
            save_settings,
            scan_rom_directory,
            validate_action_envelope,
            start_emulation,
            step_emulation,
            set_game_profile,
            capture_screenshot,
            flush_game_save,
            stop_emulation,
            running_game
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
    fn resamples_core_audio_into_web_audio_range() {
        let input = vec![0.25, -0.25, 0.25, -0.25, 0.25, -0.25, 0.25, -0.25];
        let output = resample_stereo(&input, 96_000, 48_000);
        assert_eq!(output.len(), 4);
        assert!(output.iter().all(|sample| sample.abs() <= 0.25));
    }

    #[test]
    fn state_envelope_round_trips_frame_and_bytes() {
        let encoded = encode_state(369, &[1, 2, 3, 4]);
        let (frame, state) = decode_state(&encoded).expect("decode state");
        assert_eq!(frame, 369);
        assert_eq!(state, &[1, 2, 3, 4]);
    }

    #[test]
    fn game_profile_rejects_ungoverned_speed() {
        let profile = GameProfile {
            fast_forward: 3,
            ..GameProfile::default()
        };
        assert!(profile.validate().is_err());
    }

    #[test]
    fn rung_four_maps_game_boy_extensions() {
        assert_eq!(system_id_from_extension("gb"), Some(SystemId::GameBoy));
        assert_eq!(system_id_from_extension("gbc"), Some(SystemId::GameBoyColor));
        assert_eq!(system_id_from_extension("gba"), None);
    }
}
