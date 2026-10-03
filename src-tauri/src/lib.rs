use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use phicade_libretro::{CoreIdentity, LibretroCore};
use phicade_runtime::{
    ActionEnvelope, ActionKind, ActionSource, AgentGrant, AuthorityPolicy, AudioBuffer, ControlMode,
    EmulatorCore, FrameBuffer, GameImage, PhiBotObservation, ReplayCheckpoint, ReplayLedger,
    ReplayReceipt, ReplayVerification, ReplayVerificationResult, SystemCommand, SystemId,
    PHIBOT_OBSERVATION_SCHEMA, REPLAY_RECEIPT_SCHEMA, REPLAY_SCHEMA,
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
    authority: AuthorityPolicy,
    authority_rejections: u64,
    last_authority_reason: Option<String>,
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
    control_mode: ControlMode,
    authority_rejections: u64,
    last_authority_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AuthorityStatus {
    mode: ControlMode,
    playable_ports: u8,
    agent_id: Option<String>,
    agent_seat: Option<u8>,
    allowed_buttons: Vec<String>,
    allowed_axes: Vec<String>,
    expires_at_frame: Option<u64>,
    rejected_actions: u64,
    last_reason: Option<String>,
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


fn replay_status_for(session: &EmulatorSession) -> ReplayStatus {
    let (action_count, checkpoint_count) = session
        .recording
        .as_ref()
        .map(|recording| {
            (
                recording.ledger.actions.len(),
                recording.ledger.checkpoints.len(),
            )
        })
        .or_else(|| {
            session.last_replay.as_ref().map(|replay| {
                (
                    replay.receipt.action_count,
                    replay.receipt.checkpoint_count,
                )
            })
        })
        .unwrap_or((0, 0));

    ReplayStatus {
        recording: session.recording.is_some(),
        action_count,
        checkpoint_count,
        last_replay_path: session
            .last_replay
            .as_ref()
            .map(|replay| replay.replay_path.to_string_lossy().to_string()),
        last_receipt_path: session
            .last_replay
            .as_ref()
            .map(|replay| replay.receipt_path.to_string_lossy().to_string()),
        last_replay_sha256: session
            .last_replay
            .as_ref()
            .map(|replay| replay.replay_sha256.clone()),
    }
}

fn make_replay_checkpoint(
    session: &EmulatorSession,
    frame: &FrameBuffer,
) -> Result<ReplayCheckpoint, String> {
    let state = session
        .core
        .serialize_state()
        .map_err(|error| format!("serialize replay checkpoint: {error:?}"))?;

    Ok(ReplayCheckpoint {
        frame: session.core.frame_count(),
        state_sha256: sha256_bytes(&state),
        frame_sha256: sha256_bytes(&frame.rgba8),
        input_mask: session.core.input_mask_snapshot(),
    })
}

fn upsert_replay_checkpoint(
    session: &mut EmulatorSession,
    checkpoint: ReplayCheckpoint,
) -> Result<(), String> {
    let recording = session
        .recording
        .as_mut()
        .ok_or_else(|| "replay recorder is not active".to_owned())?;

    if recording
        .ledger
        .checkpoints
        .last()
        .is_some_and(|last| last.frame == checkpoint.frame)
    {
        if let Some(last) = recording.ledger.checkpoints.last_mut() {
            *last = checkpoint;
        }
    } else {
        recording.ledger.checkpoints.push(checkpoint);
    }

    Ok(())
}

fn maybe_record_replay_checkpoint(session: &mut EmulatorSession) -> Result<(), String> {
    let frame = session.core.frame_count();
    let due = session
        .recording
        .as_ref()
        .is_some_and(|recording| frame >= recording.next_checkpoint_frame);

    if !due {
        return Ok(());
    }

    let checkpoint = make_replay_checkpoint(session, &session.last_frame)?;
    upsert_replay_checkpoint(session, checkpoint)?;

    if let Some(recording) = session.recording.as_mut() {
        recording.next_checkpoint_frame =
            frame.saturating_add(REPLAY_CHECKPOINT_INTERVAL_FRAMES);
    }

    Ok(())
}

fn validate_recording_actions(
    session: &EmulatorSession,
    actions: &[ActionEnvelope],
) -> Result<(), String> {
    if session.recording.is_none() {
        return Ok(());
    }

    for envelope in actions {
        if let ActionKind::System { command, .. } = &envelope.action {
            return Err(format!(
                "{command:?} is disabled while Replay v1 recording is active; v1 records controller/axis input only"
            ));
        }
    }

    Ok(())
}

fn record_applied_actions(
    session: &mut EmulatorSession,
    actions: &[ActionEnvelope],
) -> Result<(), String> {
    if session.recording.is_none() || actions.is_empty() {
        return Ok(());
    }

    let applied_frame = session.core.frame_count();
    let mut normalized = Vec::with_capacity(actions.len());

    for envelope in actions {
        let mut event = envelope.clone();
        event.frame = applied_frame;
        event.validate()?;
        normalized.push(event);
    }

    let recording = session
        .recording
        .as_mut()
        .ok_or_else(|| "replay recorder disappeared".to_owned())?;
    recording.ledger.actions.extend(normalized);
    Ok(())
}

fn replay_receipt(
    ledger: &ReplayLedger,
    replay_sha256: &str,
    verification: Option<ReplayVerification>,
) -> ReplayReceipt {
    ReplayReceipt {
        schema: REPLAY_RECEIPT_SCHEMA.to_owned(),
        replay_sha256: replay_sha256.to_owned(),
        game_sha256: ledger.game_sha256.clone(),
        core_name: ledger.core_name.clone(),
        core_version: ledger.core_version.clone(),
        core_sha256: ledger.core_sha256.clone(),
        start_frame: ledger.start_frame,
        end_frame: ledger.end_frame,
        action_count: ledger.actions.len(),
        checkpoint_count: ledger.checkpoints.len(),
        verification,
    }
}

fn artifact_view(export: &ReplayExport) -> ReplayArtifact {
    ReplayArtifact {
        replay_path: export.replay_path.to_string_lossy().to_string(),
        receipt_path: export.receipt_path.to_string_lossy().to_string(),
        replay_sha256: export.replay_sha256.clone(),
        receipt: export.receipt.clone(),
    }
}

fn finalize_replay_recording(session: &mut EmulatorSession) -> Result<ReplayExport, String> {
    let start_frame = session
        .recording
        .as_ref()
        .ok_or_else(|| "replay recorder is not active".to_owned())?
        .ledger
        .start_frame;
    if session.core.frame_count() <= start_frame {
        return Err("record at least one emulated frame before exporting the replay".into());
    }

    let mut recording = session
        .recording
        .take()
        .ok_or_else(|| "replay recorder is not active".to_owned())?;

    let final_checkpoint = make_replay_checkpoint(session, &session.last_frame)?;
    if recording
        .ledger
        .checkpoints
        .last()
        .is_some_and(|last| last.frame == final_checkpoint.frame)
    {
        if let Some(last) = recording.ledger.checkpoints.last_mut() {
            *last = final_checkpoint.clone();
        }
    } else {
        recording.ledger.checkpoints.push(final_checkpoint.clone());
    }

    recording.ledger.end_frame = session.core.frame_count();
    recording.ledger.final_state_sha256 = final_checkpoint.state_sha256.clone();
    recording.ledger.final_frame_sha256 = final_checkpoint.frame_sha256.clone();
    recording.ledger.validate()?;

    let replay_json = serde_json::to_vec_pretty(&recording.ledger)
        .map_err(|error| format!("serialize replay ledger: {error}"))?;
    let replay_sha256 = sha256_bytes(&replay_json);
    let replay_path = session
        .paths
        .replay_dir
        .join(format!("{replay_sha256}.replay.json"));
    let receipt_path = session
        .paths
        .replay_dir
        .join(format!("{replay_sha256}.receipt.json"));

    write_atomic(&replay_path, &replay_json)?;

    let receipt = replay_receipt(&recording.ledger, &replay_sha256, None);
    let receipt_json = serde_json::to_vec_pretty(&receipt)
        .map_err(|error| format!("serialize replay receipt: {error}"))?;
    write_atomic(&receipt_path, &receipt_json)?;

    let export = ReplayExport {
        replay_path,
        receipt_path,
        replay_sha256,
        receipt,
    };
    session.last_replay = Some(export.clone());
    Ok(export)
}

fn replay_divergence(
    checked_checkpoints: usize,
    frame: u64,
    expected: &ReplayCheckpoint,
    actual: &ReplayCheckpoint,
) -> ReplayVerification {
    ReplayVerification {
        result: ReplayVerificationResult::Diverged,
        checked_checkpoints,
        first_divergence_frame: Some(frame),
        expected_state_sha256: Some(expected.state_sha256.clone()),
        actual_state_sha256: Some(actual.state_sha256.clone()),
        expected_frame_sha256: Some(expected.frame_sha256.clone()),
        actual_frame_sha256: Some(actual.frame_sha256.clone()),
    }
}

fn compare_replay_checkpoint(
    expected: &ReplayCheckpoint,
    actual: &ReplayCheckpoint,
    checked_checkpoints: usize,
    compare_framebuffer: bool,
) -> Option<ReplayVerification> {
    let state_differs = expected.state_sha256 != actual.state_sha256;
    let input_differs = expected.input_mask != actual.input_mask;
    let frame_differs =
        compare_framebuffer && expected.frame_sha256 != actual.frame_sha256;

    if state_differs || input_differs || frame_differs {
        Some(replay_divergence(
            checked_checkpoints,
            expected.frame,
            expected,
            actual,
        ))
    } else {
        None
    }
}

fn verify_replay_execution(
    session: &mut EmulatorSession,
    ledger: &ReplayLedger,
) -> Result<ReplayVerification, String> {
    ledger.validate()?;

    if ledger.game_sha256 != session.game_key {
        return Err("replay game fingerprint does not match running game".into());
    }
    if ledger.core_name != session.core.identity().library_name
        || ledger.core_version != session.core.identity().library_version
    {
        return Err("replay core identity/version does not match running core".into());
    }

    let running_core_sha256 = sha256_file(session.core.core_path())?;
    if ledger.core_sha256 != running_core_sha256 {
        return Err("replay core binary hash does not match running core".into());
    }

    let initial_state = BASE64
        .decode(&ledger.initial_state_base64)
        .map_err(|error| format!("decode replay initial state: {error}"))?;
    session
        .core
        .restore_state(&initial_state, ledger.start_frame)
        .map_err(|error| format!("restore replay initial state: {error:?}"))?;
    session.core.restore_input_mask(ledger.initial_input_mask);

    let mut checked = 0usize;
    let mut checkpoint_index = 0usize;
    let mut video = FrameBuffer::default();
    let mut audio = AudioBuffer::default();

    if let Some(expected) = ledger.checkpoints.first() {
        if expected.frame == ledger.start_frame {
            let actual = make_replay_checkpoint(session, &FrameBuffer::default())?;
            checked += 1;

            let state_or_input_differs =
                expected.state_sha256 != actual.state_sha256
                    || expected.input_mask != actual.input_mask;

            if state_or_input_differs {
                return Ok(replay_divergence(
                    checked,
                    expected.frame,
                    expected,
                    &actual,
                ));
            }
            checkpoint_index = 1;
        }
    }

    let mut action_index = 0usize;

    while session.core.frame_count() < ledger.end_frame {
        let current_frame = session.core.frame_count();
        let mut frame_actions = Vec::new();

        while let Some(action) = ledger.actions.get(action_index) {
            if action.frame != current_frame {
                break;
            }

            if let ActionKind::System { command, .. } = &action.action {
                return Err(format!(
                    "Replay v1 contains unsupported system command {command:?}"
                ));
            }

            let mut replay_action = action.clone();
            replay_action.source = ActionSource::Replay;
            frame_actions.push(replay_action);
            action_index += 1;
        }

        session
            .core
            .step_frame(&frame_actions, &mut video, &mut audio)
            .map_err(|error| format!("verify replay frame: {error:?}"))?;

        while let Some(expected) = ledger.checkpoints.get(checkpoint_index) {
            if expected.frame > session.core.frame_count() {
                break;
            }
            if expected.frame < session.core.frame_count() {
                return Err(format!(
                    "replay checkpoint {} was skipped at runtime frame {}",
                    expected.frame,
                    session.core.frame_count()
                ));
            }

            let actual = make_replay_checkpoint(session, &video)?;
            checked += 1;
            if let Some(divergence) =
                compare_replay_checkpoint(expected, &actual, checked, true)
            {
                return Ok(divergence);
            }
            checkpoint_index += 1;
        }
    }

    let final_state = session
        .core
        .serialize_state()
        .map_err(|error| format!("serialize replay final state: {error:?}"))?;
    let final_state_sha256 = sha256_bytes(&final_state);
    let final_frame_sha256 = sha256_bytes(&video.rgba8);

    if final_state_sha256 != ledger.final_state_sha256
        || final_frame_sha256 != ledger.final_frame_sha256
    {
        let expected = ReplayCheckpoint {
            frame: ledger.end_frame,
            state_sha256: ledger.final_state_sha256.clone(),
            frame_sha256: ledger.final_frame_sha256.clone(),
            input_mask: session.core.input_mask_snapshot(),
        };
        let actual = ReplayCheckpoint {
            frame: ledger.end_frame,
            state_sha256: final_state_sha256,
            frame_sha256: final_frame_sha256,
            input_mask: session.core.input_mask_snapshot(),
        };
        return Ok(replay_divergence(
            checked.saturating_add(1),
            ledger.end_frame,
            &expected,
            &actual,
        ));
    }

    Ok(ReplayVerification {
        result: ReplayVerificationResult::Pass,
        checked_checkpoints: checked,
        first_divergence_frame: None,
        expected_state_sha256: None,
        actual_state_sha256: None,
        expected_frame_sha256: None,
        actual_frame_sha256: None,
    })
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
        authority: AuthorityPolicy::new(1),
        authority_rejections: 0,
        last_authority_reason: None,
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

    validate_recording_actions(session, &actions)?;
    record_applied_actions(session, &actions)?;
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

        maybe_record_replay_checkpoint(session)?;

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

    let replay_recording = session.recording.is_some();
    let (replay_actions, replay_checkpoints) = session
        .recording
        .as_ref()
        .map(|recording| {
            (
                recording.ledger.actions.len(),
                recording.ledger.checkpoints.len(),
            )
        })
        .or_else(|| {
            session.last_replay.as_ref().map(|replay| {
                (
                    replay.receipt.action_count,
                    replay.receipt.checkpoint_count,
                )
            })
        })
        .unwrap_or((0, 0));

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
        replay_recording,
        replay_actions,
        replay_checkpoints,
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

    if session.recording.is_some() {
        return Err("per-game profile changes are disabled during replay recording".into());
    }

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
fn replay_status(state: State<'_, EmulatorState>) -> Result<ReplayStatus, String> {
    let session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_ref()
        .ok_or_else(|| "no emulator session is running".to_owned())?;
    Ok(replay_status_for(session))
}

#[tauri::command]
fn start_replay_recording(state: State<'_, EmulatorState>) -> Result<ReplayStatus, String> {
    let mut session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_mut()
        .ok_or_else(|| "no emulator session is running".to_owned())?;

    if session.recording.is_some() {
        return Err("replay recording is already active".into());
    }
    if session.profile.fast_forward != 1 {
        return Err("replay recording requires the per-game profile to be at 1x".into());
    }

    let start_frame = session.core.frame_count();
    let initial_state = session
        .core
        .serialize_state()
        .map_err(|error| format!("serialize replay initial state: {error:?}"))?;
    let initial_checkpoint = make_replay_checkpoint(session, &session.last_frame)?;
    let core_sha256 = sha256_file(session.core.core_path())?;

    session.recording = Some(ReplayRecording {
        ledger: ReplayLedger {
            schema: REPLAY_SCHEMA.to_owned(),
            game_sha256: session.game_key.clone(),
            core_name: session.core.identity().library_name.clone(),
            core_version: session.core.identity().library_version.clone(),
            core_sha256,
            start_frame,
            end_frame: start_frame,
            initial_state_base64: BASE64.encode(initial_state),
            initial_input_mask: session.core.input_mask_snapshot(),
            actions: Vec::new(),
            checkpoints: vec![initial_checkpoint.clone()],
            final_state_sha256: initial_checkpoint.state_sha256,
            final_frame_sha256: initial_checkpoint.frame_sha256,
        },
        next_checkpoint_frame: start_frame.saturating_add(REPLAY_CHECKPOINT_INTERVAL_FRAMES),
    });

    Ok(replay_status_for(session))
}

#[tauri::command]
fn stop_replay_recording(state: State<'_, EmulatorState>) -> Result<ReplayArtifact, String> {
    let mut session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_mut()
        .ok_or_else(|| "no emulator session is running".to_owned())?;

    let export = finalize_replay_recording(session)?;
    Ok(artifact_view(&export))
}

#[tauri::command]
fn verify_last_replay(state: State<'_, EmulatorState>) -> Result<ReplayArtifact, String> {
    let mut session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_mut()
        .ok_or_else(|| "no emulator session is running".to_owned())?;

    if session.recording.is_some() {
        return Err("stop replay recording before verification".into());
    }

    let mut export = session
        .last_replay
        .clone()
        .ok_or_else(|| "no replay has been exported in this session".to_owned())?;
    let replay_json = fs::read(&export.replay_path)
        .map_err(|error| format!("cannot read {}: {error}", export.replay_path.display()))?;
    let ledger: ReplayLedger = serde_json::from_slice(&replay_json)
        .map_err(|error| format!("cannot parse replay ledger: {error}"))?;
    let observed_replay_sha256 = sha256_bytes(&replay_json);

    if observed_replay_sha256 != export.replay_sha256 {
        return Err("exported replay hash no longer matches its content".into());
    }

    let backup_state = session
        .core
        .serialize_state()
        .map_err(|error| format!("serialize live state before verification: {error:?}"))?;
    let backup_frame = session.core.frame_count();
    let backup_input_mask = session.core.input_mask_snapshot();
    let backup_save_ram = session
        .core
        .read_save_ram()
        .map_err(|error| format!("read live save RAM before verification: {error:?}"))?;
    let backup_last_frame = session.last_frame.clone();
    let backup_rewind = session.rewind.clone();
    let backup_next_rewind_frame = session.next_rewind_frame;
    let backup_last_sram_flush_frame = session.last_sram_flush_frame;

    let verification = verify_replay_execution(session, &ledger);

    let restore_result = session
        .core
        .restore_state(&backup_state, backup_frame)
        .map_err(|error| format!("restore live state after verification: {error:?}"))
        .and_then(|_| {
            session.core.restore_input_mask(backup_input_mask);
            session
                .core
                .write_save_ram(&backup_save_ram)
                .map_err(|error| format!("restore live save RAM after verification: {error:?}"))
        });

    session.last_frame = backup_last_frame;
    session.rewind = backup_rewind;
    session.next_rewind_frame = backup_next_rewind_frame;
    session.last_sram_flush_frame = backup_last_sram_flush_frame;

    restore_result?;
    let verification = verification?;

    export.receipt = replay_receipt(
        &ledger,
        &export.replay_sha256,
        Some(verification),
    );
    let receipt_json = serde_json::to_vec_pretty(&export.receipt)
        .map_err(|error| format!("serialize verified replay receipt: {error}"))?;
    write_atomic(&export.receipt_path, &receipt_json)?;

    session.last_replay = Some(export.clone());
    Ok(artifact_view(&export))
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
        if active.recording.is_some() {
            return Err("stop replay recording before ejecting the game".into());
        }
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
            replay_status,
            start_replay_recording,
            stop_replay_recording,
            verify_last_replay,
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
