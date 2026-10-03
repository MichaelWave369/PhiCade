use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use phicade_libretro::LibretroCore;
use phicade_runtime::{
    compile_agent_turn, ActionEnvelope, ActionKind, ActionSource, AgentGrant, AgentTurnAction,
    AgentTurnRequest, AgentTurnResponse, AudioBuffer, AuthorityPolicy, ControlMode, EmulatorCore,
    FrameBuffer, GameImage, PhiBotObservation, SystemId, AGENT_TURN_REQUEST_SCHEMA,
    AGENT_TURN_RESPONSE_SCHEMA, PHIBOT_OBSERVATION_SCHEMA,
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
struct DriverQualificationReceipt {
    schema: &'static str,
    result: &'static str,
    game_sha256: String,
    core_sha256: String,
    turn_id: u64,
    observation_frame: u64,
    observation_sha256: String,
    apply_frame: u64,
    end_frame: u64,
    driver_final_state_sha256: String,
    driver_final_frame_sha256: String,
    direct_final_state_sha256: String,
    direct_final_frame_sha256: String,
    driver_direct_parity_pass: bool,
    stale_response_rejected: bool,
    wrong_hash_rejected: bool,
    over_budget_rejected: bool,
    wrong_turn_rejected: bool,
    compiled_action_count: usize,
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
    eprintln!("usage: driver_qualify --core <sameboy_libretro> --rom <dmg-acid2.gb> --receipt <path>");
    process::exit(2);
}

fn run_scheduled(
    core: &mut LibretroCore,
    policy: &AuthorityPolicy,
    actions: &[ActionEnvelope],
    end_frame: u64,
) -> Result<(String, String), String> {
    let mut index = 0usize;
    let mut video = FrameBuffer::default();
    let mut audio = AudioBuffer::default();

    while core.frame_count() < end_frame {
        let current = core.frame_count();
        let mut due = Vec::new();

        while let Some(event) = actions.get(index) {
            if event.frame > current {
                break;
            }
            due.push(event.clone());
            index += 1;
        }

        let decisions = policy.authorize_batch(&due, current);
        let accepted: Vec<_> = decisions
            .into_iter()
            .map(|(event, decision)| {
                if decision.accepted {
                    Ok(event)
                } else {
                    Err(format!("qualified action rejected: {}", decision.reason))
                }
            })
            .collect::<Result<_, _>>()?;

        core.step_frame(&accepted, &mut video, &mut audio)
            .map_err(|error| format!("run governed frame: {error:?}"))?;
    }

    let state = core
        .serialize_state()
        .map_err(|error| format!("serialize final state: {error:?}"))?;

    Ok((sha256_bytes(&state), sha256_bytes(&video.rgba8)))
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Agent Driver qualification failed: {error}");
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

    let temp = env::temp_dir().join("phicade-driver-qualify");
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

    let start_frame = core.frame_count();
    let initial_state = core
        .serialize_state()
        .map_err(|error| format!("serialize initial state: {error:?}"))?;
    let initial_mask = core.input_mask_snapshot();

    let observation_sha256 = sha256_bytes(&video.rgba8);
    let observation = PhiBotObservation {
        schema: PHIBOT_OBSERVATION_SCHEMA.into(),
        frame: start_frame,
        width: video.width,
        height: video.height,
        rgba_base64: BASE64.encode(&video.rgba8),
        frame_sha256: observation_sha256.clone(),
        input_mask: initial_mask,
        game_sha256: sha256_file(&rom_path)?,
        core_name: core.identity().library_name.clone(),
        core_version: core.identity().library_version.clone(),
        agent_id: "phi-driver-qualifier".into(),
        seat: 1,
        control_mode: "phi-bot".into(),
        allowed_buttons: vec!["A".into(), "B".into(), "LEFT".into(), "RIGHT".into()],
        allowed_axes: Vec::new(),
        expires_at_frame: Some(start_frame + 3_600),
    };
    observation.validate()?;

    let request = AgentTurnRequest {
        schema: AGENT_TURN_REQUEST_SCHEMA.into(),
        turn_id: 42,
        observation: observation.clone(),
        max_actions: 4,
        max_delay_frames: 8,
        valid_until_frame: start_frame + 20,
        memory: String::new(),
        memory_sha256: sha256_bytes(b""),
        max_memory_bytes: 4096,
        max_memory_update_bytes: 1024,
    };
    request.validate()?;

    let selected = if observation_sha256.as_bytes()[0] % 2 == 0 {
        "A"
    } else {
        "B"
    };
    let response = AgentTurnResponse {
        schema: AGENT_TURN_RESPONSE_SCHEMA.into(),
        turn_id: request.turn_id,
        agent_id: observation.agent_id.clone(),
        seat: observation.seat,
        observation_frame: observation.frame,
        observation_sha256: observation.frame_sha256.clone(),
        memory_sha256: request.memory_sha256.clone(),
        memory_update: None,
        actions: vec![
            AgentTurnAction {
                delay_frames: 0,
                action: ActionKind::Button {
                    button: selected.into(),
                    pressed: true,
                },
            },
            AgentTurnAction {
                delay_frames: 2,
                action: ActionKind::Button {
                    button: selected.into(),
                    pressed: false,
                },
            },
            AgentTurnAction {
                delay_frames: 4,
                action: ActionKind::Button {
                    button: "RIGHT".into(),
                    pressed: true,
                },
            },
            AgentTurnAction {
                delay_frames: 7,
                action: ActionKind::Button {
                    button: "RIGHT".into(),
                    pressed: false,
                },
            },
        ],
    };

    let apply_frame = start_frame + 3;
    let compiled = compile_agent_turn(&request, &response, apply_frame)?;
    let end_frame = apply_frame + 60;

    let mut policy = AuthorityPolicy::new(1);
    policy
        .set_mode(
            ControlMode::PhiBot,
            Some(AgentGrant::game_boy("phi-driver-qualifier", 1)),
        )
        .map_err(|error| format!("configure agent grant: {error}"))?;

    let (driver_state, driver_frame) =
        run_scheduled(&mut core, &policy, &compiled, end_frame)?;

    core.restore_state(&initial_state, start_frame)
        .map_err(|error| format!("restore direct comparison state: {error:?}"))?;
    core.restore_input_mask(initial_mask);

    let direct: Vec<ActionEnvelope> = response
        .actions
        .iter()
        .map(|intent| ActionEnvelope {
            sequence: 0,
            frame: apply_frame + u64::from(intent.delay_frames),
            source: ActionSource::PhiBot {
                agent_id: response.agent_id.clone(),
                seat: response.seat,
            },
            action: intent.action.clone(),
        })
        .collect();

    let (direct_state, direct_frame) =
        run_scheduled(&mut core, &policy, &direct, end_frame)?;

    let driver_direct_parity_pass =
        driver_state == direct_state && driver_frame == direct_frame;
    if !driver_direct_parity_pass {
        return Err(format!(
            "driver/direct parity diverged: driver_state={driver_state} direct_state={direct_state} driver_frame={driver_frame} direct_frame={direct_frame}"
        ));
    }

    let stale_response_rejected = response
        .validate_against(&request, request.valid_until_frame + 1)
        .is_err();

    let mut wrong_hash = response.clone();
    wrong_hash.observation_sha256 = "0".repeat(64);
    let wrong_hash_rejected = wrong_hash.validate_against(&request, apply_frame).is_err();

    let mut over_budget = response.clone();
    over_budget.actions.push(AgentTurnAction {
        delay_frames: 1,
        action: ActionKind::Button {
            button: "LEFT".into(),
            pressed: true,
        },
    });
    let over_budget_rejected = over_budget.validate_against(&request, apply_frame).is_err();

    let mut wrong_turn = response.clone();
    wrong_turn.turn_id += 1;
    let wrong_turn_rejected = wrong_turn.validate_against(&request, apply_frame).is_err();

    if !(stale_response_rejected
        && wrong_hash_rejected
        && over_budget_rejected
        && wrong_turn_rejected)
    {
        return Err("one or more invalid driver responses escaped validation".into());
    }

    let receipt = DriverQualificationReceipt {
        schema: "phicade.agent-driver-qualification.v1",
        result: "PASS",
        game_sha256: sha256_file(&rom_path)?,
        core_sha256: sha256_file(&core_path)?,
        turn_id: request.turn_id,
        observation_frame: observation.frame,
        observation_sha256,
        apply_frame,
        end_frame,
        driver_final_state_sha256: driver_state,
        driver_final_frame_sha256: driver_frame,
        direct_final_state_sha256: direct_state,
        direct_final_frame_sha256: direct_frame,
        driver_direct_parity_pass,
        stale_response_rejected,
        wrong_hash_rejected,
        over_budget_rejected,
        wrong_turn_rejected,
        compiled_action_count: compiled.len(),
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
