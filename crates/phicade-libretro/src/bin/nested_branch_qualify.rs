use phicade_libretro::LibretroCore;
use phicade_runtime::{
    benchmark_task_by_id, benchmark_task_success, locate_agent_gym_player, ActionEnvelope,
    ActionKind, ActionSource, AudioBuffer, BenchmarkTaskSpec, EmulatorCore, FrameBuffer,
    GameImage, PixelPoint, SystemId, AGENT_GYM_NESTED_SQUARE_CIRCLE_ID,
    AGENT_GYM_NESTED_SQUARE_CROSS_ID, AGENT_GYM_NESTED_TRIANGLE_CIRCLE_ID,
    AGENT_GYM_NESTED_TRIANGLE_CROSS_ID,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

const WARMUP_FRAMES: u64 = 120;
const SIDE_FRAMES: u64 = 32;
// hold_button performs a separate release step. SameBoy observes one final
// Hold durations match the canonical Agent Gym oracle. The release event
// neutralizes the button before the release frame is simulated, so it does
// not contribute an extra movement tick.
const STAGE2_UP_FRAMES: u64 = 16;
const GENERATOR_UP_FRAMES: u64 = 12;
const GATE_UP_FRAMES: u64 = 8;
const TARGET_UP_FRAMES: u64 = 20;
const BLOCK_PROBE_FRAMES: u64 = 8;
const SETTLE_FRAMES: u64 = 2;

const START: PixelPoint = PixelPoint { x: 72, y: 112 };
const STAGE2_CENTER: PixelPoint = PixelPoint { x: 72, y: 80 };
const STAGE2_CENTER_TOLERANCE: i32 = 2;
const GATE_STOP_Y: i32 = 56;

#[derive(Debug, Clone, Copy)]
struct VariantPlan {
    task_id: &'static str,
    stage1_correct: &'static str,
    stage1_wrong: &'static str,
    stage1_back_correct: &'static str,
    stage1_back_wrong: &'static str,
    stage2_correct: &'static str,
    stage2_wrong: &'static str,
    stage2_back_correct: &'static str,
    stage2_back_wrong: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct VariantEvidence {
    task_id: String,
    rom_sha256: String,
    initial_frame_sha256: String,
    failed_stage1_center_sha256: String,
    failed_stage1_gate_player: PixelPoint,
    stage2_revealed_center_sha256: String,
    failed_stage2_center_player: PixelPoint,
    failed_stage2_center_sha256: String,
    failed_stage2_generator_before_sha256: String,
    failed_stage2_generator_sha256: String,
    failed_stage2_gate_player: PixelPoint,
    accepted_stage2_center_player: PixelPoint,
    accepted_stage2_center_sha256: String,
    powered_frame_sha256: String,
    open_gate_frame_sha256: String,
    final_player: PixelPoint,
    wrong_stage1_dead_end: bool,
    wrong_stage2_generator_refused: bool,
    wrong_stage2_dead_end: bool,
    correct_branch_pass: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct NestedBranchQualificationReceipt {
    schema: &'static str,
    result: &'static str,
    core_sha256: String,
    core_name: String,
    core_version: String,
    triangle_circle: VariantEvidence,
    triangle_cross: VariantEvidence,
    square_circle: VariantEvidence,
    square_cross: VariantEvidence,
    stage2_hidden_before_stage1: bool,
    stage1_family_visible: bool,
    circle_stage2_converges_across_stage1_history: bool,
    cross_stage2_converges_across_stage1_history: bool,
    stage2_conditions_are_distinct: bool,
    stage1_fail_converges_all: bool,
    stage2_fail_converges_all: bool,
    accepted_stage2_converges_all: bool,
    powered_converges_all: bool,
    open_gate_converges_all: bool,
    both_failure_depths_dead_end: bool,
    all_correct_paths_pass: bool,
}

fn point_within_tolerance(point: PixelPoint, expected: PixelPoint, tolerance: i32) -> bool {
    (point.x - expected.x).abs() <= tolerance
        && (point.y - expected.y).abs() <= tolerance
}

fn usage() -> ! {
    eprintln!(
        "usage: nested_branch_qualify --core <sameboy_libretro> --triangle-circle-rom <rom> --triangle-cross-rom <rom> --square-circle-rom <rom> --square-cross-rom <rom> --receipt <path>"
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
            name: "nested-branch-qualifier".into(),
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

fn qualify_variant(
    core_path: &Path,
    rom_path: &Path,
    plan: VariantPlan,
) -> Result<(VariantEvidence, String, String), String> {
    let task: &'static BenchmarkTaskSpec = benchmark_task_by_id(plan.task_id)
        .ok_or_else(|| format!("{} missing from benchmark registry", plan.task_id))?;

    let temp = env::temp_dir().join(format!("phicade-nested-branch-{}", task.id));
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
        "warm nested-branch start",
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

    // Negative depth 1: wrong family must commit the whole run to failure.
    hold_button(
        &mut core,
        &mut sequence,
        plan.stage1_wrong,
        SIDE_FRAMES,
        &mut video,
        &mut audio,
        "move to wrong stage1 family",
    )?;
    tap_a(
        &mut core,
        &mut sequence,
        &mut video,
        &mut audio,
        "commit wrong stage1 family",
    )?;
    hold_button(
        &mut core,
        &mut sequence,
        plan.stage1_back_wrong,
        SIDE_FRAMES,
        &mut video,
        &mut audio,
        "return from wrong stage1 family",
    )?;
    no_input(
        &mut core,
        SETTLE_FRAMES,
        &mut video,
        &mut audio,
        "settle stage1 fail center",
    )?;
    let failed_stage1_center = locate_agent_gym_player(&video)?;
    if failed_stage1_center != START {
        return Err(format!(
            "{} stage1 fail did not return to center: {:?}",
            task.id, failed_stage1_center
        ));
    }
    let failed_stage1_center_sha256 = sha256_bytes(&video.rgba8);

    hold_button(
        &mut core,
        &mut sequence,
        "UP",
        40,
        &mut video,
        &mut audio,
        "probe gate after stage1 failure",
    )?;
    tap_a(
        &mut core,
        &mut sequence,
        &mut video,
        &mut audio,
        "try gate after stage1 failure",
    )?;
    hold_button(
        &mut core,
        &mut sequence,
        "UP",
        BLOCK_PROBE_FRAMES,
        &mut video,
        &mut audio,
        "probe closed gate after stage1 failure",
    )?;
    let failed_stage1_gate_player = locate_agent_gym_player(&video)?;
    let wrong_stage1_dead_end = failed_stage1_gate_player.y == GATE_STOP_Y
        && !benchmark_task_success(task, failed_stage1_gate_player);

    restore(
        &mut core,
        &start_state,
        start_frame,
        start_mask,
        &mut video,
        &mut audio,
        "restore before correct stage1",
    )?;

    // Positive depth 1: correct family reveals a new, previously hidden selector.
    hold_button(
        &mut core,
        &mut sequence,
        plan.stage1_correct,
        SIDE_FRAMES,
        &mut video,
        &mut audio,
        "move to correct stage1 family",
    )?;
    tap_a(
        &mut core,
        &mut sequence,
        &mut video,
        &mut audio,
        "commit correct stage1 family",
    )?;
    hold_button(
        &mut core,
        &mut sequence,
        plan.stage1_back_correct,
        SIDE_FRAMES,
        &mut video,
        &mut audio,
        "return from correct stage1 family",
    )?;
    no_input(
        &mut core,
        SETTLE_FRAMES,
        &mut video,
        &mut audio,
        "settle revealed stage2",
    )?;
    let stage2_center = locate_agent_gym_player(&video)?;
    if stage2_center != START {
        return Err(format!(
            "{} stage2 reveal did not settle at stage1 center: {:?}",
            task.id, stage2_center
        ));
    }
    let stage2_revealed_center_sha256 = sha256_bytes(&video.rgba8);
    let stage1_state = core
        .serialize_state()
        .map_err(|error| format!("serialize {} stage1 state: {error:?}", task.id))?;
    let stage1_mask = core.input_mask_snapshot();
    let stage1_frame = core.frame_count();

    // Negative depth 2: wrong submodule must also be terminal.
    hold_button(
        &mut core,
        &mut sequence,
        "UP",
        STAGE2_UP_FRAMES,
        &mut video,
        &mut audio,
        "enter stage2 row",
    )?;
    hold_button(
        &mut core,
        &mut sequence,
        plan.stage2_wrong,
        SIDE_FRAMES,
        &mut video,
        &mut audio,
        "move to wrong stage2 submodule",
    )?;
    tap_a(
        &mut core,
        &mut sequence,
        &mut video,
        &mut audio,
        "commit wrong stage2 submodule",
    )?;
    hold_button(
        &mut core,
        &mut sequence,
        plan.stage2_back_wrong,
        SIDE_FRAMES,
        &mut video,
        &mut audio,
        "return from wrong stage2 submodule",
    )?;
    no_input(
        &mut core,
        SETTLE_FRAMES,
        &mut video,
        &mut audio,
        "settle stage2 fail center",
    )?;
    let failed_stage2_center = locate_agent_gym_player(&video)?;
    if !point_within_tolerance(
        failed_stage2_center,
        STAGE2_CENTER,
        STAGE2_CENTER_TOLERANCE,
    ) {
        return Err(format!(
            "{} stage2 fail left the center neighborhood: expected {:?} ±{}px, got {:?}",
            task.id, STAGE2_CENTER, STAGE2_CENTER_TOLERANCE, failed_stage2_center
        ));
    }
    let failed_stage2_center_sha256 = sha256_bytes(&video.rgba8);

    hold_button(
        &mut core,
        &mut sequence,
        "UP",
        GENERATOR_UP_FRAMES,
        &mut video,
        &mut audio,
        "move failed stage2 to generator",
    )?;
    no_input(
        &mut core,
        SETTLE_FRAMES,
        &mut video,
        &mut audio,
        "settle at failed generator",
    )?;
    let failed_stage2_generator_before_sha256 = sha256_bytes(&video.rgba8);
    tap_a(
        &mut core,
        &mut sequence,
        &mut video,
        &mut audio,
        "try generator after stage2 failure",
    )?;
    no_input(
        &mut core,
        SETTLE_FRAMES,
        &mut video,
        &mut audio,
        "settle refused failed generator",
    )?;
    let failed_stage2_generator_sha256 = sha256_bytes(&video.rgba8);
    let wrong_stage2_generator_refused =
        failed_stage2_generator_sha256 == failed_stage2_generator_before_sha256;

    hold_button(
        &mut core,
        &mut sequence,
        "UP",
        GATE_UP_FRAMES + BLOCK_PROBE_FRAMES,
        &mut video,
        &mut audio,
        "move failed stage2 to gate",
    )?;
    tap_a(
        &mut core,
        &mut sequence,
        &mut video,
        &mut audio,
        "try gate after stage2 failure",
    )?;
    hold_button(
        &mut core,
        &mut sequence,
        "UP",
        BLOCK_PROBE_FRAMES,
        &mut video,
        &mut audio,
        "probe failed stage2 gate",
    )?;
    let failed_stage2_gate_player = locate_agent_gym_player(&video)?;
    let wrong_stage2_dead_end = failed_stage2_gate_player.y == GATE_STOP_Y
        && !benchmark_task_success(task, failed_stage2_gate_player);

    restore(
        &mut core,
        &stage1_state,
        stage1_frame,
        stage1_mask,
        &mut video,
        &mut audio,
        "restore before correct stage2",
    )?;

    // Positive depth 2: correct submodule collapses all four variants together.
    hold_button(
        &mut core,
        &mut sequence,
        "UP",
        STAGE2_UP_FRAMES,
        &mut video,
        &mut audio,
        "enter correct stage2 row",
    )?;
    hold_button(
        &mut core,
        &mut sequence,
        plan.stage2_correct,
        SIDE_FRAMES,
        &mut video,
        &mut audio,
        "move to correct stage2 submodule",
    )?;
    tap_a(
        &mut core,
        &mut sequence,
        &mut video,
        &mut audio,
        "commit correct stage2 submodule",
    )?;
    hold_button(
        &mut core,
        &mut sequence,
        plan.stage2_back_correct,
        SIDE_FRAMES,
        &mut video,
        &mut audio,
        "return from correct stage2 submodule",
    )?;
    no_input(
        &mut core,
        SETTLE_FRAMES,
        &mut video,
        &mut audio,
        "settle accepted stage2 center",
    )?;
    let accepted_stage2_center = locate_agent_gym_player(&video)?;
    if !point_within_tolerance(
        accepted_stage2_center,
        STAGE2_CENTER,
        STAGE2_CENTER_TOLERANCE,
    ) {
        return Err(format!(
            "{} accepted stage2 left the center neighborhood: expected {:?} ±{}px, got {:?}",
            task.id, STAGE2_CENTER, STAGE2_CENTER_TOLERANCE, accepted_stage2_center
        ));
    }
    let accepted_stage2_center_sha256 = sha256_bytes(&video.rgba8);

    hold_button(
        &mut core,
        &mut sequence,
        "UP",
        GENERATOR_UP_FRAMES,
        &mut video,
        &mut audio,
        "move accepted stage2 to generator",
    )?;
    tap_a(
        &mut core,
        &mut sequence,
        &mut video,
        &mut audio,
        "power shared generator",
    )?;
    no_input(
        &mut core,
        SETTLE_FRAMES,
        &mut video,
        &mut audio,
        "settle powered generator",
    )?;
    let powered_frame_sha256 = sha256_bytes(&video.rgba8);

    hold_button(
        &mut core,
        &mut sequence,
        "UP",
        GATE_UP_FRAMES,
        &mut video,
        &mut audio,
        "move to shared gate",
    )?;
    tap_a(
        &mut core,
        &mut sequence,
        &mut video,
        &mut audio,
        "open shared gate",
    )?;
    no_input(
        &mut core,
        SETTLE_FRAMES,
        &mut video,
        &mut audio,
        "settle open gate",
    )?;
    let open_gate_frame_sha256 = sha256_bytes(&video.rgba8);

    hold_button(
        &mut core,
        &mut sequence,
        "UP",
        TARGET_UP_FRAMES,
        &mut video,
        &mut audio,
        "reach nested-branch target",
    )?;
    no_input(&mut core, SETTLE_FRAMES, &mut video, &mut audio, "settle target")?;
    let final_player = locate_agent_gym_player(&video)?;
    let correct_branch_pass = benchmark_task_success(task, final_player);

    Ok((
        VariantEvidence {
            task_id: task.id.into(),
            rom_sha256: sha256_file(rom_path)?,
            initial_frame_sha256,
            failed_stage1_center_sha256,
            failed_stage1_gate_player,
            stage2_revealed_center_sha256,
            failed_stage2_center_player: failed_stage2_center,
            failed_stage2_center_sha256,
            failed_stage2_generator_before_sha256,
            failed_stage2_generator_sha256,
            failed_stage2_gate_player,
            accepted_stage2_center_player: accepted_stage2_center,
            accepted_stage2_center_sha256,
            powered_frame_sha256,
            open_gate_frame_sha256,
            final_player,
            wrong_stage1_dead_end,
            wrong_stage2_generator_refused,
            wrong_stage2_dead_end,
            correct_branch_pass,
        },
        core_name,
        core_version,
    ))
}

fn all_equal(values: [&str; 4]) -> bool {
    values[1..].iter().all(|candidate| *candidate == values[0])
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Nested Branch Graph qualification failed: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let mut core_path: Option<PathBuf> = None;
    let mut triangle_circle_rom: Option<PathBuf> = None;
    let mut triangle_cross_rom: Option<PathBuf> = None;
    let mut square_circle_rom: Option<PathBuf> = None;
    let mut square_cross_rom: Option<PathBuf> = None;
    let mut receipt_path: Option<PathBuf> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--core" => core_path = args.next().map(PathBuf::from),
            "--triangle-circle-rom" => triangle_circle_rom = args.next().map(PathBuf::from),
            "--triangle-cross-rom" => triangle_cross_rom = args.next().map(PathBuf::from),
            "--square-circle-rom" => square_circle_rom = args.next().map(PathBuf::from),
            "--square-cross-rom" => square_cross_rom = args.next().map(PathBuf::from),
            "--receipt" => receipt_path = args.next().map(PathBuf::from),
            _ => usage(),
        }
    }

    let core_path = core_path.unwrap_or_else(|| usage());
    let triangle_circle_rom = triangle_circle_rom.unwrap_or_else(|| usage());
    let triangle_cross_rom = triangle_cross_rom.unwrap_or_else(|| usage());
    let square_circle_rom = square_circle_rom.unwrap_or_else(|| usage());
    let square_cross_rom = square_cross_rom.unwrap_or_else(|| usage());
    let receipt_path = receipt_path.unwrap_or_else(|| usage());

    let (triangle_circle, core_name, core_version) = qualify_variant(
        &core_path,
        &triangle_circle_rom,
        VariantPlan {
            task_id: AGENT_GYM_NESTED_TRIANGLE_CIRCLE_ID,
            stage1_correct: "LEFT",
            stage1_wrong: "RIGHT",
            stage1_back_correct: "RIGHT",
            stage1_back_wrong: "LEFT",
            stage2_correct: "LEFT",
            stage2_wrong: "RIGHT",
            stage2_back_correct: "RIGHT",
            stage2_back_wrong: "LEFT",
        },
    )?;
    let (triangle_cross, n1, v1) = qualify_variant(
        &core_path,
        &triangle_cross_rom,
        VariantPlan {
            task_id: AGENT_GYM_NESTED_TRIANGLE_CROSS_ID,
            stage1_correct: "LEFT",
            stage1_wrong: "RIGHT",
            stage1_back_correct: "RIGHT",
            stage1_back_wrong: "LEFT",
            stage2_correct: "RIGHT",
            stage2_wrong: "LEFT",
            stage2_back_correct: "LEFT",
            stage2_back_wrong: "RIGHT",
        },
    )?;
    let (square_circle, n2, v2) = qualify_variant(
        &core_path,
        &square_circle_rom,
        VariantPlan {
            task_id: AGENT_GYM_NESTED_SQUARE_CIRCLE_ID,
            stage1_correct: "RIGHT",
            stage1_wrong: "LEFT",
            stage1_back_correct: "LEFT",
            stage1_back_wrong: "RIGHT",
            stage2_correct: "LEFT",
            stage2_wrong: "RIGHT",
            stage2_back_correct: "RIGHT",
            stage2_back_wrong: "LEFT",
        },
    )?;
    let (square_cross, n3, v3) = qualify_variant(
        &core_path,
        &square_cross_rom,
        VariantPlan {
            task_id: AGENT_GYM_NESTED_SQUARE_CROSS_ID,
            stage1_correct: "RIGHT",
            stage1_wrong: "LEFT",
            stage1_back_correct: "LEFT",
            stage1_back_wrong: "RIGHT",
            stage2_correct: "RIGHT",
            stage2_wrong: "LEFT",
            stage2_back_correct: "LEFT",
            stage2_back_wrong: "RIGHT",
        },
    )?;

    if [(&n1, &v1), (&n2, &v2), (&n3, &v3)]
        .iter()
        .any(|(name, version)| {
            name.as_str() != core_name.as_str() || version.as_str() != core_version.as_str()
        })
    {
        return Err("nested-branch variants did not run under identical core identity".into());
    }

    let stage2_hidden_before_stage1 =
        triangle_circle.initial_frame_sha256 == triangle_cross.initial_frame_sha256
            && square_circle.initial_frame_sha256 == square_cross.initial_frame_sha256;
    let stage1_family_visible =
        triangle_circle.initial_frame_sha256 != square_circle.initial_frame_sha256;

    let circle_stage2_converges_across_stage1_history =
        triangle_circle.stage2_revealed_center_sha256
            == square_circle.stage2_revealed_center_sha256;
    let cross_stage2_converges_across_stage1_history =
        triangle_cross.stage2_revealed_center_sha256
            == square_cross.stage2_revealed_center_sha256;
    let stage2_conditions_are_distinct =
        triangle_circle.stage2_revealed_center_sha256
            != triangle_cross.stage2_revealed_center_sha256;

    let stage1_fail_converges_all = all_equal([
        &triangle_circle.failed_stage1_center_sha256,
        &triangle_cross.failed_stage1_center_sha256,
        &square_circle.failed_stage1_center_sha256,
        &square_cross.failed_stage1_center_sha256,
    ]);
    let stage2_fail_converges_all = all_equal([
        &triangle_circle.failed_stage2_center_sha256,
        &triangle_cross.failed_stage2_center_sha256,
        &square_circle.failed_stage2_center_sha256,
        &square_cross.failed_stage2_center_sha256,
    ]);
    let accepted_stage2_converges_all = all_equal([
        &triangle_circle.accepted_stage2_center_sha256,
        &triangle_cross.accepted_stage2_center_sha256,
        &square_circle.accepted_stage2_center_sha256,
        &square_cross.accepted_stage2_center_sha256,
    ]);
    let powered_converges_all = all_equal([
        &triangle_circle.powered_frame_sha256,
        &triangle_cross.powered_frame_sha256,
        &square_circle.powered_frame_sha256,
        &square_cross.powered_frame_sha256,
    ]);
    let open_gate_converges_all = all_equal([
        &triangle_circle.open_gate_frame_sha256,
        &triangle_cross.open_gate_frame_sha256,
        &square_circle.open_gate_frame_sha256,
        &square_cross.open_gate_frame_sha256,
    ]);

    let both_failure_depths_dead_end = [
        &triangle_circle,
        &triangle_cross,
        &square_circle,
        &square_cross,
    ]
    .iter()
    .all(|evidence| {
        evidence.wrong_stage1_dead_end
            && evidence.wrong_stage2_generator_refused
            && evidence.wrong_stage2_dead_end
    });
    let all_correct_paths_pass = [
        &triangle_circle,
        &triangle_cross,
        &square_circle,
        &square_cross,
    ]
    .iter()
    .all(|evidence| evidence.correct_branch_pass);

    if !(stage2_hidden_before_stage1
        && stage1_family_visible
        && circle_stage2_converges_across_stage1_history
        && cross_stage2_converges_across_stage1_history
        && stage2_conditions_are_distinct
        && stage1_fail_converges_all
        && stage2_fail_converges_all
        && accepted_stage2_converges_all
        && powered_converges_all
        && open_gate_converges_all
        && both_failure_depths_dead_end
        && all_correct_paths_pass)
    {
        return Err(format!(
            "nested-branch controls failed: hidden={stage2_hidden_before_stage1} stage1_visible={stage1_family_visible} circle_rejoin={circle_stage2_converges_across_stage1_history} cross_rejoin={cross_stage2_converges_across_stage1_history} stage2_distinct={stage2_conditions_are_distinct} fail1={stage1_fail_converges_all} fail2={stage2_fail_converges_all} accepted={accepted_stage2_converges_all} powered={powered_converges_all} gate={open_gate_converges_all} dead_end={both_failure_depths_dead_end} pass={all_correct_paths_pass}; tc=[fail={:?} accepted={:?} final={:?} failHash={} acceptedHash={} poweredHash={} gateHash={}] tx=[fail={:?} accepted={:?} final={:?} failHash={} acceptedHash={} poweredHash={} gateHash={}] sc=[fail={:?} accepted={:?} final={:?} failHash={} acceptedHash={} poweredHash={} gateHash={}] sx=[fail={:?} accepted={:?} final={:?} failHash={} acceptedHash={} poweredHash={} gateHash={}]",
            triangle_circle.failed_stage2_center_player,
            triangle_circle.accepted_stage2_center_player,
            triangle_circle.final_player,
            triangle_circle.failed_stage2_center_sha256,
            triangle_circle.accepted_stage2_center_sha256,
            triangle_circle.powered_frame_sha256,
            triangle_circle.open_gate_frame_sha256,
            triangle_cross.failed_stage2_center_player,
            triangle_cross.accepted_stage2_center_player,
            triangle_cross.final_player,
            triangle_cross.failed_stage2_center_sha256,
            triangle_cross.accepted_stage2_center_sha256,
            triangle_cross.powered_frame_sha256,
            triangle_cross.open_gate_frame_sha256,
            square_circle.failed_stage2_center_player,
            square_circle.accepted_stage2_center_player,
            square_circle.final_player,
            square_circle.failed_stage2_center_sha256,
            square_circle.accepted_stage2_center_sha256,
            square_circle.powered_frame_sha256,
            square_circle.open_gate_frame_sha256,
            square_cross.failed_stage2_center_player,
            square_cross.accepted_stage2_center_player,
            square_cross.final_player,
            square_cross.failed_stage2_center_sha256,
            square_cross.accepted_stage2_center_sha256,
            square_cross.powered_frame_sha256,
            square_cross.open_gate_frame_sha256,
        ));
    }

    let receipt = NestedBranchQualificationReceipt {
        schema: "phicade.nested-branch-qualification.v1",
        result: "PASS",
        core_sha256: sha256_file(&core_path)?,
        core_name,
        core_version,
        triangle_circle,
        triangle_cross,
        square_circle,
        square_cross,
        stage2_hidden_before_stage1,
        stage1_family_visible,
        circle_stage2_converges_across_stage1_history,
        cross_stage2_converges_across_stage1_history,
        stage2_conditions_are_distinct,
        stage1_fail_converges_all,
        stage2_fail_converges_all,
        accepted_stage2_converges_all,
        powered_converges_all,
        open_gate_converges_all,
        both_failure_depths_dead_end,
        all_correct_paths_pass,
    };

    let json = serde_json::to_string_pretty(&receipt)
        .map_err(|error| format!("serialize Nested Branch receipt: {error}"))?;
    println!("{json}");
    if let Some(parent) = receipt_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(&receipt_path, format!("{json}\n"))
        .map_err(|error| format!("write {}: {error}", receipt_path.display()))?;

    Ok(())
}
