use crate::{
    ActionEnvelope, ActionKind, AuthorityDecision, CapabilityStatus, RuntimeCapabilities,
    RuntimeCapabilityManifest, RuntimeExecutionModel,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fmt,
    io::{self, BufRead, BufReader, Write},
    path::Path,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

pub const PIXELFORGE_RUNTIME_PROTOCOL: &str = "pixelforge-runtime-bridge";
pub const PIXELFORGE_RUNTIME_PROTOCOL_VERSION: u32 = 1;
pub const PIXELFORGE_TRANSPORT_REQUEST_SCHEMA: &str =
    "pixelforge.runtime-transport.request.v1";
pub const PIXELFORGE_TRANSPORT_RESPONSE_SCHEMA: &str =
    "pixelforge.runtime-transport.response.v1";
pub const PIXELFORGE_ADAPTER_ID: &str = "phicade.pixelforge-jsonl-v1";
pub const PIXELFORGE_BRIDGE_QUALIFICATION_SCHEMA: &str =
    "phicade.pixelforge-bridge-qualification.v1";
pub const PIXELFORGE_CARTRIDGE_QUALIFICATION_SCHEMA: &str =
    "phicade.pixelforge-cartridge-qualification.v1";
pub const PIXELFORGE_PINNED_QUALIFICATION_REVISION: &str =
    "8fda48073b5d2ea6273df54c47c194bdac8231d6";
pub const PIXELFORGE_PINNED_REFERENCE_REVISION: &str =
    PIXELFORGE_PINNED_QUALIFICATION_REVISION;
pub const PIXELFORGE_SPARK_CHAIN_QUALIFICATION_SCHEMA: &str =
    "phicade.pixelforge-spark-chain-qualification.v1";
pub const PIXELFORGE_PINNED_SPARK_RUNTIME_REVISION: &str =
    "8790c3b00b5184136fb8fa6a2fbdabe834eeca22";
pub const SPARK_PINNED_BRIDGE_REVISION: &str =
    "fae7879820bef63a550fea486b2defbc3cee5304";

#[derive(Debug)]
pub enum PixelForgeBridgeError {
    Io(io::Error),
    Json(serde_json::Error),
    Protocol(String),
    Remote { code: String, message: String },
    Process(String),
}

impl fmt::Display for PixelForgeBridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "PixelForge transport I/O error: {error}"),
            Self::Json(error) => write!(f, "PixelForge transport JSON error: {error}"),
            Self::Protocol(message) => write!(f, "PixelForge transport protocol error: {message}"),
            Self::Remote { code, message } => {
                write!(f, "PixelForge bridge rejected request ({code}): {message}")
            }
            Self::Process(message) => write!(f, "PixelForge process error: {message}"),
        }
    }
}

impl std::error::Error for PixelForgeBridgeError {}

impl From<io::Error> for PixelForgeBridgeError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for PixelForgeBridgeError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PixelForgeBridgeDescriptor {
    pub protocol: String,
    pub version: u32,
    pub game_id: String,
    pub runtime_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deterministic: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clock_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub advance_semantics: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replay_exact: Option<bool>,
}

impl PixelForgeBridgeDescriptor {
    pub fn validate(&self) -> Result<(), PixelForgeBridgeError> {
        if self.protocol != PIXELFORGE_RUNTIME_PROTOCOL {
            return Err(PixelForgeBridgeError::Protocol(format!(
                "expected protocol {PIXELFORGE_RUNTIME_PROTOCOL}, got {}",
                self.protocol
            )));
        }
        if self.version != PIXELFORGE_RUNTIME_PROTOCOL_VERSION {
            return Err(PixelForgeBridgeError::Protocol(format!(
                "expected bridge version {PIXELFORGE_RUNTIME_PROTOCOL_VERSION}, got {}",
                self.version
            )));
        }
        if self.game_id.trim().is_empty() {
            return Err(PixelForgeBridgeError::Protocol(
                "descriptor gameId must be non-empty".into(),
            ));
        }
        if self.runtime_version.trim().is_empty() {
            return Err(PixelForgeBridgeError::Protocol(
                "descriptor runtimeVersion must be non-empty".into(),
            ));
        }
        if let Some(clock_mode) = self.clock_mode.as_deref() {
            if !matches!(clock_mode, "external" | "engine") {
                return Err(PixelForgeBridgeError::Protocol(format!(
                    "unsupported clockMode {clock_mode}"
                )));
            }
        }
        Ok(())
    }

