use phicade_runtime::{
    assess_model_evidence, ModelAdmission, ModelEvidenceClassification, ModelTrialEvidence,
};
use serde::{Deserialize, Serialize};
use std::{env, fs, path::PathBuf};

const INPUT_SCHEMA: &str = "spark.isolated-model-identity-matrix.v1";
const OUTPUT_SCHEMA: &str = "phicade.model-admission-set.v1";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MatrixModel {
    model: String,
    classification: ModelEvidenceClassification,
    #[serde(default)]
    before_digest: Option<String>,
    #[serde(default)]
    after_digest: Option<String>,
    digest_stable: bool,
    playtest_passed: bool,
    executed_actions: u64,
    runtime_hash_changed: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IdentityMatrix {
    schema: String,
    models: Vec<MatrixModel>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AdmissionSet {
    schema: String,
    source_schema: String,
    admissions: Vec<ModelAdmission>,
}

fn usage() -> ! {
    eprintln!(
        "Usage: cargo run -p phicade-runtime --example qualify_model_admission -- --input MATRIX.json --out admissions.json"
    );
    std::process::exit(2);
}

fn parse_args() -> (PathBuf, PathBuf) {
    let mut input = None;
    let mut out = None;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = args.next().map(PathBuf::from),
            "--out" => out = args.next().map(PathBuf::from),
            _ if arg.starts_with("--input=") => {
                input = Some(PathBuf::from(&arg["--input=".len()..]))
            }
            _ if arg.starts_with("--out=") => {
                out = Some(PathBuf::from(&arg["--out=".len()..]))
            }
            _ => usage(),
        }
    }
    (
        input.unwrap_or_else(|| usage()),
        out.unwrap_or_else(|| usage()),
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (input, out) = parse_args();
    let bytes = fs::read(&input)?;
    let matrix: IdentityMatrix = serde_json::from_slice(&bytes)?;
    if matrix.schema != INPUT_SCHEMA {
        return Err(format!(
            "unsupported identity-matrix schema: expected {INPUT_SCHEMA}, got {}",
            matrix.schema
        )
        .into());
    }

    let admissions = matrix
        .models
        .into_iter()
        .map(|model| {
            assess_model_evidence(&ModelTrialEvidence {
                model: model.model,
                classification: model.classification,
                before_digest: model.before_digest,
                after_digest: model.after_digest,
                digest_stable: model.digest_stable,
                playtest_passed: model.playtest_passed,
                executed_actions: model.executed_actions,
                runtime_hash_changed: model.runtime_hash_changed,
            })
        })
        .collect::<Vec<_>>();

    let output = AdmissionSet {
        schema: OUTPUT_SCHEMA.into(),
        source_schema: INPUT_SCHEMA.into(),
        admissions,
    };

    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&out, serde_json::to_vec_pretty(&output)?)?;
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
