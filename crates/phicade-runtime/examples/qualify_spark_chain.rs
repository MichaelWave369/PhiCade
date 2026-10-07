use phicade_runtime::{
    spark_threshold_move_intent_from_action, ActionEnvelope, ActionKind, ActionSource,
    AuthorityPolicy, PixelForgeJsonlClient, PixelForgeSparkChainQualificationReceipt,
    PIXELFORGE_PINNED_SPARK_RUNTIME_REVISION, PIXELFORGE_SPARK_CHAIN_QUALIFICATION_SCHEMA,
    PIXELFORGE_TRANSPORT_REQUEST_SCHEMA, PIXELFORGE_TRANSPORT_RESPONSE_SCHEMA,
    SPARK_PINNED_BRIDGE_REVISION,
};
use serde_json::json;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

fn usage() -> ! {
    eprintln!(
        "Usage: cargo run -p phicade-runtime --example qualify_spark_chain -- --pixelforge-root /path/to/parallax-pixelforge --spark-root /path/to/SparkTheSubstrate [--out artifacts/phicade-pixelforge-spark-qualification.json]"
    );
    std::process::exit(2);
}

fn parse_args() -> (PathBuf, PathBuf, PathBuf) {
    let mut args = env::args().skip(1);
    let mut pixelforge_root = None;
    let mut spark_root = None;
    let mut out = PathBuf::from("artifacts/phicade-pixelforge-spark-qualification.json");

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

    if descriptor.game_id != "spark-the-substrate" {
        return Err(format!(
            "expected SPARK gameId spark-the-substrate, got {}",
            descriptor.game_id
        )
        .into());
    }
    if descriptor.runtime_version != "spark-threshold/0.17.0-bridge-v1" {
        return Err(format!(
            "unexpected SPARK runtime version {}",
            descriptor.runtime_version
        )
        .into());
    }

    let controller_id = "phicade-spark-seat-1";
    let registration = client.register_controller(json!({
        "id": controller_id,
        "kind": "human",
        "binding": "phicade-action-bus"
    }))?;
    if registration["ok"].as_bool() != Some(true) {
        return Err("SPARK controller registration did not return ok=true".into());
    }

    let initial_observation = client.observe(controller_id)?;
    if initial_observation["state"]["room"].as_str() != Some("threshold")
        || initial_observation["state"]["form"].as_str() != Some("spark")
        || initial_observation["state"]["player"]["x"].as_f64() != Some(480.0)
        || initial_observation["state"]["player"]["y"].as_f64() != Some(390.0)
        || initial_observation["state"]["player"]["maxHp"].as_f64() != Some(112.0)
    {
        return Err(format!(
            "unexpected canonical SPARK Threshold observation: {}",
            initial_observation
        )
        .into());
    }

    let source_action = ActionEnvelope {
        sequence: 0,
        frame: 0,
        source: ActionSource::Human { seat: 1 },
        action: ActionKind::Button {
            button: "RIGHT".into(),
            pressed: true,
        },
    };

    let authority = AuthorityPolicy::new(1);
    let mut decisions = authority.authorize_batch(&[source_action.clone()], 0);
    let (_, authority_decision) = decisions
        .pop()
        .ok_or("PhiCade authority produced no decision")?;
    if !authority_decision.accepted {
        return Err(format!(
            "PhiCade authority rejected SPARK qualification action: {}",
            authority_decision.reason
        )
        .into());
    }

    let submitted_intent = spark_threshold_move_intent_from_action(&source_action)?;
    let queue = client.submit(controller_id, submitted_intent.clone(), 0)?;
    if queue["queued"].as_bool() != Some(true) || queue["tick"].as_u64() != Some(0) {
        return Err(format!("unexpected SPARK queue receipt: {queue}").into());
    }

    let direct_events = client.advance()?;
    if direct_events.len() != 1
        || direct_events[0]["type"].as_str() != Some("SPARK_PLAYER_MOVED")
    {
        return Err(format!(
            "unexpected SPARK direct event stream: {direct_events:?}"
        )
        .into());
    }

    let semantic_events = client.events(0)?;
    if semantic_events != direct_events {
        return Err("SPARK semantic event stream diverged from advance() result".into());
    }

    let final_observation = client.observe(controller_id)?;
    let initial_x = initial_observation["state"]["player"]["x"]
        .as_f64()
        .ok_or("initial SPARK x is not numeric")?;
    let final_x = final_observation["state"]["player"]["x"]
        .as_f64()
        .ok_or("final SPARK x is not numeric")?;
    let initial_y = initial_observation["state"]["player"]["y"]
        .as_f64()
        .ok_or("initial SPARK y is not numeric")?;
    let final_y = final_observation["state"]["player"]["y"]
        .as_f64()
        .ok_or("final SPARK y is not numeric")?;

    if final_x <= initial_x
        || final_y != initial_y
        || final_observation["state"]["room"].as_str() != Some("threshold")
        || final_observation["tick"].as_u64() != Some(1)
    {
        return Err(format!(
            "governed RIGHT action did not produce expected SPARK state transition: {}",
            final_observation
        )
        .into());
    }

    let final_hash = client.hash()?;
    if final_hash.len() != 64 || !final_hash.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err("SPARK runtime hash is not a 64-character hex digest".into());
    }

    let receipt = PixelForgeSparkChainQualificationReceipt {
        schema: PIXELFORGE_SPARK_CHAIN_QUALIFICATION_SCHEMA.into(),
        record_status: "PASS".into(),
        phicade_revision,
        pixelforge_revision,
        spark_revision,
        transport_request_schema: PIXELFORGE_TRANSPORT_REQUEST_SCHEMA.into(),
        transport_response_schema: PIXELFORGE_TRANSPORT_RESPONSE_SCHEMA.into(),
        descriptor,
        capability_manifest,
        controller_id: controller_id.into(),
        initial_observation,
        authority_decision,
        source_action,
        submitted_intent,
        direct_events,
        semantic_events,
        final_observation,
        final_hash,
    };

    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&out, serde_json::to_vec_pretty(&receipt)?)?;
    client.close()?;

    println!("PhiCade -> PixelForge -> SPARK qualification PASS");
    println!(
        "Pinned PixelForge revision: {}",
        PIXELFORGE_PINNED_SPARK_RUNTIME_REVISION
    );
    println!("Pinned SPARK revision: {}", SPARK_PINNED_BRIDGE_REVISION);
    println!("Receipt: {}", out.display());
    Ok(())
}
