use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ActionSource {
    Human { seat: u8 },
    Replay,
    Script { name: String },
    PhiBot {
        #[serde(rename = "agentId")]
        agent_id: String,
        seat: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SystemCommand {
    Pause,
    Reset,
    SaveState,
    LoadState,
    Rewind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ActionKind {
    Button {
        button: String,
        pressed: bool,
    },
    Axis {
        axis: String,
        value: i16,
    },
    System {
        command: SystemCommand,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        slot: Option<u8>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionEnvelope {
    pub sequence: u64,
    pub frame: u64,
    pub source: ActionSource,
    pub action: ActionKind,
}

impl ActionEnvelope {
    pub fn validate(&self) -> Result<(), String> {
        if let ActionSource::Human { seat } = &self.source {
            if *seat == 0 {
                return Err("human seat numbers start at 1".to_owned());
            }
        }

        if let ActionKind::System {
            slot: Some(slot), ..
        } = &self.action
        {
            if *slot > 9 {
                return Err("save-state slot must be in 0..=9".to_owned());
            }
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct ActionBus {
    next_sequence: u64,
    queue: VecDeque<ActionEnvelope>,
}

impl ActionBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn publish(
        &mut self,
        frame: u64,
        source: ActionSource,
        action: ActionKind,
    ) -> ActionEnvelope {
        let envelope = ActionEnvelope {
            sequence: self.next_sequence,
            frame,
            source,
            action,
        };
        self.next_sequence += 1;
        self.queue.push_back(envelope.clone());
        envelope
    }

    pub fn drain_through_frame(&mut self, frame: u64) -> Vec<ActionEnvelope> {
        let mut drained = Vec::new();
        while self.queue.front().is_some_and(|event| event.frame <= frame) {
            if let Some(event) = self.queue.pop_front() {
                drained.push(event);
            }
        }
        drained
    }

    pub fn pending(&self) -> usize {
        self.queue.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn button(name: &str) -> ActionKind {
        ActionKind::Button {
            button: name.to_owned(),
            pressed: true,
        }
    }

    #[test]
    fn sequences_are_monotonic_across_sources() {
        let mut bus = ActionBus::new();
        let human = bus.publish(7, ActionSource::Human { seat: 1 }, button("A"));
        let replay = bus.publish(7, ActionSource::Replay, button("B"));

        assert_eq!(human.sequence, 0);
        assert_eq!(replay.sequence, 1);
        assert_eq!(bus.pending(), 2);
    }

    #[test]
    fn drain_respects_frame_boundary() {
        let mut bus = ActionBus::new();
        bus.publish(10, ActionSource::Human { seat: 1 }, button("LEFT"));
        bus.publish(11, ActionSource::Human { seat: 1 }, button("RIGHT"));

        let frame_ten = bus.drain_through_frame(10);

        assert_eq!(frame_ten.len(), 1);
        assert_eq!(frame_ten[0].frame, 10);
        assert_eq!(bus.pending(), 1);
    }

    #[test]
    fn action_envelope_json_matches_ipc_contract() {
        let event = ActionEnvelope {
            sequence: 12,
            frame: 34,
            source: ActionSource::PhiBot {
                agent_id: "phi-7".to_owned(),
                seat: 1,
            },
            action: ActionKind::System {
                command: SystemCommand::SaveState,
                slot: Some(3),
            },
        };

        let json = serde_json::to_string(&event).expect("serialize action");
        assert!(json.contains("\"kind\":\"phi-bot\""));
        assert!(json.contains("\"agentId\":\"phi-7\""));
        assert!(json.contains("\"seat\":1"));
        assert!(json.contains("\"command\":\"save-state\""));

        let decoded: ActionEnvelope = serde_json::from_str(&json).expect("deserialize action");
        assert_eq!(decoded, event);
    }

    #[test]
    fn rejects_human_seat_zero() {
        let event = ActionEnvelope {
            sequence: 0,
            frame: 0,
            source: ActionSource::Human { seat: 0 },
            action: button("A"),
        };

        assert!(event.validate().is_err());
    }
}
