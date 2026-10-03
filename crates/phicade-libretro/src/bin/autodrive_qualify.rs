use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use phicade_libretro::LibretroCore;
use phicade_runtime::{
    compile_agent_turn, ActionEnvelope, ActionKind, AgentGrant, AgentTurnAction,
    AgentTurnRequest, AgentTurnResponse, AutodrivePolicy, AutodriveStatus, AutodriveStopReason,
    AudioBuffer, AuthorityPolicy, ControlMode, EmulatorCore, FrameBuffer, GameImage,
    PhiBotObservation, SystemId, AGENT_TURN_REQUEST_SCHEMA, AGENT_TURN_RESPONSE_SCHEMA,
    AUTODRIVE_STATUS_SCHEMA, PHIBOT_OBSERVATION_SCHEMA,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AutodriveQualificationReceipt {
    schema: &'static str,
    result: &'static str,
    game_sha256: String,
    core_sha256: String,
    started_frame: u64,
    ended_frame: u64,
    turns_issued: u16,
    turns_completed: u16,
    total_actions: u32,
    final_state_sha256: String,
    final_frame_sha256: String,
    turn_budget_stop_pass: bool,
    action_budget_probe_pass: bool,
    frame_budget_probe_pass: bool,
    empty_turn_probe_pass: bool,
    action_settle_probe_pass: bool,
    empty_backoff_probe_pass: bool,
    cadence_gate_probe_pass: bool,
}

fn sha256_bytes(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn sha256_file(path: &Path) -> Result<String, String> {
    fs::read(path)
        .map(|bytes| sha256_bytes(&bytes))
        .map_err(|error| format!("cannot read {}: {error}", path.display()))
}

fn usage() -> ! {
    eprintln!("usage: autodrive_qualify --core <sameboy_libretro> --rom <dmg-acid2.gb> --receipt <path>");
    process::exit(2);
}

fn run_actions(
    core: &mut LibretroCore,
    policy: &AuthorityPolicy,
    actions: &[ActionEnvelope],
    frames: u64,
    video: &mut FrameBuffer,
    audio: &mut AudioBuffer,
) -> Result<(), String> {
    let target = core.frame_count().saturating_add(frames);
    let mut index = 0usize;

    while core.frame_count() < target {
        let current = core.frame_count();
        let mut due = Vec::new();
        while let Some(event) = actions.get(index) {
            if event.frame > current {
                break;
            }
            due.push(event.clone());
            index += 1;
        }

        let accepted = policy
            .authorize_batch(&due, current)
            .into_iter()
            .map(|(event, decision)| {
                if decision.accepted {
                    Ok(event)
                } else {
                    Err(format!("autodrive qualified action rejected: {}", decision.reason))
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        core.step_frame(&accepted, video, audio)
            .map_err(|error| format!("autodrive frame: {error:?}"))?;
    }

    Ok(())
}

fn observation(
    core: &LibretroCore,
    video: &FrameBuffer,
    game_sha256: &str,
) -> PhiBotObservation {
    PhiBotObservation {
        schema: PHIBOT_OBSERVATION_SCHEMA.into(),
        frame: core.frame_count(),
        width: video.width,
        height: video.height,
        rgba_base64: BASE64.encode(&video.rgba8),
        frame_sha256: sha256_bytes(&video.rgba8),
        input_mask: core.input_mask_snapshot(),
        game_sha256: game_sha256.into(),
        core_name: core.identity().library_name.clone(),
        core_version: core.identity().library_version.clone(),
        agent_id: "phi-autodrive-qualifier".into(),
        seat: 1,
        control_mode: "phi-bot".into(),
        allowed_buttons: vec!["A".into(), "B".into(), "LEFT".into(), "RIGHT".into()],
        allowed_axes: Vec::new(),
        expires_at_frame: Some(core.frame_count() + 3_600),
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Autodrive qualification failed: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let mut core_path: Option<PathBuf> = None;
    let mut rom_path: Option<PathBuf> = None;
    let mut receipt_path: Option<PathBuf> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--core" => core_path = args.next().map(PathBuf::from),
            "--rom" => rom_path = args.next().map(PathBuf::from),
            "--receipt" => receipt_path = args.next().map(PathBuf::from),
            _ => usage(),
        }
    }

    let core_path = core_path.unwrap_or_else(|| usage());
    let rom_path = rom_path.unwrap_or_else(|| usage());
    let receipt_path = receipt_path.unwrap_or_else(|| usage());

    let temp = env::temp_dir().join("phicade-autodrive-qualify");
    let system_dir = temp.join("system");
    let save_dir = temp.join("save");
    fs::create_dir_all(&system_dir).map_err(|error| error.to_string())?;
    fs::create_dir_all(&save_dir).map_err(|error| error.to_string())?;

    let mut core = LibretroCore::open(&core_path, &system_dir, &save_dir)
        .map_err(|error| format!("open core: {error:?}"))?;
    core.load_game(&GameImage::new(&rom_path, SystemId::GameBoy, "dmg-acid2"))
        .map_err(|error| format!("load fixture: {error:?}"))?;

    let mut video = FrameBuffer::default();
    let mut audio = AudioBuffer::default();
    for _ in 0..120 {
        core.step_frame(&[], &mut video, &mut audio)
            .map_err(|error| format!("warmup frame: {error:?}"))?;
    }

    let game_sha256 = sha256_file(&rom_path)?;
    let started_frame = core.frame_count();
    let policy_config = AutodrivePolicy {
        policy_version: 1,
        max_turns: 3,
        max_total_actions: 6,
        max_consecutive_empty_turns: 2,
        max_emulated_frames: 300,
        min_observation_interval_frames: 2,
        post_action_settle_frames: 2,
        empty_turn_backoff_frames: 8,
        max_observation_interval_frames: 60,
        max_memory_bytes: 4096,
        max_memory_update_bytes: 1024,
    };
    policy_config.validate()?;

    let mut run_status = AutodriveStatus {
        schema: AUTODRIVE_STATUS_SCHEMA.into(),
        run_id: 1,
        active: true,
        provider: "qualified-driver".into(),
        model: "deterministic".into(),
        started_frame,
        current_frame: started_frame,
        turns_issued: 0,
        turns_completed: 0,
        total_actions: 0,
        consecutive_empty_turns: 0,
        next_observation_frame: started_frame,
        last_observation_frame: None,
        total_scheduled_cadence_wait_frames: 0,
        max_scheduled_cadence_wait_frames: 0,
        policy: policy_config,
        stop_reason: None,
    };
    run_status.validate()?;

    let mut authority = AuthorityPolicy::new(1);
    authority
        .set_mode(
            ControlMode::PhiBot,
            Some(AgentGrant::game_boy("phi-autodrive-qualifier", 1)),
        )
        .map_err(|error| format!("configure authority: {error}"))?;

    for turn in 0..3u64 {
        run_status.current_frame = core.frame_count();
        if let Some(reason) = run_status.pre_turn_stop_reason() {
            return Err(format!("run stopped too early before turn {}: {reason:?}", turn + 1));
        }

        if !run_status.observation_ready(core.frame_count()) {
            return Err(format!(
                "cadence refused qualified turn {} at frame {} before {}",
                turn + 1,
                core.frame_count(),
                run_status.next_observation_frame
            ));
        }

        let obs = observation(&core, &video, &game_sha256);
        obs.validate()?;
        let request = AgentTurnRequest {
            schema: AGENT_TURN_REQUEST_SCHEMA.into(),
            turn_id: turn + 1,
            observation: obs.clone(),
            max_actions: 2,
            max_delay_frames: 4,
            valid_until_frame: obs.frame + 20,
            memory: String::new(),
            memory_sha256: sha256_bytes(b""),
            max_memory_bytes: run_status.policy.max_memory_bytes,
            max_memory_update_bytes: run_status.policy.max_memory_update_bytes,
        };
        request.validate()?;
        run_status.note_turn_issued_at(core.frame_count());

        let button = if turn % 2 == 0 { "A" } else { "RIGHT" };
        let response = AgentTurnResponse {
            schema: AGENT_TURN_RESPONSE_SCHEMA.into(),
            turn_id: request.turn_id,
            agent_id: obs.agent_id.clone(),
            seat: 1,
            observation_frame: obs.frame,
            observation_sha256: obs.frame_sha256.clone(),
            memory_sha256: request.memory_sha256.clone(),
            memory_update: None,
            actions: vec![
                AgentTurnAction {
                    delay_frames: 0,
                    action: ActionKind::Button {
                        button: button.into(),
                        pressed: true,
                    },
                },
                AgentTurnAction {
                    delay_frames: 2,
                    action: ActionKind::Button {
                        button: button.into(),
                        pressed: false,
                    },
                },
            ],
        };

        let apply_frame = core.frame_count();
        let compiled = compile_agent_turn(&request, &response, apply_frame)?;
        if !run_status.can_accept_actions(compiled.len()) {
            return Err("qualified turn unexpectedly exceeded action budget".into());
        }
        run_status.note_turn_completed_with_delay(compiled.len(), 2, apply_frame);
        run_actions(
            &mut core,
            &authority,
            &compiled,
            20,
            &mut video,
            &mut audio,
        )?;
        run_status.current_frame = core.frame_count();
    }

    let turn_budget_stop_pass =
        run_status.pre_turn_stop_reason() == Some(AutodriveStopReason::TurnBudget);
    if !turn_budget_stop_pass {
        return Err(format!(
            "expected turn-budget stop after three turns, got {:?}",
            run_status.pre_turn_stop_reason()
        ));
    }

    let mut action_probe = run_status.clone();
    action_probe.turns_issued = 1;
    action_probe.turns_completed = 1;
    action_probe.total_actions = action_probe.policy.max_total_actions;
    action_probe.current_frame = action_probe.started_frame;
    let action_budget_probe_pass =
        action_probe.pre_turn_stop_reason() == Some(AutodriveStopReason::ActionBudget);

    let mut frame_probe = run_status.clone();
    frame_probe.turns_issued = 0;
    frame_probe.turns_completed = 0;
    frame_probe.total_actions = 0;
    frame_probe.current_frame =
        frame_probe.started_frame + frame_probe.policy.max_emulated_frames;
    let frame_budget_probe_pass =
        frame_probe.pre_turn_stop_reason() == Some(AutodriveStopReason::FrameBudget);

    let mut empty_probe = run_status.clone();
    empty_probe.turns_issued = 0;
    empty_probe.turns_completed = 0;
    empty_probe.total_actions = 0;
    empty_probe.consecutive_empty_turns = 0;
    empty_probe.note_turn_completed(0);
    empty_probe.note_turn_completed(0);
    let empty_turn_probe_pass =
        empty_probe.post_turn_stop_reason() == Some(AutodriveStopReason::EmptyTurnLimit);

    let action_settle_probe_pass =
        run_status.policy.cadence_wait_frames(2, 2, 0) == 4;
    let empty_backoff_probe_pass =
        run_status.policy.cadence_wait_frames(0, 0, 1) == 8
            && run_status.policy.cadence_wait_frames(0, 0, 2) == 16
            && run_status.policy.cadence_wait_frames(0, 0, 3) == 32
            && run_status.policy.cadence_wait_frames(0, 0, 4) == 60;

    let mut cadence_probe = run_status.clone();
    cadence_probe.turns_issued = 0;
    cadence_probe.turns_completed = 0;
    cadence_probe.total_actions = 0;
    cadence_probe.consecutive_empty_turns = 0;
    cadence_probe.current_frame = cadence_probe.started_frame;
    cadence_probe.next_observation_frame = cadence_probe.started_frame;
    cadence_probe.last_observation_frame = None;
    cadence_probe.total_scheduled_cadence_wait_frames = 0;
    cadence_probe.max_scheduled_cadence_wait_frames = 0;
    cadence_probe.note_turn_completed_with_delay(0, 0, cadence_probe.started_frame);
    let cadence_gate_probe_pass =
        !cadence_probe.observation_ready(cadence_probe.started_frame + 7)
            && cadence_probe.observation_ready(cadence_probe.started_frame + 8);

    if !(action_budget_probe_pass
        && frame_budget_probe_pass
        && empty_turn_probe_pass
        && action_settle_probe_pass
        && empty_backoff_probe_pass
        && cadence_gate_probe_pass)
    {
        return Err("one or more autonomous stop/cadence probes failed".into());
    }

    let final_state = core
        .serialize_state()
        .map_err(|error| format!("serialize final state: {error:?}"))?;

    let receipt = AutodriveQualificationReceipt {
        schema: "phicade.autodrive-qualification.v2",
        result: "PASS",
        game_sha256,
        core_sha256: sha256_file(&core_path)?,
        started_frame,
        ended_frame: core.frame_count(),
        turns_issued: run_status.turns_issued,
        turns_completed: run_status.turns_completed,
        total_actions: run_status.total_actions,
        final_state_sha256: sha256_bytes(&final_state),
        final_frame_sha256: sha256_bytes(&video.rgba8),
        turn_budget_stop_pass,
        action_budget_probe_pass,
        frame_budget_probe_pass,
        empty_turn_probe_pass,
        action_settle_probe_pass,
        empty_backoff_probe_pass,
        cadence_gate_probe_pass,
    };

    let json = serde_json::to_string_pretty(&receipt)
        .map_err(|error| format!("serialize receipt: {error}"))?;
    println!("{json}");
    if let Some(parent) = receipt_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(&receipt_path, format!("{json}\n"))
        .map_err(|error| format!("write {}: {error}", receipt_path.display()))?;

    Ok(())
}
