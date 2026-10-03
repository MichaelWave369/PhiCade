use crate::{ActionEnvelope, ActionKind, ActionSource, PhiBotObservation};
use serde::{Deserialize, Serialize};

pub const AGENT_TURN_REQUEST_SCHEMA: &str = "phicade.agent-turn-request.v1";
pub const AGENT_TURN_RESPONSE_SCHEMA: &str = "phicade.agent-turn-response.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentTurnAction {
    pub delay_frames: u16,
    pub action: ActionKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentTurnRequest {
    pub schema: String,
    pub turn_id: u64,
    pub observation: PhiBotObservation,
    pub max_actions: u8,
    pub max_delay_frames: u16,
    pub valid_until_frame: u64,
}

impl AgentTurnRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != AGENT_TURN_REQUEST_SCHEMA {
            return Err(format!("unsupported agent turn request schema: {}", self.schema));
        }
        self.observation.validate()?;
        if self.max_actions == 0 {
            return Err("agent turn maxActions must be greater than zero".into());
        }
        if self.max_delay_frames == 0 {
            return Err("agent turn maxDelayFrames must be greater than zero".into());
        }
        if self.valid_until_frame < self.observation.frame {
            return Err("agent turn expires before its observation frame".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentTurnResponse {
    pub schema: String,
    pub turn_id: u64,
    pub agent_id: String,
    pub seat: u8,
    pub observation_frame: u64,
    pub observation_sha256: String,
    pub actions: Vec<AgentTurnAction>,
}

impl AgentTurnResponse {
    pub fn validate_against(
        &self,
        request: &AgentTurnRequest,
        current_frame: u64,
    ) -> Result<(), String> {
        request.validate()?;

        if self.schema != AGENT_TURN_RESPONSE_SCHEMA {
            return Err(format!("unsupported agent turn response schema: {}", self.schema));
        }
        if self.turn_id != request.turn_id {
            return Err("agent turn response turnId does not match pending request".into());
        }
        if self.agent_id != request.observation.agent_id {
            return Err("agent turn response agentId does not match observation grant".into());
        }
        if self.seat != request.observation.seat {
            return Err("agent turn response seat does not match observation grant".into());
        }
        if self.observation_frame != request.observation.frame {
            return Err("agent turn response observationFrame does not match request".into());
        }
        if self.observation_sha256 != request.observation.frame_sha256 {
            return Err("agent turn response observation hash does not match request".into());
        }
        if current_frame > request.valid_until_frame {
            return Err("agent turn response arrived after request expiry".into());
        }
        if self.actions.len() > usize::from(request.max_actions) {
            return Err("agent turn response exceeds maxActions".into());
        }

        for action in &self.actions {
            if action.delay_frames > request.max_delay_frames {
                return Err("agent turn action exceeds maxDelayFrames".into());
            }
        }

        Ok(())
    }
}

pub fn compile_agent_turn(
    request: &AgentTurnRequest,
    response: &AgentTurnResponse,
    apply_frame: u64,
) -> Result<Vec<ActionEnvelope>, String> {
    response.validate_against(request, apply_frame)?;

    Ok(response
        .actions
        .iter()
        .map(|intent| ActionEnvelope {
            sequence: 0,
            frame: apply_frame.saturating_add(u64::from(intent.delay_frames)),
            source: ActionSource::PhiBot {
                agent_id: response.agent_id.clone(),
                seat: response.seat,
            },
            action: intent.action.clone(),
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PHIBOT_OBSERVATION_SCHEMA;

    fn observation() -> PhiBotObservation {
        PhiBotObservation {
            schema: PHIBOT_OBSERVATION_SCHEMA.into(),
            frame: 100,
            width: 160,
            height: 144,
            rgba_base64: "AQID".into(),
            frame_sha256: "a".repeat(64),
            input_mask: 0,
            game_sha256: "b".repeat(64),
            core_name: "SameBoy".into(),
            core_version: "1.0.3".into(),
            agent_id: "phi".into(),
            seat: 1,
            control_mode: "phi-bot".into(),
            allowed_buttons: vec!["A".into()],
            allowed_axes: Vec::new(),
            expires_at_frame: Some(500),
        }
    }

    fn request() -> AgentTurnRequest {
        AgentTurnRequest {
            schema: AGENT_TURN_REQUEST_SCHEMA.into(),
            turn_id: 7,
            observation: observation(),
            max_actions: 4,
            max_delay_frames: 8,
            valid_until_frame: 120,
        }
    }

    fn response() -> AgentTurnResponse {
        AgentTurnResponse {
            schema: AGENT_TURN_RESPONSE_SCHEMA.into(),
            turn_id: 7,
            agent_id: "phi".into(),
            seat: 1,
            observation_frame: 100,
            observation_sha256: "a".repeat(64),
            actions: vec![
                AgentTurnAction {
                    delay_frames: 0,
                    action: ActionKind::Button {
                        button: "A".into(),
                        pressed: true,
                    },
                },
                AgentTurnAction {
                    delay_frames: 1,
                    action: ActionKind::Button {
                        button: "A".into(),
                        pressed: false,
                    },
                },
            ],
        }
    }

    #[test]
    fn compiles_response_into_seat_bound_action_envelopes() {
        let actions = compile_agent_turn(&request(), &response(), 105).expect("compile");
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].frame, 105);
        assert_eq!(actions[1].frame, 106);
        assert!(matches!(
            &actions[0].source,
            ActionSource::PhiBot { agent_id, seat } if agent_id == "phi" && *seat == 1
        ));
    }

    #[test]
    fn rejects_stale_response() {
        assert!(response().validate_against(&request(), 121).is_err());
    }

    #[test]
    fn rejects_wrong_observation_hash() {
        let mut response = response();
        response.observation_sha256 = "c".repeat(64);
        assert!(response.validate_against(&request(), 105).is_err());
    }

    #[test]
    fn rejects_action_burst_over_budget() {
        let mut response = response();
        response.actions = (0..5)
            .map(|_| AgentTurnAction {
                delay_frames: 0,
                action: ActionKind::Button {
                    button: "A".into(),
                    pressed: true,
                },
            })
            .collect();
        assert!(response.validate_against(&request(), 105).is_err());
    }

    #[test]
    fn rejects_excessive_delayed_action() {
        let mut response = response();
        response.actions[0].delay_frames = 9;
        assert!(response.validate_against(&request(), 105).is_err());
    }
}
