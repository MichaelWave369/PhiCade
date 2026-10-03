use crate::{ActionEnvelope, ActionKind, ActionSource, SystemCommand};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ControlMode {
    Human,
    PhiBot,
    Coop,
    Versus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentGrant {
    pub agent_id: String,
    pub seat: u8,
    pub allowed_buttons: BTreeSet<String>,
    pub allowed_axes: BTreeSet<String>,
    pub allowed_system_commands: BTreeSet<SystemCommand>,
    pub max_actions_per_frame: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at_frame: Option<u64>,
}

impl AgentGrant {
    pub fn game_boy(agent_id: impl Into<String>, seat: u8) -> Self {
        Self {
            agent_id: agent_id.into(),
            seat,
            allowed_buttons: ["A", "B", "SELECT", "START", "UP", "DOWN", "LEFT", "RIGHT"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            allowed_axes: BTreeSet::new(),
            allowed_system_commands: BTreeSet::new(),
            max_actions_per_frame: 16,
            expires_at_frame: None,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.agent_id.trim().is_empty() {
            return Err("agent grant requires a non-empty agentId".into());
        }
        if self.seat == 0 {
            return Err("agent seat numbers start at 1".into());
        }
        if self.max_actions_per_frame == 0 {
            return Err("maxActionsPerFrame must be greater than zero".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorityDecision {
    pub accepted: bool,
    pub reason: String,
}

impl AuthorityDecision {
    pub fn accept(reason: impl Into<String>) -> Self {
        Self {
            accepted: true,
            reason: reason.into(),
        }
    }

    pub fn reject(reason: impl Into<String>) -> Self {
        Self {
            accepted: false,
            reason: reason.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorityPolicy {
    pub mode: ControlMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_grant: Option<AgentGrant>,
    pub playable_ports: u8,
}

impl Default for AuthorityPolicy {
    fn default() -> Self {
        Self {
            mode: ControlMode::Human,
            agent_grant: None,
            playable_ports: 1,
        }
    }
}

impl AuthorityPolicy {
    pub fn new(playable_ports: u8) -> Self {
        Self {
            playable_ports: playable_ports.max(1),
            ..Self::default()
        }
    }

    pub fn set_mode(
        &mut self,
        mode: ControlMode,
        grant: Option<AgentGrant>,
    ) -> Result<(), String> {
        if mode == ControlMode::Versus && self.playable_ports < 2 {
            return Err(format!(
                "versus mode requires at least 2 playable ports; core exposes {}",
                self.playable_ports
            ));
        }

        match mode {
            ControlMode::Human => {
                self.mode = mode;
                self.agent_grant = None;
            }
            ControlMode::PhiBot | ControlMode::Coop => {
                let grant = grant.ok_or_else(|| {
                    format!("{mode:?} mode requires an explicit agent grant")
                })?;
                grant.validate()?;
                if grant.seat != 1 {
                    return Err(format!("{mode:?} on the current topology requires seat 1"));
                }
                self.mode = mode;
                self.agent_grant = Some(grant);
            }
            ControlMode::Versus => {
                let grant = grant.ok_or_else(|| "versus mode requires an explicit agent grant".to_owned())?;
                grant.validate()?;
                if grant.seat != 2 {
                    return Err("versus mode assigns the Phi-Bot to seat 2".into());
                }
                self.mode = mode;
                self.agent_grant = Some(grant);
            }
        }

        Ok(())
    }

    pub fn authorize_batch(
        &self,
        actions: &[ActionEnvelope],
        frame: u64,
    ) -> Vec<(ActionEnvelope, AuthorityDecision)> {
        let mut agent_counts: BTreeMap<(String, u8), u8> = BTreeMap::new();

        actions
            .iter()
            .cloned()
            .map(|action| {
                let decision = self.authorize_one(&action, frame, &mut agent_counts);
                (action, decision)
            })
            .collect()
    }

    fn authorize_one(
        &self,
        envelope: &ActionEnvelope,
        frame: u64,
        agent_counts: &mut BTreeMap<(String, u8), u8>,
    ) -> AuthorityDecision {
        if let Err(error) = envelope.validate() {
            return AuthorityDecision::reject(error);
        }

        match &envelope.source {
            ActionSource::Human { seat } => self.authorize_human(*seat, &envelope.action),
            ActionSource::PhiBot { agent_id, seat } => {
                self.authorize_agent(agent_id, *seat, &envelope.action, frame, agent_counts)
            }
            ActionSource::Replay => AuthorityDecision::accept("replay verifier source"),
            ActionSource::Script { .. } => {
                AuthorityDecision::reject("live script authority is not enabled in Rung 6")
            }
        }
    }

    fn authorize_human(&self, seat: u8, action: &ActionKind) -> AuthorityDecision {
        if seat == 0 || seat > self.playable_ports {
            return AuthorityDecision::reject(format!(
                "human seat {seat} is outside {} playable port(s)",
                self.playable_ports
            ));
        }

        if seat == 1 && matches!(action, ActionKind::System { .. }) {
            return AuthorityDecision::accept("human operator system authority");
        }

        let accepted = match self.mode {
            ControlMode::Human | ControlMode::Coop => seat == 1,
            ControlMode::PhiBot => false,
            ControlMode::Versus => seat == 1,
        };

        if accepted {
            AuthorityDecision::accept(format!("human seat {seat} authorized in {:?}", self.mode))
        } else {
            AuthorityDecision::reject(format!("human seat {seat} not authorized in {:?}", self.mode))
        }
    }

    fn authorize_agent(
        &self,
        agent_id: &str,
        seat: u8,
        action: &ActionKind,
        frame: u64,
        agent_counts: &mut BTreeMap<(String, u8), u8>,
    ) -> AuthorityDecision {
        let mode_accepts_agent = match self.mode {
            ControlMode::PhiBot | ControlMode::Coop => seat == 1,
            ControlMode::Versus => seat == 2,
            ControlMode::Human => false,
        };
        if !mode_accepts_agent {
            return AuthorityDecision::reject(format!(
                "Phi-Bot seat {seat} not authorized in {:?}",
                self.mode
            ));
        }

        let Some(grant) = self.agent_grant.as_ref() else {
            return AuthorityDecision::reject("Phi-Bot action has no active grant");
        };

        if grant.agent_id != agent_id {
            return AuthorityDecision::reject("Phi-Bot agentId does not match active grant");
        }
        if grant.seat != seat {
            return AuthorityDecision::reject("Phi-Bot seat does not match active grant");
        }
        if grant.expires_at_frame.is_some_and(|expires| frame > expires) {
            return AuthorityDecision::reject("Phi-Bot grant has expired");
        }

        let key = (agent_id.to_owned(), seat);
        let count = agent_counts.entry(key).or_insert(0);
        if *count >= grant.max_actions_per_frame {
            return AuthorityDecision::reject("Phi-Bot per-frame action limit exceeded");
        }

        let scoped = match action {
            ActionKind::Button { button, .. } => grant.allowed_buttons.contains(button),
            ActionKind::Axis { axis, .. } => grant.allowed_axes.contains(axis),
            ActionKind::System { command, .. } => grant.allowed_system_commands.contains(command),
        };
        if !scoped {
            return AuthorityDecision::reject("Phi-Bot action is outside the active grant scope");
        }

        *count += 1;
        AuthorityDecision::accept("Phi-Bot action accepted by active grant")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn button(source: ActionSource, frame: u64, name: &str) -> ActionEnvelope {
        ActionEnvelope {
            sequence: frame,
            frame,
            source,
            action: ActionKind::Button {
                button: name.into(),
                pressed: true,
            },
        }
    }

    #[test]
    fn human_mode_rejects_phi_bot() {
        let policy = AuthorityPolicy::new(1);
        let event = button(
            ActionSource::PhiBot {
                agent_id: "phi".into(),
                seat: 1,
            },
            10,
            "A",
        );
        assert!(!policy.authorize_batch(&[event], 10)[0].1.accepted);
    }

    #[test]
    fn handoff_rejects_human_and_accepts_granted_agent() {
        let mut policy = AuthorityPolicy::new(1);
        policy
            .set_mode(ControlMode::PhiBot, Some(AgentGrant::game_boy("phi", 1)))
            .expect("handoff");

        let human = button(ActionSource::Human { seat: 1 }, 10, "A");
        let agent = button(
            ActionSource::PhiBot {
                agent_id: "phi".into(),
                seat: 1,
            },
            10,
            "A",
        );
        let decisions = policy.authorize_batch(&[human, agent], 10);
        assert!(!decisions[0].1.accepted);
        assert!(decisions[1].1.accepted);
    }

    #[test]
    fn coop_accepts_human_and_agent_on_same_seat() {
        let mut policy = AuthorityPolicy::new(1);
        policy
            .set_mode(ControlMode::Coop, Some(AgentGrant::game_boy("phi", 1)))
            .expect("coop");

        let human = button(ActionSource::Human { seat: 1 }, 20, "LEFT");
        let agent = button(
            ActionSource::PhiBot {
                agent_id: "phi".into(),
                seat: 1,
            },
            20,
            "A",
        );
        let decisions = policy.authorize_batch(&[human, agent], 20);
        assert!(decisions.iter().all(|(_, decision)| decision.accepted));
    }

    #[test]
    fn grant_scope_rejects_system_privilege() {
        let mut policy = AuthorityPolicy::new(1);
        policy
            .set_mode(ControlMode::PhiBot, Some(AgentGrant::game_boy("phi", 1)))
            .expect("handoff");

        let reset = ActionEnvelope {
            sequence: 1,
            frame: 1,
            source: ActionSource::PhiBot {
                agent_id: "phi".into(),
                seat: 1,
            },
            action: ActionKind::System {
                command: SystemCommand::Reset,
                slot: None,
            },
        };

        assert!(!policy.authorize_batch(&[reset], 1)[0].1.accepted);
    }

    #[test]
    fn expired_grant_rejects_agent() {
        let mut grant = AgentGrant::game_boy("phi", 1);
        grant.expires_at_frame = Some(50);
        let mut policy = AuthorityPolicy::new(1);
        policy.set_mode(ControlMode::PhiBot, Some(grant)).expect("handoff");

        let event = button(
            ActionSource::PhiBot {
                agent_id: "phi".into(),
                seat: 1,
            },
            51,
            "A",
        );
        assert!(!policy.authorize_batch(&[event], 51)[0].1.accepted);
    }

    #[test]
    fn versus_is_refused_on_single_port_core() {
        let mut policy = AuthorityPolicy::new(1);
        let error = policy
            .set_mode(ControlMode::Versus, Some(AgentGrant::game_boy("phi", 2)))
            .expect_err("single-port core must refuse versus");
        assert!(error.contains("2 playable ports"));
    }

    #[test]
    fn versus_topology_separates_human_and_agent_seats() {
        let mut policy = AuthorityPolicy::new(2);
        policy
            .set_mode(ControlMode::Versus, Some(AgentGrant::game_boy("phi", 2)))
            .expect("versus");

        let human = button(ActionSource::Human { seat: 1 }, 5, "A");
        let agent = button(
            ActionSource::PhiBot {
                agent_id: "phi".into(),
                seat: 2,
            },
            5,
            "B",
        );
        let decisions = policy.authorize_batch(&[human, agent], 5);
        assert!(decisions.iter().all(|(_, decision)| decision.accepted));
    }
}
