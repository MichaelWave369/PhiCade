use phicade_libretro::LibretroCore;
use phicade_runtime::{
    benchmark_task_by_id, benchmark_task_success, locate_agent_gym_player, ActionEnvelope,
    ActionKind, ActionSource, AudioBuffer, BenchmarkTaskSpec, EmulatorCore, FrameBuffer,
    GameImage, PixelPoint, SystemId, AGENT_GYM_RELAY_LEFT_ID, AGENT_GYM_RELAY_RIGHT_ID,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

const WARMUP_FRAMES: u64 = 120;
const DOWN_FRAMES: u64 = 40;
const RIGHT_FRAMES: u64 = 60;
const UP_FRAMES: u64 = 40;
const CHOICE_FRAMES: u64 = 24;
const SETTLE_FRAMES: u64 = 3;

const CORRIDOR_START: PixelPoint = PixelPoint { x: 16, y: 24 };
const TERMINAL_START: PixelPoint = PixelPoint { x: 72, y: 112 };

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct VariantEvidence {
    task_id: String,
    rom_sha256: String,
    briefing_frame_sha256: String,
    corridor_frame_sha256: String,
    terminal_frame_sha256: String,
    briefing_player: PixelPoint,
    corridor_player: PixelPoint,
    shortcut_player: PixelPoint,
    terminal_player: PixelPoint,
    wrong_final_player: PixelPoint,
    correct_final_player: PixelPoint,
    shortcut_blocked: bool,
    wrong_choice_refused: bool,
    correct_choice_pass: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RelayPairReceipt {
    schema: &'static str,
    result: &'static str,
    core_sha256: String,
    core_name: String,
    core_version: String,
    left: VariantEvidence,
    right: VariantEvidence,
    briefing_frames_differ: bool,
    corridor_frames_identical: bool,
    terminal_frames_identical: bool,
    corridor_geometry_identical: bool,
    terminal_geometry_identical: bool,
    shortcut_blocked_both: bool,
    wrong_choice_refused_both: bool,
    correct_choice_pass_both: bool,
}

fn usage() -> ! {
    eprintln!(
        "usage: relay_rooms_qualify --core <sameboy_libretro> --left-rom <left.gb> --right-rom <right.gb> --receipt <path>"
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
            name: "relay-rooms-qualifier".into(),
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

fn enter_corridor(
    core: &mut LibretroCore,
    sequence: &mut u64,
    video: &mut FrameBuffer,
    audio: &mut AudioBuffer,
) -> Result<(), String> {
    hold_button(
        core,
        sequence,
        "A",
        1,
        video,
        audio,
        "accept briefing and enter corridor",
    )?;
    no_input(core, 1, video, audio, "settle corridor entry")
}

fn run_corridor_route(
    core: &mut LibretroCore,
    sequence: &mut u64,
    video: &mut FrameBuffer,
    audio: &mut AudioBuffer,
) -> Result<(), String> {
    hold_button(
        core,
        sequence,
        "DOWN",
        DOWN_FRAMES,
        video,
        audio,
        "corridor route down",
    )?;
    hold_button(
        core,
        sequence,
        "RIGHT",
        RIGHT_FRAMES,
        video,
        audio,
        "corridor route right",
    )?;
    hold_button(
        core,
        sequence,
        "UP",
        UP_FRAMES,
        video,
        audio,
        "corridor route up",
    )?;
    no_input(core, 1, video, audio, "settle terminal entry")
}

fn choose_terminal(
    core: &mut LibretroCore,
    sequence: &mut u64,
    direction: &str,
    video: &mut FrameBuffer,
    audio: &mut AudioBuffer,
) -> Result<PixelPoint, String> {
    hold_button(
        core,
        sequence,
        direction,
        CHOICE_FRAMES,
        video,
        audio,
        "terminal choice",
    )?;
    no_input(core, SETTLE_FRAMES, video, audio, "settle terminal choice")?;
    locate_agent_gym_player(video)
}

fn qualify_variant(
    core_path: &Path,
    rom_path: &Path,
    task: &'static BenchmarkTaskSpec,
    correct: &str,
    wrong: &str,
) -> Result<(VariantEvidence, String, String), String> {
    let temp = env::temp_dir().join(format!("phicade-relay-pair-{}", task.id));
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
        "warm relay briefing",
    )?;
    let briefing_player = locate_agent_gym_player(&video)?;
    if briefing_player != task.start {
        return Err(format!(
            "{} briefing start drifted: expected {:?}, got {:?}",
            task.id, task.start, briefing_player
        ));
    }
    let briefing_frame_sha256 = sha256_bytes(&video.rgba8);

    enter_corridor(&mut core, &mut sequence, &mut video, &mut audio)?;
    let corridor_player = locate_agent_gym_player(&video)?;
    if corridor_player != CORRIDOR_START {
        return Err(format!(
            "{} corridor start drifted: expected {:?}, got {:?}",
            task.id, CORRIDOR_START, corridor_player
        ));
    }
    let corridor_frame_sha256 = sha256_bytes(&video.rgba8);
    let corridor_state = core
        .serialize_state()
        .map_err(|error| format!("serialize {} corridor state: {error:?}", task.id))?;
    let corridor_mask = core.input_mask_snapshot();
    let corridor_frame = core.frame_count();

    hold_button(
        &mut core,
        &mut sequence,
        "RIGHT",
        RIGHT_FRAMES,
        &mut video,
        &mut audio,
        "direct-right shortcut probe",
    )?;
    no_input(&mut core, 1, &mut video, &mut audio, "settle shortcut probe")?;
    let shortcut_player = locate_agent_gym_player(&video)?;
    let shortcut_blocked = shortcut_player.y < 96
        && shortcut_player.x <= 64
        && !benchmark_task_success(task, shortcut_player);

    core.restore_state(&corridor_state, corridor_frame)
        .map_err(|error| format!("restore {} corridor state: {error:?}", task.id))?;
    core.restore_input_mask(corridor_mask);
    no_input(
        &mut core,
        1,
        &mut video,
        &mut audio,
        "render restored corridor state",
    )?;

    run_corridor_route(&mut core, &mut sequence, &mut video, &mut audio)?;
    let terminal_player = locate_agent_gym_player(&video)?;
    if terminal_player != TERMINAL_START {
        return Err(format!(
            "{} terminal start drifted: expected {:?}, got {:?}",
            task.id, TERMINAL_START, terminal_player
        ));
    }
    let terminal_frame_sha256 = sha256_bytes(&video.rgba8);
    let terminal_state = core
        .serialize_state()
        .map_err(|error| format!("serialize {} terminal state: {error:?}", task.id))?;
    let terminal_mask = core.input_mask_snapshot();
    let terminal_frame = core.frame_count();

    let wrong_final_player = choose_terminal(
        &mut core,
        &mut sequence,
        wrong,
        &mut video,
        &mut audio,
    )?;
    let wrong_choice_refused = !benchmark_task_success(task, wrong_final_player);

    core.restore_state(&terminal_state, terminal_frame)
        .map_err(|error| format!("restore {} terminal state: {error:?}", task.id))?;
    core.restore_input_mask(terminal_mask);
    no_input(
        &mut core,
        1,
        &mut video,
        &mut audio,
        "render restored terminal state",
    )?;

    let correct_final_player = choose_terminal(
        &mut core,
        &mut sequence,
        correct,
        &mut video,
        &mut audio,
    )?;
    let correct_choice_pass = benchmark_task_success(task, correct_final_player);

    Ok((
        VariantEvidence {
            task_id: task.id.into(),
            rom_sha256: sha256_file(rom_path)?,
            briefing_frame_sha256,
            corridor_frame_sha256,
            terminal_frame_sha256,
            briefing_player,
            corridor_player,
            shortcut_player,
            terminal_player,
            wrong_final_player,
            correct_final_player,
            shortcut_blocked,
            wrong_choice_refused,
            correct_choice_pass,
        },
        core_name,
        core_version,
    ))
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Relay Rooms pair qualification failed: {error}");
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

    let left_task = benchmark_task_by_id(AGENT_GYM_RELAY_LEFT_ID)
        .ok_or_else(|| "relay LEFT task missing from registry".to_owned())?;
    let right_task = benchmark_task_by_id(AGENT_GYM_RELAY_RIGHT_ID)
        .ok_or_else(|| "relay RIGHT task missing from registry".to_owned())?;

    let (left, core_name, core_version) =
        qualify_variant(&core_path, &left_rom, left_task, "LEFT", "RIGHT")?;
    let (right, right_core_name, right_core_version) =
        qualify_variant(&core_path, &right_rom, right_task, "RIGHT", "LEFT")?;

    if core_name != right_core_name || core_version != right_core_version {
        return Err("relay variants did not run under identical core identity".into());
    }

    let briefing_frames_differ = left.briefing_frame_sha256 != right.briefing_frame_sha256;
    let corridor_frames_identical = left.corridor_frame_sha256 == right.corridor_frame_sha256;
    let terminal_frames_identical = left.terminal_frame_sha256 == right.terminal_frame_sha256;
    let corridor_geometry_identical =
        left.corridor_player == right.corridor_player && left.corridor_player == CORRIDOR_START;
    let terminal_geometry_identical =
        left.terminal_player == right.terminal_player && left.terminal_player == TERMINAL_START;
    let shortcut_blocked_both = left.shortcut_blocked && right.shortcut_blocked;
    let wrong_choice_refused_both = left.wrong_choice_refused && right.wrong_choice_refused;
    let correct_choice_pass_both = left.correct_choice_pass && right.correct_choice_pass;

    if !(briefing_frames_differ
        && corridor_frames_identical
        && terminal_frames_identical
        && corridor_geometry_identical
        && terminal_geometry_identical
        && shortcut_blocked_both
        && wrong_choice_refused_both
        && correct_choice_pass_both)
    {
        return Err(format!(
            "relay pair controls failed: briefing_diff={briefing_frames_differ} corridor_same={corridor_frames_identical} terminal_same={terminal_frames_identical} corridor_geometry={corridor_geometry_identical} terminal_geometry={terminal_geometry_identical} shortcut={shortcut_blocked_both} wrong_refused={wrong_choice_refused_both} correct_pass={correct_choice_pass_both} left_shortcut={:?} right_shortcut={:?} left_wrong={:?} right_wrong={:?} left_correct={:?} right_correct={:?}",
            left.shortcut_player,
            right.shortcut_player,
            left.wrong_final_player,
            right.wrong_final_player,
            left.correct_final_player,
            right.correct_final_player
        ));
    }

    let receipt = RelayPairReceipt {
        schema: "phicade.relay-rooms-pair-qualification.v1",
        result: "PASS",
        core_sha256: sha256_file(&core_path)?,
        core_name,
        core_version,
        left,
        right,
        briefing_frames_differ,
        corridor_frames_identical,
        terminal_frames_identical,
        corridor_geometry_identical,
        terminal_geometry_identical,
        shortcut_blocked_both,
        wrong_choice_refused_both,
        correct_choice_pass_both,
    };

    let json = serde_json::to_string_pretty(&receipt)
        .map_err(|error| format!("serialize relay receipt: {error}"))?;
    println!("{json}");
    if let Some(parent) = receipt_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(&receipt_path, format!("{json}\n"))
        .map_err(|error| format!("write {}: {error}", receipt_path.display()))?;
    Ok(())
}
