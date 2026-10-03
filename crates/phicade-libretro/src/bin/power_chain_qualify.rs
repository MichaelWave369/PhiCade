use phicade_libretro::LibretroCore;
use phicade_runtime::{
    benchmark_task_by_id, benchmark_task_success, locate_agent_gym_player, ActionEnvelope,
    ActionKind, ActionSource, AudioBuffer, BenchmarkTaskSpec, EmulatorCore, FrameBuffer,
    GameImage, PixelPoint, SystemId, AGENT_GYM_POWER_CHAIN_LEFT_ID,
    AGENT_GYM_POWER_CHAIN_RIGHT_ID,
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
    generator_before_fuse_frame_sha256: String,
    gate_before_power_player: PixelPoint,
    empty_side_player: PixelPoint,
    empty_side_before_a_sha256: String,
    empty_side_after_a_sha256: String,
    empty_side_gate_player: PixelPoint,
    post_fuse_center_frame_sha256: String,
    powered_frame_sha256: String,
    open_gate_frame_sha256: String,
    final_player: PixelPoint,
    generator_before_fuse_refused: bool,
    gate_before_power_refused: bool,
    empty_side_pickup_refused: bool,
    empty_side_chain_refused: bool,
    fuse_pickup_changes_frame: bool,
    correct_chain_pass: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PowerChainPairReceipt {
    schema: &'static str,
    result: &'static str,
    core_sha256: String,
    core_name: String,
    core_version: String,
    left: VariantEvidence,
    right: VariantEvidence,
    initial_frames_differ: bool,
    post_fuse_frames_identical: bool,
    powered_frames_identical: bool,
    open_gate_frames_identical: bool,
    generator_before_fuse_refused_both: bool,
    gate_before_power_refused_both: bool,
    empty_side_pickup_refused_both: bool,
    empty_side_chain_refused_both: bool,
    correct_chain_pass_both: bool,
}

fn usage() -> ! {
    eprintln!(
        "usage: power_chain_qualify --core <sameboy_libretro> --left-rom <left.gb> --right-rom <right.gb> --receipt <path>"
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
            name: "power-chain-qualifier".into(),
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

fn gate_probe(
    core: &mut LibretroCore,
    sequence: &mut u64,
    task: &BenchmarkTaskSpec,
    video: &mut FrameBuffer,
    audio: &mut AudioBuffer,
    context: &str,
) -> Result<(PixelPoint, bool), String> {
    hold_button(
        core,
        sequence,
        "UP",
        TO_GATE_FRAMES + BLOCK_PROBE_FRAMES,
        video,
        audio,
        context,
    )?;
    tap_a(core, sequence, video, audio, "press A at unpowered gate")?;
    hold_button(
        core,
        sequence,
        "UP",
        BLOCK_PROBE_FRAMES,
        video,
        audio,
        "probe closed gate",
    )?;
    let player = locate_agent_gym_player(video)?;
    Ok((
        player,
        player.y == GATE_STOP.y && !benchmark_task_success(task, player),
    ))
}

fn qualify_variant(
    core_path: &Path,
    rom_path: &Path,
    task: &'static BenchmarkTaskSpec,
    fuse_side: &str,
    empty_side: &str,
    back_from_fuse: &str,
    back_from_empty: &str,
) -> Result<(VariantEvidence, String, String), String> {
    let temp = env::temp_dir().join(format!("phicade-power-chain-pair-{}", task.id));
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
        "warm power-chain start",
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

    // Wrong order #1: generator interaction before fuse must change nothing.
    tap_a(
        &mut core,
        &mut sequence,
        &mut video,
        &mut audio,
        "try generator before fuse",
    )?;
    no_input(&mut core, SETTLE_FRAMES, &mut video, &mut audio, "settle generator refusal")?;
    let generator_before_fuse_frame_sha256 = sha256_bytes(&video.rgba8);
    let generator_before_fuse_refused =
        generator_before_fuse_frame_sha256 == initial_frame_sha256;

    let (gate_before_power_player, gate_before_power_refused) = gate_probe(
        &mut core,
        &mut sequence,
        task,
        &mut video,
        &mut audio,
        "move to gate before power",
    )?;

    restore(
        &mut core,
        &start_state,
        start_frame,
        start_mask,
        &mut video,
        &mut audio,
        "restore before empty-pedestal probe",
    )?;

    // Wrong order #2: A on the empty pedestal must not create fuse state.
    hold_button(
        &mut core,
        &mut sequence,
        empty_side,
        SIDE_FRAMES,
        &mut video,
        &mut audio,
        "move to empty fuse pedestal",
    )?;
    no_input(&mut core, 1, &mut video, &mut audio, "settle empty pedestal")?;
    let empty_side_player = locate_agent_gym_player(&video)?;
    let empty_side_before_a_sha256 = sha256_bytes(&video.rgba8);
    tap_a(
        &mut core,
        &mut sequence,
        &mut video,
        &mut audio,
        "press A on empty fuse pedestal",
    )?;
    no_input(&mut core, 1, &mut video, &mut audio, "settle fake pickup")?;
    let empty_side_after_a_sha256 = sha256_bytes(&video.rgba8);
    let empty_side_pickup_refused = empty_side_before_a_sha256 == empty_side_after_a_sha256;

    hold_button(
        &mut core,
        &mut sequence,
        back_from_empty,
        SIDE_FRAMES,
        &mut video,
        &mut audio,
        "return from empty pedestal",
    )?;
    tap_a(
        &mut core,
        &mut sequence,
        &mut video,
        &mut audio,
        "try generator after fake pickup",
    )?;
    let (empty_side_gate_player, empty_side_chain_refused) = gate_probe(
        &mut core,
        &mut sequence,
        task,
        &mut video,
        &mut audio,
        "probe gate after fake pickup",
    )?;

    restore(
        &mut core,
        &start_state,
        start_frame,
        start_mask,
        &mut video,
        &mut audio,
        "restore before valid chain",
    )?;

    // Correct causal chain.
    hold_button(
        &mut core,
        &mut sequence,
        fuse_side,
        SIDE_FRAMES,
        &mut video,
        &mut audio,
        "move to real fuse",
    )?;
    tap_a(&mut core, &mut sequence, &mut video, &mut audio, "acquire fuse")?;
    hold_button(
        &mut core,
        &mut sequence,
        back_from_fuse,
        SIDE_FRAMES,
        &mut video,
        &mut audio,
        "return to generator with fuse",
    )?;
    no_input(&mut core, SETTLE_FRAMES, &mut video, &mut audio, "settle post-fuse center")?;
    let center = locate_agent_gym_player(&video)?;
    if center != START {
        return Err(format!("post-fuse center drifted: expected {:?}, got {:?}", START, center));
    }
    let post_fuse_center_frame_sha256 = sha256_bytes(&video.rgba8);
    let fuse_pickup_changes_frame = post_fuse_center_frame_sha256 != initial_frame_sha256;

    tap_a(
        &mut core,
        &mut sequence,
        &mut video,
        &mut audio,
        "install fuse in generator",
    )?;
    no_input(&mut core, SETTLE_FRAMES, &mut video, &mut audio, "settle powered generator")?;
    let powered_frame_sha256 = sha256_bytes(&video.rgba8);

    hold_button(
        &mut core,
        &mut sequence,
        "UP",
        TO_GATE_FRAMES,
        &mut video,
        &mut audio,
        "move to powered gate",
    )?;
    let gate_player = locate_agent_gym_player(&video)?;
    if gate_player != GATE_STOP {
        return Err(format!(
            "powered gate approach drifted: expected {:?}, got {:?}",
            GATE_STOP, gate_player
        ));
    }
    tap_a(
        &mut core,
        &mut sequence,
        &mut video,
        &mut audio,
        "open powered gate",
    )?;
    no_input(&mut core, SETTLE_FRAMES, &mut video, &mut audio, "settle open gate")?;
    let open_gate_frame_sha256 = sha256_bytes(&video.rgba8);

    hold_button(
        &mut core,
        &mut sequence,
        "UP",
        THROUGH_GATE_FRAMES,
        &mut video,
        &mut audio,
        "move through powered gate",
    )?;
    no_input(&mut core, SETTLE_FRAMES, &mut video, &mut audio, "settle target")?;
    let final_player = locate_agent_gym_player(&video)?;
    let correct_chain_pass = benchmark_task_success(task, final_player);

    Ok((
        VariantEvidence {
            task_id: task.id.into(),
            rom_sha256: sha256_file(rom_path)?,
            initial_frame_sha256,
            generator_before_fuse_frame_sha256,
            gate_before_power_player,
            empty_side_player,
            empty_side_before_a_sha256,
            empty_side_after_a_sha256,
            empty_side_gate_player,
            post_fuse_center_frame_sha256,
            powered_frame_sha256,
            open_gate_frame_sha256,
            final_player,
            generator_before_fuse_refused,
            gate_before_power_refused,
            empty_side_pickup_refused,
            empty_side_chain_refused,
            fuse_pickup_changes_frame,
            correct_chain_pass,
        },
        core_name,
        core_version,
    ))
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Power Chain pair qualification failed: {error}");
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

    let left_task = benchmark_task_by_id(AGENT_GYM_POWER_CHAIN_LEFT_ID)
        .ok_or_else(|| "Power Chain LEFT task missing from registry".to_owned())?;
    let right_task = benchmark_task_by_id(AGENT_GYM_POWER_CHAIN_RIGHT_ID)
        .ok_or_else(|| "Power Chain RIGHT task missing from registry".to_owned())?;

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
        return Err("Power Chain variants did not run under identical core identity".into());
    }

    let initial_frames_differ = left.initial_frame_sha256 != right.initial_frame_sha256;
    let post_fuse_frames_identical =
        left.post_fuse_center_frame_sha256 == right.post_fuse_center_frame_sha256;
    let powered_frames_identical = left.powered_frame_sha256 == right.powered_frame_sha256;
    let open_gate_frames_identical = left.open_gate_frame_sha256 == right.open_gate_frame_sha256;
    let generator_before_fuse_refused_both =
        left.generator_before_fuse_refused && right.generator_before_fuse_refused;
    let gate_before_power_refused_both =
        left.gate_before_power_refused && right.gate_before_power_refused;
    let empty_side_pickup_refused_both =
        left.empty_side_pickup_refused && right.empty_side_pickup_refused;
    let empty_side_chain_refused_both =
        left.empty_side_chain_refused && right.empty_side_chain_refused;
    let correct_chain_pass_both = left.correct_chain_pass && right.correct_chain_pass;

    if !(initial_frames_differ
        && post_fuse_frames_identical
        && powered_frames_identical
        && open_gate_frames_identical
        && generator_before_fuse_refused_both
        && gate_before_power_refused_both
        && empty_side_pickup_refused_both
        && empty_side_chain_refused_both
        && left.fuse_pickup_changes_frame
        && right.fuse_pickup_changes_frame
        && correct_chain_pass_both)
    {
        return Err(format!(
            "power-chain controls failed: initial_diff={initial_frames_differ} post_fuse_same={post_fuse_frames_identical} powered_same={powered_frames_identical} gate_same={open_gate_frames_identical} generator_refused={generator_before_fuse_refused_both} gate_refused={gate_before_power_refused_both} empty_pickup_refused={empty_side_pickup_refused_both} empty_chain_refused={empty_side_chain_refused_both} correct={correct_chain_pass_both} left_gate={:?} right_gate={:?} left_empty_gate={:?} right_empty_gate={:?} left_final={:?} right_final={:?}",
            left.gate_before_power_player,
            right.gate_before_power_player,
            left.empty_side_gate_player,
            right.empty_side_gate_player,
            left.final_player,
            right.final_player
        ));
    }

    let receipt = PowerChainPairReceipt {
        schema: "phicade.power-chain-pair-qualification.v1",
        result: "PASS",
        core_sha256: sha256_file(&core_path)?,
        core_name,
        core_version,
        left,
        right,
        initial_frames_differ,
        post_fuse_frames_identical,
        powered_frames_identical,
        open_gate_frames_identical,
        generator_before_fuse_refused_both,
        gate_before_power_refused_both,
        empty_side_pickup_refused_both,
        empty_side_chain_refused_both,
        correct_chain_pass_both,
    };

    let json = serde_json::to_string_pretty(&receipt)
        .map_err(|error| format!("serialize Power Chain receipt: {error}"))?;
    println!("{json}");
    if let Some(parent) = receipt_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(&receipt_path, format!("{json}\n"))
        .map_err(|error| format!("write {}: {error}", receipt_path.display()))?;
    Ok(())
}