    fn is_externally_stepped(&self) -> bool {
        match self.clock_mode.as_deref() {
            Some("external") => true,
            Some("engine") => false,
            _ => self.deterministic == Some(true),
        }
    }
}

#[derive(Debug, Serialize)]
struct PixelForgeTransportRequest<'a> {
    schema: &'static str,
    id: u64,
    method: &'a str,
    params: Vec<Value>,
}

#[derive(Debug, Deserialize)]
struct PixelForgeTransportResponse {
    schema: String,
    id: Value,
    ok: bool,
    #[serde(default)]
    result: Option<Value>,
    #[serde(default)]
    error: Option<PixelForgeTransportRemoteError>,
}

#[derive(Debug, Deserialize)]
struct PixelForgeTransportRemoteError {
    code: String,
    message: String,
}

/// Local-process client for PixelForge Runtime Bridge JSONL Transport v1.
///
/// The client transports bridge calls. It does not grant authority, reinterpret
/// cartridge rules, or promote runtime capabilities beyond what PhiCade can
/// actually prove.
pub struct PixelForgeJsonlClient {
    child: Child,
    stdin: Option<ChildStdin>,
    stdout: BufReader<ChildStdout>,
    next_request_id: u64,
    descriptor: PixelForgeBridgeDescriptor,
}

impl PixelForgeJsonlClient {
    pub fn spawn_node_reference(
        pixelforge_root: &Path,
        seed: i64,
    ) -> Result<Self, PixelForgeBridgeError> {
        Self::spawn_node_reference_with_program(pixelforge_root, seed, "node")
    }

    pub fn spawn_node_cartridge(
        pixelforge_root: &Path,
        cartridge: &str,
        scene: &str,
    ) -> Result<Self, PixelForgeBridgeError> {
        let server = pixelforge_root.join("scripts/serve_cartridge_runtime.mjs");
        if !server.is_file() {
            return Err(PixelForgeBridgeError::Process(format!(
                "PixelForge cartridge server not found: {}",
                server.display()
            )));
        }

        let mut command = Command::new("node");
        command
            .arg(&server)
            .arg("--cartridge")
            .arg(cartridge)
            .arg("--scene")
            .arg(scene)
            .current_dir(pixelforge_root);

        Self::spawn(command)
    }

    pub fn spawn_external_spark(
        pixelforge_root: &Path,
        spark_root: &Path,
    ) -> Result<Self, PixelForgeBridgeError> {
        let server = pixelforge_root.join("scripts/serve_external_spark_runtime.mjs");
        if !server.is_file() {
            return Err(PixelForgeBridgeError::Process(format!(
                "PixelForge SPARK server not found: {}",
                server.display()
            )));
        }
        if !spark_root.is_dir() {
            return Err(PixelForgeBridgeError::Process(format!(
                "SPARK root was not found: {}",
                spark_root.display()
            )));
        }

        let mut command = Command::new("node");
        command
            .arg(&server)
            .arg("--spark-root")
            .arg(spark_root)
            .current_dir(pixelforge_root);

        Self::spawn(command)
    }

    pub fn spawn_node_reference_with_program(
        pixelforge_root: &Path,
        seed: i64,
        node_program: &str,
    ) -> Result<Self, PixelForgeBridgeError> {
        let server = pixelforge_root.join("scripts/serve_reference_runtime.mjs");
        if !server.is_file() {
            return Err(PixelForgeBridgeError::Process(format!(
                "PixelForge reference server not found: {}",
                server.display()
            )));
        }

        let mut command = Command::new(node_program);
        command
            .arg(&server)
            .arg("--seed")
            .arg(seed.to_string())
            .current_dir(pixelforge_root);

        Self::spawn(command)
    }

