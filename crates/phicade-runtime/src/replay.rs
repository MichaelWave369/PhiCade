use crate::ActionEnvelope;
use serde::{Deserialize, Serialize};

pub const REPLAY_SCHEMA: &str = "phicade.replay.v1";
pub const REPLAY_RECEIPT_SCHEMA: &str = "phicade.replay-receipt.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayCheckpoint {
    pub frame: u64,
    pub state_sha256: String,
    pub frame_sha256: String,
    pub input_mask: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayLedger {
    pub schema: String,
    pub game_sha256: String,
    pub core_name: String,
    pub core_version: String,
    pub core_sha256: String,
    pub start_frame: u64,
    pub end_frame: u64,
    pub initial_state_base64: String,
    pub initial_input_mask: u16,
    pub actions: Vec<ActionEnvelope>,
    pub checkpoints: Vec<ReplayCheckpoint>,
    pub final_state_sha256: String,
    pub final_frame_sha256: String,
}

impl ReplayLedger {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != REPLAY_SCHEMA {
            return Err(format!("unsupported replay schema: {}", self.schema));
        }
        if self.game_sha256.len() != 64 || self.core_sha256.len() != 64 {
            return Err("replay provenance hashes must be SHA-256 hex".into());
        }
        if self.end_frame < self.start_frame {
            return Err("replay end frame precedes start frame".into());
        }
        if self.initial_state_base64.is_empty() {
            return Err("replay is missing initial state".into());
        }

        let mut previous_sequence = None;
        let mut previous_frame = self.start_frame;
        for action in &self.actions {
            action.validate()?;
            if action.frame < self.start_frame || action.frame >= self.end_frame {
                return Err(format!(
                    "replay action frame {} is outside [{}, {})",
                    action.frame, self.start_frame, self.end_frame
                ));
            }
            if action.frame < previous_frame {
                return Err("replay actions are not frame-monotonic".into());
            }
            if let Some(sequence) = previous_sequence {
                if action.sequence <= sequence {
                    return Err("replay action sequences are not strictly monotonic".into());
                }
            }
            previous_frame = action.frame;
            previous_sequence = Some(action.sequence);
        }

        let mut checkpoint_frame = None;
        for checkpoint in &self.checkpoints {
            if checkpoint.frame < self.start_frame || checkpoint.frame > self.end_frame {
                return Err("checkpoint frame is outside replay bounds".into());
            }
            if let Some(previous) = checkpoint_frame {
                if checkpoint.frame <= previous {
                    return Err("checkpoint frames are not strictly increasing".into());
                }
            }
            checkpoint_frame = Some(checkpoint.frame);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReplayVerificationResult {
    Pass,
    Diverged,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayVerification {
    pub result: ReplayVerificationResult,
    pub checked_checkpoints: usize,
    pub first_divergence_frame: Option<u64>,
    pub expected_state_sha256: Option<String>,
    pub actual_state_sha256: Option<String>,
    pub expected_frame_sha256: Option<String>,
    pub actual_frame_sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayReceipt {
    pub schema: String,
    pub replay_sha256: String,
    pub game_sha256: String,
    pub core_name: String,
    pub core_version: String,
    pub core_sha256: String,
    pub start_frame: u64,
    pub end_frame: u64,
    pub action_count: usize,
    pub checkpoint_count: usize,
    pub verification: Option<ReplayVerification>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ActionKind, ActionSource};

    fn action(sequence: u64, frame: u64) -> ActionEnvelope {
        ActionEnvelope {
            sequence,
            frame,
            source: ActionSource::Human { seat: 1 },
            action: ActionKind::Button {
                button: "A".into(),
                pressed: sequence % 2 == 0,
            },
        }
    }

    fn ledger() -> ReplayLedger {
        ReplayLedger {
            schema: REPLAY_SCHEMA.into(),
            game_sha256: "a".repeat(64),
            core_name: "SameBoy".into(),
            core_version: "1.0.3".into(),
            core_sha256: "b".repeat(64),
            start_frame: 100,
            end_frame: 104,
            initial_state_base64: "AQID".into(),
            initial_input_mask: 0,
            actions: vec![action(7, 100), action(8, 102)],
            checkpoints: vec![
                ReplayCheckpoint {
                    frame: 100,
                    state_sha256: "c".repeat(64),
                    frame_sha256: "d".repeat(64),
                    input_mask: 0,
                },
                ReplayCheckpoint {
                    frame: 104,
                    state_sha256: "e".repeat(64),
                    frame_sha256: "f".repeat(64),
                    input_mask: 1,
                },
            ],
            final_state_sha256: "e".repeat(64),
            final_frame_sha256: "f".repeat(64),
        }
    }

    #[test]
    fn replay_contract_round_trips_json() {
        let original = ledger();
        original.validate().expect("valid replay");
        let json = serde_json::to_string(&original).expect("serialize");
        let decoded: ReplayLedger = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, original);
    }

    #[test]
    fn rejects_action_outside_bounds() {
        let mut invalid = ledger();
        invalid.actions[0].frame = 104;
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn rejects_non_monotonic_sequences() {
        let mut invalid = ledger();
        invalid.actions[1].sequence = 7;
        assert!(invalid.validate().is_err());
    }
}
