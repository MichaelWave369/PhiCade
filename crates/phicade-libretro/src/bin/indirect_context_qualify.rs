use phicade_libretro::LibretroCore;
use phicade_runtime::{
    agent_gym::{
        AGENT_GYM_IND_A_NOR_MOON_SQR_ID, AGENT_GYM_IND_A_NOR_MOON_TRI_ID,
        AGENT_GYM_IND_A_NOR_STAR_SQR_ID, AGENT_GYM_IND_A_NOR_STAR_TRI_ID,
        AGENT_GYM_IND_A_SWP_MOON_SQR_ID, AGENT_GYM_IND_A_SWP_MOON_TRI_ID,
        AGENT_GYM_IND_A_SWP_STAR_SQR_ID, AGENT_GYM_IND_A_SWP_STAR_TRI_ID,
        AGENT_GYM_IND_B_NOR_MOON_SQR_ID, AGENT_GYM_IND_B_NOR_MOON_TRI_ID,
        AGENT_GYM_IND_B_NOR_STAR_SQR_ID, AGENT_GYM_IND_B_NOR_STAR_TRI_ID,
        AGENT_GYM_IND_B_SWP_MOON_SQR_ID, AGENT_GYM_IND_B_SWP_MOON_TRI_ID,
        AGENT_GYM_IND_B_SWP_STAR_SQR_ID, AGENT_GYM_IND_B_SWP_STAR_TRI_ID,
    },
    benchmark_task_by_id, benchmark_task_success, locate_agent_gym_player, ActionEnvelope,
    ActionKind, ActionSource, AudioBuffer, BenchmarkTaskSpec, EmulatorCore, FrameBuffer,
    GameImage, PixelPoint, SystemId,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    env, fs,
    path::{Path, PathBuf},
    process,
};

const WARMUP_FRAMES: u64 = 120;
const WAIT_AFTER_BRIEF_FRAMES: u64 = 101;
const MOVE_FRAMES: u64 = 24;
const SETTLE_FRAMES: u64 = 4;
const START: PixelPoint = PixelPoint { x: 72, y: 96 };

#[derive(Debug, Clone, Copy)]
struct VariantPlan {
    task_id: &'static str,
    layout: &'static str,
    pointer_map: &'static str,
    pointer: &'static str,
    query: &'static str,
    resolved_bank: &'static str,
    correct: &'static str,
    wrong: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct VariantEvidence {
    task_id: String,
    layout: String,
    pointer_map: String,
    pointer: String,
    query: String,
    resolved_bank: String,
    correct_side: String,
    rom_sha256: String,
    registry_hash_match: bool,
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
    briefing_groups_condition_independent: bool,
    briefing_histories_visibly_distinct: bool,
    same_pointer_query_choice_converges_across_histories: bool,
    pointer_tokens_visibly_distinct: bool,
    queries_visibly_distinct: bool,
    all_choice_geometry_identical: bool,
    all_registry_hashes_match: bool,
    all_correct_paths_pass: bool,
    all_wrong_commits_terminal: bool,
    fixed_left_succeeds_exactly_eight: bool,
    fixed_right_succeeds_exactly_eight: bool,
    assume_normal_map_succeeds_exactly_eight: bool,
    assume_swapped_map_succeeds_exactly_eight: bool,
    always_circle_succeeds_exactly_eight: bool,
    always_cross_succeeds_exactly_eight: bool,
    always_triangle_succeeds_exactly_eight: bool,
    always_square_succeeds_exactly_eight: bool,
}

fn usage() -> ! {
    eprintln!("usage: indirect_context_qualify --core <sameboy> --rom <task-id=path> [--rom ... x16] --receipt <path>");
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
            name: "indirect-context-qualifier".into(),
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
        .map_err(|error| format!("{context}: restore: {error:?}"))?;
    core.restore_input_mask(mask);
    no_input(core, 1, video, audio, context)
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
        core, sequence, direction, MOVE_FRAMES, video, audio, context,
    )?;
    tap_a(
        core,
        sequence,
        video,
        audio,
        "commit indirect-context door",
    )?;
    no_input(
        core,
        SETTLE_FRAMES,
        video,
        audio,
        "settle indirect-context commitment",
    )?;
    locate_agent_gym_player(video)
}

