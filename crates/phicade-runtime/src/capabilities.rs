use serde::{Deserialize, Serialize};

pub const RUNTIME_CAPABILITY_MANIFEST_SCHEMA: &str =
    "phicade.runtime-capability-manifest.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CapabilityStatus {
    Unsupported,
    Supported,
    Qualified,
}

impl CapabilityStatus {
    pub fn is_supported(self) -> bool {
        !matches!(self, Self::Unsupported)
    }

    pub fn is_qualified(self) -> bool {
        matches!(self, Self::Qualified)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RuntimeExecutionModel {
    EmbeddedFrameCore,
    ExternalProcess,
    WebRuntime,
    BridgedRuntime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeCapabilities {
    pub frame_step: CapabilityStatus,
    pub rendered_framebuffer: CapabilityStatus,
    pub audio_stream: CapabilityStatus,
    pub governed_actions: CapabilityStatus,
    pub reset: CapabilityStatus,
    pub state_snapshots: CapabilityStatus,
    pub exact_replay: CapabilityStatus,
    pub persistent_save_data: CapabilityStatus,
    pub game_detection: CapabilityStatus,
    pub external_process_lifecycle: CapabilityStatus,
    pub semantic_events: CapabilityStatus,
}

impl RuntimeCapabilities {
    /// Capabilities guaranteed by the core-neutral EmulatorCore contract itself.
    ///
    /// Anything not present in that trait stays UNSUPPORTED until an adapter
    /// explicitly declares more. This is deliberately conservative.
    pub fn core_baseline() -> Self {
        Self {
            frame_step: CapabilityStatus::Supported,
            rendered_framebuffer: CapabilityStatus::Supported,
            audio_stream: CapabilityStatus::Supported,
            governed_actions: CapabilityStatus::Supported,
            reset: CapabilityStatus::Supported,
            state_snapshots: CapabilityStatus::Unsupported,
            exact_replay: CapabilityStatus::Unsupported,
            persistent_save_data: CapabilityStatus::Unsupported,
            game_detection: CapabilityStatus::Unsupported,
            external_process_lifecycle: CapabilityStatus::Unsupported,
            semantic_events: CapabilityStatus::Unsupported,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeQualificationProfile {
    pub profile_id: String,
    pub source_revision: Option<String>,
    pub binary_evidence_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeCapabilityManifest {
    pub schema: String,
    pub runtime_id: String,
    pub runtime_version: Option<String>,
    pub adapter_id: String,
    pub execution_model: RuntimeExecutionModel,
    pub qualification_profile: Option<RuntimeQualificationProfile>,
    pub capabilities: RuntimeCapabilities,
}

impl RuntimeCapabilityManifest {
    pub fn new(
        runtime_id: impl Into<String>,
        runtime_version: Option<String>,
        adapter_id: impl Into<String>,
        execution_model: RuntimeExecutionModel,
        capabilities: RuntimeCapabilities,
    ) -> Self {
        Self {
            schema: RUNTIME_CAPABILITY_MANIFEST_SCHEMA.to_owned(),
            runtime_id: runtime_id.into(),
            runtime_version,
            adapter_id: adapter_id.into(),
            execution_model,
            qualification_profile: None,
            capabilities,
        }
    }

    pub fn with_qualification_profile(mut self, profile: RuntimeQualificationProfile) -> Self {
        self.qualification_profile = Some(profile);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_baseline_is_conservative_about_state_and_replay() {
        let caps = RuntimeCapabilities::core_baseline();
        assert!(caps.frame_step.is_supported());
        assert!(caps.rendered_framebuffer.is_supported());
        assert!(caps.audio_stream.is_supported());
        assert!(caps.governed_actions.is_supported());
        assert!(caps.reset.is_supported());

        assert!(!caps.state_snapshots.is_supported());
        assert!(!caps.exact_replay.is_supported());
        assert!(!caps.persistent_save_data.is_supported());
        assert!(!caps.game_detection.is_supported());
        assert!(!caps.external_process_lifecycle.is_supported());
        assert!(!caps.semantic_events.is_supported());
    }

    #[test]
    fn manifest_serializes_stable_schema_and_status_names() {
        let manifest = RuntimeCapabilityManifest::new(
            "example-core",
            Some("1.2.3".into()),
            "phicade.test-adapter",
            RuntimeExecutionModel::EmbeddedFrameCore,
            RuntimeCapabilities::core_baseline(),
        );
        let value = serde_json::to_value(manifest).expect("manifest json");
        assert_eq!(value["schema"], RUNTIME_CAPABILITY_MANIFEST_SCHEMA);
        assert_eq!(value["runtimeId"], "example-core");
        assert_eq!(value["runtimeVersion"], "1.2.3");
        assert_eq!(value["executionModel"], "EMBEDDED_FRAME_CORE");
        assert_eq!(value["capabilities"]["frameStep"], "SUPPORTED");
        assert_eq!(value["capabilities"]["exactReplay"], "UNSUPPORTED");
    }

    #[test]
    fn qualified_status_is_distinct_from_merely_supported() {
        assert!(CapabilityStatus::Supported.is_supported());
        assert!(!CapabilityStatus::Supported.is_qualified());
        assert!(CapabilityStatus::Qualified.is_supported());
        assert!(CapabilityStatus::Qualified.is_qualified());
    }
}
