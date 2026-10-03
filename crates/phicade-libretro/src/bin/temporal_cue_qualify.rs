use phicade_libretro::LibretroCore;
use phicade_runtime::{
    benchmark_task_by_id, locate_agent_gym_player, ActionEnvelope, ActionKind, ActionSource,
    AudioBuffer, BenchmarkTaskSpec, EmulatorCore, FrameBuffer, GameImage, PixelPoint, SystemId,
    AGENT_GYM_TEMPORAL_LEFT_ID, AGENT_GYM_TEMPORAL_RIGHT_ID,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

const WARMUP_FRAMES: u64 = 120;
const WAIT_AFTER_CUE_FRAMES: u64 = 100;
const CHOICE_MOVE_FRAMES: u64 = 24;
const SETTLE_FRAMES: u64 = 6;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct VariantEvidence {
    task_id: String,
    rom_sha256: String,
    cue_frame_sha256: String,
    decision_frame_sha256: String,
    cue_player: PixelPoint,
    decision_player: PixelPoint,
    carried_direction_player: PixelPoint,
    recovered_final_player: PixelPoint,
    carry_through_refused: bool,
    neutral_rearm_recovery_pass: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TemporalCuePairReceipt {
    schema: &'static str,
    result: &'static str,
    core_sha256: String,
    core_name: String,
    core_version: String,
    left: VariantEvidence,
    right: VariantEvidence,
    cue_frames_differ: bool,
    decision_frames_identical: bool,
    decision_geometry_identical: bool,
    carried_direction_refused_both: bool,
    neutral_rearm_recovery_both: bool,
}

fn usage() -> ! {
    eprintln!(
        "usage: temporal_cue_qualify --core <sameboy_libretro> --left-rom <left.gb> --right-rom <right.gb> --receipt <path>"
    );
    process::exit(2);
}

fn sha256_bytes(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn sha256_file(path: &Path) -> Result<String, String> {
    fs::read(path)
        .map(|bytes| sha256_bytes(&bytes))
        .map_err(|error| format!("cannot read {}: {error}", path.display()))
}

fn button_event(sequence: u64, frame: u64, button: &str, pressed: bool) -> ActionEnvelope {
    ActionEnvelope {
        sequence,
        frame,
        source: ActionSource::Script {
            name: "temporal-cue-qualifier".into(),
        },
        action: ActionKind::Button {
            button: button.into(),
            pressed,
        },
    }
}

fn step(
    core: &mut LibretroCore,
    events: &[ActionEnvelope],
    video: &mut FrameBuffer,
    audio: &mut AudioBuffer,
    context: &str,
) -> Result<(), String> {
    core.step_frame(events, video, audio)
        .map_err(|error| format!("{context}: {error:?}"))
}

fn no_input(
    core: &mut LibretroCore,
    frames: u64,
    video: &mut FrameBuffer,
    audio: &mut AudioBuffer,
    context: &str,
) -> Result<(), String> {
    for _ in 0..frames {
        step(core, &[], video, audio, context)?;
    }
    Ok(())
}

fn dismiss_and_wait(
    core: &mut LibretroCore,
    video: &mut FrameBuffer,
    audio: &mut AudioBuffer,
) -> Result<(), String> {
    let frame = core.frame_count();
    step(
        core,
        &[button_event(0, frame, "A", true)],
        video,
        audio,
        "press A to dismiss cue",
    )?;
    let frame = core.frame_count();
    step(
        core,
        &[button_event(1, frame, "A", false)],
        video,
        audio,
        "release A after cue",
    )?;
    no_input(
        core,
        WAIT_AFTER_CUE_FRAMES.saturating_sub(1),
        video,
        audio,
        "wait for decision chamber",
    )
}

fn carried_direction_probe(
    core: &mut LibretroCore,
    direction: &str,
    start: PixelPoint,
    video: &mut FrameBuffer,
    audio: &mut AudioBuffer,
) -> Result<(PixelPoint, bool), String> {
    let frame = core.frame_count();
    step(
        core,
        &[
            button_event(0, frame, "A", true),
            button_event(1, frame, direction, true),
        ],
        video,
        audio,
        "press A plus early direction",
    )?;
    let frame = core.frame_count();
    step(
        core,
        &[button_event(2, frame, "A", false)],
        video,
        audio,
        "release A while carrying direction",
    )?;
    no_input(
        core,
        WAIT_AFTER_CUE_FRAMES.saturating_sub(1),
        video,
        audio,
        "carry direction through lockout",
    )?;
    let player = locate_agent_gym_player(video)?;
    Ok((player, player == start))
}

fn neutral_rearm_and_move(
    core: &mut LibretroCore,
    direction: &str,
    target: PixelPoint,
    video: &mut FrameBuffer,
    audio: &mut AudioBuffer,
) -> Result<(PixelPoint, bool), String> {
    let frame = core.frame_count();
    step(
        core,
        &[button_event(3, frame, direction, false)],
        video,
        audio,
        "release carried direction to arm choice",
    )?;

    let frame = core.frame_count();
    step(
        core,
        &[button_event(4, frame, direction, true)],
        video,
        audio,
        "press direction after neutral rearm",
    )?;
    for _ in 1..CHOICE_MOVE_FRAMES {
        step(core, &[], video, audio, "hold recovered direction")?;
    }
    let frame = core.frame_count();
    step(
        core,
        &[button_event(5, frame, direction, false)],
        video,
        audio,
        "release recovered direction",
    )?;
    no_input(core, SETTLE_FRAMES, video, audio, "settle recovered choice")?;
    let player = locate_agent_gym_player(video)?;
    Ok((player, player == target))
}

fn qualify_variant(
    core_path: &Path,
    rom_path: &Path,
    task: &'static BenchmarkTaskSpec,
    direction: &str,
) -> Result<(VariantEvidence, String, String), String> {
    let temp = env::temp_dir().join(format!("phicade-temporal-pair-{}", task.id));
    let system_dir = temp.join("system");
    let save_dir = temp.join("save");
    fs::create_dir_all(&system_dir).map_err(|error| error.to_string())?;
    fs::create_dir_all(&save_dir).map_err(|error| error.to_string())?;

    let mut core = LibretroCore::open(core_path, &system_dir, &save_dir)
        .map_err(|error| format!("open SameBoy for {}: {error:?}", task.id))?;
    core.load_game(&GameImage::new(rom_path, SystemId::GameBoy, task.id))
        .map_err(|error| format!("load {}: {error:?}", task.id))?;

    let core_name = core.identity().library_name.clone();
    let core_version = core.identity().library_version.clone();
    let mut video = FrameBuffer::default();
    let mut audio = AudioBuffer::default();

    no_input(
        &mut core,
        WARMUP_FRAMES,
        &mut video,
        &mut audio,
        "warm temporal cue",
    )?;
    let cue_player = locate_agent_gym_player(&video)?;
    if cue_player != task.start {
        return Err(format!(
            "{} cue start drifted: expected {:?}, got {:?}",
            task.id, task.start, cue_player
        ));
    }
    let cue_frame_sha256 = sha256_bytes(&video.rgba8);
    let frozen_state = core
        .serialize_state()
        .map_err(|error| format!("serialize {} cue state: {error:?}", task.id))?;
    let frozen_mask = core.input_mask_snapshot();
    let frozen_frame = core.frame_count();

    dismiss_and_wait(&mut core, &mut video, &mut audio)?;
    let decision_player = locate_agent_gym_player(&video)?;
    let decision_frame_sha256 = sha256_bytes(&video.rgba8);
    if decision_player != task.start {
        return Err(format!(
            "{} moved before decision: expected {:?}, got {:?}",
            task.id, task.start, decision_player
        ));
    }

    core.restore_state(&frozen_state, frozen_frame)
        .map_err(|error| format!("restore {} cue state: {error:?}", task.id))?;
    core.restore_input_mask(frozen_mask);

    let (carried_direction_player, carry_through_refused) = carried_direction_probe(
        &mut core,
        direction,
        task.start,
        &mut video,
        &mut audio,
    )?;

    let (recovered_final_player, neutral_rearm_recovery_pass) = neutral_rearm_and_move(
        &mut core,
        direction,
        task.target,
        &mut video,
        &mut audio,
    )?;

    Ok((
        VariantEvidence {
            task_id: task.id.into(),
            rom_sha256: sha256_file(rom_path)?,
            cue_frame_sha256,
            decision_frame_sha256,
            cue_player,
            decision_player,
            carried_direction_player,
            recovered_final_player,
            carry_through_refused,
            neutral_rearm_recovery_pass,
        },
        core_name,
        core_version,
    ))
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Temporal Cue pair qualification failed: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let mut core_path: Option<PathBuf> = None;
    let mut left_rom: Option<PathBuf> = None;
    let mut right_rom: Option<PathBuf> = None;
    let mut receipt_path: Option<PathBuf> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--core" => core_path = args.next().map(PathBuf::from),
            "--left-rom" => left_rom = args.next().map(PathBuf::from),
            "--right-rom" => right_rom = args.next().map(PathBuf::from),
            "--receipt" => receipt_path = args.next().map(PathBuf::from),
            _ => usage(),
        }
    }

    let core_path = core_path.unwrap_or_else(|| usage());
    let left_rom = left_rom.unwrap_or_else(|| usage());
    let right_rom = right_rom.unwrap_or_else(|| usage());
    let receipt_path = receipt_path.unwrap_or_else(|| usage());

    let left_task = benchmark_task_by_id(AGENT_GYM_TEMPORAL_LEFT_ID)
        .ok_or_else(|| "temporal LEFT task missing from registry".to_owned())?;
    let right_task = benchmark_task_by_id(AGENT_GYM_TEMPORAL_RIGHT_ID)
        .ok_or_else(|| "temporal RIGHT task missing from registry".to_owned())?;

    let (left, core_name, core_version) =
        qualify_variant(&core_path, &left_rom, left_task, "LEFT")?;
    let (right, right_core_name, right_core_version) =
        qualify_variant(&core_path, &right_rom, right_task, "RIGHT")?;

    if core_name != right_core_name || core_version != right_core_version {
        return Err("temporal variants did not run under identical core identity".into());
    }

    let cue_frames_differ = left.cue_frame_sha256 != right.cue_frame_sha256;
    let decision_frames_identical = left.decision_frame_sha256 == right.decision_frame_sha256;
    let decision_geometry_identical = left.decision_player == right.decision_player
        && left.decision_player == left_task.start
        && right.decision_player == right_task.start;
    let carried_direction_refused_both =
        left.carry_through_refused && right.carry_through_refused;
    let neutral_rearm_recovery_both =
        left.neutral_rearm_recovery_pass && right.neutral_rearm_recovery_pass;

    if !(cue_frames_differ
        && decision_frames_identical
        && decision_geometry_identical
        && carried_direction_refused_both
        && neutral_rearm_recovery_both)
    {
        return Err(format!(
            "temporal pair controls failed: cue_diff={cue_frames_differ} decision_same={decision_frames_identical} geometry_same={decision_geometry_identical} carry_refused={carried_direction_refused_both} recovery={neutral_rearm_recovery_both}"
        ));
    }

    let receipt = TemporalCuePairReceipt {
        schema: "phicade.temporal-cue-pair-qualification.v1",
        result: "PASS",
        core_sha256: sha256_file(&core_path)?,
        core_name,
        core_version,
        left,
        right,
        cue_frames_differ,
        decision_frames_identical,
        decision_geometry_identical,
        carried_direction_refused_both,
        neutral_rearm_recovery_both,
    };

    let json = serde_json::to_string_pretty(&receipt)
        .map_err(|error| format!("serialize temporal cue receipt: {error}"))?;
    println!("{json}");
    if let Some(parent) = receipt_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(&receipt_path, format!("{json}\n"))
        .map_err(|error| format!("write {}: {error}", receipt_path.display()))?;
    Ok(())
}
