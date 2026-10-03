use phicade_libretro::LibretroCore;
use phicade_runtime::{
    benchmark_task_by_id, benchmark_task_success, locate_agent_gym_player, ActionEnvelope,
    ActionKind, ActionSource, AudioBuffer, BenchmarkTaskSpec, EmulatorCore, FrameBuffer,
    GameImage, PixelPoint, SystemId, AGENT_GYM_BRANCH_SELECTOR_SQUARE_ID,
    AGENT_GYM_BRANCH_SELECTOR_TRIANGLE_ID,
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
const GATE_STOP_Y: i32 = 80;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct VariantEvidence {
    task_id: String,
    rom_sha256: String,
    initial_frame_sha256: String,
    wrong_side_player: PixelPoint,
    wrong_before_a_sha256: String,
    wrong_after_a_sha256: String,
    failed_center_frame_sha256: String,
    failed_generator_frame_sha256: String,
    failed_gate_player: PixelPoint,
    post_selection_center_frame_sha256: String,
    powered_frame_sha256: String,
    open_gate_frame_sha256: String,
    final_player: PixelPoint,
    wrong_selection_changes_frame: bool,
    wrong_generator_refused: bool,
    wrong_branch_dead_end: bool,
    correct_selection_changes_frame: bool,
    correct_branch_pass: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BranchSelectorPairReceipt {
    schema: &'static str,
    result: &'static str,
    core_sha256: String,
    core_name: String,
    core_version: String,
    triangle: VariantEvidence,
    square: VariantEvidence,
    initial_frames_differ: bool,
    failed_center_frames_identical: bool,
    post_selection_frames_identical: bool,
    powered_frames_identical: bool,
    open_gate_frames_identical: bool,
    wrong_selection_changes_frame_both: bool,
    wrong_generator_refused_both: bool,
    wrong_branch_dead_end_both: bool,
    correct_selection_changes_frame_both: bool,
    correct_branch_pass_both: bool,
}

fn usage() -> ! {
    eprintln!(
        "usage: branch_selector_qualify --core <sameboy_libretro> --triangle-rom <triangle.gb> --square-rom <square.gb> --receipt <path>"
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
            name: "branch-selector-qualifier".into(),
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
    task: &'static BenchmarkTaskSpec,
    correct_side: &str,
    wrong_side: &str,
    back_from_correct: &str,
    back_from_wrong: &str,
) -> Result<(VariantEvidence, String, String), String> {
    let temp = env::temp_dir().join(format!("phicade-branch-selector-{}", task.id));
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
        "warm branch-selector start",
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

    // Negative control: deliberately choose the module that does not match the cue.
    hold_button(
        &mut core,
        &mut sequence,
        wrong_side,
        SIDE_FRAMES,
        &mut video,
        &mut audio,
        "move to wrong branch module",
    )?;
    no_input(&mut core, 1, &mut video, &mut audio, "settle wrong module")?;
    let wrong_side_player = locate_agent_gym_player(&video)?;
    let wrong_before_a_sha256 = sha256_bytes(&video.rgba8);
    tap_a(
        &mut core,
        &mut sequence,
        &mut video,
        &mut audio,
        "select wrong branch module",
    )?;
    no_input(
        &mut core,
        SETTLE_FRAMES,
        &mut video,
        &mut audio,
        "settle irreversible fail state",
    )?;
    let wrong_after_a_sha256 = sha256_bytes(&video.rgba8);
    let wrong_selection_changes_frame = wrong_before_a_sha256 != wrong_after_a_sha256;

    hold_button(
        &mut core,
        &mut sequence,
        back_from_wrong,
        SIDE_FRAMES,
        &mut video,
        &mut audio,
        "return from wrong branch",
    )?;
    no_input(&mut core, SETTLE_FRAMES, &mut video, &mut audio, "settle failed center")?;
    let failed_center = locate_agent_gym_player(&video)?;
    if failed_center != START {
        return Err(format!(
            "{} failed branch did not converge to center: expected {:?}, got {:?}",
            task.id, START, failed_center
        ));
    }
    let failed_center_frame_sha256 = sha256_bytes(&video.rgba8);

    tap_a(
        &mut core,
        &mut sequence,
        &mut video,
        &mut audio,
        "try generator after wrong branch",
    )?;
    no_input(&mut core, SETTLE_FRAMES, &mut video, &mut audio, "settle failed generator")?;
    let failed_generator_frame_sha256 = sha256_bytes(&video.rgba8);
    let wrong_generator_refused =
        failed_generator_frame_sha256 == failed_center_frame_sha256;

    hold_button(
        &mut core,
        &mut sequence,
        "UP",
        TO_GATE_FRAMES + BLOCK_PROBE_FRAMES,
        &mut video,
        &mut audio,
        "move failed branch to gate",
    )?;
    tap_a(
        &mut core,
        &mut sequence,
        &mut video,
        &mut audio,
        "try gate after wrong branch",
    )?;
    hold_button(
        &mut core,
        &mut sequence,
        "UP",
        BLOCK_PROBE_FRAMES,
        &mut video,
        &mut audio,
        "probe failed gate",
    )?;
    let failed_gate_player = locate_agent_gym_player(&video)?;
    let wrong_branch_dead_end =
        failed_gate_player.y == GATE_STOP_Y && !benchmark_task_success(task, failed_gate_player);

    restore(
        &mut core,
        &start_state,
        start_frame,
        start_mask,
        &mut video,
        &mut audio,
        "restore before correct branch",
    )?;

    // Positive control: use the selector cue and choose the matching module.
    hold_button(
        &mut core,
        &mut sequence,
        correct_side,
        SIDE_FRAMES,
        &mut video,
        &mut audio,
        "move to correct branch module",
    )?;
    tap_a(
        &mut core,
        &mut sequence,
        &mut video,
        &mut audio,
        "select correct branch module",
    )?;
    hold_button(
        &mut core,
        &mut sequence,
        back_from_correct,
        SIDE_FRAMES,
        &mut video,
        &mut audio,
        "return with accepted module",
    )?;
    no_input(
        &mut core,
        SETTLE_FRAMES,
        &mut video,
        &mut audio,
        "settle accepted center",
    )?;
    let correct_center = locate_agent_gym_player(&video)?;
    if correct_center != START {
        return Err(format!(
            "{} correct branch did not converge to center: expected {:?}, got {:?}",
            task.id, START, correct_center
        ));
    }
    let post_selection_center_frame_sha256 = sha256_bytes(&video.rgba8);
    let correct_selection_changes_frame =
        post_selection_center_frame_sha256 != initial_frame_sha256;

    tap_a(
        &mut core,
        &mut sequence,
        &mut video,
        &mut audio,
        "install accepted module",
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
        TO_GATE_FRAMES,
        &mut video,
        &mut audio,
        "move to powered gate",
    )?;
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
        "move through gate",
    )?;
    no_input(&mut core, SETTLE_FRAMES, &mut video, &mut audio, "settle target")?;
    let final_player = locate_agent_gym_player(&video)?;
    let correct_branch_pass = benchmark_task_success(task, final_player);

    Ok((
        VariantEvidence {
            task_id: task.id.into(),
            rom_sha256: sha256_file(rom_path)?,
            initial_frame_sha256,
            wrong_side_player,
            wrong_before_a_sha256,
            wrong_after_a_sha256,
            failed_center_frame_sha256,
            failed_generator_frame_sha256,
            failed_gate_player,
            post_selection_center_frame_sha256,
            powered_frame_sha256,
            open_gate_frame_sha256,
            final_player,
            wrong_selection_changes_frame,
            wrong_generator_refused,
            wrong_branch_dead_end,
            correct_selection_changes_frame,
            correct_branch_pass,
        },
        core_name,
        core_version,
    ))
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Branch Selector pair qualification failed: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let mut core_path: Option<PathBuf> = None;
    let mut triangle_rom: Option<PathBuf> = None;
    let mut square_rom: Option<PathBuf> = None;
    let mut receipt_path: Option<PathBuf> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--core" => core_path = args.next().map(PathBuf::from),
            "--triangle-rom" => triangle_rom = args.next().map(PathBuf::from),
            "--square-rom" => square_rom = args.next().map(PathBuf::from),
            "--receipt" => receipt_path = args.next().map(PathBuf::from),
            _ => usage(),
        }
    }

    let core_path = core_path.unwrap_or_else(|| usage());
    let triangle_rom = triangle_rom.unwrap_or_else(|| usage());
    let square_rom = square_rom.unwrap_or_else(|| usage());
    let receipt_path = receipt_path.unwrap_or_else(|| usage());

    let triangle_task = benchmark_task_by_id(AGENT_GYM_BRANCH_SELECTOR_TRIANGLE_ID)
        .ok_or_else(|| "Branch Selector TRIANGLE task missing from registry".to_owned())?;
    let square_task = benchmark_task_by_id(AGENT_GYM_BRANCH_SELECTOR_SQUARE_ID)
        .ok_or_else(|| "Branch Selector SQUARE task missing from registry".to_owned())?;

    let (triangle, core_name, core_version) = qualify_variant(
        &core_path,
        &triangle_rom,
        triangle_task,
        "LEFT",
        "RIGHT",
        "RIGHT",
        "LEFT",
    )?;
    let (square, square_core_name, square_core_version) = qualify_variant(
        &core_path,
        &square_rom,
        square_task,
        "RIGHT",
        "LEFT",
        "LEFT",
        "RIGHT",
    )?;

    if core_name != square_core_name || core_version != square_core_version {
        return Err("Branch Selector variants did not run under identical core identity".into());
    }

    let initial_frames_differ =
        triangle.initial_frame_sha256 != square.initial_frame_sha256;
    let failed_center_frames_identical =
        triangle.failed_center_frame_sha256 == square.failed_center_frame_sha256;
    let post_selection_frames_identical =
        triangle.post_selection_center_frame_sha256 == square.post_selection_center_frame_sha256;
    let powered_frames_identical =
        triangle.powered_frame_sha256 == square.powered_frame_sha256;
    let open_gate_frames_identical =
        triangle.open_gate_frame_sha256 == square.open_gate_frame_sha256;
    let wrong_selection_changes_frame_both =
        triangle.wrong_selection_changes_frame && square.wrong_selection_changes_frame;
    let wrong_generator_refused_both =
        triangle.wrong_generator_refused && square.wrong_generator_refused;
    let wrong_branch_dead_end_both =
        triangle.wrong_branch_dead_end && square.wrong_branch_dead_end;
    let correct_selection_changes_frame_both =
        triangle.correct_selection_changes_frame && square.correct_selection_changes_frame;
    let correct_branch_pass_both =
        triangle.correct_branch_pass && square.correct_branch_pass;

    if !(initial_frames_differ
        && failed_center_frames_identical
        && post_selection_frames_identical
        && powered_frames_identical
        && open_gate_frames_identical
        && wrong_selection_changes_frame_both
        && wrong_generator_refused_both
        && wrong_branch_dead_end_both
        && correct_selection_changes_frame_both
        && correct_branch_pass_both)
    {
        return Err(format!(
            "branch-selector controls failed: initial_diff={initial_frames_differ} failed_same={failed_center_frames_identical} selected_same={post_selection_frames_identical} powered_same={powered_frames_identical} gate_same={open_gate_frames_identical} wrong_changed={wrong_selection_changes_frame_both} wrong_generator_refused={wrong_generator_refused_both} dead_end={wrong_branch_dead_end_both} correct_changed={correct_selection_changes_frame_both} correct={correct_branch_pass_both} triangle_failed={:?} square_failed={:?} triangle_final={:?} square_final={:?}",
            triangle.failed_gate_player,
            square.failed_gate_player,
            triangle.final_player,
            square.final_player
        ));
    }

    let receipt = BranchSelectorPairReceipt {
        schema: "phicade.branch-selector-pair-qualification.v1",
        result: "PASS",
        core_sha256: sha256_file(&core_path)?,
        core_name,
        core_version,
        triangle,
        square,
        initial_frames_differ,
        failed_center_frames_identical,
        post_selection_frames_identical,
        powered_frames_identical,
        open_gate_frames_identical,
        wrong_selection_changes_frame_both,
        wrong_generator_refused_both,
        wrong_branch_dead_end_both,
        correct_selection_changes_frame_both,
        correct_branch_pass_both,
    };

    let json = serde_json::to_string_pretty(&receipt)
        .map_err(|error| format!("serialize Branch Selector receipt: {error}"))?;
    println!("{json}");
    if let Some(parent) = receipt_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(&receipt_path, format!("{json}\n"))
        .map_err(|error| format!("write {}: {error}", receipt_path.display()))?;

    Ok(())
}