fn resolved_bank(pointer_map: &str, pointer: &str) -> &'static str {
    match (pointer_map, pointer) {
        ("NORMAL", "STAR") => "CIRCLE",
        ("NORMAL", "MOON") => "CROSS",
        ("SWAPPED", "STAR") => "CROSS",
        ("SWAPPED", "MOON") => "CIRCLE",
        _ => unreachable!(),
    }
}

fn remembered_side(layout: &str, bank: &str, query: &str) -> &'static str {
    match (layout, bank, query) {
        ("A", "CIRCLE", "TRIANGLE") => "LEFT",
        ("A", "CIRCLE", "SQUARE") => "RIGHT",
        ("A", "CROSS", "TRIANGLE") => "RIGHT",
        ("A", "CROSS", "SQUARE") => "LEFT",
        ("B", "CIRCLE", "TRIANGLE") => "RIGHT",
        ("B", "CIRCLE", "SQUARE") => "LEFT",
        ("B", "CROSS", "TRIANGLE") => "LEFT",
        ("B", "CROSS", "SQUARE") => "RIGHT",
        _ => unreachable!(),
    }
}

fn plan(
    task_id: &'static str,
    layout: &'static str,
    pointer_map: &'static str,
    pointer: &'static str,
    query: &'static str,
) -> VariantPlan {
    let bank = resolved_bank(pointer_map, pointer);
    let correct = remembered_side(layout, bank, query);
    let wrong = if correct == "LEFT" { "RIGHT" } else { "LEFT" };
    VariantPlan {
        task_id,
        layout,
        pointer_map,
        pointer,
        query,
        resolved_bank: bank,
        correct,
        wrong,
    }
}

