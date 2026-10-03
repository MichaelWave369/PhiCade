use serde::{Deserialize, Serialize};

pub const AUTODRIVE_STATUS_SCHEMA: &str = "phicade.autodrive-status.v1";
pub const AUTODRIVE_RECEIPT_SCHEMA: &str = "phicade.autodrive-receipt.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AutodrivePolicy {
    pub max_turns: u16,
    pub max_total_actions: u32,
    pub max_consecutive_empty_turns: u8,
    pub max_emulated_frames: u64,
}

impl Default for AutodrivePolicy {
    fn default() -> Self {
        Self {
            max_turns: 32,
            max_total_actions: 128,
            max_consecutive_empty_turns: 4,
            max_emulated_frames: 3_600,
        }
    }
}

impl AutodrivePolicy {
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=500).contains(&self.max_turns) {
            return Err("maxTurns must be in 1..=500".into());
        }
        if !(1..=4096).contains(&self.max_total_actions) {
            return Err("maxTotalActions must be in 1..=4096".into());
        }
        if !(1..=20).contains(&self.max_consecutive_empty_turns) {
            return Err("maxConsecutiveEmptyTurns must be in 1..=20".into());
        }
        if !(60..=216_000).contains(&self.max_emulated_frames) {
            return Err("maxEmulatedFrames must be in 60..=216000".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AutodriveStopReason {
    OperatorStop,
    HumanTakeover,
    TurnBudget,
    ActionBudget,
    FrameBudget,
    EmptyTurnLimit,
    ProviderFailure,
    GrantExpired,
    CoreShutdown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutodriveStatus {
    pub schema: String,
    pub run_id: u64,
    pub active: bool,
    pub provider: String,
    pub model: String,
    pub started_frame: u64,
    pub current_frame: u64,
    pub turns_issued: u16,
    pub turns_completed: u16,
    pub total_actions: u32,
    pub consecutive_empty_turns: u8,
    pub policy: AutodrivePolicy,
    pub stop_reason: Option<AutodriveStopReason>,
}

impl AutodriveStatus {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != AUTODRIVE_STATUS_SCHEMA {
            return Err(format!("unsupported autodrive status schema: {}", self.schema));
        }
        self.policy.validate()?;
        if self.provider.trim().is_empty() || self.model.trim().is_empty() {
            return Err("autodrive provider and model must be non-empty".into());
        }
        if self.current_frame < self.started_frame {
            return Err("autodrive current frame precedes started frame".into());
        }
        Ok(())
    }

    pub fn frame_span(&self) -> u64 {
        self.current_frame.saturating_sub(self.started_frame)
    }

    pub fn pre_turn_stop_reason(&self) -> Option<AutodriveStopReason> {
        if self.frame_span() >= self.policy.max_emulated_frames {
            return Some(AutodriveStopReason::FrameBudget);
        }
        if self.turns_issued >= self.policy.max_turns {
            return Some(AutodriveStopReason::TurnBudget);
        }
        if self.total_actions >= self.policy.max_total_actions {
            return Some(AutodriveStopReason::ActionBudget);
        }
        None
    }

    pub fn note_turn_issued(&mut self) {
        self.turns_issued = self.turns_issued.saturating_add(1);
    }

    pub fn can_accept_actions(&self, action_count: usize) -> bool {
        self.total_actions
            .saturating_add(u32::try_from(action_count).unwrap_or(u32::MAX))
            <= self.policy.max_total_actions
    }

    pub fn note_turn_completed(&mut self, action_count: usize) {
        self.turns_completed = self.turns_completed.saturating_add(1);
        self.total_actions = self
            .total_actions
            .saturating_add(u32::try_from(action_count).unwrap_or(u32::MAX));
        if action_count == 0 {
            self.consecutive_empty_turns = self.consecutive_empty_turns.saturating_add(1);
        } else {
            self.consecutive_empty_turns = 0;
        }
    }

    pub fn post_turn_stop_reason(&self) -> Option<AutodriveStopReason> {
        if self.consecutive_empty_turns >= self.policy.max_consecutive_empty_turns {
            return Some(AutodriveStopReason::EmptyTurnLimit);
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutodriveReceipt {
    pub schema: String,
    pub run_id: u64,
    pub provider: String,
    pub model: String,
    pub game_sha256: String,
    pub core_name: String,
    pub core_version: String,
    pub started_frame: u64,
    pub ended_frame: u64,
    pub turns_issued: u16,
    pub turns_completed: u16,
    pub total_actions: u32,
    pub stop_reason: AutodriveStopReason,
    pub final_frame_sha256: String,
    pub policy: AutodrivePolicy,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_policy_is_bounded_and_valid() {
        AutodrivePolicy::default().validate().expect("valid");
    }

    #[test]
    fn rejects_unbounded_turn_budget() {
        let policy = AutodrivePolicy {
            max_turns: 0,
            ..AutodrivePolicy::default()
        };
        assert!(policy.validate().is_err());
    }

    #[test]
    fn turn_budget_stops_before_issuing_one_too_many() {
        let mut status = AutodriveStatus {
            schema: AUTODRIVE_STATUS_SCHEMA.into(),
            run_id: 1,
            active: true,
            provider: "ollama".into(),
            model: "vision".into(),
            started_frame: 100,
            current_frame: 100,
            turns_issued: 0,
            turns_completed: 0,
            total_actions: 0,
            consecutive_empty_turns: 0,
            policy: AutodrivePolicy {
                max_turns: 2,
                ..AutodrivePolicy::default()
            },
            stop_reason: None,
        };
        assert_eq!(status.pre_turn_stop_reason(), None);
        status.note_turn_issued();
        status.note_turn_issued();
        assert_eq!(
            status.pre_turn_stop_reason(),
            Some(AutodriveStopReason::TurnBudget)
        );
    }

    #[test]
    fn empty_turn_limit_is_explicit() {
        let mut status = AutodriveStatus {
            schema: AUTODRIVE_STATUS_SCHEMA.into(),
            run_id: 1,
            active: true,
            provider: "ollama".into(),
            model: "vision".into(),
            started_frame: 100,
            current_frame: 100,
            turns_issued: 2,
            turns_completed: 0,
            total_actions: 0,
            consecutive_empty_turns: 0,
            policy: AutodrivePolicy {
                max_consecutive_empty_turns: 2,
                ..AutodrivePolicy::default()
            },
            stop_reason: None,
        };
        status.note_turn_completed(0);
        assert_eq!(status.post_turn_stop_reason(), None);
        status.note_turn_completed(0);
        assert_eq!(
            status.post_turn_stop_reason(),
            Some(AutodriveStopReason::EmptyTurnLimit)
        );
    }

    #[test]
    fn action_budget_rejects_oversized_completion() {
        let mut status = AutodriveStatus {
            schema: AUTODRIVE_STATUS_SCHEMA.into(),
            run_id: 1,
            active: true,
            provider: "ollama".into(),
            model: "vision".into(),
            started_frame: 100,
            current_frame: 100,
            turns_issued: 1,
            turns_completed: 0,
            total_actions: 3,
            consecutive_empty_turns: 0,
            policy: AutodrivePolicy {
                max_total_actions: 4,
                ..AutodrivePolicy::default()
            },
            stop_reason: None,
        };
        assert!(!status.can_accept_actions(2));
        assert!(status.can_accept_actions(1));
        status.note_turn_completed(1);
        assert_eq!(
            status.pre_turn_stop_reason(),
            Some(AutodriveStopReason::ActionBudget)
        );
    }

    #[test]
    fn status_reports_frame_span() {
        let status = AutodriveStatus {
            schema: AUTODRIVE_STATUS_SCHEMA.into(),
            run_id: 1,
            active: true,
            provider: "ollama".into(),
            model: "vision".into(),
            started_frame: 100,
            current_frame: 160,
            turns_issued: 1,
            turns_completed: 1,
            total_actions: 2,
            consecutive_empty_turns: 0,
            policy: AutodrivePolicy::default(),
            stop_reason: None,
        };
        assert_eq!(status.frame_span(), 60);
    }
}
