use phicade_runtime::{
    legend_bouncehome_intent_from_action, ActionEnvelope, ActionKind, ActionSource,
    AuthorityPolicy, PixelForgeCartridgeQualificationReceipt, PixelForgeJsonlClient,
    PIXELFORGE_CARTRIDGE_QUALIFICATION_SCHEMA, PIXELFORGE_PINNED_QUALIFICATION_REVISION,
    PIXELFORGE_TRANSPORT_REQUEST_SCHEMA, PIXELFORGE_TRANSPORT_RESPONSE_SCHEMA,
};
use serde_json::json;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

const CARTRIDGE_ID: &str = "the-legend-of-more-bounce";
const SCENE_ID: &str = "bouncehome-grove";

fn usage() -> ! {
    eprintln!(
        "Usage: cargo run -p phicade-runtime --example qualify_pixelforge -- --pixelforge-root /path/to/parallax-pixelforge [--out artifacts/pixelforge-cartridge-qualification.json]"
    );
    std::process::exit(2);
}

fn parse_args() -> (PathBuf, PathBuf) {
    let mut args = env::args().skip(1);
    let mut root = None;
    let mut out = PathBuf::from("artifacts/pixelforge-cartridge-qualification.json");

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--pixelforge-root" => root = args.next().map(PathBuf::from),
            "--out" => out = args.next().map(PathBuf::from).unwrap_or_else(|| usage()),
            _ if arg.starts_with("--pixelforge-root=") => {
                root = Some(PathBuf::from(&arg["--pixelforge-root=".len()..]))
            }
            _ if arg.starts_with("--out=") => {
                out = PathBuf::from(&arg["--out=".len()..])
            }
            _ => usage(),
        }
    }

    (root.unwrap_or_else(|| usage()), out)
}

fn git_head(root: &Path) -> Result<String, String> {
    let root = root
        .to_str()
        .ok_or_else(|| "PixelForge path is not valid UTF-8".to_owned())?;
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
    let (root, out) = parse_args();

    let revision = git_head(&root)?;
    if revision != PIXELFORGE_PINNED_QUALIFICATION_REVISION {
        return Err(format!(
            "PixelForge revision mismatch: expected {}, got {}",
            PIXELFORGE_PINNED_QUALIFICATION_REVISION, revision
        )
        .into());
    }

    let mut client = PixelForgeJsonlClient::spawn_node_cartridge(
        &root,
        CARTRIDGE_ID,
        SCENE_ID,
    )?;
    let descriptor = client.descriptor().clone();
    let capability_manifest = client.capability_manifest();

    if descriptor.game_id != CARTRIDGE_ID {
        return Err(format!(
            "expected PixelForge gameId {}, got {}",
            CARTRIDGE_ID, descriptor.game_id
        )
        .into());
    }
    if descriptor.runtime_version != "legend-bouncehome/1" {
        return Err(format!(
            "expected Legend runtime version legend-bouncehome/1, got {}",
            descriptor.runtime_version
        )
        .into());
    }

    let controller_id = "phicade-pixelforge-seat-1";
    let registration = client.register_controller(json!({
        "id": controller_id,
        "kind": "human",
        "binding": "phicade-action-bus"
    }))?;
    if registration["ok"].as_bool() != Some(true) {
        return Err("PixelForge controller registration did not return ok=true".into());
    }

    let initial_observation = client.observe(controller_id)?;
    if initial_observation["cartridgeId"].as_str() != Some(CARTRIDGE_ID)
        || initial_observation["sceneId"].as_str() != Some(SCENE_ID)
    {
        return Err(format!(
            "unexpected initial cartridge observation: {}",
            initial_observation
        )
        .into());
    }
    if initial_observation["self"]["x"].as_f64() != Some(2.5)
        || initial_observation["self"]["y"].as_f64() != Some(8.5)
    {
        return Err(format!(
            "expected Bouncehome player spawn at (2.5, 8.5), got {}",
            initial_observation["self"]
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
            "PhiCade authority rejected qualification action: {}",
            authority_decision.reason
        )
        .into());
    }

    let submitted_intent = legend_bouncehome_intent_from_action(&source_action)?;
    let queue = client.submit(controller_id, submitted_intent.clone(), 0)?;
    if queue["queued"].as_bool() != Some(true) || queue["tick"].as_u64() != Some(0) {
        return Err(format!("unexpected PixelForge queue receipt: {queue}").into());
    }

    let direct_events = client.advance()?;
    if direct_events.len() != 1 || direct_events[0]["type"].as_str() != Some("PLAYER_MOVED") {
        return Err(format!("unexpected direct event stream: {direct_events:?}").into());
    }
    if direct_events[0]["payload"]["direction"].as_str() != Some("RIGHT") {
        return Err(format!(
            "expected RIGHT movement event, got {}",
            direct_events[0]
        )
        .into());
    }

    let semantic_events = client.events(0)?;
    if semantic_events != direct_events {
        return Err("PixelForge semantic event stream diverged from direct event stream".into());
    }

    let final_observation = client.observe(controller_id)?;
    if final_observation["self"]["x"].as_f64() != Some(3.5)
        || final_observation["self"]["y"].as_f64() != Some(8.5)
    {
        return Err(format!(
            "expected governed RIGHT press to move player to (3.5, 8.5), got {}",
            final_observation["self"]
        )
        .into());
    }

    let final_hash = client.hash()?;
    if final_hash.trim().is_empty() || !final_hash.contains(CARTRIDGE_ID) {
        return Err("PixelForge returned an invalid cartridge runtime hash".into());
    }

    let receipt = PixelForgeCartridgeQualificationReceipt {
        schema: PIXELFORGE_CARTRIDGE_QUALIFICATION_SCHEMA.into(),
        record_status: "PASS".into(),
        pixelforge_revision: revision,
        cartridge_id: CARTRIDGE_ID.into(),
        scene_id: SCENE_ID.into(),
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

    println!("PixelForge real-cartridge qualification PASS");
    println!("Cartridge: {CARTRIDGE_ID}");
    println!("Scene: {SCENE_ID}");
    println!(
        "Pinned PixelForge revision: {}",
        PIXELFORGE_PINNED_QUALIFICATION_REVISION
    );
    println!("Receipt: {}", out.display());
    Ok(())
}
