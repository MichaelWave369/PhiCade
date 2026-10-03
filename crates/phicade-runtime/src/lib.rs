//! PhiCade core-neutral runtime.
//!
//! This crate intentionally does not embed emulator cores. It defines the trust
//! boundary that future core adapters must satisfy.

pub mod action_bus;
pub mod core;
pub mod library;
pub mod replay;

pub use action_bus::{
    ActionBus, ActionEnvelope, ActionKind, ActionSource, SystemCommand,
};
pub use core::{AudioBuffer, CoreError, EmulatorCore, FrameBuffer};
pub use library::{GameImage, SystemId};
pub use replay::{
    ReplayCheckpoint, ReplayLedger, ReplayReceipt, ReplayVerification,
    ReplayVerificationResult, REPLAY_RECEIPT_SCHEMA, REPLAY_SCHEMA,
};
