use phicade_libretro::LibretroCore;
use phicade_runtime::{
    benchmark_task_by_id, benchmark_task_success, locate_agent_gym_player, ActionEnvelope,
    ActionKind, ActionSource, AudioBuffer, BenchmarkTaskSpec, EmulatorCore, FrameBuffer,
    GameImage, PixelPoint, SystemId,
    AGENT_GYM_CTX_A_CIR_TM_ID, AGENT_GYM_CTX_A_CIR_TF_ID,
    AGENT_GYM_CTX_A_CIR_SM_ID, AGENT_GYM_CTX_A_CIR_SF_ID,
    AGENT_GYM_CTX_A_CRS_TM_ID, AGENT_GYM_CTX_A_CRS_TF_ID,
    AGENT_GYM_CTX_A_CRS_SM_ID, AGENT_GYM_CTX_A_CRS_SF_ID,
    AGENT_GYM_CTX_B_CIR_TM_ID, AGENT_GYM_CTX_B_CIR_TF_ID,
    AGENT_GYM_CTX_B_CIR_SM_ID, AGENT_GYM_CTX_B_CIR_SF_ID,
    AGENT_GYM_CTX_B_CRS_TM_ID, AGENT_GYM_CTX_B_CRS_TF_ID,
    AGENT_GYM_CTX_B_CRS_SM_ID, AGENT_GYM_CTX_B_CRS_SF_ID,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, env, fs, path::{Path, PathBuf}, process};

const WARMUP_FRAMES: u64 = 120;
const WAIT_AFTER_BRIEF_FRAMES: u64 = 101;
const MOVE_FRAMES: u64 = 24;
const SETTLE_FRAMES: u64 = 4;
const START: PixelPoint = PixelPoint { x: 72, y: 96 };

