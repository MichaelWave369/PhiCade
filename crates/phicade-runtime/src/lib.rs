//! PhiCade core-neutral runtime.
//!
//! This crate intentionally does not embed emulator cores. It defines the trust
//! boundary that future core adapters must satisfy.

pub mod action_bus;
pub mod authority;
pub mod agent_gym;
pub mod autodrive;
pub mod core;
pub mod driver;
pub mod library;
pub mod replay;
pub mod observation;

pub use action_bus::{
    live_source_order, ActionBus, ActionEnvelope, ActionKind, ActionSource, SystemCommand,
};
pub use authority::{AgentGrant, AuthorityDecision, AuthorityPolicy, ControlMode};
pub use agent_gym::{
    agent_gym_distance, agent_gym_score_1000, agent_gym_success, locate_agent_gym_player,
    PixelPoint, AGENT_GYM_ID, AGENT_GYM_INITIAL_DISTANCE, AGENT_GYM_ROM_SHA256,
    AGENT_GYM_SOURCE_SHA256, AGENT_GYM_START, AGENT_GYM_SUCCESS_DISTANCE,
    AGENT_GYM_TARGET, AGENT_GYM_WARMUP_FRAMES,
};
pub use autodrive::{
    AutodrivePolicy, AutodriveReceipt, AutodriveStatus, AutodriveStopReason,
    AUTODRIVE_RECEIPT_SCHEMA, AUTODRIVE_STATUS_SCHEMA,
};
pub use core::{AudioBuffer, CoreError, EmulatorCore, FrameBuffer};
pub use driver::{
    compile_agent_turn, AgentTurnAction, AgentTurnRequest, AgentTurnResponse,
    AGENT_TURN_REQUEST_SCHEMA, AGENT_TURN_RESPONSE_SCHEMA,
};
pub use library::{GameImage, SystemId};
pub use replay::{
    ReplayCheckpoint, ReplayLedger, ReplayReceipt, ReplayVerification,
    ReplayVerificationResult, REPLAY_RECEIPT_SCHEMA, REPLAY_SCHEMA,
};

pub use observation::{PhiBotObservation, PHIBOT_OBSERVATION_SCHEMA};