    pub fn spawn(mut command: Command) -> Result<Self, PixelForgeBridgeError> {
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());

        let mut child = command.spawn()?;
        let stdin = child.stdin.take().ok_or_else(|| {
            PixelForgeBridgeError::Process("child stdin pipe was not created".into())
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            PixelForgeBridgeError::Process("child stdout pipe was not created".into())
        })?;

        let placeholder = PixelForgeBridgeDescriptor {
            protocol: String::new(),
            version: 0,
            game_id: String::new(),
            runtime_version: String::new(),
            deterministic: None,
            clock_mode: None,
            advance_semantics: None,
            replay_exact: None,
        };

        let mut client = Self {
            child,
            stdin: Some(stdin),
            stdout: BufReader::new(stdout),
            next_request_id: 1,
            descriptor: placeholder,
        };

        let descriptor = client.fetch_descriptor()?;
        descriptor.validate()?;
        client.descriptor = descriptor;
        Ok(client)
    }

    pub fn descriptor(&self) -> &PixelForgeBridgeDescriptor {
        &self.descriptor
    }

    pub fn capability_manifest(&self) -> RuntimeCapabilityManifest {
        let externally_stepped = self.descriptor.is_externally_stepped();
        let capabilities = RuntimeCapabilities {
            frame_step: if externally_stepped {
                CapabilityStatus::Supported
            } else {
                CapabilityStatus::Unsupported
            },
            rendered_framebuffer: CapabilityStatus::Unsupported,
            audio_stream: CapabilityStatus::Unsupported,
            governed_actions: CapabilityStatus::Supported,
            reset: CapabilityStatus::Unsupported,
            state_snapshots: CapabilityStatus::Unsupported,
            exact_replay: CapabilityStatus::Unsupported,
            persistent_save_data: CapabilityStatus::Unsupported,
            game_detection: CapabilityStatus::Unsupported,
            external_process_lifecycle: CapabilityStatus::Supported,
            semantic_events: CapabilityStatus::Supported,
        };

        RuntimeCapabilityManifest::new(
            self.descriptor.game_id.clone(),
            Some(self.descriptor.runtime_version.clone()),
            PIXELFORGE_ADAPTER_ID,
            RuntimeExecutionModel::BridgedRuntime,
            capabilities,
        )
    }

    pub fn register_controller(
        &mut self,
        controller: Value,
    ) -> Result<Value, PixelForgeBridgeError> {
        self.call_value("registerController", vec![controller])
    }

    pub fn observe(&mut self, controller_id: &str) -> Result<Value, PixelForgeBridgeError> {
        self.call_value("observe", vec![json!(controller_id)])
    }

    pub fn submit(
        &mut self,
        controller_id: &str,
        intent: Value,
        tick: u64,
    ) -> Result<Value, PixelForgeBridgeError> {
        self.call_value(
            "submit",
            vec![json!(controller_id), intent, json!(tick)],
        )
    }

    pub fn advance(&mut self) -> Result<Vec<Value>, PixelForgeBridgeError> {
        let value = self.call_value("advance", Vec::new())?;
        Ok(serde_json::from_value(value)?)
    }

    pub fn events(&mut self, since: u64) -> Result<Vec<Value>, PixelForgeBridgeError> {
        let value = self.call_value("events", vec![json!(since)])?;
        Ok(serde_json::from_value(value)?)
    }

    pub fn snapshot(&mut self) -> Result<Value, PixelForgeBridgeError> {
        self.call_value("snapshot", Vec::new())
    }

    pub fn recording(&mut self) -> Result<Value, PixelForgeBridgeError> {
        self.call_value("recording", Vec::new())
    }

    pub fn authority(&mut self) -> Result<Value, PixelForgeBridgeError> {
        self.call_value("authority", Vec::new())
    }

    pub fn hash(&mut self) -> Result<String, PixelForgeBridgeError> {
        let value = self.call_value("hash", Vec::new())?;
        Ok(serde_json::from_value(value)?)
    }

    pub fn close(mut self) -> Result<(), PixelForgeBridgeError> {
        self.stdin.take();
        let status = self.child.wait()?;
        if !status.success() {
            return Err(PixelForgeBridgeError::Process(format!(
                "PixelForge runtime exited with status {status}"
            )));
        }
        Ok(())
    }

    fn fetch_descriptor(&mut self) -> Result<PixelForgeBridgeDescriptor, PixelForgeBridgeError> {
        let value = self.call_value("describe", Vec::new())?;
        Ok(serde_json::from_value(value)?)
    }

    fn call_value(
        &mut self,
        method: &str,
        params: Vec<Value>,
    ) -> Result<Value, PixelForgeBridgeError> {
        let id = self.next_request_id;
        self.next_request_id = self.next_request_id.saturating_add(1);

        let request = PixelForgeTransportRequest {
            schema: PIXELFORGE_TRANSPORT_REQUEST_SCHEMA,
            id,
            method,
            params,
        };

        let line = serde_json::to_string(&request)?;
        let stdin = self.stdin.as_mut().ok_or_else(|| {
            PixelForgeBridgeError::Process("PixelForge runtime stdin is closed".into())
        })?;
        stdin.write_all(line.as_bytes())?;
        stdin.write_all(b"\n")?;
        stdin.flush()?;

        let mut response_line = String::new();
        let bytes = self.stdout.read_line(&mut response_line)?;
        if bytes == 0 {
            return Err(PixelForgeBridgeError::Process(
                "PixelForge runtime closed stdout before replying".into(),
            ));
        }

        let response: PixelForgeTransportResponse =
            serde_json::from_str(response_line.trim_end())?;

        if response.schema != PIXELFORGE_TRANSPORT_RESPONSE_SCHEMA {
            return Err(PixelForgeBridgeError::Protocol(format!(
                "unexpected response schema {}",
                response.schema
            )));
        }
        if response.id != json!(id) {
            return Err(PixelForgeBridgeError::Protocol(format!(
                "response id {} does not match request id {id}",
                response.id
            )));
        }

        if !response.ok {
            let error = response.error.unwrap_or(PixelForgeTransportRemoteError {
                code: "UNKNOWN_REMOTE_ERROR".into(),
                message: "PixelForge returned ok=false without an error body".into(),
            });
            return Err(PixelForgeBridgeError::Remote {
                code: error.code,
                message: error.message,
            });
        }

        Ok(response.result.unwrap_or(Value::Null))
    }
}

