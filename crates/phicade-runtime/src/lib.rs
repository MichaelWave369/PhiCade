//! PhiCade core-neutral runtime.
//!
//! This crate intentionally does not embed emulator cores. It defines the trust
//! boundary that future core adapters must satisfy.

pub mod action_bus;
pub mod authority;
pub mod agent_gym;
pub mod autodrive;
pub mod benchmark_campaign;
pub mod campaign_comparison;
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
    agent_gym_distance, agent_gym_score_1000, agent_gym_success, benchmark_suite_v1_tasks,
    benchmark_task_by_id, benchmark_task_by_rom_sha256, benchmark_task_distance,
    benchmark_task_success, locate_agent_gym_player, score_agent_gym_frame,
    score_benchmark_task_frame, AgentGymScore, BenchmarkTaskSpec, OracleLeg, PixelPoint,
    AGENT_GYM_ID, AGENT_GYM_INITIAL_DISTANCE, AGENT_GYM_MIRROR_ID,
    AGENT_GYM_MIRROR_ROM_SHA256, AGENT_GYM_MIRROR_SOURCE_SHA256,
    AGENT_GYM_MIRROR_START, AGENT_GYM_MIRROR_TARGET, AGENT_GYM_MIRROR_TASK,
    AGENT_GYM_ROM_SHA256, AGENT_GYM_SOURCE_SHA256, AGENT_GYM_START,
    AGENT_GYM_SUCCESS_DISTANCE, AGENT_GYM_TARGET, AGENT_GYM_TASK,
    AGENT_GYM_WARMUP_FRAMES, BENCHMARK_SUITE_V1_ID,
};
pub use autodrive::{
    AutodrivePolicy, AutodriveReceipt, AutodriveStatus, AutodriveStopReason,
    AUTODRIVE_RECEIPT_SCHEMA, AUTODRIVE_STATUS_SCHEMA,
};
pub use benchmark_campaign::{
    summarize_benchmark_trials, BenchmarkCampaignStats, BenchmarkTrialOutcome,
    BENCHMARK_CAMPAIGN_SCHEMA,
};
pub use campaign_comparison::{
    compare_campaign_samples, CampaignComparisonStats, CampaignSampleSummary,
    CAMPAIGN_COMPARISON_SCHEMA,
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
