use phicade_libretro::LibretroCore;
use phicade_runtime::{
    agent_gym_score_1000, benchmark_task_by_id, benchmark_task_distance,
    benchmark_task_success, locate_agent_gym_player, ActionEnvelope, ActionKind, ActionSource,
    AudioBuffer, BenchmarkTaskSpec, EmulatorCore, FrameBuffer, GameImage, OracleLeg, PixelPoint,
    SystemId,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

const SETTLE_FRAMES: u64 = 6;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentGymQualificationReceipt {
    schema: &'static str,
    result: &'static str,
    suite_id: String,
    task_id: String,
    task_title: String,
    rom_sha256: String,
    source_sha256: String,
    registry_rom_sha256: String,
    registry_source_sha256: String,
    core_sha256: String,
    core_name: String,
    core_version: String,
    start_frame: u64,
    end_frame: u64,
    start_player: PixelPoint,
    target: PixelPoint,
    initial_distance: i32,
    no_input_final_player: PixelPoint,
    no_input_distance: i32,
    no_input_progress: i32,
    oracle_final_player: PixelPoint,
    oracle_final_distance: i32,
    oracle_progress: i32,
    oracle_score_1000: u16,
    oracle_success: bool,
    no_input_control_pass: bool,
    oracle_control_pass: bool,
    deterministic_replay_pass: bool,
    registry_hashes_match: bool,
    oracle_final_frame_sha256: String,
    oracle_replay_frame_sha256: String,
}

fn usage() -> ! {
    eprintln!(
        "usage: agent_gym_qualify --core <sameboy_libretro> --rom <benchmark.gb> --source <main.asm> --task <task-id> --receipt <path>"
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
            name: "phi-agent-gym-oracle".into(),
        },
        action: ActionKind::Button {
            button: button.into(),
            pressed,
        },
    }
}

fn run_no_input(
    core: &mut LibretroCore,
    frames: u64,
    video: &mut FrameBuffer,
    audio: &mut AudioBuffer,
) -> Result<(), String> {
    for _ in 0..frames {
        core.step_frame(&[], video, audio)
            .map_err(|error| format!("gym no-input frame: {error:?}"))?;
    }
    Ok(())
}

fn validate_oracle_leg(leg: OracleLeg) -> Result<(), String> {
    if leg.frames == 0 {
        return Err(format!("oracle leg {} has zero frames", leg.button));
    }
    if !matches!(leg.button, "UP" | "DOWN" | "LEFT" | "RIGHT") {
        return Err(format!("oracle uses unsupported button {}", leg.button));
    }
    Ok(())
}

fn run_oracle(
    core: &mut LibretroCore,
    task: &BenchmarkTaskSpec,
    video: &mut FrameBuffer,
    audio: &mut AudioBuffer,
) -> Result<(), String> {
    let first = task.oracle[0];
    let second = task.oracle[1];
    validate_oracle_leg(first)?;
    validate_oracle_leg(second)?;

    let mut sequence = 0u64;

    let first_press = button_event(sequence, core.frame_count(), first.button, true);
    sequence += 1;
    core.step_frame(&[first_press], video, audio)
        .map_err(|error| format!("gym oracle {} press: {error:?}", first.button))?;

    for _ in 1..first.frames {
        core.step_frame(&[], video, audio)
            .map_err(|error| format!("gym oracle {} hold: {error:?}", first.button))?;
    }

    let frame = core.frame_count();
    let first_release = button_event(sequence, frame, first.button, false);
    sequence += 1;
    let second_press = button_event(sequence, frame, second.button, true);
    sequence += 1;
    core.step_frame(&[first_release, second_press], video, audio)
        .map_err(|error| {
            format!(
                "gym oracle switch {} -> {}: {error:?}",
                first.button, second.button
            )
        })?;

    for _ in 1..second.frames {
        core.step_frame(&[], video, audio)
            .map_err(|error| format!("gym oracle {} hold: {error:?}", second.button))?;
    }

    let second_release = button_event(sequence, core.frame_count(), second.button, false);
    core.step_frame(&[second_release], video, audio)
        .map_err(|error| format!("gym oracle {} release: {error:?}", second.button))?;

    for _ in 0..SETTLE_FRAMES {
        core.step_frame(&[], video, audio)
            .map_err(|error| format!("gym oracle settle: {error:?}"))?;
    }

    Ok(())
}

