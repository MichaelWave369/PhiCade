use serde::{Deserialize, Serialize};

pub const MODEL_ADMISSION_SCHEMA: &str = "phicade.model-admission.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModelEvidenceClassification {
    PassStable,
    IdentityDrift,
    IdentityUnresolved,
    PlaytestFailStableIdentity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModelRouteDisposition {
    Eligible,
    QuarantinedIdentityDrift,
    QuarantinedIdentityUnresolved,
    ExcludedBehaviorFailure,
    InvalidEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelTrialEvidence {
    pub model: String,
    pub classification: ModelEvidenceClassification,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after_digest: Option<String>,
    pub digest_stable: bool,
    pub playtest_passed: bool,
    pub executed_actions: u64,
    pub runtime_hash_changed: bool,
}

impl ModelTrialEvidence {
    pub fn validate(&self) -> Result<(), String> {
        if self.model.trim().is_empty() {
            return Err("model evidence requires a non-empty model name".into());
        }

        for (label, digest) in [
            ("beforeDigest", self.before_digest.as_deref()),
            ("afterDigest", self.after_digest.as_deref()),
        ] {
            if let Some(digest) = digest {
                if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                    return Err(format!(
                        "model evidence {label} must be a 64-character hex digest"
                    ));
                }
            }
        }

        if self.digest_stable {
            let Some(before) = self.before_digest.as_deref() else {
                return Err(
                    "digestStable=true requires both beforeDigest and afterDigest".into(),
                );
            };
            let Some(after) = self.after_digest.as_deref() else {
                return Err(
                    "digestStable=true requires both beforeDigest and afterDigest".into(),
                );
            };
            if before != after {
                return Err(
                    "digestStable=true conflicts with different before/after digests".into(),
                );
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelAdmission {
    pub schema: String,
    pub model: String,
    pub disposition: ModelRouteDisposition,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admitted_digest: Option<String>,
}

pub fn assess_model_evidence(evidence: &ModelTrialEvidence) -> ModelAdmission {
    if let Err(error) = evidence.validate() {
        return ModelAdmission {
            schema: MODEL_ADMISSION_SCHEMA.into(),
            model: evidence.model.clone(),
            disposition: ModelRouteDisposition::InvalidEvidence,
            reason: error,
            admitted_digest: None,
        };
    }

    let invalid = |reason: &str| ModelAdmission {
        schema: MODEL_ADMISSION_SCHEMA.into(),
        model: evidence.model.clone(),
        disposition: ModelRouteDisposition::InvalidEvidence,
        reason: reason.into(),
        admitted_digest: None,
    };

    match evidence.classification {
        ModelEvidenceClassification::IdentityDrift => ModelAdmission {
            schema: MODEL_ADMISSION_SCHEMA.into(),
            model: evidence.model.clone(),
            disposition: ModelRouteDisposition::QuarantinedIdentityDrift,
            reason: "model identity changed during the governed trial; behavior is not admissible for routing".into(),
            admitted_digest: None,
        },
        ModelEvidenceClassification::IdentityUnresolved => ModelAdmission {
            schema: MODEL_ADMISSION_SCHEMA.into(),
            model: evidence.model.clone(),
            disposition: ModelRouteDisposition::QuarantinedIdentityUnresolved,
            reason: "model identity could not be resolved before and after the governed trial".into(),
            admitted_digest: None,
        },
        ModelEvidenceClassification::PlaytestFailStableIdentity => {
            if !evidence.digest_stable {
                return invalid(
                    "stable-identity behavior failure classification requires digestStable=true",
                );
            }
            ModelAdmission {
                schema: MODEL_ADMISSION_SCHEMA.into(),
                model: evidence.model.clone(),
                disposition: ModelRouteDisposition::ExcludedBehaviorFailure,
                reason: "model identity was stable, but the governed playtest did not pass".into(),
                admitted_digest: evidence.after_digest.clone(),
            }
        }
        ModelEvidenceClassification::PassStable => {
            if !evidence.digest_stable {
                return invalid("PASS_STABLE requires digestStable=true");
            }
            if !evidence.playtest_passed {
                return invalid("PASS_STABLE requires playtestPassed=true");
            }
            if evidence.executed_actions == 0 {
                return invalid("PASS_STABLE requires at least one executed action");
            }
            if !evidence.runtime_hash_changed {
                return invalid("PASS_STABLE requires runtimeHashChanged=true");
            }

            ModelAdmission {
                schema: MODEL_ADMISSION_SCHEMA.into(),
                model: evidence.model.clone(),
                disposition: ModelRouteDisposition::Eligible,
                reason: "stable model identity and governed playtest evidence are sufficient for routing admission".into(),
                admitted_digest: evidence.after_digest.clone(),
            }
        }
    }
}

pub fn eligible_models<'a>(
    evidence: &'a [ModelTrialEvidence],
) -> Vec<ModelAdmission> {
    evidence
        .iter()
        .map(assess_model_evidence)
        .filter(|admission| admission.disposition == ModelRouteDisposition::Eligible)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(ch: char) -> String {
        std::iter::repeat(ch).take(64).collect()
    }

    fn stable_pass(model: &str, ch: char) -> ModelTrialEvidence {
        let digest = digest(ch);
        ModelTrialEvidence {
            model: model.into(),
            classification: ModelEvidenceClassification::PassStable,
            before_digest: Some(digest.clone()),
            after_digest: Some(digest),
            digest_stable: true,
            playtest_passed: true,
            executed_actions: 5,
            runtime_hash_changed: true,
        }
    }

    #[test]
    fn rung51_qwen_and_llama_are_admissible_but_gemma_is_quarantined() {
        let qwen = stable_pass("qwen2.5:0.5b-instruct", 'a');
        let llama = stable_pass("llama3.2:1b", 'b');
        let gemma = ModelTrialEvidence {
            model: "gemma3:1b".into(),
            classification: ModelEvidenceClassification::IdentityDrift,
            before_digest: Some(digest('c')),
            after_digest: Some(digest('d')),
            digest_stable: false,
            playtest_passed: false,
            executed_actions: 0,
            runtime_hash_changed: false,
        };

        assert_eq!(
            assess_model_evidence(&qwen).disposition,
            ModelRouteDisposition::Eligible
        );
        assert_eq!(
            assess_model_evidence(&llama).disposition,
            ModelRouteDisposition::Eligible
        );
        assert_eq!(
            assess_model_evidence(&gemma).disposition,
            ModelRouteDisposition::QuarantinedIdentityDrift
        );

        let admitted = eligible_models(&[qwen, gemma, llama]);
        assert_eq!(admitted.len(), 2);
        assert_eq!(admitted[0].model, "qwen2.5:0.5b-instruct");
        assert_eq!(admitted[1].model, "llama3.2:1b");
    }

    #[test]
    fn pass_stable_claim_fails_closed_when_evidence_is_contradictory() {
        let mut evidence = stable_pass("contradictory", 'e');
        evidence.runtime_hash_changed = false;
        let admission = assess_model_evidence(&evidence);
        assert_eq!(
            admission.disposition,
            ModelRouteDisposition::InvalidEvidence
        );
    }

    #[test]
    fn stable_behavior_failure_is_not_identity_quarantine() {
        let digest = digest('f');
        let evidence = ModelTrialEvidence {
            model: "stable-but-failed".into(),
            classification: ModelEvidenceClassification::PlaytestFailStableIdentity,
            before_digest: Some(digest.clone()),
            after_digest: Some(digest.clone()),
            digest_stable: true,
            playtest_passed: false,
            executed_actions: 0,
            runtime_hash_changed: false,
        };

        let admission = assess_model_evidence(&evidence);
        assert_eq!(
            admission.disposition,
            ModelRouteDisposition::ExcludedBehaviorFailure
        );
        assert_eq!(admission.admitted_digest.as_deref(), Some(digest.as_str()));
    }
}
