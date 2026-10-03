use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use phicade_libretro::LibretroCore;
use phicade_runtime::{
    ActionEnvelope, ActionKind, ActionSource, AgentGrant, AudioBuffer, AuthorityPolicy, ControlMode,
    EmulatorCore, FrameBuffer, GameImage, PhiBotObservation, SystemCommand, SystemId,
    PHIBOT_OBSERVATION_SCHEMA,
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
struct PhiBotQualificationReceipt {
    schema: &'static str,
    result: &'static str,
    game_sha256: String,
    core_sha256: String,
    start_frame: u64,
    end_frame: u64,
    human_final_state_sha256: String,
    human_final_frame_sha256: String,
    agent_final_state_sha256: String,
    agent_final_frame_sha256: String,
    human_agent_parity_pass: bool,
    scoped_privilege_rejection_pass: bool,
    takeover_pass: bool,
    coop_pass: bool,
    versus_single_port_refusal_pass: bool,
    observation_contract_pass: bool,
    rejected_probe_actions: usize,
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
    eprintln!("usage: phibot_qualify --core <sameboy_libretro> --rom <dmg-acid2.gb> --receipt <path>");
    process::exit(2);
}

fn action(
    sequence: u64,
    frame: u64,
    source: ActionSource,
    action: ActionKind,
) -> ActionEnvelope {
    ActionEnvelope {
        sequence,
        frame,
        source,
        action,
    }
}

fn tape(start: u64, source: ActionSource) -> Vec<ActionEnvelope> {
    vec![
        action(
            0,
            start + 5,
            source.clone(),
            ActionKind::Button {
                button: "A".into(),
                pressed: true,
            },
        ),
        action(
            1,
            start + 12,
            source.clone(),
            ActionKind::Button {
                button: "A".into(),
                pressed: false,
            },
        ),
        action(
            2,
            start + 20,
            source.clone(),
            ActionKind::Button {
                button: "RIGHT".into(),
                pressed: true,
            },
        ),
        action(
            3,
            start + 42,
            source,
            ActionKind::Button {
                button: "RIGHT".into(),
                pressed: false,
            },
        ),
    ]
}

fn run_tape(
    core: &mut LibretroCore,
    policy: &AuthorityPolicy,
    actions: &[ActionEnvelope],
    end_frame: u64,
) -> Result<(String, String, usize), String> {
    let mut video = FrameBuffer::default();
    let mut audio = AudioBuffer::default();
    let mut index = 0usize;
    let mut rejected = 0usize;

    while core.frame_count() < end_frame {
        let current = core.frame_count();
        let mut due = Vec::new();
        while let Some(event) = actions.get(index) {
            if event.frame != current {
                break;
            }
            due.push(event.clone());
            index += 1;
        }

        let mut accepted = Vec::new();
        for (event, decision) in policy.authorize_batch(&due, current) {
            if decision.accepted {
                accepted.push(event);
            } else {
                rejected += 1;
            }
        }

        core.step_frame(&accepted, &mut video, &mut audio)
            .map_err(|error| format!("run governed frame: {error:?}"))?;
    }

    let state = core
        .serialize_state()
        .map_err(|error| format!("serialize final state: {error:?}"))?;

    Ok((
        sha256_bytes(&state),
        sha256_bytes(&video.rgba8),
        rejected,
    ))
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Phi-Bot qualification failed: {error}");
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

    let temp = env::temp_dir().join("phicade-phibot-qualify");
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
    let end_frame = start_frame + 120;

    let human_policy = AuthorityPolicy::new(1);
    let human_actions = tape(start_frame, ActionSource::Human { seat: 1 });
    let (human_state, human_frame, human_rejected) =
        run_tape(&mut core, &human_policy, &human_actions, end_frame)?;
    if human_rejected != 0 {
        return Err(format!("human parity tape unexpectedly rejected {human_rejected} actions"));
    }

    core.restore_state(&initial_state, start_frame)
        .map_err(|error| format!("restore for agent parity: {error:?}"))?;
    core.restore_input_mask(initial_mask);

    let mut agent_policy = AuthorityPolicy::new(1);
    agent_policy
        .set_mode(
            ControlMode::PhiBot,
            Some(AgentGrant::game_boy("phi-qualifier", 1)),
        )
        .map_err(|error| format!("configure Phi-Bot handoff: {error}"))?;
    let agent_actions = tape(
        start_frame,
        ActionSource::PhiBot {
            agent_id: "phi-qualifier".into(),
            seat: 1,
        },
    );
    let (agent_state, agent_frame, agent_rejected) =
        run_tape(&mut core, &agent_policy, &agent_actions, end_frame)?;
    if agent_rejected != 0 {
        return Err(format!("agent parity tape unexpectedly rejected {agent_rejected} actions"));
    }

    let parity_pass = human_state == agent_state && human_frame == agent_frame;
    if !parity_pass {
        return Err(format!(
            "human/agent parity diverged: human_state={human_state} agent_state={agent_state} human_frame={human_frame} agent_frame={agent_frame}"
        ));
    }

    let probe_frame = core.frame_count();
    let privileged_probe = vec![
        action(
            100,
            probe_frame,
            ActionSource::PhiBot {
                agent_id: "phi-qualifier".into(),
                seat: 1,
            },
            ActionKind::System {
                command: SystemCommand::Reset,
                slot: None,
            },
        ),
        action(
            101,
            probe_frame,
            ActionSource::PhiBot {
                agent_id: "phi-qualifier".into(),
                seat: 1,
            },
            ActionKind::Button {
                button: "A".into(),
                pressed: true,
            },
        ),
    ];
    let probe = agent_policy.authorize_batch(&privileged_probe, probe_frame);
    let rejected_probe_actions = probe.iter().filter(|(_, decision)| !decision.accepted).count();
    let scoped_privilege_rejection_pass =
        !probe[0].1.accepted && probe[1].1.accepted && rejected_probe_actions == 1;
    if !scoped_privilege_rejection_pass {
        return Err("Phi-Bot grant scope failed to reject privileged RESET".into());
    }

    let mut takeover_policy = agent_policy.clone();
    takeover_policy
        .set_mode(ControlMode::Human, None)
        .map_err(|error| format!("human takeover: {error}"))?;
    let takeover_actions = vec![
        action(
            200,
            probe_frame,
            ActionSource::PhiBot {
                agent_id: "phi-qualifier".into(),
                seat: 1,
            },
            ActionKind::Button {
                button: "A".into(),
                pressed: true,
            },
        ),
        action(
            201,
            probe_frame,
            ActionSource::Human { seat: 1 },
            ActionKind::Button {
                button: "B".into(),
                pressed: true,
            },
        ),
    ];
    let takeover = takeover_policy.authorize_batch(&takeover_actions, probe_frame);
    let takeover_pass = !takeover[0].1.accepted && takeover[1].1.accepted;
    if !takeover_pass {
        return Err("human takeover failed to revoke Phi-Bot gameplay authority".into());
    }

    let mut coop_policy = AuthorityPolicy::new(1);
    coop_policy
        .set_mode(
            ControlMode::Coop,
            Some(AgentGrant::game_boy("phi-qualifier", 1)),
        )
        .map_err(|error| format!("configure co-op: {error}"))?;
    let coop_actions = vec![
        action(
            300,
            probe_frame,
            ActionSource::Human { seat: 1 },
            ActionKind::Button {
                button: "LEFT".into(),
                pressed: true,
            },
        ),
        action(
            301,
            probe_frame,
            ActionSource::PhiBot {
                agent_id: "phi-qualifier".into(),
                seat: 1,
            },
            ActionKind::Button {
                button: "A".into(),
                pressed: true,
            },
        ),
    ];
    let coop = coop_policy.authorize_batch(&coop_actions, probe_frame);
    let coop_pass = coop.iter().all(|(_, decision)| decision.accepted);
    if !coop_pass {
        return Err("co-op policy did not admit both human and Phi-Bot sources".into());
    }

    let mut versus_policy = AuthorityPolicy::new(1);
    let versus_single_port_refusal_pass = versus_policy
        .set_mode(
            ControlMode::Versus,
            Some(AgentGrant::game_boy("phi-qualifier", 2)),
        )
        .is_err();
    if !versus_single_port_refusal_pass {
        return Err("single-port SameBoy topology unexpectedly accepted versus mode".into());
    }

    let observation = PhiBotObservation {
        schema: PHIBOT_OBSERVATION_SCHEMA.into(),
        frame: core.frame_count(),
        width: video.width,
        height: video.height,
        rgba_base64: BASE64.encode(&video.rgba8),
        frame_sha256: sha256_bytes(&video.rgba8),
        input_mask: core.input_mask_snapshot(),
        game_sha256: sha256_file(&rom_path)?,
        core_name: core.identity().library_name.clone(),
        core_version: core.identity().library_version.clone(),
        agent_id: "phi-qualifier".into(),
        seat: 1,
        control_mode: "phi-bot".into(),
        allowed_buttons: AgentGrant::game_boy("phi-qualifier", 1)
            .allowed_buttons
            .into_iter()
            .collect(),
        allowed_axes: Vec::new(),
        expires_at_frame: Some(core.frame_count() + 3_600),
    };
    let observation_contract_pass = observation.validate().is_ok();
    if !observation_contract_pass {
        return Err("Phi-Bot observation contract did not validate".into());
    }

    let receipt = PhiBotQualificationReceipt {
        schema: "phicade.phibot-qualification.v1",
        result: "PASS",
        game_sha256: sha256_file(&rom_path)?,
        core_sha256: sha256_file(&core_path)?,
        start_frame,
        end_frame,
        human_final_state_sha256: human_state,
        human_final_frame_sha256: human_frame,
        agent_final_state_sha256: agent_state,
        agent_final_frame_sha256: agent_frame,
        human_agent_parity_pass: parity_pass,
        scoped_privilege_rejection_pass,
        takeover_pass,
        coop_pass,
        versus_single_port_refusal_pass,
        observation_contract_pass,
        rejected_probe_actions,
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
