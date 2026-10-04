use phicade_libretro::LibretroCore;
use phicade_runtime::{
    benchmark_task_by_id, benchmark_task_success, locate_agent_gym_player, ActionEnvelope,
    ActionKind, ActionSource, AudioBuffer, BenchmarkTaskSpec, EmulatorCore, FrameBuffer,
    GameImage, PixelPoint, SystemId, AGENT_GYM_COMP_NSF_ID, AGENT_GYM_COMP_NSM_ID,
    AGENT_GYM_COMP_NTF_ID, AGENT_GYM_COMP_NTM_ID, AGENT_GYM_COMP_SSF_ID,
    AGENT_GYM_COMP_SSM_ID, AGENT_GYM_COMP_STF_ID, AGENT_GYM_COMP_STM_ID,
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
    arrangement: &'static str,
    query: &'static str,
    operator: &'static str,
    correct: &'static str,
    wrong: &'static str,
    remembered: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct VariantEvidence {
    task_id: String,
    arrangement: String,
    query: String,
    operator: String,
    rom_sha256: String,
    briefing_frame_sha256: String,
    choice_frame_sha256: String,
    briefing_player: PixelPoint,
    choice_player: PixelPoint,
    correct_final_player: PixelPoint,
    wrong_final_player: PixelPoint,
    neutral_terminal_player: PixelPoint,
    recovery_probe_player: PixelPoint,
    neutral_terminal_frame_sha256: String,
    recovery_probe_frame_sha256: String,
    correct_path_pass: bool,
    wrong_commit_refused: bool,
    wrong_commit_terminal: bool,
    correct_side: String,
    remembered_side: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CompositionalRecallQualificationReceipt {
    schema: &'static str,
    result: &'static str,
    core_sha256: String,
    core_name: String,
    core_version: String,
    variants: Vec<VariantEvidence>,
    normal_briefing_condition_independent: bool,
    swapped_briefing_condition_independent: bool,
    arrangements_visibly_distinct: bool,
    same_query_operator_choice_converges_across_arrangements: bool,
    operators_visibly_distinct: bool,
    queries_visibly_distinct: bool,
    all_choice_geometry_identical: bool,
    all_correct_paths_pass: bool,
    all_wrong_commits_terminal: bool,
    fixed_left_succeeds_exactly_four: bool,
    fixed_right_succeeds_exactly_four: bool,
    ignore_operator_succeeds_exactly_four: bool,
    always_flip_succeeds_exactly_four: bool,
}

fn usage() -> ! {
    eprintln!(
        "usage: compositional_recall_qualify --core <sameboy> \
         --ntm-rom <gb> --ntf-rom <gb> --nsm-rom <gb> --nsf-rom <gb> \
         --stm-rom <gb> --stf-rom <gb> --ssm-rom <gb> --ssf-rom <gb> \
         --receipt <path>"
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
            name: "compositional-recall-qualifier".into(),
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
    tap_a(core, sequence, video, audio, "dismiss compositional briefing")?;
    no_input(
        core,
        WAIT_AFTER_BRIEF_FRAMES,
        video,
        audio,
        "wait through compositional lockout",
    )?;
    no_input(core, 1, video, audio, "arm compositional choice")?;
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
    tap_a(core, sequence, video, audio, "commit compositional door")?;
    no_input(core, SETTLE_FRAMES, video, audio, "settle compositional commitment")?;
    locate_agent_gym_player(video)
}

fn qualify_variant(
    core_path: &Path,
    rom_path: &Path,
    plan: VariantPlan,
) -> Result<(VariantEvidence, String, String), String> {
    let task: &'static BenchmarkTaskSpec = benchmark_task_by_id(plan.task_id)
        .ok_or_else(|| format!("{} missing from benchmark registry", plan.task_id))?;

    let temp = env::temp_dir().join(format!("phicade-compositional-{}", task.id));
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
        "warm compositional briefing",
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
        "move to correct compositional door",
    )?;
    let correct_path_pass = benchmark_task_success(task, correct_final_player);

    restore(
        &mut core,
        &choice_state,
        choice_frame,
        choice_mask,
        &mut video,
        &mut audio,
        "restore before wrong compositional commitment",
    )?;
    let wrong_final_player = choose_and_commit(
        &mut core,
        &mut sequence,
        plan.wrong,
        &mut video,
        &mut audio,
        "move to wrong compositional door",
    )?;
    let wrong_commit_refused = !benchmark_task_success(task, wrong_final_player);

    let failed_state = core
        .serialize_state()
        .map_err(|error| format!("serialize {} failed state: {error:?}", task.id))?;
    let failed_mask = core.input_mask_snapshot();
    let failed_frame = core.frame_count();

    no_input(
        &mut core,
        CHOICE_MOVE_FRAMES + 1 + 2 + SETTLE_FRAMES,
        &mut video,
        &mut audio,
        "advance neutral terminal control",
    )?;
    let neutral_terminal_player = locate_agent_gym_player(&video)?;
    let neutral_terminal_frame_sha256 = sha256_bytes(&video.rgba8);

    restore(
        &mut core,
        &failed_state,
        failed_frame,
        failed_mask,
        &mut video,
        &mut audio,
        "restore fail before recovery probe",
    )?;
    hold_button(
        &mut core,
        &mut sequence,
        plan.correct,
        CHOICE_MOVE_FRAMES.saturating_sub(1),
        &mut video,
        &mut audio,
        "probe recovery after wrong compositional commitment",
    )?;
    tap_a(
        &mut core,
        &mut sequence,
        &mut video,
        &mut audio,
        "probe A after compositional failure",
    )?;
    no_input(
        &mut core,
        SETTLE_FRAMES,
        &mut video,
        &mut audio,
        "settle compositional recovery probe",
    )?;
    let recovery_probe_player = locate_agent_gym_player(&video)?;
    let recovery_probe_frame_sha256 = sha256_bytes(&video.rgba8);
    let wrong_commit_terminal =
        recovery_probe_frame_sha256 == neutral_terminal_frame_sha256
            && recovery_probe_player == neutral_terminal_player
            && !benchmark_task_success(task, recovery_probe_player);

    Ok((
        VariantEvidence {
            task_id: task.id.into(),
            arrangement: plan.arrangement.into(),
            query: plan.query.into(),
            operator: plan.operator.into(),
            rom_sha256: sha256_file(rom_path)?,
            briefing_frame_sha256,
            choice_frame_sha256,
            briefing_player,
            choice_player,
            correct_final_player,
            wrong_final_player,
            neutral_terminal_player,
            recovery_probe_player,
            neutral_terminal_frame_sha256,
            recovery_probe_frame_sha256,
            correct_path_pass,
            wrong_commit_refused,
            wrong_commit_terminal,
            correct_side: plan.correct.into(),
            remembered_side: plan.remembered.into(),
        },
        core_name,
        core_version,
    ))
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Compositional Recall qualification failed: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let mut core_path: Option<PathBuf> = None;
    let mut roms: [Option<PathBuf>; 8] = Default::default();
    let mut receipt_path: Option<PathBuf> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--core" => core_path = args.next().map(PathBuf::from),
            "--ntm-rom" => roms[0] = args.next().map(PathBuf::from),
            "--ntf-rom" => roms[1] = args.next().map(PathBuf::from),
            "--nsm-rom" => roms[2] = args.next().map(PathBuf::from),
            "--nsf-rom" => roms[3] = args.next().map(PathBuf::from),
            "--stm-rom" => roms[4] = args.next().map(PathBuf::from),
            "--stf-rom" => roms[5] = args.next().map(PathBuf::from),
            "--ssm-rom" => roms[6] = args.next().map(PathBuf::from),
            "--ssf-rom" => roms[7] = args.next().map(PathBuf::from),
            "--receipt" => receipt_path = args.next().map(PathBuf::from),
            _ => usage(),
        }
    }

    let core_path = core_path.unwrap_or_else(|| usage());
    let receipt_path = receipt_path.unwrap_or_else(|| usage());
    let roms: Vec<PathBuf> = roms
        .into_iter()
        .map(|rom| rom.unwrap_or_else(|| usage()))
        .collect();

    let plans = [
        VariantPlan { task_id: AGENT_GYM_COMP_NTM_ID, arrangement: "NORMAL", query: "TRIANGLE", operator: "MATCH", correct: "LEFT",  wrong: "RIGHT", remembered: "LEFT" },
        VariantPlan { task_id: AGENT_GYM_COMP_NTF_ID, arrangement: "NORMAL", query: "TRIANGLE", operator: "FLIP",  correct: "RIGHT", wrong: "LEFT",  remembered: "LEFT" },
        VariantPlan { task_id: AGENT_GYM_COMP_NSM_ID, arrangement: "NORMAL", query: "SQUARE",   operator: "MATCH", correct: "RIGHT", wrong: "LEFT",  remembered: "RIGHT" },
        VariantPlan { task_id: AGENT_GYM_COMP_NSF_ID, arrangement: "NORMAL", query: "SQUARE",   operator: "FLIP",  correct: "LEFT",  wrong: "RIGHT", remembered: "RIGHT" },
        VariantPlan { task_id: AGENT_GYM_COMP_STM_ID, arrangement: "SWAPPED", query: "TRIANGLE", operator: "MATCH", correct: "RIGHT", wrong: "LEFT",  remembered: "RIGHT" },
        VariantPlan { task_id: AGENT_GYM_COMP_STF_ID, arrangement: "SWAPPED", query: "TRIANGLE", operator: "FLIP",  correct: "LEFT",  wrong: "RIGHT", remembered: "RIGHT" },
        VariantPlan { task_id: AGENT_GYM_COMP_SSM_ID, arrangement: "SWAPPED", query: "SQUARE",   operator: "MATCH", correct: "LEFT",  wrong: "RIGHT", remembered: "LEFT" },
        VariantPlan { task_id: AGENT_GYM_COMP_SSF_ID, arrangement: "SWAPPED", query: "SQUARE",   operator: "FLIP",  correct: "RIGHT", wrong: "LEFT",  remembered: "LEFT" },
    ];

    let mut variants = Vec::with_capacity(8);
    let mut core_name = String::new();
    let mut core_version = String::new();
    for (index, plan) in plans.iter().copied().enumerate() {
        let (evidence, name, version) =
            qualify_variant(&core_path, &roms[index], plan)?;
        if index == 0 {
            core_name = name;
            core_version = version;
        } else if name != core_name || version != core_version {
            return Err("compositional variants did not run under identical core identity".into());
        }
        variants.push(evidence);
    }

    let normal_briefing_condition_independent =
        variants[..4].iter().all(|v| v.briefing_frame_sha256 == variants[0].briefing_frame_sha256);
    let swapped_briefing_condition_independent =
        variants[4..].iter().all(|v| v.briefing_frame_sha256 == variants[4].briefing_frame_sha256);
    let arrangements_visibly_distinct =
        variants[0].briefing_frame_sha256 != variants[4].briefing_frame_sha256;

    // Pair NORMAL/SWAPPED histories under identical query + operator.
    let same_query_operator_choice_converges_across_arrangements =
        variants[0].choice_frame_sha256 == variants[4].choice_frame_sha256
            && variants[1].choice_frame_sha256 == variants[5].choice_frame_sha256
            && variants[2].choice_frame_sha256 == variants[6].choice_frame_sha256
            && variants[3].choice_frame_sha256 == variants[7].choice_frame_sha256;

    let operators_visibly_distinct =
        variants[0].choice_frame_sha256 != variants[1].choice_frame_sha256
            && variants[2].choice_frame_sha256 != variants[3].choice_frame_sha256
            && variants[4].choice_frame_sha256 != variants[5].choice_frame_sha256
            && variants[6].choice_frame_sha256 != variants[7].choice_frame_sha256;

    let queries_visibly_distinct =
        variants[0].choice_frame_sha256 != variants[2].choice_frame_sha256
            && variants[1].choice_frame_sha256 != variants[3].choice_frame_sha256
            && variants[4].choice_frame_sha256 != variants[6].choice_frame_sha256
            && variants[5].choice_frame_sha256 != variants[7].choice_frame_sha256;

    let all_choice_geometry_identical =
        variants.iter().all(|v| v.choice_player == START);
    let all_correct_paths_pass =
        variants.iter().all(|v| v.correct_path_pass);
    let all_wrong_commits_terminal =
        variants.iter().all(|v| v.wrong_commit_refused && v.wrong_commit_terminal);

    let fixed_left_succeeds_exactly_four =
        variants.iter().filter(|v| v.correct_side == "LEFT").count() == 4;
    let fixed_right_succeeds_exactly_four =
        variants.iter().filter(|v| v.correct_side == "RIGHT").count() == 4;

    // Ignoring the operator means choosing the remembered side every time.
    // MATCH variants succeed and FLIP variants fail: exactly 4/8.
    let ignore_operator_succeeds_exactly_four =
        variants.iter().filter(|v| v.correct_side == v.remembered_side).count() == 4;

    // Always applying FLIP succeeds exactly on the four FLIP variants and fails
    // the four MATCH variants.
    let always_flip_succeeds_exactly_four =
        variants.iter().filter(|v| v.correct_side != v.remembered_side).count() == 4;

    if !(normal_briefing_condition_independent
        && swapped_briefing_condition_independent
        && arrangements_visibly_distinct
        && same_query_operator_choice_converges_across_arrangements
        && operators_visibly_distinct
        && queries_visibly_distinct
        && all_choice_geometry_identical
        && all_correct_paths_pass
        && all_wrong_commits_terminal
        && fixed_left_succeeds_exactly_four
        && fixed_right_succeeds_exactly_four
        && ignore_operator_succeeds_exactly_four
        && always_flip_succeeds_exactly_four)
    {
        return Err(format!(
            "compositional controls failed: normal_brief={normal_briefing_condition_independent} swapped_brief={swapped_briefing_condition_independent} arrangements={arrangements_visibly_distinct} pair_convergence={same_query_operator_choice_converges_across_arrangements} operators={operators_visibly_distinct} queries={queries_visibly_distinct} geometry={all_choice_geometry_identical} correct={all_correct_paths_pass} terminal={all_wrong_commits_terminal} left4={fixed_left_succeeds_exactly_four} right4={fixed_right_succeeds_exactly_four} ignore_op4={ignore_operator_succeeds_exactly_four} flip4={always_flip_succeeds_exactly_four}"
        ));
    }

    let receipt = CompositionalRecallQualificationReceipt {
        schema: "phicade.compositional-recall-qualification.v1",
        result: "PASS",
        core_sha256: sha256_file(&core_path)?,
        core_name,
        core_version,
        variants,
        normal_briefing_condition_independent,
        swapped_briefing_condition_independent,
        arrangements_visibly_distinct,
        same_query_operator_choice_converges_across_arrangements,
        operators_visibly_distinct,
        queries_visibly_distinct,
        all_choice_geometry_identical,
        all_correct_paths_pass,
        all_wrong_commits_terminal,
        fixed_left_succeeds_exactly_four,
        fixed_right_succeeds_exactly_four,
        ignore_operator_succeeds_exactly_four,
        always_flip_succeeds_exactly_four,
    };

    let json = serde_json::to_string_pretty(&receipt)
        .map_err(|error| format!("serialize compositional receipt: {error}"))?;
    println!("{json}");
    if let Some(parent) = receipt_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(&receipt_path, format!("{json}\n"))
        .map_err(|error| format!("write {}: {error}", receipt_path.display()))?;
    Ok(())
}
