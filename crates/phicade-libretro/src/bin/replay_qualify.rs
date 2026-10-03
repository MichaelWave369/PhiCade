use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use phicade_libretro::LibretroCore;
use phicade_runtime::{
    ActionEnvelope, ActionKind, ActionSource, AudioBuffer, EmulatorCore, FrameBuffer, GameImage,
    ReplayCheckpoint, ReplayLedger, SystemCommand, SystemId, REPLAY_SCHEMA,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ReplayQualificationReceipt {
    schema: &'static str,
    result: &'static str,
    game_sha256: String,
    core_sha256: String,
    replay_sha256: String,
    start_frame: u64,
    end_frame: u64,
    action_count: usize,
    checkpoint_count: usize,
    exact_replay_pass: bool,
    divergence_probe_detected: bool,
    first_divergence_frame: Option<u64>,
}

fn sha256_bytes(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn sha256_file(path: &Path) -> Result<String, String> {
    fs::read(path)
        .map(|bytes| sha256_bytes(&bytes))
        .map_err(|error| format!("cannot read {}: {error}", path.display()))
}

fn checkpoint(core: &LibretroCore, frame: &FrameBuffer) -> Result<ReplayCheckpoint, String> {
    let state = core
        .serialize_state()
        .map_err(|error| format!("serialize replay checkpoint: {error:?}"))?;
    Ok(ReplayCheckpoint {
        frame: core.frame_count(),
        state_sha256: sha256_bytes(&state),
        frame_sha256: sha256_bytes(&frame.rgba8),
        input_mask: core.input_mask_snapshot(),
    })
}

fn verify(core: &mut LibretroCore, ledger: &ReplayLedger) -> Result<Option<u64>, String> {
    ledger.validate()?;
    let initial = BASE64
        .decode(&ledger.initial_state_base64)
        .map_err(|error| format!("decode initial state: {error}"))?;
    core.restore_state(&initial, ledger.start_frame)
        .map_err(|error| format!("restore initial state: {error:?}"))?;
    core.restore_input_mask(ledger.initial_input_mask);

    let first = ledger
        .checkpoints
        .first()
        .ok_or_else(|| "replay has no checkpoints".to_owned())?;
    let state = core
        .serialize_state()
        .map_err(|error| format!("serialize initial verification state: {error:?}"))?;
    if first.frame != ledger.start_frame
        || first.state_sha256 != sha256_bytes(&state)
        || first.input_mask != core.input_mask_snapshot()
    {
        return Ok(Some(ledger.start_frame));
    }

    let mut checkpoint_index = 1usize;
    let mut action_index = 0usize;
    let mut video = FrameBuffer::default();
    let mut audio = AudioBuffer::default();

    while core.frame_count() < ledger.end_frame {
        let current = core.frame_count();
        let mut actions = Vec::new();

        while let Some(action) = ledger.actions.get(action_index) {
            if action.frame != current {
                break;
            }
            let mut replay_action = action.clone();
            replay_action.source = ActionSource::Replay;
            actions.push(replay_action);
            action_index += 1;
        }

        core.step_frame(&actions, &mut video, &mut audio)
            .map_err(|error| format!("verify replay frame: {error:?}"))?;

        if let Some(expected) = ledger.checkpoints.get(checkpoint_index) {
            if expected.frame == core.frame_count() {
                let actual = checkpoint(core, &video)?;
                if expected.state_sha256 != actual.state_sha256
                    || expected.frame_sha256 != actual.frame_sha256
                    || expected.input_mask != actual.input_mask
                {
                    return Ok(Some(expected.frame));
                }
                checkpoint_index += 1;
            }
        }
    }

    let final_state = core
        .serialize_state()
        .map_err(|error| format!("serialize final verification state: {error:?}"))?;
    if sha256_bytes(&final_state) != ledger.final_state_sha256
        || sha256_bytes(&video.rgba8) != ledger.final_frame_sha256
    {
        return Ok(Some(ledger.end_frame));
    }

    Ok(None)
}

fn action(sequence: u64, frame: u64, action: ActionKind) -> ActionEnvelope {
    ActionEnvelope {
        sequence,
        frame,
        source: ActionSource::Human { seat: 1 },
        action,
    }
}

fn usage() -> ! {
    eprintln!("usage: replay_qualify --core <sameboy_libretro> --rom <dmg-acid2.gb> --receipt <path>");
    process::exit(2);
}

fn main() {
    if let Err(error) = run() {
        eprintln!("replay qualification failed: {error}");
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

    let temp = env::temp_dir().join("phicade-replay-qualify");
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
        .map_err(|error| format!("serialize replay start: {error:?}"))?;
    let initial_input_mask = core.input_mask_snapshot();
    let initial_checkpoint = checkpoint(&core, &video)?;

    let reset_frame = start_frame + 75;
    let actions = vec![
        action(
            0,
            start_frame + 5,
            ActionKind::Button {
                button: "A".into(),
                pressed: true,
            },
        ),
        action(
            1,
            start_frame + 12,
            ActionKind::Button {
                button: "A".into(),
                pressed: false,
            },
        ),
        action(
            2,
            start_frame + 20,
            ActionKind::Button {
                button: "RIGHT".into(),
                pressed: true,
            },
        ),
        action(
            3,
            start_frame + 35,
            ActionKind::Button {
                button: "RIGHT".into(),
                pressed: false,
            },
        ),
        action(
            4,
            reset_frame,
            ActionKind::System {
                command: SystemCommand::Reset,
                slot: None,
            },
        ),
    ];

    let end_frame = start_frame + 180;
    let mut checkpoints = vec![initial_checkpoint];
    let mut action_index = 0usize;

    while core.frame_count() < end_frame {
        let current = core.frame_count();
        let mut frame_actions = Vec::new();
        while let Some(event) = actions.get(action_index) {
            if event.frame != current {
                break;
            }
            frame_actions.push(event.clone());
            action_index += 1;
        }

        core.step_frame(&frame_actions, &mut video, &mut audio)
            .map_err(|error| format!("record replay frame: {error:?}"))?;

        if (core.frame_count() - start_frame) % 30 == 0 {
            checkpoints.push(checkpoint(&core, &video)?);
        }
    }

    let final_state = core
        .serialize_state()
        .map_err(|error| format!("serialize replay final: {error:?}"))?;

    let ledger = ReplayLedger {
        schema: REPLAY_SCHEMA.into(),
        game_sha256: sha256_file(&rom_path)?,
        core_name: core.identity().library_name.clone(),
        core_version: core.identity().library_version.clone(),
        core_sha256: sha256_file(&core_path)?,
        start_frame,
        end_frame,
        initial_state_base64: BASE64.encode(&initial_state),
        initial_input_mask,
        actions,
        checkpoints,
        final_state_sha256: sha256_bytes(&final_state),
        final_frame_sha256: sha256_bytes(&video.rgba8),
    };
    ledger.validate()?;

    let replay_json =
        serde_json::to_vec_pretty(&ledger).map_err(|error| format!("serialize replay: {error}"))?;
    let replay_sha256 = sha256_bytes(&replay_json);

    let exact_divergence = verify(&mut core, &ledger)?;
    if exact_divergence.is_some() {
        return Err(format!(
            "exact replay diverged at {:?}",
            exact_divergence
        ));
    }

    let mut mutated = ledger.clone();
    let reset = mutated
        .actions
        .iter_mut()
        .find(|event| {
            matches!(
                &event.action,
                ActionKind::System {
                    command: SystemCommand::Reset,
                    ..
                }
            )
        })
        .ok_or_else(|| "qualification replay is missing reset action".to_owned())?;
    reset.action = ActionKind::Button {
        button: "B".into(),
        pressed: true,
    };

    let divergence = verify(&mut core, &mutated)?;
    if divergence.is_none() {
        return Err("mutated replay was not detected as divergent".into());
    }

    let receipt = ReplayQualificationReceipt {
        schema: "phicade.replay-qualification.v1",
        result: "PASS",
        game_sha256: ledger.game_sha256.clone(),
        core_sha256: ledger.core_sha256.clone(),
        replay_sha256,
        start_frame,
        end_frame,
        action_count: ledger.actions.len(),
        checkpoint_count: ledger.checkpoints.len(),
        exact_replay_pass: true,
        divergence_probe_detected: true,
        first_divergence_frame: divergence,
    };

    let json =
        serde_json::to_string_pretty(&receipt).map_err(|error| format!("serialize receipt: {error}"))?;
    println!("{json}");
    if let Some(parent) = receipt_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(&receipt_path, format!("{json}\n"))
        .map_err(|error| format!("write {}: {error}", receipt_path.display()))?;

    Ok(())
}
