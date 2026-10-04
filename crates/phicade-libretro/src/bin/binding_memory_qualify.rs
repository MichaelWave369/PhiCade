use phicade_libretro::LibretroCore;
use phicade_runtime::{
    benchmark_task_by_id, benchmark_task_success, locate_agent_gym_player, ActionEnvelope,
    ActionKind, ActionSource, AudioBuffer, BenchmarkTaskSpec, EmulatorCore, FrameBuffer,
    GameImage, PixelPoint, SystemId, AGENT_GYM_BINDING_NORMAL_SQUARE_ID,
    AGENT_GYM_BINDING_NORMAL_TRIANGLE_ID, AGENT_GYM_BINDING_SWAPPED_SQUARE_ID,
    AGENT_GYM_BINDING_SWAPPED_TRIANGLE_ID,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

const WARMUP_FRAMES: u64 = 120;
const WAIT_AFTER_BRIEF_FRAMES: u64 = 101;
const CHOICE_MOVE_FRAMES: u64 = 24;
const SETTLE_FRAMES: u64 = 4;
const START: PixelPoint = PixelPoint { x: 72, y: 96 };

#[derive(Debug, Clone, Copy)]
struct VariantPlan {
    task_id: &'static str,
    correct: &'static str,
    wrong: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct VariantEvidence {
    task_id: String,
    rom_sha256: String,
    briefing_frame_sha256: String,
    choice_frame_sha256: String,
    briefing_player: PixelPoint,
    choice_player: PixelPoint,
    correct_final_player: PixelPoint,
    wrong_final_player: PixelPoint,
    post_recovery_probe_player: PixelPoint,
    correct_path_pass: bool,
    wrong_commit_refused: bool,
    wrong_commit_terminal: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BindingMemoryQualificationReceipt {
    schema: &'static str,
    result: &'static str,
    core_sha256: String,
    core_name: String,
    core_version: String,
    normal_triangle: VariantEvidence,
    normal_square: VariantEvidence,
    swapped_triangle: VariantEvidence,
    swapped_square: VariantEvidence,
    normal_briefing_query_independent: bool,
    swapped_briefing_query_independent: bool,
    arrangements_visibly_distinct: bool,
    triangle_choice_converges_across_arrangements: bool,
    square_choice_converges_across_arrangements: bool,
    query_symbols_visibly_distinct: bool,
    all_choice_geometry_identical: bool,
    all_correct_paths_pass: bool,
    all_wrong_commits_terminal: bool,
    fixed_left_succeeds_exactly_two: bool,
    fixed_right_succeeds_exactly_two: bool,
}

fn usage() -> ! {
    eprintln!(
        "usage: binding_memory_qualify --core <sameboy_libretro> --normal-triangle-rom <a.gb> --normal-square-rom <b.gb> --swapped-triangle-rom <c.gb> --swapped-square-rom <d.gb> --receipt <path>"
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
            name: "binding-memory-qualifier".into(),
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

fn enter_choice(
    core: &mut LibretroCore,
    sequence: &mut u64,
    video: &mut FrameBuffer,
    audio: &mut AudioBuffer,
) -> Result<(), String> {
    tap_a(core, sequence, video, audio, "dismiss binding briefing")?;
    no_input(
        core,
        WAIT_AFTER_BRIEF_FRAMES,
        video,
        audio,
        "wait through binding lockout",
    )?;
    // One explicit neutral choice frame arms the D-pad after lockout.
    no_input(core, 1, video, audio, "arm binding choice")?;
    Ok(())
}

fn choose_and_commit(
    core: &mut LibretroCore,
    sequence: &mut u64,
    direction: &str,
    video: &mut FrameBuffer,
    audio: &mut AudioBuffer,
    context: &str,
) -> Result<PixelPoint, String> {
    hold_button(
        core,
        sequence,
        direction,
        CHOICE_MOVE_FRAMES,
        video,
        audio,
        context,
    )?;
    tap_a(core, sequence, video, audio, "commit binding door")?;
    no_input(core, SETTLE_FRAMES, video, audio, "settle binding commitment")?;
    locate_agent_gym_player(video)
}

fn qualify_variant(
    core_path: &Path,
    rom_path: &Path,
    plan: VariantPlan,
) -> Result<(VariantEvidence, String, String), String> {
    let task: &'static BenchmarkTaskSpec = benchmark_task_by_id(plan.task_id)
        .ok_or_else(|| format!("{} missing from benchmark registry", plan.task_id))?;

    let temp = env::temp_dir().join(format!("phicade-binding-memory-{}", task.id));
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
        "warm binding briefing",
    )?;
    let briefing_player = locate_agent_gym_player(&video)?;
    if briefing_player != START || briefing_player != task.start {
        return Err(format!(
            "{} briefing start drifted: expected {:?}, got {:?}",
            task.id, task.start, briefing_player
        ));
    }
    let briefing_frame_sha256 = sha256_bytes(&video.rgba8);

    enter_choice(&mut core, &mut sequence, &mut video, &mut audio)?;
    let choice_player = locate_agent_gym_player(&video)?;
    if choice_player != task.start {
        return Err(format!(
            "{} moved before choice: expected {:?}, got {:?}",
            task.id, task.start, choice_player
        ));
    }
    let choice_frame_sha256 = sha256_bytes(&video.rgba8);
    let choice_state = core
        .serialize_state()
        .map_err(|error| format!("serialize {} choice state: {error:?}", task.id))?;
    let choice_mask = core.input_mask_snapshot();
    let choice_frame = core.frame_count();

    let correct_final_player = choose_and_commit(
        &mut core,
        &mut sequence,
        plan.correct,
        &mut video,
        &mut audio,
        "move to correct binding door",
    )?;
    let correct_path_pass = benchmark_task_success(task, correct_final_player);

    restore(
        &mut core,
        &choice_state,
        choice_frame,
        choice_mask,
        &mut video,
        &mut audio,
        "restore before wrong binding commitment",
    )?;
    let wrong_final_player = choose_and_commit(
        &mut core,
        &mut sequence,
        plan.wrong,
        &mut video,
        &mut audio,
        "move to wrong binding door",
    )?;
    let wrong_commit_refused = !benchmark_task_success(task, wrong_final_player);

    // Try to recover after terminal failure. The ROM must ignore this.
    hold_button(
        &mut core,
        &mut sequence,
        plan.correct,
        CHOICE_MOVE_FRAMES,
        &mut video,
        &mut audio,
        "probe recovery after wrong binding commitment",
    )?;
    tap_a(
        &mut core,
        &mut sequence,
        &mut video,
        &mut audio,
        "probe correct A after binding failure",
    )?;
    no_input(
        &mut core,
        SETTLE_FRAMES,
        &mut video,
        &mut audio,
        "settle binding recovery probe",
    )?;
    let post_recovery_probe_player = locate_agent_gym_player(&video)?;
    let wrong_commit_terminal =
        post_recovery_probe_player == wrong_final_player
            && !benchmark_task_success(task, post_recovery_probe_player);

    Ok((
        VariantEvidence {
            task_id: task.id.into(),
            rom_sha256: sha256_file(rom_path)?,
            briefing_frame_sha256,
            choice_frame_sha256,
            briefing_player,
            choice_player,
            correct_final_player,
            wrong_final_player,
            post_recovery_probe_player,
            correct_path_pass,
            wrong_commit_refused,
            wrong_commit_terminal,
        },
        core_name,
        core_version,
    ))
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Binding Memory qualification failed: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let mut core_path: Option<PathBuf> = None;
    let mut nt_rom: Option<PathBuf> = None;
    let mut ns_rom: Option<PathBuf> = None;
    let mut st_rom: Option<PathBuf> = None;
    let mut ss_rom: Option<PathBuf> = None;
    let mut receipt_path: Option<PathBuf> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--core" => core_path = args.next().map(PathBuf::from),
            "--normal-triangle-rom" => nt_rom = args.next().map(PathBuf::from),
            "--normal-square-rom" => ns_rom = args.next().map(PathBuf::from),
            "--swapped-triangle-rom" => st_rom = args.next().map(PathBuf::from),
            "--swapped-square-rom" => ss_rom = args.next().map(PathBuf::from),
            "--receipt" => receipt_path = args.next().map(PathBuf::from),
            _ => usage(),
        }
    }

    let core_path = core_path.unwrap_or_else(|| usage());
    let nt_rom = nt_rom.unwrap_or_else(|| usage());
    let ns_rom = ns_rom.unwrap_or_else(|| usage());
    let st_rom = st_rom.unwrap_or_else(|| usage());
    let ss_rom = ss_rom.unwrap_or_else(|| usage());
    let receipt_path = receipt_path.unwrap_or_else(|| usage());

    let (normal_triangle, core_name, core_version) = qualify_variant(
        &core_path,
        &nt_rom,
        VariantPlan {
            task_id: AGENT_GYM_BINDING_NORMAL_TRIANGLE_ID,
            correct: "LEFT",
            wrong: "RIGHT",
        },
    )?;
    let (normal_square, nscn, nscv) = qualify_variant(
        &core_path,
        &ns_rom,
        VariantPlan {
            task_id: AGENT_GYM_BINDING_NORMAL_SQUARE_ID,
            correct: "RIGHT",
            wrong: "LEFT",
        },
    )?;
    let (swapped_triangle, stcn, stcv) = qualify_variant(
        &core_path,
        &st_rom,
        VariantPlan {
            task_id: AGENT_GYM_BINDING_SWAPPED_TRIANGLE_ID,
            correct: "RIGHT",
            wrong: "LEFT",
        },
    )?;
    let (swapped_square, sscn, sscv) = qualify_variant(
        &core_path,
        &ss_rom,
        VariantPlan {
            task_id: AGENT_GYM_BINDING_SWAPPED_SQUARE_ID,
            correct: "LEFT",
            wrong: "RIGHT",
        },
    )?;

    if [(&nscn, &nscv), (&stcn, &stcv), (&sscn, &sscv)]
        .iter()
        .any(|(name, version)| **name != core_name || **version != core_version)
    {
        return Err("binding variants did not run under identical core identity".into());
    }

    let normal_briefing_query_independent =
        normal_triangle.briefing_frame_sha256 == normal_square.briefing_frame_sha256;
    let swapped_briefing_query_independent =
        swapped_triangle.briefing_frame_sha256 == swapped_square.briefing_frame_sha256;
    let arrangements_visibly_distinct =
        normal_triangle.briefing_frame_sha256 != swapped_triangle.briefing_frame_sha256;
    let triangle_choice_converges_across_arrangements =
        normal_triangle.choice_frame_sha256 == swapped_triangle.choice_frame_sha256;
    let square_choice_converges_across_arrangements =
        normal_square.choice_frame_sha256 == swapped_square.choice_frame_sha256;
    let query_symbols_visibly_distinct =
        normal_triangle.choice_frame_sha256 != normal_square.choice_frame_sha256;
    let all_choice_geometry_identical = [
        normal_triangle.choice_player,
        normal_square.choice_player,
        swapped_triangle.choice_player,
        swapped_square.choice_player,
    ]
    .iter()
    .all(|point| *point == START);
    let all_correct_paths_pass = [
        normal_triangle.correct_path_pass,
        normal_square.correct_path_pass,
        swapped_triangle.correct_path_pass,
        swapped_square.correct_path_pass,
    ]
    .iter()
    .all(|value| *value);
    let all_wrong_commits_terminal = [
        normal_triangle.wrong_commit_refused && normal_triangle.wrong_commit_terminal,
        normal_square.wrong_commit_refused && normal_square.wrong_commit_terminal,
        swapped_triangle.wrong_commit_refused && swapped_triangle.wrong_commit_terminal,
        swapped_square.wrong_commit_refused && swapped_square.wrong_commit_terminal,
    ]
    .iter()
    .all(|value| *value);

    // By construction, left is correct for NORMAL/TRIANGLE + SWAPPED/SQUARE;
    // right is correct for NORMAL/SQUARE + SWAPPED/TRIANGLE.
    let fixed_left_succeeds_exactly_two =
        normal_triangle.correct_path_pass
            && swapped_square.correct_path_pass
            && normal_square.wrong_commit_refused
            && swapped_triangle.wrong_commit_refused;
    let fixed_right_succeeds_exactly_two =
        normal_square.correct_path_pass
            && swapped_triangle.correct_path_pass
            && normal_triangle.wrong_commit_refused
            && swapped_square.wrong_commit_refused;

    if !(normal_briefing_query_independent
        && swapped_briefing_query_independent
        && arrangements_visibly_distinct
        && triangle_choice_converges_across_arrangements
        && square_choice_converges_across_arrangements
        && query_symbols_visibly_distinct
        && all_choice_geometry_identical
        && all_correct_paths_pass
        && all_wrong_commits_terminal
        && fixed_left_succeeds_exactly_two
        && fixed_right_succeeds_exactly_two)
    {
        return Err(format!(
            "binding controls failed: normal_brief_same={normal_briefing_query_independent} swapped_brief_same={swapped_briefing_query_independent} arrangements_distinct={arrangements_visibly_distinct} triangle_choice_same={triangle_choice_converges_across_arrangements} square_choice_same={square_choice_converges_across_arrangements} queries_distinct={query_symbols_visibly_distinct} geometry={all_choice_geometry_identical} correct={all_correct_paths_pass} terminal={all_wrong_commits_terminal} left2={fixed_left_succeeds_exactly_two} right2={fixed_right_succeeds_exactly_two}"
        ));
    }

    let receipt = BindingMemoryQualificationReceipt {
        schema: "phicade.binding-memory-qualification.v1",
        result: "PASS",
        core_sha256: sha256_file(&core_path)?,
        core_name,
        core_version,
        normal_triangle,
        normal_square,
        swapped_triangle,
        swapped_square,
        normal_briefing_query_independent,
        swapped_briefing_query_independent,
        arrangements_visibly_distinct,
        triangle_choice_converges_across_arrangements,
        square_choice_converges_across_arrangements,
        query_symbols_visibly_distinct,
        all_choice_geometry_identical,
        all_correct_paths_pass,
        all_wrong_commits_terminal,
        fixed_left_succeeds_exactly_two,
        fixed_right_succeeds_exactly_two,
    };

    let json = serde_json::to_string_pretty(&receipt)
        .map_err(|error| format!("serialize binding-memory receipt: {error}"))?;
    println!("{json}");
    if let Some(parent) = receipt_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(&receipt_path, format!("{json}\n"))
        .map_err(|error| format!("write {}: {error}", receipt_path.display()))?;
    Ok(())
}