fn qualify_variant(
    core_path: &Path,
    rom_path: &Path,
    plan: VariantPlan,
) -> Result<(VariantEvidence, String, String), String> {
    let task: &'static BenchmarkTaskSpec = benchmark_task_by_id(plan.task_id)
        .ok_or_else(|| format!("{} missing from benchmark registry", plan.task_id))?;
    let temp = env::temp_dir().join(format!("phicade-indirect-context-{}", task.id));
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

    no_input(
        &mut core,
        WARMUP_FRAMES,
        &mut video,
        &mut audio,
        "warm indirect-context briefing",
    )?;
    let briefing_player = locate_agent_gym_player(&video)?;
    if briefing_player != START || briefing_player != task.start {
        return Err(format!(
            "{} briefing geometry drifted: {:?}",
            task.id, briefing_player
        ));
    }
    let briefing_frame_sha256 = sha256_bytes(&video.rgba8);

    tap_a(
        &mut core,
        &mut sequence,
        &mut video,
        &mut audio,
        "erase indirect-context briefing",
    )?;
    no_input(
        &mut core,
        WAIT_AFTER_BRIEF_FRAMES,
        &mut video,
        &mut audio,
        "wait for indirect-context choice",
    )?;
    no_input(
        &mut core,
        1,
        &mut video,
        &mut audio,
        "arm indirect-context choice",
    )?;
    let choice_player = locate_agent_gym_player(&video)?;
    if choice_player != task.start {
        return Err(format!("{} moved before choice", task.id));
    }
    let choice_frame_sha256 = sha256_bytes(&video.rgba8);

    let choice_state = core
        .serialize_state()
        .map_err(|error| format!("serialize {} choice: {error:?}", task.id))?;
    let choice_mask = core.input_mask_snapshot();
    let choice_frame = core.frame_count();

    let correct_final_player = choose_and_commit(
        &mut core,
        &mut sequence,
        plan.correct,
        &mut video,
        &mut audio,
        "correct indirect-context door",
    )?;
    let correct_path_pass = benchmark_task_success(task, correct_final_player);

    restore(
        &mut core,
        &choice_state,
        choice_frame,
        choice_mask,
        &mut video,
        &mut audio,
        "restore indirect-context choice",
    )?;
    let wrong_final_player = choose_and_commit(
        &mut core,
        &mut sequence,
        plan.wrong,
        &mut video,
        &mut audio,
        "wrong indirect-context door",
    )?;
    let wrong_commit_refused = !benchmark_task_success(task, wrong_final_player);

    let fail_state = core
        .serialize_state()
        .map_err(|error| format!("serialize {} fail: {error:?}", task.id))?;
    let fail_mask = core.input_mask_snapshot();
    let fail_frame = core.frame_count();

    no_input(
        &mut core,
        MOVE_FRAMES + 1 + 2 + SETTLE_FRAMES,
        &mut video,
        &mut audio,
        "neutral failed future",
    )?;
    let neutral_terminal_player = locate_agent_gym_player(&video)?;
    let neutral_terminal_frame_sha256 = sha256_bytes(&video.rgba8);

    restore(
        &mut core,
        &fail_state,
        fail_frame,
        fail_mask,
        &mut video,
        &mut audio,
        "restore fail for recovery probe",
    )?;
    hold_button(
        &mut core,
        &mut sequence,
        plan.correct,
        MOVE_FRAMES.saturating_sub(1),
        &mut video,
        &mut audio,
        "probe failed recovery",
    )?;
    tap_a(
        &mut core,
        &mut sequence,
        &mut video,
        &mut audio,
        "probe A after failure",
    )?;
    no_input(
        &mut core,
        SETTLE_FRAMES,
        &mut video,
        &mut audio,
        "settle failed recovery",
    )?;
    let recovery_probe_player = locate_agent_gym_player(&video)?;
    let recovery_probe_frame_sha256 = sha256_bytes(&video.rgba8);
    let wrong_commit_terminal = neutral_terminal_player == recovery_probe_player
        && neutral_terminal_frame_sha256 == recovery_probe_frame_sha256
        && !benchmark_task_success(task, recovery_probe_player);

    let rom_sha256 = sha256_file(rom_path)?;
    let registry_hash_match = rom_sha256 == task.rom_sha256;

    Ok((
        VariantEvidence {
            task_id: task.id.into(),
            layout: plan.layout.into(),
            pointer_map: plan.pointer_map.into(),
            pointer: plan.pointer.into(),
            query: plan.query.into(),
            resolved_bank: plan.resolved_bank.into(),
            correct_side: plan.correct.into(),
            rom_sha256,
            registry_hash_match,
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
        },
        core_name,
        core_version,
    ))
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Indirect Context qualification failed: {error}");
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
        plan(AGENT_GYM_IND_A_NOR_STAR_TRI_ID, "A", "NORMAL", "STAR", "TRIANGLE"),
        plan(AGENT_GYM_IND_A_NOR_STAR_SQR_ID, "A", "NORMAL", "STAR", "SQUARE"),
        plan(AGENT_GYM_IND_A_NOR_MOON_TRI_ID, "A", "NORMAL", "MOON", "TRIANGLE"),
        plan(AGENT_GYM_IND_A_NOR_MOON_SQR_ID, "A", "NORMAL", "MOON", "SQUARE"),
        plan(AGENT_GYM_IND_A_SWP_STAR_TRI_ID, "A", "SWAPPED", "STAR", "TRIANGLE"),
        plan(AGENT_GYM_IND_A_SWP_STAR_SQR_ID, "A", "SWAPPED", "STAR", "SQUARE"),
        plan(AGENT_GYM_IND_A_SWP_MOON_TRI_ID, "A", "SWAPPED", "MOON", "TRIANGLE"),
        plan(AGENT_GYM_IND_A_SWP_MOON_SQR_ID, "A", "SWAPPED", "MOON", "SQUARE"),
        plan(AGENT_GYM_IND_B_NOR_STAR_TRI_ID, "B", "NORMAL", "STAR", "TRIANGLE"),
        plan(AGENT_GYM_IND_B_NOR_STAR_SQR_ID, "B", "NORMAL", "STAR", "SQUARE"),
        plan(AGENT_GYM_IND_B_NOR_MOON_TRI_ID, "B", "NORMAL", "MOON", "TRIANGLE"),
        plan(AGENT_GYM_IND_B_NOR_MOON_SQR_ID, "B", "NORMAL", "MOON", "SQUARE"),
        plan(AGENT_GYM_IND_B_SWP_STAR_TRI_ID, "B", "SWAPPED", "STAR", "TRIANGLE"),
        plan(AGENT_GYM_IND_B_SWP_STAR_SQR_ID, "B", "SWAPPED", "STAR", "SQUARE"),
        plan(AGENT_GYM_IND_B_SWP_MOON_TRI_ID, "B", "SWAPPED", "MOON", "TRIANGLE"),
        plan(AGENT_GYM_IND_B_SWP_MOON_SQR_ID, "B", "SWAPPED", "MOON", "SQUARE"),
    ];

    if supplied_roms.len() != plans.len() {
        return Err(format!(
            "expected {} --rom arguments, got {}",
            plans.len(),
            supplied_roms.len()
        ));
    }

    let mut variants = Vec::with_capacity(plans.len());
    let mut core_name = String::new();
    let mut core_version = String::new();

    for (index, p) in plans.iter().copied().enumerate() {
        let rom_path = supplied_roms
            .iter()
            .find(|(id, _)| id == p.task_id)
            .map(|(_, path)| path)
            .ok_or_else(|| format!("missing ROM for {}", p.task_id))?;
        let (evidence, name, version) = qualify_variant(&core_path, rom_path, p)?;
        if index == 0 {
            core_name = name;
            core_version = version;
        } else if name != core_name || version != core_version {
            return Err("indirect-context variants did not run under identical core identity".into());
        }
        variants.push(evidence);
    }

    let briefing_groups_condition_independent = [0usize, 4, 8, 12].iter().all(|&base| {
        variants[base..base + 4]
            .iter()
            .all(|v| v.briefing_frame_sha256 == variants[base].briefing_frame_sha256)
    });
    let briefing_hashes: BTreeSet<_> = [0usize, 4, 8, 12]
        .iter()
        .map(|&i| variants[i].briefing_frame_sha256.as_str())
        .collect();
    let briefing_histories_visibly_distinct = briefing_hashes.len() == 4;

    let same_pointer_query_choice_converges_across_histories = (0..4).all(|i| {
        variants[i].choice_frame_sha256 == variants[i + 4].choice_frame_sha256
            && variants[i].choice_frame_sha256 == variants[i + 8].choice_frame_sha256
            && variants[i].choice_frame_sha256 == variants[i + 12].choice_frame_sha256
    });

    let pointer_tokens_visibly_distinct = [0usize, 4, 8, 12].iter().all(|&base| {
        variants[base].choice_frame_sha256 != variants[base + 2].choice_frame_sha256
            && variants[base + 1].choice_frame_sha256
                != variants[base + 3].choice_frame_sha256
    });
    let queries_visibly_distinct = [0usize, 4, 8, 12].iter().all(|&base| {
        variants[base].choice_frame_sha256 != variants[base + 1].choice_frame_sha256
            && variants[base + 2].choice_frame_sha256
                != variants[base + 3].choice_frame_sha256
    });

    let all_choice_geometry_identical = variants.iter().all(|v| v.choice_player == START);
    let all_registry_hashes_match = variants.iter().all(|v| v.registry_hash_match);
    let all_correct_paths_pass = variants.iter().all(|v| v.correct_path_pass);
    let all_wrong_commits_terminal = variants
        .iter()
        .all(|v| v.wrong_commit_refused && v.wrong_commit_terminal);

    let fixed_left_succeeds_exactly_eight =
        variants.iter().filter(|v| v.correct_side == "LEFT").count() == 8;
    let fixed_right_succeeds_exactly_eight =
        variants.iter().filter(|v| v.correct_side == "RIGHT").count() == 8;

    let predict_map = |v: &VariantEvidence, assumed_map: &str| {
        remembered_side(
            &v.layout,
            resolved_bank(assumed_map, &v.pointer),
            &v.query,
        )
    };
    let assume_normal_map_succeeds_exactly_eight = variants
        .iter()
        .filter(|v| predict_map(v, "NORMAL") == v.correct_side)
        .count()
        == 8;
    let assume_swapped_map_succeeds_exactly_eight = variants
        .iter()
        .filter(|v| predict_map(v, "SWAPPED") == v.correct_side)
        .count()
        == 8;

    let always_circle_succeeds_exactly_eight = variants
        .iter()
        .filter(|v| remembered_side(&v.layout, "CIRCLE", &v.query) == v.correct_side)
        .count()
        == 8;
    let always_cross_succeeds_exactly_eight = variants
        .iter()
        .filter(|v| remembered_side(&v.layout, "CROSS", &v.query) == v.correct_side)
        .count()
        == 8;

    let predict_query = |v: &VariantEvidence, query: &str| {
        remembered_side(
            &v.layout,
            resolved_bank(&v.pointer_map, &v.pointer),
            query,
        )
    };
    let always_triangle_succeeds_exactly_eight = variants
        .iter()
        .filter(|v| predict_query(v, "TRIANGLE") == v.correct_side)
        .count()
        == 8;
    let always_square_succeeds_exactly_eight = variants
        .iter()
        .filter(|v| predict_query(v, "SQUARE") == v.correct_side)
        .count()
        == 8;

    if !(briefing_groups_condition_independent
        && briefing_histories_visibly_distinct
        && same_pointer_query_choice_converges_across_histories
        && pointer_tokens_visibly_distinct
        && queries_visibly_distinct
        && all_choice_geometry_identical
        && all_registry_hashes_match
        && all_correct_paths_pass
        && all_wrong_commits_terminal
        && fixed_left_succeeds_exactly_eight
        && fixed_right_succeeds_exactly_eight
        && assume_normal_map_succeeds_exactly_eight
        && assume_swapped_map_succeeds_exactly_eight
        && always_circle_succeeds_exactly_eight
        && always_cross_succeeds_exactly_eight
        && always_triangle_succeeds_exactly_eight
        && always_square_succeeds_exactly_eight)
    {
        let rom_hashes: BTreeSet<_> =
            variants.iter().map(|v| v.rom_sha256.as_str()).collect();
        return Err(format!(
            "indirect controls failed: briefing_groups={briefing_groups_condition_independent} briefing_distinct={briefing_histories_visibly_distinct} converge={same_pointer_query_choice_converges_across_histories} pointers={pointer_tokens_visibly_distinct} queries={queries_visibly_distinct} geometry={all_choice_geometry_identical} hashes={all_registry_hashes_match}/{} correct={all_correct_paths_pass} terminal={all_wrong_commits_terminal} left8={fixed_left_succeeds_exactly_eight} right8={fixed_right_succeeds_exactly_eight} normal8={assume_normal_map_succeeds_exactly_eight} swapped8={assume_swapped_map_succeeds_exactly_eight} circle8={always_circle_succeeds_exactly_eight} cross8={always_cross_succeeds_exactly_eight} tri8={always_triangle_succeeds_exactly_eight} sq8={always_square_succeeds_exactly_eight}",
            rom_hashes.len()
        ));
    }

    let receipt = Receipt {
        schema: "phicade.indirect-context-routing-qualification.v1",
        result: "PASS",
        core_sha256: sha256_file(&core_path)?,
        core_name,
        core_version,
        variants,
        briefing_groups_condition_independent,
        briefing_histories_visibly_distinct,
        same_pointer_query_choice_converges_across_histories,
        pointer_tokens_visibly_distinct,
        queries_visibly_distinct,
        all_choice_geometry_identical,
        all_registry_hashes_match,
        all_correct_paths_pass,
        all_wrong_commits_terminal,
        fixed_left_succeeds_exactly_eight,
        fixed_right_succeeds_exactly_eight,
        assume_normal_map_succeeds_exactly_eight,
        assume_swapped_map_succeeds_exactly_eight,
        always_circle_succeeds_exactly_eight,
        always_cross_succeeds_exactly_eight,
        always_triangle_succeeds_exactly_eight,
        always_square_succeeds_exactly_eight,
    };

    let json = serde_json::to_string_pretty(&receipt)
        .map_err(|error| format!("serialize indirect-context receipt: {error}"))?;
    println!("{json}");
    if let Some(parent) = receipt_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(&receipt_path, format!("{json}\n"))
        .map_err(|error| format!("write {}: {error}", receipt_path.display()))?;
    Ok(())
}