impl Drop for PixelForgeJsonlClient {
    fn drop(&mut self) {
        self.stdin.take();
        match self.child.try_wait() {
            Ok(Some(_)) => {}
            _ => {
                let _ = self.child.kill();
                let _ = self.child.wait();
            }
        }
    }
}

/// Qualification-only mapping used by the first PhiCade ↔ PixelForge proof.
///
/// It intentionally does not become a generic gameplay mapping. A real
/// cartridge, including SPARK, owns its own action grammar.
pub fn reference_counter_intent_from_action(
    action: &ActionEnvelope,
) -> Result<Value, PixelForgeBridgeError> {
    match &action.action {
        ActionKind::Button { button, pressed } if button == "A" && *pressed => Ok(json!({
            "type": "ADD",
            "actorId": "counter",
            "params": { "amount": 1 }
        })),
        _ => Err(PixelForgeBridgeError::Protocol(
            "reference counter accepts only an authorized A-button press".into(),
        )),
    }
}

pub fn legend_bouncehome_intent_from_action(
    action: &ActionEnvelope,
) -> Result<Value, PixelForgeBridgeError> {
    let direction = match &action.action {
        ActionKind::Button { button, pressed } if *pressed => match button.as_str() {
            "UP" | "DOWN" | "LEFT" | "RIGHT" => button.as_str(),
            _ => {
                return Err(PixelForgeBridgeError::Protocol(
                    "Legend qualification accepts only directional button presses".into(),
                ))
            }
        },
        _ => {
            return Err(PixelForgeBridgeError::Protocol(
                "Legend qualification requires a pressed directional button".into(),
            ))
        }
    };

    Ok(json!({
        "type": "MOVE",
        "actorId": "more-bounce",
        "params": { "direction": direction }
    }))
}

