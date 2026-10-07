use phicade_runtime::{
    spark_semantic_observation_from_bridge, spark_threshold_intent_from_action, ActionEnvelope,
    ActionKind, ActionSource, AgentGrant, AuthorityPolicy, ControlMode, PixelForgeJsonlClient,
    PixelForgeSparkPhiBotQualificationReceipt, PIXELFORGE_PINNED_SPARK_RUNTIME_REVISION,
    PIXELFORGE_SPARK_PHIBOT_QUALIFICATION_SCHEMA, PIXELFORGE_TRANSPORT_REQUEST_SCHEMA,
    PIXELFORGE_TRANSPORT_RESPONSE_SCHEMA, SPARK_PINNED_BRIDGE_REVISION,
};
use serde_json::{json, Value};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

fn usage() -> ! {
    eprintln!(
        "Usage: cargo run -p phicade-runtime --example qualify_spark_phibot -- --pixelforge-root /path/to/parallax-pixelforge --spark-root /path/to/SparkTheSubstrate [--out artifacts/phicade-pixelforge-spark-phibot-qualification.json]"
    );
    std::process::exit(2);
}

fn parse_args() -> (PathBuf, PathBuf, PathBuf) {
    let mut args = env::args().skip(1);
    let mut pixelforge_root = None;
    let mut spark_root = None;
    let mut out =
        PathBuf::from("artifacts/phicade-pixelforge-spark-phibot-qualification.json");

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--pixelforge-root" => pixelforge_root = args.next().map(PathBuf::from),
            "--spark-root" => spark_root = args.next().map(PathBuf::from),
            "--out" => out = args.next().map(PathBuf::from).unwrap_or_else(|| usage()),
            _ if arg.starts_with("--pixelforge-root=") => {
                pixelforge_root = Some(PathBuf::from(&arg["--pixelforge-root=".len()..]))
            }
            _ if arg.starts_with("--spark-root=") => {
                spark_root = Some(PathBuf::from(&arg["--spark-root=".len()..]))
            }
            _ if arg.starts_with("--out=") => out = PathBuf::from(&arg["--out=".len()..]),
            _ => usage(),
        }
    }

    (
        pixelforge_root.unwrap_or_else(|| usage()),
        spark_root.unwrap_or_else(|| usage()),
        out,
    )
}

