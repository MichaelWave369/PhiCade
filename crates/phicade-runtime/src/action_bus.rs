use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionSource {
    Human { seat: u8 },
    Replay,
    Script { name: String },
    PhiBot { agent_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemAction {
    Pause,
    Reset,
    SaveState { slot: u8 },
    LoadState { slot: u8 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionKind {
    Button { button: String, pressed: bool },
    Axis { axis: String, value: i16 },
    System(SystemAction),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionEnvelope {
    pub sequence: u64,
    pub frame: u64,
    pub source: ActionSource,
    pub action: ActionKind,
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
}