pub fn spark_threshold_move_intent_from_action(
    action: &ActionEnvelope,
) -> Result<Value, PixelForgeBridgeError> {
    let (x, y) = match &action.action {
        ActionKind::Button { button, pressed } if *pressed => match button.as_str() {
            "UP" => (0, -1),
            "DOWN" => (0, 1),
            "LEFT" => (-1, 0),
            "RIGHT" => (1, 0),
            _ => {
                return Err(PixelForgeBridgeError::Protocol(
                    "SPARK Threshold qualification accepts only directional button presses".into(),
                ))
            }
        },
        _ => {
            return Err(PixelForgeBridgeError::Protocol(
                "SPARK Threshold qualification requires a pressed directional button".into(),
            ))
        }
    };

    Ok(json!({
        "type": "MOVE",
        "actorId": "spark",
        "params": { "x": x, "y": y }
    }))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PixelForgeSparkChainQualificationReceipt {
    pub schema: String,
    pub record_status: String,
    pub phicade_revision: String,
    pub pixelforge_revision: String,
    pub spark_revision: String,
    pub transport_request_schema: String,
    pub transport_response_schema: String,
    pub descriptor: PixelForgeBridgeDescriptor,
    pub capability_manifest: RuntimeCapabilityManifest,
    pub controller_id: String,
    pub initial_observation: Value,
    pub authority_decision: AuthorityDecision,
    pub source_action: ActionEnvelope,
    pub submitted_intent: Value,
    pub direct_events: Vec<Value>,
    pub semantic_events: Vec<Value>,
    pub final_observation: Value,
    pub final_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PixelForgeCartridgeQualificationReceipt {
    pub schema: String,
    pub record_status: String,
    pub pixelforge_revision: String,
    pub cartridge_id: String,
    pub scene_id: String,
    pub transport_request_schema: String,
    pub transport_response_schema: String,
    pub descriptor: PixelForgeBridgeDescriptor,
    pub capability_manifest: RuntimeCapabilityManifest,
    pub controller_id: String,
    pub initial_observation: Value,
    pub authority_decision: AuthorityDecision,
    pub source_action: ActionEnvelope,
    pub submitted_intent: Value,
    pub direct_events: Vec<Value>,
    pub semantic_events: Vec<Value>,
    pub final_observation: Value,
    pub final_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PixelForgeBridgeQualificationReceipt {
    pub schema: String,
    pub record_status: String,
    pub pixelforge_revision: String,
    pub transport_request_schema: String,
    pub transport_response_schema: String,
    pub descriptor: PixelForgeBridgeDescriptor,
    pub capability_manifest: RuntimeCapabilityManifest,
    pub controller_id: String,
    pub initial_observation: Value,
    pub authority_decision: AuthorityDecision,
    pub source_action: ActionEnvelope,
    pub submitted_intent: Value,
    pub direct_events: Vec<Value>,
    pub semantic_events: Vec<Value>,
    pub final_observation: Value,
    pub final_hash: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ActionSource;

    fn descriptor() -> PixelForgeBridgeDescriptor {
        PixelForgeBridgeDescriptor {
            protocol: PIXELFORGE_RUNTIME_PROTOCOL.into(),
            version: PIXELFORGE_RUNTIME_PROTOCOL_VERSION,
            game_id: "test-game".into(),
            runtime_version: "test/1".into(),
            deterministic: Some(true),
            clock_mode: None,
            advance_semantics: None,
            replay_exact: None,
        }
    }

    #[test]
    fn descriptor_validation_rejects_wrong_protocol() {
        let mut value = descriptor();
        value.protocol = "something-else".into();
        assert!(value.validate().is_err());
    }

    #[test]
    fn deterministic_descriptor_is_externally_stepped_by_compatibility_rule() {
        assert!(descriptor().is_externally_stepped());
    }

    #[test]
    fn reference_action_mapping_is_deliberately_narrow() {
        let accepted = ActionEnvelope {
            sequence: 0,
            frame: 0,
            source: ActionSource::Human { seat: 1 },
            action: ActionKind::Button {
                button: "A".into(),
                pressed: true,
            },
        };
        let rejected = ActionEnvelope {
            action: ActionKind::Button {
                button: "B".into(),
                pressed: true,
            },
            ..accepted.clone()
        };

        assert_eq!(
            reference_counter_intent_from_action(&accepted).unwrap()["type"],
            "ADD"
        );
        assert!(reference_counter_intent_from_action(&rejected).is_err());
    }

    #[test]
    fn legend_mapping_accepts_only_directional_pressed_buttons() {
        let right = ActionEnvelope {
            sequence: 0,
            frame: 0,
            source: ActionSource::Human { seat: 1 },
            action: ActionKind::Button {
                button: "RIGHT".into(),
                pressed: true,
            },
        };
        let a = ActionEnvelope {
            action: ActionKind::Button {
                button: "A".into(),
                pressed: true,
            },
            ..right.clone()
        };

        let intent = legend_bouncehome_intent_from_action(&right).unwrap();
        assert_eq!(intent["type"], "MOVE");
        assert_eq!(intent["actorId"], "more-bounce");
        assert_eq!(intent["params"]["direction"], "RIGHT");
        assert!(legend_bouncehome_intent_from_action(&a).is_err());
    }

    #[test]
    fn spark_threshold_mapping_is_deliberately_move_only() {
        let right = ActionEnvelope {
            sequence: 0,
            frame: 0,
            source: ActionSource::Human { seat: 1 },
            action: ActionKind::Button {
                button: "RIGHT".into(),
                pressed: true,
            },
        };
        let dash = ActionEnvelope {
            action: ActionKind::Button {
                button: "DASH".into(),
                pressed: true,
            },
            ..right.clone()
        };

        let intent = spark_threshold_move_intent_from_action(&right).unwrap();
        assert_eq!(intent["type"], "MOVE");
        assert_eq!(intent["actorId"], "spark");
        assert_eq!(intent["params"]["x"], 1);
        assert_eq!(intent["params"]["y"], 0);
        assert!(spark_threshold_move_intent_from_action(&dash).is_err());
    }

    #[test]
    fn bridged_manifest_does_not_invent_snapshot_or_replay_support() {
        let descriptor = descriptor();
        let capabilities = RuntimeCapabilities {
            frame_step: if descriptor.is_externally_stepped() {
                CapabilityStatus::Supported
            } else {
                CapabilityStatus::Unsupported
            },
            rendered_framebuffer: CapabilityStatus::Unsupported,
            audio_stream: CapabilityStatus::Unsupported,
            governed_actions: CapabilityStatus::Supported,
            reset: CapabilityStatus::Unsupported,
            state_snapshots: CapabilityStatus::Unsupported,
            exact_replay: CapabilityStatus::Unsupported,
            persistent_save_data: CapabilityStatus::Unsupported,
            game_detection: CapabilityStatus::Unsupported,
            external_process_lifecycle: CapabilityStatus::Supported,
            semantic_events: CapabilityStatus::Supported,
        };
        let manifest = RuntimeCapabilityManifest::new(
            descriptor.game_id,
            Some(descriptor.runtime_version),
            PIXELFORGE_ADAPTER_ID,
            RuntimeExecutionModel::BridgedRuntime,
            capabilities,
        );

        assert_eq!(manifest.execution_model, RuntimeExecutionModel::BridgedRuntime);
        assert!(manifest.capabilities.governed_actions.is_supported());
        assert!(manifest.capabilities.semantic_events.is_supported());
        assert!(!manifest.capabilities.state_snapshots.is_supported());
        assert!(!manifest.capabilities.exact_replay.is_supported());
    }
}
