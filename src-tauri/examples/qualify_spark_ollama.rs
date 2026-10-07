use phicade_lib::providers::ollama::{complete_spark_semantic_turn, inspect_model};
use phicade_runtime::{
    compile_spark_agent_turn, spark_semantic_observation_from_bridge,
    spark_threshold_intent_from_action, ActionEnvelope, AgentGrant, AuthorityDecision,
    AuthorityPolicy, ControlMode, PixelForgeJsonlClient, SparkAgentTurnRequest,
    SparkAgentTurnResponse, SparkSemanticObservation, PIXELFORGE_PINNED_SPARK_RUNTIME_REVISION,
    SPARK_AGENT_TURN_REQUEST_SCHEMA, SPARK_PINNED_BRIDGE_REVISION,
};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

const RECEIPT_SCHEMA: &str = "phicade.spark-ollama-playtest.v2";
const DEFAULT_TURNS: u64 = 3;
const MAX_TURNS: u64 = 8;
const MAX_MEMORY_BYTES: u32 = 4096;
const MAX_MEMORY_UPDATE_BYTES: u32 = 1024;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SparkOllamaTurnEvidence {
    turn_id: u64,
    observation: SparkSemanticObservation,
    observation_runtime_hash: String,
    provider_model: String,
    provider_total_duration_ns: Option<u64>,
    provider_eval_count: Option<u64>,
    provider_response: SparkAgentTurnResponse,
    authority_decision: Option<AuthorityDecision>,
    source_action: Option<ActionEnvelope>,
    submitted_intent: Option<Value>,
    direct_events: Vec<Value>,
    memory_after: String,
    memory_sha256_after: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SparkOllamaPlaytestReceipt {
    schema: String,
    record_status: String,
    phicade_revision: String,
    pixelforge_revision: String,
    spark_revision: String,
    provider: String,
    model: String,
    model_digest: String,
    model_capabilities: Vec<String>,
    agent_id: String,
    requested_turns: u64,
    executed_actions: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    objective: Option<String>,
    initial_observation: SparkSemanticObservation,
    final_observation: SparkSemanticObservation,
    initial_runtime_hash: String,
    final_runtime_hash: String,
    semantic_events: Vec<Value>,
    turns: Vec<SparkOllamaTurnEvidence>,
}

fn usage() -> ! {
    eprintln!(
        "Usage: cargo run --manifest-path src-tauri/Cargo.toml --example qualify_spark_ollama -- --pixelforge-root /path/to/parallax-pixelforge --spark-root /path/to/SparkTheSubstrate --model MODEL [--ollama-base-url http://127.0.0.1:11434] [--turns 3] [--objective TEXT] [--out artifacts/phicade-spark-ollama-playtest.json]"
    );
    std::process::exit(2);
}

#[derive(Debug)]
struct Args {
    pixelforge_root: PathBuf,
    spark_root: PathBuf,
    model: String,
    ollama_base_url: String,
    turns: u64,
    objective: Option<String>,
    out: PathBuf,
}

fn parse_args() -> Args {
    let mut args = env::args().skip(1);
    let mut pixelforge_root = None;
    let mut spark_root = None;
    let mut model = None;
    let mut ollama_base_url = "http://127.0.0.1:11434".to_owned();
    let mut turns = DEFAULT_TURNS;
    let mut objective = None;
    let mut out = PathBuf::from("artifacts/phicade-spark-ollama-playtest.json");

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--pixelforge-root" => pixelforge_root = args.next().map(PathBuf::from),
            "--spark-root" => spark_root = args.next().map(PathBuf::from),
            "--model" => model = args.next(),
            "--ollama-base-url" => {
                ollama_base_url = args.next().unwrap_or_else(|| usage())
            }
            "--turns" => {
                turns = args
                    .next()
                    .and_then(|value| value.parse::<u64>().ok())
                    .unwrap_or_else(|| usage())
            }
            "--objective" => objective = args.next(),
            "--out" => out = args.next().map(PathBuf::from).unwrap_or_else(|| usage()),
            _ if arg.starts_with("--pixelforge-root=") => {
                pixelforge_root = Some(PathBuf::from(&arg["--pixelforge-root=".len()..]))
            }
            _ if arg.starts_with("--spark-root=") => {
                spark_root = Some(PathBuf::from(&arg["--spark-root=".len()..]))
            }
            _ if arg.starts_with("--model=") => {
                model = Some(arg["--model=".len()..].to_owned())
            }
            _ if arg.starts_with("--ollama-base-url=") => {
                ollama_base_url = arg["--ollama-base-url=".len()..].to_owned()
            }
            _ if arg.starts_with("--turns=") => {
                turns = arg["--turns=".len()..]
                    .parse::<u64>()
                    .unwrap_or_else(|_| usage())
            }
            _ if arg.starts_with("--objective=") => {
                objective = Some(arg["--objective=".len()..].to_owned())
            }
            _ if arg.starts_with("--out=") => {
                out = PathBuf::from(&arg["--out=".len()..])
            }
            _ => usage(),
        }
    }

    if !(1..=MAX_TURNS).contains(&turns) {
        eprintln!("--turns must be in 1..={MAX_TURNS}");
        usage();
    }

    Args {
        pixelforge_root: pixelforge_root.unwrap_or_else(|| usage()),
        spark_root: spark_root.unwrap_or_else(|| usage()),
        model: model.filter(|value| !value.trim().is_empty()).unwrap_or_else(|| usage()),
        ollama_base_url,
        turns,
        objective: objective
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty()),
        out,
    }
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

