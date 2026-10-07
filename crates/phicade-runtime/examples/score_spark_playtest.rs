use phicade_runtime::{score_spark_microtask, SparkMicrotask, SparkSemanticObservation};
use serde::{Deserialize, Serialize};
use std::{env, fs, path::PathBuf};

const PLAYTEST_SCHEMA: &str = "phicade.spark-ollama-playtest.v2";
const SCORED_RECEIPT_SCHEMA: &str = "phicade.spark-task-scored-playtest.v1";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlaytestReceipt {
    schema: String,
    record_status: String,
    model: String,
    model_digest: String,
    requested_turns: u64,
    executed_actions: u64,
    objective: Option<String>,
    initial_observation: SparkSemanticObservation,
    final_observation: SparkSemanticObservation,
    initial_runtime_hash: String,
    final_runtime_hash: String,
    turns: Vec<TurnEvidence>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TurnEvidence {
    #[serde(default)]
    provider_total_duration_ns: Option<u64>,
    #[serde(default)]
    provider_eval_count: Option<u64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScoredPlaytest {
    schema: String,
    source_schema: String,
    record_status: String,
    task: phicade_runtime::SparkTaskQualityEvidence,
    model: String,
    model_digest: String,
    requested_turns: u64,
    executed_actions: u64,
    average_provider_turn_seconds: Option<f64>,
    total_eval_count: Option<u64>,
    initial_runtime_hash: String,
    final_runtime_hash: String,
}

fn usage() -> ! {
    eprintln!(
        "Usage: cargo run -p phicade-runtime --example score_spark_playtest -- --input playtest.json --task move-east|use-dash|use-pulse --out scored.json"
    );
    std::process::exit(2);
}

fn parse_task(value: &str) -> Option<SparkMicrotask> {
    match value {
        "move-east" => Some(SparkMicrotask::MoveEast),
        "use-dash" => Some(SparkMicrotask::UseDash),
        "use-pulse" => Some(SparkMicrotask::UsePulse),
        _ => None,
    }
}

fn parse_args() -> (PathBuf, SparkMicrotask, PathBuf) {
    let mut input = None;
    let mut task = None;
    let mut out = None;
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = args.next().map(PathBuf::from),
            "--task" => task = args.next().as_deref().and_then(parse_task),
            "--out" => out = args.next().map(PathBuf::from),
            _ if arg.starts_with("--input=") => {
                input = Some(PathBuf::from(&arg["--input=".len()..]))
            }
            _ if arg.starts_with("--task=") => {
                task = parse_task(&arg["--task=".len()..])
            }
            _ if arg.starts_with("--out=") => {
                out = Some(PathBuf::from(&arg["--out=".len()..]))
            }
            _ => usage(),
        }
    }

    (
        input.unwrap_or_else(|| usage()),
        task.unwrap_or_else(|| usage()),
        out.unwrap_or_else(|| usage()),
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (input, task, out) = parse_args();
    let receipt: PlaytestReceipt = serde_json::from_slice(&fs::read(&input)?)?;

    if receipt.schema != PLAYTEST_SCHEMA {
        return Err(format!(
            "unsupported playtest schema: expected {PLAYTEST_SCHEMA}, got {}",
            receipt.schema
        )
        .into());
    }
    if receipt.record_status != "PASS" {
        return Err("task quality can only be scored from a PASS playtest receipt".into());
    }

    let expected_objective = task.objective();
    if receipt.objective.as_deref() != Some(expected_objective) {
        return Err(format!(
            "playtest objective does not match frozen task objective: expected {:?}, got {:?}",
            expected_objective, receipt.objective
        )
        .into());
    }

    let task_evidence =
        score_spark_microtask(task, &receipt.initial_observation, &receipt.final_observation)?;

    let durations = receipt
        .turns
        .iter()
        .filter_map(|turn| turn.provider_total_duration_ns)
        .collect::<Vec<_>>();
    let average_provider_turn_seconds = (!durations.is_empty()).then(|| {
        durations.iter().sum::<u64>() as f64 / durations.len() as f64 / 1_000_000_000.0
    });
    let eval_counts = receipt
        .turns
        .iter()
        .filter_map(|turn| turn.provider_eval_count)
        .collect::<Vec<_>>();
    let total_eval_count = (!eval_counts.is_empty()).then(|| eval_counts.iter().sum());

    let scored = ScoredPlaytest {
        schema: SCORED_RECEIPT_SCHEMA.into(),
        source_schema: PLAYTEST_SCHEMA.into(),
        record_status: "PASS".into(),
        task: task_evidence,
        model: receipt.model,
        model_digest: receipt.model_digest,
        requested_turns: receipt.requested_turns,
        executed_actions: receipt.executed_actions,
        average_provider_turn_seconds,
        total_eval_count,
        initial_runtime_hash: receipt.initial_runtime_hash,
        final_runtime_hash: receipt.final_runtime_hash,
    };

    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&out, serde_json::to_vec_pretty(&scored)?)?;
    println!("{}", serde_json::to_string_pretty(&scored)?);
    Ok(())
}
