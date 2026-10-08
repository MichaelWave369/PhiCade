use crate::{
    ActionEnvelope, ActionKind, ActionSource, AgentTurnAction, SparkSemanticObservation,
    SPARK_SEMANTIC_OBSERVATION_SCHEMA,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const SPARK_AGENT_TURN_REQUEST_SCHEMA: &str = "phicade.spark-agent-turn-request.v1";
pub const SPARK_AGENT_TURN_RESPONSE_SCHEMA: &str = "phicade.spark-agent-turn-response.v1";
pub const SPARK_AGENT_MEMORY_MAX_BYTES: u32 = 8_192;
pub const SPARK_AGENT_OBJECTIVE_MAX_BYTES: usize = 256;

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SparkAgentTurnRequest {
    pub schema: String,
    pub turn_id: u64,
    pub agent_id: String,
    pub seat: u8,
    pub observation: SparkSemanticObservation,
    pub observation_runtime_hash: String,
    pub allowed_buttons: Vec<String>,
    pub max_actions: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub objective: Option<String>,
    pub memory: String,
    pub memory_sha256: String,
    pub max_memory_bytes: u32,
    pub max_memory_update_bytes: u32,
}

impl SparkAgentTurnRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != SPARK_AGENT_TURN_REQUEST_SCHEMA {
            return Err(format!(
                "unsupported SPARK agent turn request schema: {}",
                self.schema
            ));
        }
        if self.agent_id.trim().is_empty() {
            return Err("SPARK agent turn requires agentId".into());
        }
        if self.seat == 0 {
            return Err("SPARK agent seat numbers start at 1".into());
        }
        if self.observation.schema != SPARK_SEMANTIC_OBSERVATION_SCHEMA {
            return Err(format!(
                "SPARK agent turn requires {}, got {}",
                SPARK_SEMANTIC_OBSERVATION_SCHEMA, self.observation.schema
            ));
        }
        if !is_sha256_hex(&self.observation_runtime_hash) {
            return Err(
                "SPARK agent turn observationRuntimeHash must be a 64-character hex digest".into(),
            );
        }
        if self.allowed_buttons.is_empty() {
            return Err("SPARK agent turn requires at least one allowed button".into());
        }
        let mut unique = BTreeSet::new();
        for button in &self.allowed_buttons {
            if button.trim().is_empty() {
                return Err("SPARK allowed buttons must be non-empty".into());
            }
            if !unique.insert(button) {
                return Err("SPARK allowed buttons must not contain duplicates".into());
            }
        }
        if self.max_actions != 1 {
            return Err("SPARK semantic driver currently requires maxActions=1".into());
        }
        if let Some(objective) = &self.objective {
            let objective = objective.trim();
            if objective.is_empty() {
                return Err("SPARK objective must be non-empty when provided".into());
            }
            if objective.as_bytes().len() > SPARK_AGENT_OBJECTIVE_MAX_BYTES {
                return Err(format!(
                    "SPARK objective exceeds {SPARK_AGENT_OBJECTIVE_MAX_BYTES} bytes"
                ));
            }
        }
        if !(1..=SPARK_AGENT_MEMORY_MAX_BYTES).contains(&self.max_memory_bytes) {
            return Err(format!(
                "SPARK maxMemoryBytes must be in 1..={SPARK_AGENT_MEMORY_MAX_BYTES}"
            ));
        }
        if self.max_memory_update_bytes == 0
            || self.max_memory_update_bytes > self.max_memory_bytes
        {
            return Err(
                "SPARK maxMemoryUpdateBytes must be in 1..=maxMemoryBytes".into(),
            );
        }
        if self.memory.len() > self.max_memory_bytes as usize {
            return Err("SPARK agent memory exceeds maxMemoryBytes".into());
        }
        if !is_sha256_hex(&self.memory_sha256) {
            return Err("SPARK memorySha256 must be a 64-character hex digest".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SparkAgentTurnResponse {
    pub schema: String,
    pub turn_id: u64,
    pub agent_id: String,
    pub seat: u8,
    pub observation_tick: u64,
    pub observation_runtime_hash: String,
    pub memory_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_update: Option<String>,
    pub actions: Vec<AgentTurnAction>,
}

impl SparkAgentTurnResponse {
    pub fn validate_against(
        &self,
        request: &SparkAgentTurnRequest,
        current_tick: u64,
    ) -> Result<(), String> {
        request.validate()?;

        if self.schema != SPARK_AGENT_TURN_RESPONSE_SCHEMA {
            return Err(format!(
                "unsupported SPARK agent turn response schema: {}",
                self.schema
            ));
        }
        if self.turn_id != request.turn_id {
            return Err("SPARK response turnId does not match request".into());
        }
        if self.agent_id != request.agent_id {
            return Err("SPARK response agentId does not match request".into());
        }
        if self.seat != request.seat {
            return Err("SPARK response seat does not match request".into());
        }
        if self.observation_tick != request.observation.tick {
            return Err("SPARK response observationTick does not match request".into());
        }
        if self.observation_runtime_hash != request.observation_runtime_hash {
            return Err(
                "SPARK response observationRuntimeHash does not match request".into(),
            );
        }
        if self.memory_sha256 != request.memory_sha256 {
            return Err("SPARK response memorySha256 does not match request".into());
        }
        if current_tick != request.observation.tick {
            return Err(
                "SPARK semantic response may only be applied to its observed tick".into(),
            );
        }
        if self.actions.len() > usize::from(request.max_actions) {
            return Err("SPARK response exceeds maxActions".into());
        }
        if let Some(memory_update) = &self.memory_update {
            if memory_update.len() > request.max_memory_update_bytes as usize {
                return Err("SPARK memoryUpdate exceeds maxMemoryUpdateBytes".into());
            }
            if memory_update.len() > request.max_memory_bytes as usize {
                return Err("SPARK memoryUpdate exceeds maxMemoryBytes".into());
            }
        }

        for action in &self.actions {
            if action.delay_frames != 0 {
                return Err("SPARK semantic actions must apply at delayFrames=0".into());
            }
            match &action.action {
                ActionKind::Button { button, pressed } => {
                    if !pressed {
                        return Err("SPARK semantic actions must be pressed button actions".into());
                    }
                    if !request.allowed_buttons.iter().any(|allowed| allowed == button) {
                        return Err(format!(
                            "SPARK response button {button} is outside the request grant"
                        ));
                    }
                }
                _ => {
                    return Err(
                        "SPARK semantic driver currently accepts button actions only".into(),
                    )
                }
            }
        }

        Ok(())
    }
}

pub fn compile_spark_agent_turn(
    request: &SparkAgentTurnRequest,
    response: &SparkAgentTurnResponse,
    current_tick: u64,
) -> Result<Vec<ActionEnvelope>, String> {
    response.validate_against(request, current_tick)?;

    Ok(response
        .actions
        .iter()
        .map(|action| ActionEnvelope {
            sequence: 0,
            frame: current_tick,
            source: ActionSource::PhiBot {
                agent_id: response.agent_id.clone(),
                seat: response.seat,
            },
            action: action.action.clone(),
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        SparkSemanticPlayer, SPARK_SEMANTIC_OBSERVATION_SCHEMA,
    };

    fn observation() -> SparkSemanticObservation {
        SparkSemanticObservation {
            schema: SPARK_SEMANTIC_OBSERVATION_SCHEMA.into(),
            tick: 3,
            room: "threshold".into(),
            world: "threshold".into(),
            phase: "playing".into(),
            form: "spark".into(),
            player: SparkSemanticPlayer {
                x: 480.0,
                y: 390.0,
                hp: 112.0,
                max_hp: 112.0,
                dash_ready: true,
            },
            power_name: "Lumen pulse".into(),
            power_cooldown: 0.0,
            enemy_count: 0,
            nearest_enemy: None,
            fragment_count: 0,
            exit_directions: vec!["north".into()],
            allowed_actions: vec!["MOVE".into(), "DASH".into(), "PULSE".into()],
            dashes: 0,
            powers: 0,
        }
    }

    fn request() -> SparkAgentTurnRequest {
        SparkAgentTurnRequest {
            schema: SPARK_AGENT_TURN_REQUEST_SCHEMA.into(),
            turn_id: 9,
            agent_id: "phi-spark".into(),
            seat: 1,
            observation: observation(),
            observation_runtime_hash: "a".repeat(64),
            allowed_buttons: vec![
                "UP".into(),
                "DOWN".into(),
                "LEFT".into(),
                "RIGHT".into(),
                "DASH_RIGHT".into(),
                "PULSE".into(),
            ],
            max_actions: 1,
            objective: None,
            memory: "entered threshold".into(),
            memory_sha256: "b".repeat(64),
            max_memory_bytes: 4096,
            max_memory_update_bytes: 1024,
        }
    }

    fn response(button: &str) -> SparkAgentTurnResponse {
        SparkAgentTurnResponse {
            schema: SPARK_AGENT_TURN_RESPONSE_SCHEMA.into(),
            turn_id: 9,
            agent_id: "phi-spark".into(),
            seat: 1,
            observation_tick: 3,
            observation_runtime_hash: "a".repeat(64),
            memory_sha256: "b".repeat(64),
            memory_update: Some("moved east".into()),
            actions: vec![AgentTurnAction {
                delay_frames: 0,
                action: ActionKind::Button {
                    button: button.into(),
                    pressed: true,
                },
            }],
        }
    }

    #[test]
    fn compiles_bound_semantic_turn_into_phi_bot_action() {
        let request = request();
        let response = response("RIGHT");
        let actions = compile_spark_agent_turn(&request, &response, 3).unwrap();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].frame, 3);
        assert!(matches!(
            &actions[0].source,
            ActionSource::PhiBot { agent_id, seat }
                if agent_id == "phi-spark" && *seat == 1
        ));
    }

    #[test]
    fn validates_bounded_explicit_objective() {
        let mut request = request();
        request.objective = Some("Move east while staying within the granted controls.".into());
        assert!(request.validate().is_ok());

        request.objective = Some(" ".into());
        assert!(request.validate().is_err());

        request.objective = Some("x".repeat(SPARK_AGENT_OBJECTIVE_MAX_BYTES + 1));
        assert!(request.validate().is_err());
    }

    #[test]
    fn rejects_out_of_scope_button() {
        assert!(response("START").validate_against(&request(), 3).is_err());
    }

    #[test]
    fn rejects_stale_semantic_response() {
        assert!(response("RIGHT").validate_against(&request(), 4).is_err());
    }

    #[test]
    fn rejects_multiple_actions_for_one_tick() {
        let mut response = response("RIGHT");
        response.actions.push(AgentTurnAction {
            delay_frames: 0,
            action: ActionKind::Button {
                button: "PULSE".into(),
                pressed: true,
            },
        });
        assert!(response.validate_against(&request(), 3).is_err());
    }
}