fn sha256_text(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hex::encode(hasher.finalize())
}

fn allowed_buttons(grant: &AgentGrant) -> Vec<String> {
    grant.allowed_buttons.iter().cloned().collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args();

    let phicade_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()?;
    let phicade_revision = git_head(&phicade_root)?;
    let pixelforge_revision = git_head(&args.pixelforge_root)?;
    let spark_revision = git_head(&args.spark_root)?;

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

    let model_details = tauri::async_runtime::block_on(inspect_model(
        &args.ollama_base_url,
        &args.model,
    ))?;

    let agent_id = "phi-spark-local-model";
    let grant = AgentGrant::spark_threshold(agent_id, 1);
    let mut policy = AuthorityPolicy::new(1);
    policy.set_mode(ControlMode::PhiBot, Some(grant.clone()))?;

    let mut client = PixelForgeJsonlClient::spawn_external_spark(
        &args.pixelforge_root,
        &args.spark_root,
    )?;
    let controller_id = "phicade-spark-local-model-seat-1";
    let registration = client.register_controller(serde_json::json!({
        "id": controller_id,
        "kind": "phi-bot",
        "binding": "phicade-ollama-semantic-driver",
        "agentId": agent_id,
        "model": model_details.name.clone(),
        "modelDigest": model_details.digest.clone()
    }))?;
    if registration["ok"].as_bool() != Some(true) {
        return Err("SPARK local-model controller registration did not return ok=true".into());
    }

    let initial_raw_observation = client.observe(controller_id)?;
    let initial_observation = spark_semantic_observation_from_bridge(&initial_raw_observation)?;
    let initial_runtime_hash = client.hash()?;
    let mut memory = String::new();
    let mut memory_sha256 = sha256_text(&memory);
    let mut evidence = Vec::new();
    let mut executed_actions = 0u64;

    for turn_id in 0..args.turns {
        let raw_observation = client.observe(controller_id)?;
        let observation = spark_semantic_observation_from_bridge(&raw_observation)?;
        let observation_runtime_hash = client.hash()?;
        let current_tick = observation.tick;

        let request = SparkAgentTurnRequest {
            schema: SPARK_AGENT_TURN_REQUEST_SCHEMA.into(),
            turn_id,
            agent_id: agent_id.into(),
            seat: 1,
            observation: observation.clone(),
            observation_runtime_hash: observation_runtime_hash.clone(),
            allowed_buttons: allowed_buttons(&grant),
            max_actions: 1,
            objective: args.objective.clone(),
            memory: memory.clone(),
            memory_sha256: memory_sha256.clone(),
            max_memory_bytes: MAX_MEMORY_BYTES,
            max_memory_update_bytes: MAX_MEMORY_UPDATE_BYTES,
        };
        request.validate()?;

        let provider = tauri::async_runtime::block_on(complete_spark_semantic_turn(
            request.clone(),
            &args.ollama_base_url,
            &args.model,
        ))?;
        let provider_model = provider.model.clone();
        let provider_total_duration_ns = provider.total_duration_ns;
        let provider_eval_count = provider.eval_count;
        let response = provider.response;
        let compiled = compile_spark_agent_turn(&request, &response, current_tick)?;

        let mut authority_decision = None;
        let mut source_action = None;
        let mut submitted_intent = None;

        if let Some(action) = compiled.first().cloned() {
            let mut decisions = policy.authorize_batch(&[action.clone()], current_tick);
            let (_, decision) = decisions
                .pop()
                .ok_or("PhiCade authority produced no decision for model action")?;
            if !decision.accepted {
                return Err(format!(
                    "PhiCade authority rejected model action: {}",
                    decision.reason
                )
                .into());
            }

            let intent = spark_threshold_intent_from_action(&action)?;
            let queued = client.submit(controller_id, intent.clone(), current_tick)?;
            if queued["queued"].as_bool() != Some(true)
                || queued["tick"].as_u64() != Some(current_tick)
            {
                return Err(format!("unexpected SPARK queue receipt: {queued}").into());
            }

            executed_actions += 1;
            authority_decision = Some(decision);
            source_action = Some(action);
            submitted_intent = Some(intent);
        }

        let direct_events = client.advance()?;

        if let Some(memory_update) = response.memory_update.as_ref() {
            memory = memory_update.clone();
            memory_sha256 = sha256_text(&memory);
        }

        evidence.push(SparkOllamaTurnEvidence {
            turn_id,
            observation,
            observation_runtime_hash,
            provider_model,
            provider_total_duration_ns,
            provider_eval_count,
            provider_response: response,
            authority_decision,
            source_action,
            submitted_intent,
            direct_events,
            memory_after: memory.clone(),
            memory_sha256_after: memory_sha256.clone(),
        });
    }

    if executed_actions == 0 {
        return Err(
            "local model completed the bounded playtest without proposing any executable action"
                .into(),
        );
    }

    let final_raw_observation = client.observe(controller_id)?;
    let final_observation = spark_semantic_observation_from_bridge(&final_raw_observation)?;
    let final_runtime_hash = client.hash()?;
    if final_runtime_hash == initial_runtime_hash {
        return Err("SPARK runtime hash did not change during the model playtest".into());
    }

    let semantic_events = client.events(0)?;
    let direct_event_count: usize = evidence.iter().map(|turn| turn.direct_events.len()).sum();
    if semantic_events.len() != direct_event_count {
        return Err(format!(
            "SPARK semantic event count {} diverged from {} direct event(s)",
            semantic_events.len(),
            direct_event_count
        )
        .into());
    }

    let final_model_details = tauri::async_runtime::block_on(inspect_model(
        &args.ollama_base_url,
        &args.model,
    ))?;
    if final_model_details.digest != model_details.digest {
        return Err(format!(
            "Ollama model digest changed during playtest: {} -> {}",
            model_details.digest, final_model_details.digest
        )
        .into());
    }

    let receipt = SparkOllamaPlaytestReceipt {
        schema: RECEIPT_SCHEMA.into(),
        record_status: "PASS".into(),
        phicade_revision,
        pixelforge_revision,
        spark_revision,
        provider: "ollama".into(),
        model: model_details.name,
        model_digest: model_details.digest,
        model_capabilities: model_details.capabilities,
        agent_id: agent_id.into(),
        requested_turns: args.turns,
        executed_actions,
        objective: args.objective,
        initial_observation,
        final_observation,
        initial_runtime_hash,
        final_runtime_hash,
        semantic_events,
        turns: evidence,
    };

    if let Some(parent) = args.out.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&args.out, serde_json::to_vec_pretty(&receipt)?)?;
    client.close()?;

    println!("SPARK local-model semantic playtest PASS");
    println!("Model: {} ({})", receipt.model, receipt.model_digest);
    println!("Turns: {}", receipt.requested_turns);
    println!("Executed actions: {}", receipt.executed_actions);
    println!("Receipt: {}", args.out.display());
    Ok(())
}
