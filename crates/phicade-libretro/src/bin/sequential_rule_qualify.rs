use phicade_libretro::LibretroCore;
use phicade_runtime::{
    benchmark_task_by_id, benchmark_task_success, locate_agent_gym_player, ActionEnvelope,
    ActionKind, ActionSource, AudioBuffer, BenchmarkTaskSpec, EmulatorCore, FrameBuffer,
    GameImage, PixelPoint, SystemId,
    AGENT_GYM_SEQ_NTMM_ID, AGENT_GYM_SEQ_NTMF_ID, AGENT_GYM_SEQ_NTFM_ID, AGENT_GYM_SEQ_NTFF_ID,
    AGENT_GYM_SEQ_NSMM_ID, AGENT_GYM_SEQ_NSMF_ID, AGENT_GYM_SEQ_NSFM_ID, AGENT_GYM_SEQ_NSFF_ID,
    AGENT_GYM_SEQ_STMM_ID, AGENT_GYM_SEQ_STMF_ID, AGENT_GYM_SEQ_STFM_ID, AGENT_GYM_SEQ_STFF_ID,
    AGENT_GYM_SEQ_SSMM_ID, AGENT_GYM_SEQ_SSMF_ID, AGENT_GYM_SEQ_SSFM_ID, AGENT_GYM_SEQ_SSFF_ID,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{env, fs, path::{Path, PathBuf}, process};

const WARMUP_FRAMES: u64 = 120;
const STAGE_WAIT_FRAMES: u64 = 101;
const MOVE_FRAMES: u64 = 24;
const SETTLE_FRAMES: u64 = 4;
const START: PixelPoint = PixelPoint { x: 72, y: 96 };

#[derive(Debug, Clone, Copy)]
struct VariantPlan {
    task_id: &'static str,
    arrangement: &'static str,
    query: &'static str,
    op1: &'static str,
    op2: &'static str,
    remembered: &'static str,
    after1: &'static str,
    correct: &'static str,
    wrong: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct VariantEvidence {
    task_id: String,
    arrangement: String,
    query: String,
    operator1: String,
    operator2: String,
    remembered_side: String,
    intermediate_side: String,
    correct_side: String,
    rom_sha256: String,
    registry_hash_match: bool,
    briefing_frame_sha256: String,
    stage1_frame_sha256: String,
    stage2_frame_sha256: String,
    briefing_player: PixelPoint,
    stage1_player: PixelPoint,
    stage2_player: PixelPoint,
    correct_final_player: PixelPoint,
    wrong_final_player: PixelPoint,
    neutral_terminal_player: PixelPoint,
    recovery_probe_player: PixelPoint,
    neutral_terminal_frame_sha256: String,
    recovery_probe_frame_sha256: String,
    correct_path_pass: bool,
    wrong_commit_refused: bool,
    wrong_commit_terminal: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Receipt {
    schema: &'static str,
    result: &'static str,
    core_sha256: String,
    core_name: String,
    core_version: String,
    variants: Vec<VariantEvidence>,
    normal_briefing_condition_independent: bool,
    swapped_briefing_condition_independent: bool,
    arrangements_visibly_distinct: bool,
    stage1_independent_of_operator2: bool,
    stage1_conditions_visibly_distinct: bool,
    stage2_converges_by_operator2: bool,
    stage2_operators_visibly_distinct: bool,
    all_geometry_identical: bool,
    all_registry_hashes_match: bool,
    all_correct_paths_pass: bool,
    all_wrong_commits_terminal: bool,
    fixed_left_succeeds_exactly_eight: bool,
    fixed_right_succeeds_exactly_eight: bool,
    ignore_operator1_succeeds_exactly_eight: bool,
    ignore_operator2_succeeds_exactly_eight: bool,
    use_only_operator1_succeeds_exactly_eight: bool,
    use_only_operator2_succeeds_exactly_eight: bool,
}

fn usage() -> ! {
    eprintln!("usage: sequential_rule_qualify --core <sameboy> --rom <task-id=path> [--rom ... x16] --receipt <path>");
    process::exit(2);
}

fn sha256_bytes(bytes: &[u8]) -> String { hex::encode(Sha256::digest(bytes)) }

fn sha256_file(path: &Path) -> Result<String, String> {
    fs::read(path).map(|bytes| sha256_bytes(&bytes))
        .map_err(|error| format!("cannot read {}: {error}", path.display()))
}

fn button_event(sequence: u64, frame: u64, button: &str, pressed: bool) -> ActionEnvelope {
    ActionEnvelope {
        sequence,
        frame,
        source: ActionSource::Script { name: "sequential-rule-qualifier".into() },
        action: ActionKind::Button { button: button.into(), pressed },
    }
}

fn step(core: &mut LibretroCore, events: &[ActionEnvelope], video: &mut FrameBuffer, audio: &mut AudioBuffer, context: &str) -> Result<(), String> {
    core.step_frame(events, video, audio).map_err(|error| format!("{context}: {error:?}"))
}

fn no_input(core: &mut LibretroCore, frames: u64, video: &mut FrameBuffer, audio: &mut AudioBuffer, context: &str) -> Result<(), String> {
    for _ in 0..frames { step(core, &[], video, audio, context)?; }
    Ok(())
}

fn hold_button(core: &mut LibretroCore, sequence: &mut u64, button: &str, frames: u64, video: &mut FrameBuffer, audio: &mut AudioBuffer, context: &str) -> Result<(), String> {
    if frames == 0 { return Ok(()); }
    let frame = core.frame_count();
    step(core, &[button_event(*sequence, frame, button, true)], video, audio, context)?;
    *sequence = sequence.saturating_add(1);
    for _ in 1..frames { step(core, &[], video, audio, context)?; }
    let frame = core.frame_count();
    step(core, &[button_event(*sequence, frame, button, false)], video, audio, context)?;
    *sequence = sequence.saturating_add(1);
    Ok(())
}

fn tap_a(core: &mut LibretroCore, sequence: &mut u64, video: &mut FrameBuffer, audio: &mut AudioBuffer, context: &str) -> Result<(), String> {
    hold_button(core, sequence, "A", 1, video, audio, context)
}

fn restore(core: &mut LibretroCore, state: &[u8], frame: u64, mask: u16, video: &mut FrameBuffer, audio: &mut AudioBuffer, context: &str) -> Result<(), String> {
    core.restore_state(state, frame).map_err(|error| format!("{context}: restore state: {error:?}"))?;
    core.restore_input_mask(mask);
    no_input(core, 1, video, audio, context)
}

fn choose_and_commit(core: &mut LibretroCore, sequence: &mut u64, direction: &str, video: &mut FrameBuffer, audio: &mut AudioBuffer, context: &str) -> Result<PixelPoint, String> {
    hold_button(core, sequence, direction, MOVE_FRAMES, video, audio, context)?;
    tap_a(core, sequence, video, audio, "commit sequential-rule door")?;
    no_input(core, SETTLE_FRAMES, video, audio, "settle sequential-rule commitment")?;
    locate_agent_gym_player(video)
}

fn qualify_variant(core_path: &Path, rom_path: &Path, plan: VariantPlan) -> Result<(VariantEvidence, String, String), String> {
    let task: &'static BenchmarkTaskSpec = benchmark_task_by_id(plan.task_id)
        .ok_or_else(|| format!("{} missing from benchmark registry", plan.task_id))?;

    let temp = env::temp_dir().join(format!("phicade-sequential-{}", task.id));
    let system_dir = temp.join("system");
    let save_dir = temp.join("save");
    fs::create_dir_all(&system_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&save_dir).map_err(|e| e.to_string())?;

    let mut core = LibretroCore::open(core_path, &system_dir, &save_dir)
        .map_err(|error| format!("open SameBoy for {}: {error:?}", task.id))?;
    core.load_game(&GameImage::new(rom_path, SystemId::GameBoy, task.id))
        .map_err(|error| format!("load {}: {error:?}", task.id))?;

    let core_name = core.identity().library_name.clone();
    let core_version = core.identity().library_version.clone();
    let mut video = FrameBuffer::default();
    let mut audio = AudioBuffer::default();
    let mut sequence = 0u64;

    no_input(&mut core, WARMUP_FRAMES, &mut video, &mut audio, "warm sequential briefing")?;
    let briefing_player = locate_agent_gym_player(&video)?;
    if briefing_player != START || briefing_player != task.start {
        return Err(format!("{} briefing geometry drifted: {:?}", task.id, briefing_player));
    }
    let briefing_frame_sha256 = sha256_bytes(&video.rgba8);

    tap_a(&mut core, &mut sequence, &mut video, &mut audio, "erase briefing")?;
    no_input(&mut core, STAGE_WAIT_FRAMES, &mut video, &mut audio, "wait for Stage 1")?;
    let stage1_player = locate_agent_gym_player(&video)?;
    let stage1_frame_sha256 = sha256_bytes(&video.rgba8);

    tap_a(&mut core, &mut sequence, &mut video, &mut audio, "acknowledge and erase Stage 1")?;
    no_input(&mut core, STAGE_WAIT_FRAMES, &mut video, &mut audio, "wait for Stage 2")?;
    no_input(&mut core, 1, &mut video, &mut audio, "arm Stage 2")?;
    let stage2_player = locate_agent_gym_player(&video)?;
    let stage2_frame_sha256 = sha256_bytes(&video.rgba8);

    if stage1_player != task.start || stage2_player != task.start {
        return Err(format!("{} player moved before final choice", task.id));
    }

    let stage2_state = core.serialize_state()
        .map_err(|error| format!("serialize {} Stage 2: {error:?}", task.id))?;
    let stage2_mask = core.input_mask_snapshot();
    let stage2_frame = core.frame_count();

    let correct_final_player = choose_and_commit(
        &mut core, &mut sequence, plan.correct, &mut video, &mut audio, "correct sequential-rule door"
    )?;
    let correct_path_pass = benchmark_task_success(task, correct_final_player);

    restore(&mut core, &stage2_state, stage2_frame, stage2_mask, &mut video, &mut audio, "restore before wrong door")?;
    let wrong_final_player = choose_and_commit(
        &mut core, &mut sequence, plan.wrong, &mut video, &mut audio, "wrong sequential-rule door"
    )?;
    let wrong_commit_refused = !benchmark_task_success(task, wrong_final_player);

    let fail_state = core.serialize_state()
        .map_err(|error| format!("serialize {} fail state: {error:?}", task.id))?;
    let fail_mask = core.input_mask_snapshot();
    let fail_frame = core.frame_count();

    no_input(&mut core, MOVE_FRAMES + 1 + 2 + SETTLE_FRAMES, &mut video, &mut audio, "neutral terminal future")?;
    let neutral_terminal_player = locate_agent_gym_player(&video)?;
    let neutral_terminal_frame_sha256 = sha256_bytes(&video.rgba8);

    restore(&mut core, &fail_state, fail_frame, fail_mask, &mut video, &mut audio, "restore fail for recovery probe")?;
    hold_button(&mut core, &mut sequence, plan.correct, MOVE_FRAMES.saturating_sub(1), &mut video, &mut audio, "recovery movement")?;
    tap_a(&mut core, &mut sequence, &mut video, &mut audio, "recovery A")?;
    no_input(&mut core, SETTLE_FRAMES, &mut video, &mut audio, "settle recovery probe")?;
    let recovery_probe_player = locate_agent_gym_player(&video)?;
    let recovery_probe_frame_sha256 = sha256_bytes(&video.rgba8);

    let wrong_commit_terminal =
        neutral_terminal_player == recovery_probe_player
        && neutral_terminal_frame_sha256 == recovery_probe_frame_sha256
        && !benchmark_task_success(task, recovery_probe_player);

    let rom_sha256 = sha256_file(rom_path)?;
    let registry_hash_match = rom_sha256 == task.rom_sha256;

    Ok((VariantEvidence {
        task_id: task.id.into(),
        arrangement: plan.arrangement.into(),
        query: plan.query.into(),
        operator1: plan.op1.into(),
        operator2: plan.op2.into(),
        remembered_side: plan.remembered.into(),
        intermediate_side: plan.after1.into(),
        correct_side: plan.correct.into(),
        rom_sha256,
        registry_hash_match,
        briefing_frame_sha256,
        stage1_frame_sha256,
        stage2_frame_sha256,
        briefing_player,
        stage1_player,
        stage2_player,
        correct_final_player,
        wrong_final_player,
        neutral_terminal_player,
        recovery_probe_player,
        neutral_terminal_frame_sha256,
        recovery_probe_frame_sha256,
        correct_path_pass,
        wrong_commit_refused,
        wrong_commit_terminal,
    }, core_name, core_version))
}

fn plan(task_id: &'static str, arrangement: &'static str, query: &'static str, op1: &'static str, op2: &'static str, remembered: &'static str) -> VariantPlan {
    let after1 = if op1 == "MATCH" { remembered } else if remembered == "LEFT" { "RIGHT" } else { "LEFT" };
    let correct = if op2 == "MATCH" { after1 } else if after1 == "LEFT" { "RIGHT" } else { "LEFT" };
    let wrong = if correct == "LEFT" { "RIGHT" } else { "LEFT" };
    VariantPlan { task_id, arrangement, query, op1, op2, remembered, after1, correct, wrong }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Sequential Rule qualification failed: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let mut core_path: Option<PathBuf> = None;
    let mut receipt_path: Option<PathBuf> = None;
    let mut supplied_roms: Vec<(String, PathBuf)> = Vec::new();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--core" => core_path = args.next().map(PathBuf::from),
            "--receipt" => receipt_path = args.next().map(PathBuf::from),
            "--rom" => {
                let value = args.next().unwrap_or_else(|| usage());
                let (id, path) = value.split_once('=').unwrap_or_else(|| usage());
                supplied_roms.push((id.to_owned(), PathBuf::from(path)));
            }
            _ => usage(),
        }
    }

    let core_path = core_path.unwrap_or_else(|| usage());
    let receipt_path = receipt_path.unwrap_or_else(|| usage());

    let plans = [
        plan(AGENT_GYM_SEQ_NTMM_ID, "NORMAL", "TRIANGLE", "MATCH", "MATCH", "LEFT"),
        plan(AGENT_GYM_SEQ_NTMF_ID, "NORMAL", "TRIANGLE", "MATCH", "FLIP", "LEFT"),
        plan(AGENT_GYM_SEQ_NTFM_ID, "NORMAL", "TRIANGLE", "FLIP", "MATCH", "LEFT"),
        plan(AGENT_GYM_SEQ_NTFF_ID, "NORMAL", "TRIANGLE", "FLIP", "FLIP", "LEFT"),
        plan(AGENT_GYM_SEQ_NSMM_ID, "NORMAL", "SQUARE", "MATCH", "MATCH", "RIGHT"),
        plan(AGENT_GYM_SEQ_NSMF_ID, "NORMAL", "SQUARE", "MATCH", "FLIP", "RIGHT"),
        plan(AGENT_GYM_SEQ_NSFM_ID, "NORMAL", "SQUARE", "FLIP", "MATCH", "RIGHT"),
        plan(AGENT_GYM_SEQ_NSFF_ID, "NORMAL", "SQUARE", "FLIP", "FLIP", "RIGHT"),
        plan(AGENT_GYM_SEQ_STMM_ID, "SWAPPED", "TRIANGLE", "MATCH", "MATCH", "RIGHT"),
        plan(AGENT_GYM_SEQ_STMF_ID, "SWAPPED", "TRIANGLE", "MATCH", "FLIP", "RIGHT"),
        plan(AGENT_GYM_SEQ_STFM_ID, "SWAPPED", "TRIANGLE", "FLIP", "MATCH", "RIGHT"),
        plan(AGENT_GYM_SEQ_STFF_ID, "SWAPPED", "TRIANGLE", "FLIP", "FLIP", "RIGHT"),
        plan(AGENT_GYM_SEQ_SSMM_ID, "SWAPPED", "SQUARE", "MATCH", "MATCH", "LEFT"),
        plan(AGENT_GYM_SEQ_SSMF_ID, "SWAPPED", "SQUARE", "MATCH", "FLIP", "LEFT"),
        plan(AGENT_GYM_SEQ_SSFM_ID, "SWAPPED", "SQUARE", "FLIP", "MATCH", "LEFT"),
        plan(AGENT_GYM_SEQ_SSFF_ID, "SWAPPED", "SQUARE", "FLIP", "FLIP", "LEFT"),
    ];

    if supplied_roms.len() != plans.len() {
        return Err(format!("expected {} --rom arguments, got {}", plans.len(), supplied_roms.len()));
    }

    let mut variants = Vec::with_capacity(plans.len());
    let mut core_name = String::new();
    let mut core_version = String::new();

    for (index, p) in plans.iter().copied().enumerate() {
        let rom_path = supplied_roms.iter()
            .find(|(id, _)| id == p.task_id)
            .map(|(_, path)| path)
            .ok_or_else(|| format!("missing ROM path for {}", p.task_id))?;
        let (evidence, name, version) = qualify_variant(&core_path, rom_path, p)?;
        if index == 0 {
            core_name = name;
            core_version = version;
        } else if name != core_name || version != core_version {
            return Err("sequential variants did not run under identical core identity".into());
        }
        variants.push(evidence);
    }

    let normal_briefing_condition_independent =
        variants[..8].iter().all(|v| v.briefing_frame_sha256 == variants[0].briefing_frame_sha256);
    let swapped_briefing_condition_independent =
        variants[8..].iter().all(|v| v.briefing_frame_sha256 == variants[8].briefing_frame_sha256);
    let arrangements_visibly_distinct =
        variants[0].briefing_frame_sha256 != variants[8].briefing_frame_sha256;

    // Within each arrangement/query/op1 triple, changing future op2 cannot alter Stage 1.
    let stage1_independent_of_operator2 =
        (0..variants.len()).step_by(2).all(|i| variants[i].stage1_frame_sha256 == variants[i + 1].stage1_frame_sha256);

    // Stage 1 must actually expose query/op1 distinctions.
    let stage1_conditions_visibly_distinct = {
        let hashes: std::collections::BTreeSet<_> =
            variants.iter().map(|v| v.stage1_frame_sha256.as_str()).collect();
        hashes.len() == 4
    };

    // Stage 2 exposes op2 only; every MATCH history converges, every FLIP history converges.
    let match_stage2 = variants.iter().filter(|v| v.operator2 == "MATCH").map(|v| v.stage2_frame_sha256.as_str()).collect::<std::collections::BTreeSet<_>>();
    let flip_stage2 = variants.iter().filter(|v| v.operator2 == "FLIP").map(|v| v.stage2_frame_sha256.as_str()).collect::<std::collections::BTreeSet<_>>();
    let stage2_converges_by_operator2 = match_stage2.len() == 1 && flip_stage2.len() == 1;
    let stage2_operators_visibly_distinct =
        match_stage2.iter().next() != flip_stage2.iter().next();

    let all_geometry_identical = variants.iter().all(|v|
        v.briefing_player == START && v.stage1_player == START && v.stage2_player == START
    );
    let all_registry_hashes_match = variants.iter().all(|v| v.registry_hash_match);
    let all_correct_paths_pass = variants.iter().all(|v| v.correct_path_pass);
    let all_wrong_commits_terminal = variants.iter().all(|v| v.wrong_commit_refused && v.wrong_commit_terminal);

    let fixed_left_succeeds_exactly_eight = variants.iter().filter(|v| v.correct_side == "LEFT").count() == 8;
    let fixed_right_succeeds_exactly_eight = variants.iter().filter(|v| v.correct_side == "RIGHT").count() == 8;

    // Ignore op1: apply only op2 to remembered side. Correct exactly when op1 is MATCH.
    let ignore_operator1_succeeds_exactly_eight = variants.iter().filter(|v| {
        let predicted = if v.operator2 == "MATCH" { v.remembered_side.as_str() }
            else if v.remembered_side == "LEFT" { "RIGHT" } else { "LEFT" };
        predicted == v.correct_side
    }).count() == 8;

    // Ignore op2: stop at the hidden intermediate. Correct exactly when op2 is MATCH.
    let ignore_operator2_succeeds_exactly_eight =
        variants.iter().filter(|v| v.intermediate_side == v.correct_side).count() == 8;

    let use_only_operator1_succeeds_exactly_eight = ignore_operator2_succeeds_exactly_eight;
    let use_only_operator2_succeeds_exactly_eight = ignore_operator1_succeeds_exactly_eight;

    if !(normal_briefing_condition_independent
        && swapped_briefing_condition_independent
        && arrangements_visibly_distinct
        && stage1_independent_of_operator2
        && stage1_conditions_visibly_distinct
        && stage2_converges_by_operator2
        && stage2_operators_visibly_distinct
        && all_geometry_identical
        && all_registry_hashes_match
        && all_correct_paths_pass
        && all_wrong_commits_terminal
        && fixed_left_succeeds_exactly_eight
        && fixed_right_succeeds_exactly_eight
        && ignore_operator1_succeeds_exactly_eight
        && ignore_operator2_succeeds_exactly_eight
        && use_only_operator1_succeeds_exactly_eight
        && use_only_operator2_succeeds_exactly_eight)
    {
        eprintln!(
            "SEQUENTIAL DEBUG EVIDENCE:\n{}",
            serde_json::to_string_pretty(&variants).unwrap_or_else(|_| "<serialize failed>".into())
        );
        return Err(format!(
            "sequential controls failed: normal={normal_briefing_condition_independent} swapped={swapped_briefing_condition_independent} arrangements={arrangements_visibly_distinct} stage1_future={stage1_independent_of_operator2} stage1_visible={stage1_conditions_visibly_distinct} stage2_converges={stage2_converges_by_operator2} stage2_visible={stage2_operators_visibly_distinct} geometry={all_geometry_identical} hashes={all_registry_hashes_match} correct={all_correct_paths_pass} terminal={all_wrong_commits_terminal} left8={fixed_left_succeeds_exactly_eight} right8={fixed_right_succeeds_exactly_eight} ignore1={ignore_operator1_succeeds_exactly_eight} ignore2={ignore_operator2_succeeds_exactly_eight}"
        ));
    }

    let receipt = Receipt {
        schema: "phicade.sequential-rule-qualification.v1",
        result: "PASS",
        core_sha256: sha256_file(&core_path)?,
        core_name,
        core_version,
        variants,
        normal_briefing_condition_independent,
        swapped_briefing_condition_independent,
        arrangements_visibly_distinct,
        stage1_independent_of_operator2,
        stage1_conditions_visibly_distinct,
        stage2_converges_by_operator2,
        stage2_operators_visibly_distinct,
        all_geometry_identical,
        all_registry_hashes_match,
        all_correct_paths_pass,
        all_wrong_commits_terminal,
        fixed_left_succeeds_exactly_eight,
        fixed_right_succeeds_exactly_eight,
        ignore_operator1_succeeds_exactly_eight,
        ignore_operator2_succeeds_exactly_eight,
        use_only_operator1_succeeds_exactly_eight,
        use_only_operator2_succeeds_exactly_eight,
    };

    let json = serde_json::to_string_pretty(&receipt)
        .map_err(|error| format!("serialize sequential receipt: {error}"))?;
    println!("{json}");
    if let Some(parent) = receipt_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(&receipt_path, format!("{json}\n"))
        .map_err(|error| format!("write {}: {error}", receipt_path.display()))?;
    Ok(())
}
