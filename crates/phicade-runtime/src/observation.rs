use serde::{Deserialize, Serialize};

pub const PHIBOT_OBSERVATION_SCHEMA: &str = "phicade.phibot-observation.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhiBotObservation {
    pub schema: String,
    pub frame: u64,
    pub width: u32,
    pub height: u32,
    pub rgba_base64: String,
    pub frame_sha256: String,
    pub input_mask: u16,
    pub game_sha256: String,
    pub core_name: String,
    pub core_version: String,
    pub agent_id: String,
    pub seat: u8,
    pub control_mode: String,
    pub allowed_buttons: Vec<String>,
    pub allowed_axes: Vec<String>,
    pub expires_at_frame: Option<u64>,
}

impl PhiBotObservation {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != PHIBOT_OBSERVATION_SCHEMA {
            return Err(format!("unsupported observation schema: {}", self.schema));
        }
        if self.agent_id.trim().is_empty() {
            return Err("observation requires agentId".into());
        }
        if self.seat == 0 {
            return Err("observation seat numbers start at 1".into());
        }
        if self.width == 0 || self.height == 0 || self.rgba_base64.is_empty() {
            return Err("observation requires a rendered framebuffer".into());
        }
        if self.frame_sha256.len() != 64 || self.game_sha256.len() != 64 {
            return Err("observation hashes must be SHA-256 hex".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observation_contract_rejects_privilege_free_blank_frame() {
        let observation = PhiBotObservation {
            schema: PHIBOT_OBSERVATION_SCHEMA.into(),
            frame: 1,
            width: 0,
            height: 0,
            rgba_base64: String::new(),
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
            expires_at_frame: None,
        };

        assert!(observation.validate().is_err());
    }
}