#[derive(Debug, Clone, Copy)]
struct VariantPlan {
    task_id: &'static str,
    layout: &'static str,
    bank: &'static str,
    query: &'static str,
    operator: &'static str,
    remembered: &'static str,
    correct: &'static str,
    wrong: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct VariantEvidence {
    task_id: String,
    layout: String,
    bank: String,
    query: String,
    operator: String,
    remembered_side: String,
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
    layout_a_briefing_condition_independent: bool,
    layout_b_briefing_condition_independent: bool,
    layouts_visibly_distinct: bool,
    same_condition_choice_converges_across_layouts: bool,
    bank_selectors_visibly_distinct: bool,
    queries_visibly_distinct: bool,
    operators_visibly_distinct: bool,
    all_choice_geometry_identical: bool,
    all_registry_hashes_match: bool,
    all_correct_paths_pass: bool,
    all_wrong_commits_terminal: bool,
    fixed_left_succeeds_exactly_eight: bool,
    fixed_right_succeeds_exactly_eight: bool,
    ignore_operator_succeeds_exactly_eight: bool,
    always_flip_succeeds_exactly_eight: bool,
    always_circle_succeeds_exactly_eight: bool,
    always_cross_succeeds_exactly_eight: bool,
    always_triangle_succeeds_exactly_eight: bool,
    always_square_succeeds_exactly_eight: bool,
}

fn usage() -> ! {
    eprintln!("usage: selective_context_qualify --core <sameboy> --rom <task-id=path> [--rom ... x16] --receipt <path>");
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
        source: ActionSource::Script { name: "selective-context-qualifier".into() },
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
    core.restore_state(state, frame).map_err(|error| format!("{context}: restore: {error:?}"))?;
    core.restore_input_mask(mask);
    no_input(core, 1, video, audio, context)
}

fn choose_and_commit(core: &mut LibretroCore, sequence: &mut u64, direction: &str, video: &mut FrameBuffer, audio: &mut AudioBuffer, context: &str) -> Result<PixelPoint, String> {
    hold_button(core, sequence, direction, MOVE_FRAMES, video, audio, context)?;
    tap_a(core, sequence, video, audio, "commit selective-context door")?;
    no_input(core, SETTLE_FRAMES, video, audio, "settle selective-context commitment")?;
    locate_agent_gym_player(video)
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

fn apply_operator(side: &str, operator: &str) -> &'static str {
    if operator == "MATCH" {
        if side == "LEFT" { "LEFT" } else { "RIGHT" }
    } else if side == "LEFT" {
        "RIGHT"
    } else {
        "LEFT"
    }
}

fn plan(task_id: &'static str, layout: &'static str, bank: &'static str, query: &'static str, operator: &'static str) -> VariantPlan {
    let remembered = remembered_side(layout, bank, query);
    let correct = apply_operator(remembered, operator);
    let wrong = if correct == "LEFT" { "RIGHT" } else { "LEFT" };
    VariantPlan { task_id, layout, bank, query, operator, remembered, correct, wrong }
}

fn qualify_variant(core_path: &Path, rom_path: &Path, plan: VariantPlan) -> Result<(VariantEvidence, String, String), String> {
    let task: &'static BenchmarkTaskSpec = benchmark_task_by_id(plan.task_id)
        .ok_or_else(|| format!("{} missing from benchmark registry", plan.task_id))?;
    let temp = env::temp_dir().join(format!("phicade-context-{}", task.id));
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

    no_input(&mut core, WARMUP_FRAMES, &mut video, &mut audio, "warm context briefing")?;
    let briefing_player = locate_agent_gym_player(&video)?;
    if briefing_player != START || briefing_player != task.start {
        return Err(format!("{} briefing geometry drifted: {:?}", task.id, briefing_player));
    }
    let briefing_frame_sha256 = sha256_bytes(&video.rgba8);

    tap_a(&mut core, &mut sequence, &mut video, &mut audio, "erase context briefing")?;
    no_input(&mut core, WAIT_AFTER_BRIEF_FRAMES, &mut video, &mut audio, "wait for context choice")?;
    no_input(&mut core, 1, &mut video, &mut audio, "arm context choice")?;
    let choice_player = locate_agent_gym_player(&video)?;
    if choice_player != task.start {
        return Err(format!("{} moved before choice", task.id));
    }
    let choice_frame_sha256 = sha256_bytes(&video.rgba8);

    let choice_state = core.serialize_state()
        .map_err(|error| format!("serialize {} choice: {error:?}", task.id))?;
    let choice_mask = core.input_mask_snapshot();
    let choice_frame = core.frame_count();

    let correct_final_player = choose_and_commit(&mut core, &mut sequence, plan.correct, &mut video, &mut audio, "correct context door")?;
    let correct_path_pass = benchmark_task_success(task, correct_final_player);

    restore(&mut core, &choice_state, choice_frame, choice_mask, &mut video, &mut audio, "restore context choice")?;
    let wrong_final_player = choose_and_commit(&mut core, &mut sequence, plan.wrong, &mut video, &mut audio, "wrong context door")?;
    let wrong_commit_refused = !benchmark_task_success(task, wrong_final_player);

    let fail_state = core.serialize_state()
        .map_err(|error| format!("serialize {} fail: {error:?}", task.id))?;
    let fail_mask = core.input_mask_snapshot();
    let fail_frame = core.frame_count();

    no_input(&mut core, MOVE_FRAMES + 1 + 2 + SETTLE_FRAMES, &mut video, &mut audio, "neutral failed future")?;
    let neutral_terminal_player = locate_agent_gym_player(&video)?;
    let neutral_terminal_frame_sha256 = sha256_bytes(&video.rgba8);

    restore(&mut core, &fail_state, fail_frame, fail_mask, &mut video, &mut audio, "restore fail for recovery probe")?;
    hold_button(&mut core, &mut sequence, plan.correct, MOVE_FRAMES.saturating_sub(1), &mut video, &mut audio, "probe failed recovery")?;
    tap_a(&mut core, &mut sequence, &mut video, &mut audio, "probe A after failure")?;
    no_input(&mut core, SETTLE_FRAMES, &mut video, &mut audio, "settle failed recovery")?;
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
        layout: plan.layout.into(),
        bank: plan.bank.into(),
        query: plan.query.into(),
        operator: plan.operator.into(),
        remembered_side: plan.remembered.into(),
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
    }, core_name, core_version))
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Selective Context qualification failed: {error}");
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
        plan(AGENT_GYM_CTX_A_CIR_TM_ID, "A", "CIRCLE", "TRIANGLE", "MATCH"),
        plan(AGENT_GYM_CTX_A_CIR_TF_ID, "A", "CIRCLE", "TRIANGLE", "FLIP"),
        plan(AGENT_GYM_CTX_A_CIR_SM_ID, "A", "CIRCLE", "SQUARE", "MATCH"),
        plan(AGENT_GYM_CTX_A_CIR_SF_ID, "A", "CIRCLE", "SQUARE", "FLIP"),
        plan(AGENT_GYM_CTX_A_CRS_TM_ID, "A", "CROSS", "TRIANGLE", "MATCH"),
        plan(AGENT_GYM_CTX_A_CRS_TF_ID, "A", "CROSS", "TRIANGLE", "FLIP"),
        plan(AGENT_GYM_CTX_A_CRS_SM_ID, "A", "CROSS", "SQUARE", "MATCH"),
        plan(AGENT_GYM_CTX_A_CRS_SF_ID, "A", "CROSS", "SQUARE", "FLIP"),
        plan(AGENT_GYM_CTX_B_CIR_TM_ID, "B", "CIRCLE", "TRIANGLE", "MATCH"),
        plan(AGENT_GYM_CTX_B_CIR_TF_ID, "B", "CIRCLE", "TRIANGLE", "FLIP"),
        plan(AGENT_GYM_CTX_B_CIR_SM_ID, "B", "CIRCLE", "SQUARE", "MATCH"),
        plan(AGENT_GYM_CTX_B_CIR_SF_ID, "B", "CIRCLE", "SQUARE", "FLIP"),
        plan(AGENT_GYM_CTX_B_CRS_TM_ID, "B", "CROSS", "TRIANGLE", "MATCH"),
        plan(AGENT_GYM_CTX_B_CRS_TF_ID, "B", "CROSS", "TRIANGLE", "FLIP"),
        plan(AGENT_GYM_CTX_B_CRS_SM_ID, "B", "CROSS", "SQUARE", "MATCH"),
        plan(AGENT_GYM_CTX_B_CRS_SF_ID, "B", "CROSS", "SQUARE", "FLIP"),
    ];

    if supplied_roms.len() != plans.len() {
        return Err(format!("expected {} --rom arguments, got {}", plans.len(), supplied_roms.len()));
    }

    let mut variants = Vec::with_capacity(plans.len());
    let mut core_name = String::new();
    let mut core_version = String::new();
    for (index, p) in plans.iter().copied().enumerate() {
        let rom_path = supplied_roms.iter().find(|(id, _)| id == p.task_id)
            .map(|(_, path)| path).ok_or_else(|| format!("missing ROM for {}", p.task_id))?;
        let (evidence, name, version) = qualify_variant(&core_path, rom_path, p)?;
        if index == 0 { core_name = name; core_version = version; }
        else if name != core_name || version != core_version {
            return Err("context variants did not run under identical core identity".into());
        }
        variants.push(evidence);
    }

    let layout_a_briefing_condition_independent =
        variants[..8].iter().all(|v| v.briefing_frame_sha256 == variants[0].briefing_frame_sha256);
    let layout_b_briefing_condition_independent =
        variants[8..].iter().all(|v| v.briefing_frame_sha256 == variants[8].briefing_frame_sha256);
    let layouts_visibly_distinct =
        variants[0].briefing_frame_sha256 != variants[8].briefing_frame_sha256;

    let same_condition_choice_converges_across_layouts =
        (0..8).all(|i| variants[i].choice_frame_sha256 == variants[i + 8].choice_frame_sha256);

    let bank_selectors_visibly_distinct = (0..4).all(|i|
        variants[i].choice_frame_sha256 != variants[i + 4].choice_frame_sha256
        && variants[i + 8].choice_frame_sha256 != variants[i + 12].choice_frame_sha256
    );

    let queries_visibly_distinct =
        [0usize, 1, 4, 5, 8, 9, 12, 13].iter().all(|&i|
            variants[i].choice_frame_sha256 != variants[i + 2].choice_frame_sha256
        );

    let operators_visibly_distinct =
        (0..variants.len()).step_by(2).all(|i|
            variants[i].choice_frame_sha256 != variants[i + 1].choice_frame_sha256
        );

    let all_choice_geometry_identical = variants.iter().all(|v| v.choice_player == START);
    let all_registry_hashes_match = variants.iter().all(|v| v.registry_hash_match);
    let all_correct_paths_pass = variants.iter().all(|v| v.correct_path_pass);
    let all_wrong_commits_terminal = variants.iter().all(|v| v.wrong_commit_refused && v.wrong_commit_terminal);

    let fixed_left_succeeds_exactly_eight = variants.iter().filter(|v| v.correct_side == "LEFT").count() == 8;
    let fixed_right_succeeds_exactly_eight = variants.iter().filter(|v| v.correct_side == "RIGHT").count() == 8;
    let ignore_operator_succeeds_exactly_eight =
        variants.iter().filter(|v| v.remembered_side == v.correct_side).count() == 8;
    let always_flip_succeeds_exactly_eight =
        variants.iter().filter(|v| v.remembered_side != v.correct_side).count() == 8;

    let predict = |v: &VariantEvidence, bank: &str, query: &str| {
        let side = remembered_side(&v.layout, bank, query);
        apply_operator(side, &v.operator)
    };
    let always_circle_succeeds_exactly_eight =
        variants.iter().filter(|v| predict(v, "CIRCLE", &v.query) == v.correct_side).count() == 8;
    let always_cross_succeeds_exactly_eight =
        variants.iter().filter(|v| predict(v, "CROSS", &v.query) == v.correct_side).count() == 8;
    let always_triangle_succeeds_exactly_eight =
        variants.iter().filter(|v| predict(v, &v.bank, "TRIANGLE") == v.correct_side).count() == 8;
    let always_square_succeeds_exactly_eight =
        variants.iter().filter(|v| predict(v, &v.bank, "SQUARE") == v.correct_side).count() == 8;

    if !(layout_a_briefing_condition_independent
        && layout_b_briefing_condition_independent
        && layouts_visibly_distinct
        && same_condition_choice_converges_across_layouts
        && bank_selectors_visibly_distinct
        && queries_visibly_distinct
        && operators_visibly_distinct
        && all_choice_geometry_identical
        && all_registry_hashes_match
        && all_correct_paths_pass
        && all_wrong_commits_terminal
        && fixed_left_succeeds_exactly_eight
        && fixed_right_succeeds_exactly_eight
        && ignore_operator_succeeds_exactly_eight
        && always_flip_succeeds_exactly_eight
        && always_circle_succeeds_exactly_eight
        && always_cross_succeeds_exactly_eight
        && always_triangle_succeeds_exactly_eight
        && always_square_succeeds_exactly_eight)
    {
        let hashes: BTreeSet<_> = variants.iter().map(|v| v.rom_sha256.as_str()).collect();
        return Err(format!(
            "context controls failed: a_brief={layout_a_briefing_condition_independent} b_brief={layout_b_briefing_condition_independent} layouts={layouts_visibly_distinct} converge={same_condition_choice_converges_across_layouts} banks={bank_selectors_visibly_distinct} queries={queries_visibly_distinct} operators={operators_visibly_distinct} geometry={all_choice_geometry_identical} hashes={all_registry_hashes_match}/{} correct={all_correct_paths_pass} terminal={all_wrong_commits_terminal} left8={fixed_left_succeeds_exactly_eight} right8={fixed_right_succeeds_exactly_eight} ignore_op8={ignore_operator_succeeds_exactly_eight} flip8={always_flip_succeeds_exactly_eight} circle8={always_circle_succeeds_exactly_eight} cross8={always_cross_succeeds_exactly_eight} tri8={always_triangle_succeeds_exactly_eight} sq8={always_square_succeeds_exactly_eight}",
            hashes.len()
        ));
    }

    let receipt = Receipt {
        schema: "phicade.selective-context-routing-qualification.v1",
        result: "PASS",
        core_sha256: sha256_file(&core_path)?,
        core_name,
        core_version,
        variants,
        layout_a_briefing_condition_independent,
        layout_b_briefing_condition_independent,
        layouts_visibly_distinct,
        same_condition_choice_converges_across_layouts,
        bank_selectors_visibly_distinct,
        queries_visibly_distinct,
        operators_visibly_distinct,
        all_choice_geometry_identical,
        all_registry_hashes_match,
        all_correct_paths_pass,
        all_wrong_commits_terminal,
        fixed_left_succeeds_exactly_eight,
        fixed_right_succeeds_exactly_eight,
        ignore_operator_succeeds_exactly_eight,
        always_flip_succeeds_exactly_eight,
        always_circle_succeeds_exactly_eight,
        always_cross_succeeds_exactly_eight,
        always_triangle_succeeds_exactly_eight,
        always_square_succeeds_exactly_eight,
    };

    let json = serde_json::to_string_pretty(&receipt)
        .map_err(|error| format!("serialize context receipt: {error}"))?;
    println!("{json}");
    if let Some(parent) = receipt_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(&receipt_path, format!("{json}\n"))
        .map_err(|error| format!("write {}: {error}", receipt_path.display()))?;
    Ok(())
}
