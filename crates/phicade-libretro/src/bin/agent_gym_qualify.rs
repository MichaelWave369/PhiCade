use phicade_libretro::LibretroCore;
use phicade_runtime::{
    agent_gym_distance, agent_gym_score_1000, agent_gym_success, locate_agent_gym_player,
    ActionEnvelope, ActionKind, ActionSource, AudioBuffer, EmulatorCore, FrameBuffer, GameImage,
    PixelPoint, SystemId, AGENT_GYM_INITIAL_DISTANCE, AGENT_GYM_TARGET, AGENT_GYM_AGENT_GYM_WARMUP_FRAMES,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

const RIGHT_FRAMES: u64 = 60;
const DOWN_FRAMES: u64 = 44;
const SETTLE_FRAMES: u64 = 6;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentGymQualificationReceipt {
    schema: &'static str,
    result: &'static str,
    rom_sha256: String,
    source_sha256: String,
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
    oracle_final_frame_sha256: String,
    oracle_replay_frame_sha256: String,
}

fn usage() -> ! {
    eprintln!(
        "usage: agent_gym_qualify --core <sameboy_libretro> --rom <phi-agent-gym.gb> --source <main.asm> --receipt <path>"
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

fn dark(pixel: &[u8]) -> bool {
    if pixel.len() < 4 {
        return false;
    }
    let r = u16::from(pixel[0]);
    let g = u16::from(pixel[1]);
    let b = u16::from(pixel[2]);
    (r + g + b) / 3 < 112
}

fn locate_player(video: &FrameBuffer) -> Result<PixelPoint, String> {
    let width = usize::try_from(video.width).map_err(|_| "video width overflow")?;
    let height = usize::try_from(video.height).map_err(|_| "video height overflow")?;
    if width < PLAYER_SIZE || height < PLAYER_SIZE {
        return Err(format!("unexpected gym framebuffer {}x{}", width, height));
    }
    if video.rgba8.len() != width * height * 4 {
        return Err("gym framebuffer byte length mismatch".into());
    }

    let mut best: Option<(usize, usize, usize)> = None;
    for y in 0..=height - PLAYER_SIZE {
        for x in 0..=width - PLAYER_SIZE {
            let mut dark_count = 0usize;
            for py in y..y + PLAYER_SIZE {
                let row = py * width * 4;
                for px in x..x + PLAYER_SIZE {
                    let offset = row + px * 4;
                    if dark(&video.rgba8[offset..offset + 4]) {
                        dark_count += 1;
                    }
                }
            }

            if best.is_none_or(|(_, _, count)| dark_count > count) {
                best = Some((x, y, dark_count));
            }
        }
    }

    let (x, y, count) = best.ok_or_else(|| "no candidate player patch found".to_owned())?;
    if count < 52 {
        return Err(format!(
            "solid player patch not found: best 8x8 dark-pixel count was {count}"
        ));
    }

    Ok(PixelPoint {
        x: i32::try_from(x).map_err(|_| "player x overflow")?,
        y: i32::try_from(y).map_err(|_| "player y overflow")?,
    })
}

fn distance(point: PixelPoint) -> i32 {
    (point.x - TARGET_X).abs() + (point.y - TARGET_Y).abs()
}

fn score(initial: i32, final_distance: i32) -> u16 {
    if initial <= 0 {
        return 1000;
    }
    let progress = (initial - final_distance).clamp(0, initial);
    u16::try_from((i64::from(progress) * 1000) / i64::from(initial)).unwrap_or(0)
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

fn run_oracle(
    core: &mut LibretroCore,
    video: &mut FrameBuffer,
    audio: &mut AudioBuffer,
) -> Result<(), String> {
    let mut sequence = 0u64;

    let right_press = button_event(sequence, core.frame_count(), "RIGHT", true);
    sequence += 1;
    core.step_frame(&[right_press], video, audio)
        .map_err(|error| format!("gym oracle RIGHT press: {error:?}"))?;

    for _ in 1..RIGHT_FRAMES {
        core.step_frame(&[], video, audio)
            .map_err(|error| format!("gym oracle RIGHT hold: {error:?}"))?;
    }

    let frame = core.frame_count();
    let right_release = button_event(sequence, frame, "RIGHT", false);
    sequence += 1;
    let down_press = button_event(sequence, frame, "DOWN", true);
    sequence += 1;
    core.step_frame(&[right_release, down_press], video, audio)
        .map_err(|error| format!("gym oracle direction switch: {error:?}"))?;

    for _ in 1..DOWN_FRAMES {
        core.step_frame(&[], video, audio)
            .map_err(|error| format!("gym oracle DOWN hold: {error:?}"))?;
    }

    let down_release = button_event(sequence, core.frame_count(), "DOWN", false);
    core.step_frame(&[down_release], video, audio)
        .map_err(|error| format!("gym oracle DOWN release: {error:?}"))?;

    for _ in 0..SETTLE_FRAMES {
        core.step_frame(&[], video, audio)
            .map_err(|error| format!("gym oracle settle: {error:?}"))?;
    }

    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Phi-Agent Gym qualification failed: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let mut core_path: Option<PathBuf> = None;
    let mut rom_path: Option<PathBuf> = None;
    let mut source_path: Option<PathBuf> = None;
    let mut receipt_path: Option<PathBuf> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--core" => core_path = args.next().map(PathBuf::from),
            "--rom" => rom_path = args.next().map(PathBuf::from),
            "--source" => source_path = args.next().map(PathBuf::from),
            "--receipt" => receipt_path = args.next().map(PathBuf::from),
            _ => usage(),
        }
    }

    let core_path = core_path.unwrap_or_else(|| usage());
    let rom_path = rom_path.unwrap_or_else(|| usage());
    let source_path = source_path.unwrap_or_else(|| usage());
    let receipt_path = receipt_path.unwrap_or_else(|| usage());

    let temp = env::temp_dir().join("phicade-agent-gym-qualify");
    let system_dir = temp.join("system");
    let save_dir = temp.join("save");
    fs::create_dir_all(&system_dir).map_err(|error| error.to_string())?;
    fs::create_dir_all(&save_dir).map_err(|error| error.to_string())?;

    let mut core = LibretroCore::open(&core_path, &system_dir, &save_dir)
        .map_err(|error| format!("open SameBoy: {error:?}"))?;
    core.load_game(&GameImage::new(&rom_path, SystemId::GameBoy, "phi-agent-gym"))
        .map_err(|error| format!("load Phi-Agent Gym: {error:?}"))?;

    let mut video = FrameBuffer::default();
    let mut audio = AudioBuffer::default();
    run_no_input(&mut core, AGENT_GYM_WARMUP_FRAMES, &mut video, &mut audio)?;

    let start_frame = core.frame_count();
    let start_player = locate_player(&video)?;
    let initial_distance = agent_gym_distance(start_player);
    if initial_distance != AGENT_GYM_INITIAL_DISTANCE {
        return Err(format!(
            "gym start geometry drifted: expected {AGENT_GYM_INITIAL_DISTANCE}, got {initial_distance}"
        ));
    }

    let frozen_state = core
        .serialize_state()
        .map_err(|error| format!("freeze gym start state: {error:?}"))?;
    let frozen_mask = core.input_mask_snapshot();

    let run_frames = RIGHT_FRAMES + DOWN_FRAMES + SETTLE_FRAMES + 1;
    run_no_input(&mut core, run_frames, &mut video, &mut audio)?;
    let no_input_final_player = locate_player(&video)?;
    let no_input_distance = agent_gym_distance(no_input_final_player);
    let no_input_progress = initial_distance - no_input_distance;
    let no_input_control_pass = no_input_progress == 0;

    core.restore_state(&frozen_state, start_frame)
        .map_err(|error| format!("restore gym oracle state: {error:?}"))?;
    core.restore_input_mask(frozen_mask);
    run_oracle(&mut core, &mut video, &mut audio)?;
    let oracle_final_player = locate_player(&video)?;
    let oracle_final_distance = agent_gym_distance(oracle_final_player);
    let oracle_progress = initial_distance - oracle_final_distance;
    let oracle_score_1000 = agent_gym_score_1000(initial_distance, oracle_final_distance);
    let oracle_success = agent_gym_success(oracle_final_player);
    let oracle_control_pass = oracle_success && oracle_score_1000 >= 980;
    let oracle_final_frame_sha256 = sha256_bytes(&video.rgba8);
    let end_frame = core.frame_count();

    core.restore_state(&frozen_state, start_frame)
        .map_err(|error| format!("restore gym deterministic replay state: {error:?}"))?;
    core.restore_input_mask(frozen_mask);
    run_oracle(&mut core, &mut video, &mut audio)?;
    let oracle_replay_frame_sha256 = sha256_bytes(&video.rgba8);
    let deterministic_replay_pass = oracle_final_frame_sha256 == oracle_replay_frame_sha256;

    if !(no_input_control_pass && oracle_control_pass && deterministic_replay_pass) {
        return Err(format!(
            "gym controls failed: no_input={no_input_control_pass} oracle={oracle_control_pass} deterministic={deterministic_replay_pass} start={start_player:?} final={oracle_final_player:?} distance={oracle_final_distance}"
        ));
    }

    let receipt = AgentGymQualificationReceipt {
        schema: "phicade.agent-gym-qualification.v1",
        result: "PASS",
        rom_sha256: sha256_file(&rom_path)?,
        source_sha256: sha256_file(&source_path)?,
        core_sha256: sha256_file(&core_path)?,
        core_name: core.identity().library_name.clone(),
        core_version: core.identity().library_version.clone(),
        start_frame,
        end_frame,
        start_player,
        target: AGENT_GYM_TARGET,
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
        oracle_final_frame_sha256,
        oracle_replay_frame_sha256,
    };

    let json = serde_json::to_string_pretty(&receipt)
        .map_err(|error| format!("serialize gym receipt: {error}"))?;
    println!("{json}");

    if let Some(parent) = receipt_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(&receipt_path, format!("{json}\n"))
        .map_err(|error| format!("write {}: {error}", receipt_path.display()))?;

    Ok(())
}
