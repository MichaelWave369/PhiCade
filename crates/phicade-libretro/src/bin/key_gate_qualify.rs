use phicade_libretro::LibretroCore;
use phicade_runtime::{
    benchmark_task_by_id, benchmark_task_success, locate_agent_gym_player, ActionEnvelope,
    ActionKind, ActionSource, AudioBuffer, BenchmarkTaskSpec, EmulatorCore, FrameBuffer,
    GameImage, PixelPoint, SystemId, AGENT_GYM_KEY_GATE_LEFT_ID, AGENT_GYM_KEY_GATE_RIGHT_ID,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

const WARMUP_FRAMES: u64 = 120;
const SIDE_FRAMES: u64 = 28;
const TO_GATE_FRAMES: u64 = 20;
const THROUGH_GATE_FRAMES: u64 = 32;
const BLOCK_PROBE_FRAMES: u64 = 8;
const SETTLE_FRAMES: u64 = 2;

const START: PixelPoint = PixelPoint { x: 72, y: 112 };
const GATE_STOP: PixelPoint = PixelPoint { x: 72, y: 80 };

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct VariantEvidence {
    task_id: String,
    rom_sha256: String,
    initial_frame_sha256: String,
    direct_gate_player: PixelPoint,
    wrong_side_gate_player: PixelPoint,
    post_key_center_frame_sha256: String,
    post_key_center_player: PixelPoint,
    open_gate_frame_sha256: String,
    open_gate_player: PixelPoint,
    final_player: PixelPoint,
    direct_gate_refused: bool,
    wrong_side_refused: bool,
    pickup_changes_frame: bool,
    correct_key_pass: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct KeyGatePairReceipt {
    schema: &'static str,
    result: &'static str,
    core_sha256: String,
    core_name: String,
    core_version: String,
    left: VariantEvidence,
    right: VariantEvidence,
    initial_frames_differ: bool,
    post_key_frames_identical: bool,
    open_gate_frames_identical: bool,
    direct_gate_refused_both: bool,
    wrong_side_refused_both: bool,
    pickup_changes_frame_both: bool,
    correct_key_pass_both: bool,
}

fn usage() -> ! {
    eprintln!(
        "usage: key_gate_qualify --core <sameboy_libretro> --left-rom <left.gb> --right-rom <right.gb> --receipt <path>"
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
            name: "key-gate-qualifier".into(),
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

fn hold_button(
    core: &mut LibretroCore,
    sequence: &mut u64,
    button: &str,
    frames: u64,
    video: &mut FrameBuffer,
    audio: &mut AudioBuffer,
    context: &str,
) -> Result<(), String> {
    if frames == 0 {
        return Ok(());
    }

    let frame = core.frame_count();
    step(
        core,
        &[button_event(*sequence, frame, button, true)],
        video,
        audio,
        context,
    )?;
    *sequence = sequence.saturating_add(1);

    for _ in 1..frames {
        step(core, &[], video, audio, context)?;
    }

    let frame = core.frame_count();
    step(
        core,
        &[button_event(*sequence, frame, button, false)],
        video,
        audio,
        context,
    )?;
    *sequence = sequence.saturating_add(1);
    Ok(())
}

fn tap_a(
    core: &mut LibretroCore,
    sequence: &mut u64,
    video: &mut FrameBuffer,
    audio: &mut AudioBuffer,
    context: &str,
) -> Result<(), String> {
    hold_button(core, sequence, "A", 1, video, audio, context)
}

fn restore(
    core: &mut LibretroCore,
    state: &[u8],
    frame: u64,
    mask: u16,
    video: &mut FrameBuffer,
    audio: &mut AudioBuffer,
    context: &str,
) -> Result<(), String> {
    core.restore_state(state, frame)
        .map_err(|error| format!("{context}: restore state: {error:?}"))?;
    core.restore_input_mask(mask);
    no_input(core, 1, video, audio, context)
}

fn direct_gate_probe(
    core: &mut LibretroCore,
    sequence: &mut u64,
    task: &BenchmarkTaskSpec,
    video: &mut FrameBuffer,
    audio: &mut AudioBuffer,
) -> Result<(PixelPoint, bool), String> {
    hold_button(
        core,
        sequence,
        "UP",
        TO_GATE_FRAMES + BLOCK_PROBE_FRAMES,
        video,
        audio,
        "move directly into locked gate",
    )?;
    tap_a(core, sequence, video, audio, "try to unlock gate without key")?;
    hold_button(
        core,
        sequence,
        "UP",
        BLOCK_PROBE_FRAMES,
        video,
        audio,
        "probe gate after keyless unlock attempt",
    )?;
    let player = locate_agent_gym_player(video)?;
    Ok((
        player,
        player == GATE_STOP && !benchmark_task_success(task, player),
    ))
}

fn wrong_side_probe(
    core: &mut LibretroCore,
    sequence: &mut u64,
    task: &BenchmarkTaskSpec,
    wrong_side: &str,
    back_to_center: &str,
    video: &mut FrameBuffer,
    audio: &mut AudioBuffer,
) -> Result<(PixelPoint, bool), String> {
    hold_button(
        core,
        sequence,
        wrong_side,
        SIDE_FRAMES,
        video,
        audio,
        "visit empty key side",
    )?;
    tap_a(core, sequence, video, audio, "press A on empty key side")?;
    hold_button(
        core,
        sequence,
        back_to_center,
        SIDE_FRAMES,
        video,
        audio,
        "return from empty key side",
    )?;
    hold_button(
        core,
        sequence,
        "UP",
        TO_GATE_FRAMES,
        video,
        audio,
        "move to gate after empty pickup attempt",
    )?;
    tap_a(
        core,
        sequence,
        video,
        audio,
        "try gate after empty pickup attempt",
    )?;
    hold_button(
        core,
        sequence,
        "UP",
        BLOCK_PROBE_FRAMES,
        video,
        audio,
        "probe gate after empty pickup attempt",
    )?;
    let player = locate_agent_gym_player(video)?;
    Ok((
        player,
        player.y == GATE_STOP.y && !benchmark_task_success(task, player),
    ))
}

fn correct_key_path(
    core: &mut LibretroCore,
    sequence: &mut u64,
    task: &BenchmarkTaskSpec,
    key_side: &str,
    back_to_center: &str,
    initial_frame_sha256: &str,
    video: &mut FrameBuffer,
    audio: &mut AudioBuffer,
) -> Result<(String, PixelPoint, String, PixelPoint, PixelPoint, bool, bool), String> {
    hold_button(
        core,
        sequence,
        key_side,
        SIDE_FRAMES,
        video,
        audio,
        "move to visible key",
    )?;
    tap_a(core, sequence, video, audio, "acquire visible key")?;
    hold_button(
        core,
        sequence,
        back_to_center,
        SIDE_FRAMES,
        video,
        audio,
        "return to center with key",
    )?;
    no_input(
        core,
        SETTLE_FRAMES,
        video,
        audio,
        "settle post-key center state",
    )?;

    let post_key_center_player = locate_agent_gym_player(video)?;
    if post_key_center_player != START {
        return Err(format!(
            "post-key center drifted: expected {:?}, got {:?}",
            START, post_key_center_player
        ));
    }
    let post_key_center_frame_sha256 = sha256_bytes(&video.rgba8);
    let pickup_changes_frame = post_key_center_frame_sha256 != initial_frame_sha256;

    hold_button(
        core,
        sequence,
        "UP",
        TO_GATE_FRAMES,
        video,
        audio,
        "move to locked gate with key",
    )?;
    let at_gate = locate_agent_gym_player(video)?;
    if at_gate != GATE_STOP {
        return Err(format!(
            "gate approach drifted: expected {:?}, got {:?}",
            GATE_STOP, at_gate
        ));
    }

    tap_a(core, sequence, video, audio, "unlock gate with acquired key")?;
    no_input(core, SETTLE_FRAMES, video, audio, "settle open gate")?;
    let open_gate_player = locate_agent_gym_player(video)?;
    let open_gate_frame_sha256 = sha256_bytes(&video.rgba8);

    hold_button(
        core,
        sequence,
        "UP",
        THROUGH_GATE_FRAMES,
        video,
        audio,
        "move through unlocked gate to target",
    )?;
    no_input(core, SETTLE_FRAMES, video, audio, "settle target")?;
    let final_player = locate_agent_gym_player(video)?;
    let correct_key_pass = benchmark_task_success(task, final_player);

    Ok((
        post_key_center_frame_sha256,
        post_key_center_player,
        open_gate_frame_sha256,
        open_gate_player,
        final_player,
        pickup_changes_frame,
        correct_key_pass,
    ))
}

fn qualify_variant(
    core_path: &Path,
    rom_path: &Path,
    task: &'static BenchmarkTaskSpec,
    key_side: &str,
    wrong_side: &str,
    back_from_key: &str,
    back_from_wrong: &str,
) -> Result<(VariantEvidence, String, String), String> {
    let temp = env::temp_dir().join(format!("phicade-key-gate-pair-{}", task.id));
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
    let mut sequence = 0u64;

    no_input(
        &mut core,
        WARMUP_FRAMES,
        &mut video,
        &mut audio,
        "warm key-gate start",
    )?;
    let start_player = locate_agent_gym_player(&video)?;
    if start_player != START || start_player != task.start {
        return Err(format!(
            "{} start drifted: registry={:?} expected={:?} got={:?}",
            task.id, task.start, START, start_player
        ));
    }
    let initial_frame_sha256 = sha256_bytes(&video.rgba8);
    let start_state = core
        .serialize_state()
        .map_err(|error| format!("serialize {} start state: {error:?}", task.id))?;
    let start_mask = core.input_mask_snapshot();
    let start_frame = core.frame_count();

    let (direct_gate_player, direct_gate_refused) = direct_gate_probe(
        &mut core,
        &mut sequence,
        task,
        &mut video,
        &mut audio,
    )?;

    restore(
        &mut core,
        &start_state,
        start_frame,
        start_mask,
        &mut video,
        &mut audio,
        "restore before wrong-side probe",
    )?;

    let (wrong_side_gate_player, wrong_side_refused) = wrong_side_probe(
        &mut core,
        &mut sequence,
        task,
        wrong_side,
        back_from_wrong,
        &mut video,
        &mut audio,
    )?;

    restore(
        &mut core,
        &start_state,
        start_frame,
        start_mask,
        &mut video,
        &mut audio,
        "restore before correct-key path",
    )?;

    let (
        post_key_center_frame_sha256,
        post_key_center_player,
        open_gate_frame_sha256,
        open_gate_player,
        final_player,
        pickup_changes_frame,
        correct_key_pass,
    ) = correct_key_path(
        &mut core,
        &mut sequence,
        task,
        key_side,
        back_from_key,
        &initial_frame_sha256,
        &mut video,
        &mut audio,
    )?;

    Ok((
        VariantEvidence {
            task_id: task.id.into(),
            rom_sha256: sha256_file(rom_path)?,
            initial_frame_sha256,
            direct_gate_player,
            wrong_side_gate_player,
            post_key_center_frame_sha256,
            post_key_center_player,
            open_gate_frame_sha256,
            open_gate_player,
            final_player,
            direct_gate_refused,
            wrong_side_refused,
            pickup_changes_frame,
            correct_key_pass,
        },
        core_name,
        core_version,
    ))
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Key Gate pair qualification failed: {error}");
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

    let left_task = benchmark_task_by_id(AGENT_GYM_KEY_GATE_LEFT_ID)
        .ok_or_else(|| "Key Gate LEFT task missing from registry".to_owned())?;
    let right_task = benchmark_task_by_id(AGENT_GYM_KEY_GATE_RIGHT_ID)
        .ok_or_else(|| "Key Gate RIGHT task missing from registry".to_owned())?;

    let (left, core_name, core_version) = qualify_variant(
        &core_path,
        &left_rom,
        left_task,
        "LEFT",
        "RIGHT",
        "RIGHT",
        "LEFT",
    )?;
    let (right, right_core_name, right_core_version) = qualify_variant(
        &core_path,
        &right_rom,
        right_task,
        "RIGHT",
        "LEFT",
        "LEFT",
        "RIGHT",
    )?;

    if core_name != right_core_name || core_version != right_core_version {
        return Err("Key Gate variants did not run under identical core identity".into());
    }

    let initial_frames_differ = left.initial_frame_sha256 != right.initial_frame_sha256;
    let post_key_frames_identical =
        left.post_key_center_frame_sha256 == right.post_key_center_frame_sha256;
    let open_gate_frames_identical = left.open_gate_frame_sha256 == right.open_gate_frame_sha256;
    let direct_gate_refused_both = left.direct_gate_refused && right.direct_gate_refused;
    let wrong_side_refused_both = left.wrong_side_refused && right.wrong_side_refused;
    let pickup_changes_frame_both = left.pickup_changes_frame && right.pickup_changes_frame;
    let correct_key_pass_both = left.correct_key_pass && right.correct_key_pass;

    if !(initial_frames_differ
        && post_key_frames_identical
        && open_gate_frames_identical
        && direct_gate_refused_both
        && wrong_side_refused_both
        && pickup_changes_frame_both
        && correct_key_pass_both)
    {
        return Err(format!(
            "key-gate pair controls failed: initial_diff={initial_frames_differ} post_key_same={post_key_frames_identical} open_gate_same={open_gate_frames_identical} direct_refused={direct_gate_refused_both} wrong_side_refused={wrong_side_refused_both} pickup_changes={pickup_changes_frame_both} correct_pass={correct_key_pass_both} left_direct={:?} right_direct={:?} left_wrong={:?} right_wrong={:?} left_final={:?} right_final={:?}",
            left.direct_gate_player,
            right.direct_gate_player,
            left.wrong_side_gate_player,
            right.wrong_side_gate_player,
            left.final_player,
            right.final_player
        ));
    }

    let receipt = KeyGatePairReceipt {
        schema: "phicade.key-gate-pair-qualification.v1",
        result: "PASS",
        core_sha256: sha256_file(&core_path)?,
        core_name,
        core_version,
        left,
        right,
        initial_frames_differ,
        post_key_frames_identical,
        open_gate_frames_identical,
        direct_gate_refused_both,
        wrong_side_refused_both,
        pickup_changes_frame_both,
        correct_key_pass_both,
    };

    let json = serde_json::to_string_pretty(&receipt)
        .map_err(|error| format!("serialize Key Gate receipt: {error}"))?;
    println!("{json}");
    if let Some(parent) = receipt_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(&receipt_path, format!("{json}\n"))
        .map_err(|error| format!("write {}: {error}", receipt_path.display()))?;
    Ok(())
}
