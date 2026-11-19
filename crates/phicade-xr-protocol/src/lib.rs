use phicade_runtime::{ActionEnvelope, ActionKind, ActionSource};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};

pub const PROTOCOL: &str = "phicade.xr.v1";
pub const INPUT_TIMEOUT_MS: u64 = 300;
pub const MAX_GRANT_MS: u64 = 30 * 60 * 1000;
const BUTTONS: &[&str] = &[
    "A",
    "B",
    "X",
    "Y",
    "START",
    "SELECT",
    "UP",
    "DOWN",
    "LEFT",
    "RIGHT",
    "L",
    "R",
    "L2",
    "R2",
    "L3",
    "R3",
    "GUN_TRIGGER",
    "GUN_RELOAD",
    "GUN_OFFSCREEN",
];
const AXES: &[&str] = &["LEFT_X", "LEFT_Y", "RIGHT_X", "RIGHT_Y", "GUN_X", "GUN_Y"];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum XrAction {
    Button { button: String, pressed: bool },
    Axis { axis: String, value: i16 },
}
impl XrAction {
    fn validate(&self, lightgun: bool) -> Result<(), String> {
        let (name, valid) = match self {
            Self::Button { button, .. } => (button, BUTTONS.contains(&button.as_str())),
            Self::Axis { axis, .. } => (axis, AXES.contains(&axis.as_str())),
        };
        if !valid || (name.starts_with("GUN_") && !lightgun) {
            return Err("input outside approved XR device profile".into());
        }
        Ok(())
    }
    fn key(&self) -> String {
        match self {
            Self::Button { button, .. } => button.clone(),
            Self::Axis { axis, .. } => axis.clone(),
        }
    }
    fn neutral(&self) -> Self {
        match self {
            Self::Button { button, .. } => Self::Button {
                button: button.clone(),
                pressed: button == "GUN_OFFSCREEN",
            },
            Self::Axis { axis, .. } => Self::Axis {
                axis: axis.clone(),
                value: if axis.starts_with("GUN_") { -32768 } else { 0 },
            },
        }
    }
    fn into_action(self) -> ActionKind {
        match self {
            Self::Button { button, pressed } => ActionKind::Button { button, pressed },
            Self::Axis { axis, value } => ActionKind::Axis { axis, value },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InputPacket {
    pub kind: String,
    pub protocol: String,
    pub sequence: u64,
    pub sent_at: f64,
    pub actions: Vec<XrAction>,
}
#[derive(Debug, Default)]
pub struct XrGateway {
    peer_id: Option<String>,
    token: String,
    lightgun: bool,
    expires_ms: u64,
    last_input_ms: u64,
    last_sequence: u64,
    rate_start_ms: u64,
    rate_count: u16,
    pending: VecDeque<XrAction>,
    held: BTreeMap<String, XrAction>,
    needs_release: bool,
}
impl XrGateway {
    /// Called only from the local operator command; never from network messages.
    pub fn approve(
        &mut self,
        peer_id: String,
        token: String,
        lightgun: bool,
        now_ms: u64,
    ) -> Result<(), String> {
        if peer_id.is_empty()
            || peer_id.len() > 64
            || !peer_id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            || token.len() != 48
            || !token.chars().all(|c| c.is_ascii_hexdigit())
        {
            return Err("invalid XR pairing identity".into());
        }
        self.revoke();
        self.peer_id = Some(peer_id);
        self.token = token;
        self.lightgun = lightgun;
        self.expires_ms = now_ms.saturating_add(MAX_GRANT_MS);
        self.last_input_ms = now_ms;
        self.last_sequence = 0;
        self.rate_start_ms = now_ms;
        self.rate_count = 0;
        Ok(())
    }
    pub fn submit(
        &mut self,
        peer_id: &str,
        token: &str,
        packet: InputPacket,
        now_ms: u64,
    ) -> Result<(), String> {
        if self.peer_id.as_deref() != Some(peer_id)
            || self.token != token
            || now_ms >= self.expires_ms
        {
            return Err("XR grant absent, expired or does not match peer".into());
        }
        if packet.kind != "input"
            || packet.protocol != PROTOCOL
            || !packet.sent_at.is_finite()
            || packet.sequence <= self.last_sequence
            || packet.actions.is_empty()
            || packet.actions.len() > 32
        {
            return Err("invalid or replayed XR packet".into());
        }
        if now_ms.saturating_sub(self.rate_start_ms) >= 1000 {
            self.rate_start_ms = now_ms;
            self.rate_count = 0;
        }
        if self.rate_count >= 180 || self.pending.len() + packet.actions.len() > 256 {
            return Err("XR input rate or queue limit exceeded".into());
        }
        for a in &packet.actions {
            a.validate(self.lightgun)?;
        }
        self.rate_count += 1;
        self.last_sequence = packet.sequence;
        self.last_input_ms = now_ms;
        self.pending.extend(packet.actions);
        Ok(())
    }
    pub fn revoke(&mut self) {
        self.peer_id = None;
        self.token.clear();
        self.pending.clear();
        self.needs_release = true;
    }
    /// A stalled/hidden/disconnected headset cannot leave a button held forever.
    pub fn drain(&mut self, frame: u64, now_ms: u64) -> Vec<ActionEnvelope> {
        if now_ms >= self.expires_ms {
            self.revoke();
        }
        if now_ms.saturating_sub(self.last_input_ms) > INPUT_TIMEOUT_MS {
            self.pending.clear();
            self.needs_release = true;
        }
        let mut actions = if self.needs_release {
            self.needs_release = false;
            let releases = self
                .held
                .values()
                .map(XrAction::neutral)
                .collect::<Vec<_>>();
            self.held.clear();
            releases
        } else {
            Vec::new()
        };
        for a in self.pending.drain(..) {
            self.held.insert(a.key(), a.clone());
            actions.push(a);
        }
        actions
            .into_iter()
            .enumerate()
            .map(|(sequence, a)| ActionEnvelope {
                sequence: sequence as u64,
                frame,
                source: ActionSource::Human { seat: 1 },
                action: a.into_action(),
            })
            .collect()
    }
    pub fn active(&self) -> bool {
        self.peer_id.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn packet(sequence: u64) -> InputPacket {
        InputPacket {
            kind: "input".into(),
            protocol: PROTOCOL.into(),
            sequence,
            sent_at: 12.0,
            actions: vec![XrAction::Button {
                button: "A".into(),
                pressed: true,
            }],
        }
    }
    fn gateway() -> XrGateway {
        let mut g = XrGateway::default();
        g.approve("quest".into(), "a".repeat(48), false, 100)
            .unwrap();
        g.drain(0, 100);
        g
    }
    #[test]
    fn unauthenticated_and_replayed_input_is_rejected() {
        let mut g = gateway();
        assert!(g.submit("other", &"a".repeat(48), packet(1), 110).is_err());
        assert!(g.submit("quest", &"b".repeat(48), packet(1), 110).is_err());
        g.submit("quest", &"a".repeat(48), packet(1), 110).unwrap();
        assert!(g.submit("quest", &"a".repeat(48), packet(1), 110).is_err());
    }
    #[test]
    fn canonical_human_source_still_passes_through_authority() {
        let mut g = gateway();
        g.submit("quest", &"a".repeat(48), packet(1), 110).unwrap();
        let actions = g.drain(99, 111);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].frame, 99);
        let mut policy = phicade_runtime::AuthorityPolicy::new(1);
        policy
            .set_mode(
                phicade_runtime::ControlMode::PhiBot,
                Some(phicade_runtime::AgentGrant::game_boy("bot", 1)),
            )
            .unwrap();
        assert!(!policy.authorize_batch(&actions, 99)[0].1.accepted);
    }
    #[test]
    fn disconnect_and_timeout_release_held_inputs() {
        let mut g = gateway();
        g.submit("quest", &"a".repeat(48), packet(1), 110).unwrap();
        g.drain(1, 111);
        let actions = g.drain(2, 411);
        assert!(matches!(
            actions[0].action,
            ActionKind::Button { pressed: false, .. }
        ));
        assert!(g.drain(3, 412).is_empty());
        g.approve("quest".into(), "a".repeat(48), true, 500)
            .unwrap();
        let mut p = packet(1);
        p.actions = vec![XrAction::Button {
            button: "GUN_OFFSCREEN".into(),
            pressed: false,
        }];
        g.submit("quest", &"a".repeat(48), p, 500).unwrap();
        g.drain(4, 500);
        assert!(g.drain(5,801).iter().any(|e|matches!(&e.action,ActionKind::Button{button,pressed:true} if button=="GUN_OFFSCREEN")));
    }
    #[test]
    fn expires_and_does_not_accept_system_or_source_spoofing() {
        let mut g = gateway();
        assert!(g
            .submit("quest", &"a".repeat(48), packet(1), MAX_GRANT_MS + 100)
            .is_err());
        for wire in [
            r#"{"kind":"input","protocol":"phicade.xr.v1","sequence":1,"sentAt":1,"actions":[{"kind":"system","command":"reset"}]}"#,
            r#"{"kind":"input","protocol":"phicade.xr.v1","sequence":1,"sentAt":1,"source":{"kind":"replay"},"actions":[]}"#,
        ] {
            assert!(serde_json::from_str::<InputPacket>(wire).is_err());
        }
    }
    #[test]
    fn device_and_packet_limits_are_enforced() {
        let mut g = gateway();
        let mut p = packet(1);
        p.actions = vec![XrAction::Axis {
            axis: "GUN_X".into(),
            value: 0,
        }];
        assert!(g.submit("quest", &"a".repeat(48), p, 110).is_err());
        let mut p = packet(2);
        p.actions = vec![
            XrAction::Button {
                button: "A".into(),
                pressed: true
            };
            33
        ];
        assert!(g.submit("quest", &"a".repeat(48), p, 110).is_err());
    }
    #[test]
    fn revoked_grant_cannot_reenter() {
        let mut g = gateway();
        g.revoke();
        assert!(g.submit("quest", &"a".repeat(48), packet(1), 110).is_err());
    }
}