fn git_head(root: &Path) -> Result<String, String> {
    let root = root
        .to_str()
        .ok_or_else(|| "repository path is not valid UTF-8".to_owned())?;
    let output = Command::new("git")
        .args(["-C", root, "rev-parse", "HEAD"])
        .output()
        .map_err(|error| format!("could not run git rev-parse: {error}"))?;

    if !output.status.success() {
        return Err(format!(
            "git rev-parse failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn phi_action(sequence: u64, frame: u64, button: &str, agent_id: &str) -> ActionEnvelope {
    ActionEnvelope {
        sequence,
        frame,
        source: ActionSource::PhiBot {
            agent_id: agent_id.into(),
            seat: 1,
        },
        action: ActionKind::Button {
            button: button.into(),
            pressed: true,
        },
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (pixelforge_root, spark_root, out) = parse_args();

    let phicade_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let phicade_revision = git_head(&phicade_root)?;
    let pixelforge_revision = git_head(&pixelforge_root)?;
    let spark_revision = git_head(&spark_root)?;

    if pixelforge_revision != PIXELFORGE_PINNED_SPARK_RUNTIME_REVISION {
        return Err(format!(
            "PixelForge revision mismatch: expected {}, got {}",
            PIXELFORGE_PINNED_SPARK_RUNTIME_REVISION, pixelforge_revision
        )
        .into());
    }
    if spark_revision != SPARK_PINNED_BRIDGE_REVISION {
        return Err(format!(
            "SPARK revision mismatch: expected {}, got {}",
            SPARK_PINNED_BRIDGE_REVISION, spark_revision
        )
        .into());
    }

    let mut client = PixelForgeJsonlClient::spawn_external_spark(
        &pixelforge_root,
        &spark_root,
    )?;
    let descriptor = client.descriptor().clone();
    let capability_manifest = client.capability_manifest();

    if descriptor.game_id != "spark-the-substrate"
        || descriptor.runtime_version != "spark-threshold/0.17.0-bridge-v1"
    {
        return Err(format!("unexpected SPARK descriptor: {descriptor:?}").into());
    }

    let agent_id = "phi-spark-rung47";
    let controller_id = "phicade-spark-phibot-seat-1";
    let registration = client.register_controller(json!({
        "id": controller_id,
        "kind": "phi-bot",
        "binding": "phicade-action-bus",
        "agentId": agent_id
    }))?;
    if registration["ok"].as_bool() != Some(true) {
        return Err("SPARK Phi-Bot controller registration did not return ok=true".into());
    }

    let initial_raw = client.observe(controller_id)?;
    let initial = spark_semantic_observation_from_bridge(&initial_raw)?;
    if initial.room != "threshold"
        || initial.form != "spark"
        || initial.player.x != 480.0
        || initial.player.y != 390.0
        || initial.player.max_hp != 112.0
    {
        return Err(format!(
            "unexpected canonical SPARK semantic observation: {:?}",
            initial
        )
        .into());
    }

    let mut policy = AuthorityPolicy::new(1);
    policy.set_mode(
        ControlMode::PhiBot,
        Some(AgentGrant::spark_threshold(agent_id, 1)),
    )?;

    let buttons = ["RIGHT", "DASH_RIGHT", "PULSE"];
    let expected_events = [
        "SPARK_PLAYER_MOVED",
        "SPARK_DASH_STARTED",
        "SPARK_PULSE_USED",
    ];

    let mut source_actions = Vec::new();
    let mut authority_decisions = Vec::new();
    let mut submitted_intents = Vec::new();
    let mut direct_events = Vec::new();
    let mut semantic_observations = vec![initial];

    for (index, (button, expected_event)) in buttons
        .into_iter()
        .zip(expected_events)
        .enumerate()
    {
        let tick = index as u64;
        let action = phi_action(tick, tick, button, agent_id);
        let mut decisions = policy.authorize_batch(&[action.clone()], tick);
        let (_, decision) = decisions
            .pop()
            .ok_or("PhiCade authority produced no decision")?;
        if !decision.accepted {
            return Err(format!(
                "PhiCade authority rejected {button}: {}",
                decision.reason
            )
            .into());
        }

        let intent = spark_threshold_intent_from_action(&action)?;
        let queue = client.submit(controller_id, intent.clone(), tick)?;
        if queue["queued"].as_bool() != Some(true) || queue["tick"].as_u64() != Some(tick) {
            return Err(format!("unexpected queue receipt for {button}: {queue}").into());
        }

        let events = client.advance()?;
        if events.len() != 1 || events[0]["type"].as_str() != Some(expected_event) {
            return Err(format!(
                "expected {expected_event} for {button}, got {events:?}"
            )
            .into());
        }

        let raw = client.observe(controller_id)?;
        let semantic = spark_semantic_observation_from_bridge(&raw)?;

        source_actions.push(action);
        authority_decisions.push(decision);
        submitted_intents.push(intent);
        direct_events.extend(events);
        semantic_observations.push(semantic);
    }

    let semantic_events = client.events(0)?;
    if semantic_events != direct_events {
        return Err("SPARK semantic event stream diverged from direct events".into());
    }

    let after_move = &semantic_observations[1];
    let after_dash = &semantic_observations[2];
    let after_pulse = &semantic_observations[3];

    if after_move.player.x <= semantic_observations[0].player.x {
        return Err("Phi-Bot RIGHT did not move SPARK positively on X".into());
    }
    if after_dash.dashes != 1 || after_dash.player.dash_ready {
        return Err(format!(
            "Phi-Bot DASH did not engage canonical dash state: {:?}",
            after_dash
        )
        .into());
    }
    if after_pulse.powers != 1 || after_pulse.power_cooldown <= 0.0 {
        return Err(format!(
            "Phi-Bot PULSE did not engage canonical power cooldown: {:?}",
            after_pulse
        )
        .into());
    }

    let final_hash = client.hash()?;
    if final_hash.len() != 64 || !final_hash.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err("SPARK runtime hash is not a 64-character hex digest".into());
    }

    let receipt = PixelForgeSparkPhiBotQualificationReceipt {
        schema: PIXELFORGE_SPARK_PHIBOT_QUALIFICATION_SCHEMA.into(),
        record_status: "PASS".into(),
        phicade_revision,
        pixelforge_revision,
        spark_revision,
        transport_request_schema: PIXELFORGE_TRANSPORT_REQUEST_SCHEMA.into(),
        transport_response_schema: PIXELFORGE_TRANSPORT_RESPONSE_SCHEMA.into(),
        descriptor,
        capability_manifest,
        controller_id: controller_id.into(),
        agent_id: agent_id.into(),
        semantic_observations,
        authority_decisions,
        source_actions,
        submitted_intents,
        direct_events,
        semantic_events,
        final_hash,
    };

    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&out, serde_json::to_vec_pretty(&receipt)?)?;
    client.close()?;

    println!("PhiCade Phi-Bot -> PixelForge -> SPARK qualification PASS");
    println!("Actions: RIGHT -> DASH_RIGHT -> PULSE");
    println!("Receipt: {}", out.display());
    Ok(())
}