fn frozen_registry_hash_matches(expected: &str, observed: &str) -> bool {
    expected == observed
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Phi-Agent Gym task qualification failed: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let mut core_path: Option<PathBuf> = None;
    let mut rom_path: Option<PathBuf> = None;
    let mut source_path: Option<PathBuf> = None;
    let mut task_id: Option<String> = None;
    let mut receipt_path: Option<PathBuf> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--core" => core_path = args.next().map(PathBuf::from),
            "--rom" => rom_path = args.next().map(PathBuf::from),
            "--source" => source_path = args.next().map(PathBuf::from),
            "--task" => task_id = args.next(),
            "--receipt" => receipt_path = args.next().map(PathBuf::from),
            _ => usage(),
        }
    }

    let core_path = core_path.unwrap_or_else(|| usage());
    let rom_path = rom_path.unwrap_or_else(|| usage());
    let source_path = source_path.unwrap_or_else(|| usage());
    let task_id = task_id.unwrap_or_else(|| usage());
    let receipt_path = receipt_path.unwrap_or_else(|| usage());
    let task = benchmark_task_by_id(&task_id)
        .ok_or_else(|| format!("unknown benchmark task id {task_id}"))?;

    let temp = env::temp_dir().join(format!("phicade-agent-gym-{}", task.id));
    let system_dir = temp.join("system");
    let save_dir = temp.join("save");
    fs::create_dir_all(&system_dir).map_err(|error| error.to_string())?;
    fs::create_dir_all(&save_dir).map_err(|error| error.to_string())?;

    let mut core = LibretroCore::open(&core_path, &system_dir, &save_dir)
        .map_err(|error| format!("open SameBoy: {error:?}"))?;
    core.load_game(&GameImage::new(&rom_path, SystemId::GameBoy, task.id))
        .map_err(|error| format!("load benchmark {}: {error:?}", task.id))?;

    let mut video = FrameBuffer::default();
    let mut audio = AudioBuffer::default();
    run_no_input(&mut core, task.warmup_frames, &mut video, &mut audio)?;

    let start_frame = core.frame_count();
    let start_player = locate_agent_gym_player(&video)?;
    let initial_distance = benchmark_task_distance(task, start_player);
    if start_player != task.start || initial_distance != task.initial_distance {
        return Err(format!(
            "task {} start geometry drifted: expected {:?}/{} got {:?}/{}",
            task.id, task.start, task.initial_distance, start_player, initial_distance
        ));
    }

    let frozen_state = core
        .serialize_state()
        .map_err(|error| format!("freeze benchmark start state: {error:?}"))?;
    let frozen_mask = core.input_mask_snapshot();

    let run_frames = task.oracle[0]
        .frames
        .saturating_add(task.oracle[1].frames)
        .saturating_add(SETTLE_FRAMES)
        .saturating_add(1);
    run_no_input(&mut core, run_frames, &mut video, &mut audio)?;
    let no_input_final_player = locate_agent_gym_player(&video)?;
    let no_input_distance = benchmark_task_distance(task, no_input_final_player);
    let no_input_progress = initial_distance - no_input_distance;
    let no_input_control_pass = no_input_final_player == task.start && no_input_progress == 0;

    core.restore_state(&frozen_state, start_frame)
        .map_err(|error| format!("restore benchmark oracle state: {error:?}"))?;
    core.restore_input_mask(frozen_mask);
    run_oracle(&mut core, task, &mut video, &mut audio)?;
    let oracle_final_player = locate_agent_gym_player(&video)?;
    let oracle_final_distance = benchmark_task_distance(task, oracle_final_player);
    let oracle_progress = initial_distance - oracle_final_distance;
    let oracle_score_1000 = agent_gym_score_1000(initial_distance, oracle_final_distance);
    let oracle_success = benchmark_task_success(task, oracle_final_player);
    let oracle_control_pass = oracle_success && oracle_score_1000 >= 980;
    let oracle_final_frame_sha256 = sha256_bytes(&video.rgba8);
    let end_frame = core.frame_count();

    core.restore_state(&frozen_state, start_frame)
        .map_err(|error| format!("restore benchmark deterministic replay state: {error:?}"))?;
    core.restore_input_mask(frozen_mask);
    run_oracle(&mut core, task, &mut video, &mut audio)?;
    let oracle_replay_frame_sha256 = sha256_bytes(&video.rgba8);
    let deterministic_replay_pass = oracle_final_frame_sha256 == oracle_replay_frame_sha256;

    let observed_rom_sha256 = sha256_file(&rom_path)?;
    let observed_source_sha256 = sha256_file(&source_path)?;
    let registry_hashes_match =
        frozen_registry_hash_matches(task.rom_sha256, &observed_rom_sha256)
            && frozen_registry_hash_matches(task.source_sha256, &observed_source_sha256);

    if !(no_input_control_pass
        && oracle_control_pass
        && deterministic_replay_pass
        && registry_hashes_match)
    {
        return Err(format!(
            "task {} qualification failed: no_input={} oracle={} deterministic={} registry={} start={start_player:?} final={oracle_final_player:?} distance={oracle_final_distance}",
            task.id,
            no_input_control_pass,
            oracle_control_pass,
            deterministic_replay_pass,
            registry_hashes_match,
        ));
    }

    let receipt = AgentGymQualificationReceipt {
        schema: "phicade.agent-gym-qualification.v2",
        result: "PASS",
        suite_id: task.suite_id.into(),
        task_id: task.id.into(),
        task_title: task.title.into(),
        rom_sha256: observed_rom_sha256,
        source_sha256: observed_source_sha256,
        registry_rom_sha256: task.rom_sha256.into(),
        registry_source_sha256: task.source_sha256.into(),
        core_sha256: sha256_file(&core_path)?,
        core_name: core.identity().library_name.clone(),
        core_version: core.identity().library_version.clone(),
        start_frame,
        end_frame,
        start_player,
        target: task.target,
        initial_distance,
        no_input_final_player,
        no_input_distance,
        no_input_progress,
        oracle_final_player,
        oracle_final_distance,
        oracle_progress,
        oracle_score_1000,
        oracle_success,
        no_input_control_pass,
        oracle_control_pass,
        deterministic_replay_pass,
        registry_hashes_match,
        oracle_final_frame_sha256,
        oracle_replay_frame_sha256,
    };

    let json = serde_json::to_string_pretty(&receipt)
        .map_err(|error| format!("serialize benchmark receipt: {error}"))?;
    println!("{json}");

    if let Some(parent) = receipt_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(&receipt_path, format!("{json}\n"))
        .map_err(|error| format!("write {}: {error}", receipt_path.display()))?;

    Ok(())
}
