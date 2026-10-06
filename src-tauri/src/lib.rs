use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use phicade_libretro::{CoreIdentity, LibretroCore};
use phicade_runtime::{
    compile_agent_turn, live_source_order, ActionEnvelope, ActionKind, ActionSource, AgentGrant,
    AgentGymScore, AgentTurnRequest, AgentTurnResponse, AutodrivePolicy, AutodriveReceipt,
    AutodriveStatus, AutodriveStopReason, AuthorityPolicy, AudioBuffer, BenchmarkCampaignStats,
    BenchmarkSuiteAggregateStats, BenchmarkSuiteComparisonStats, BenchmarkTaskSpec,
    BenchmarkTrialOutcome, CampaignComparisonStats, ContentDescriptor, ControlMode, EmulatorCore,
    FrameBuffer, GameImage,
    PhiBotObservation, PixelPoint, ReplayCheckpoint, ReplayLedger, ReplayReceipt,
    ReplayVerification, ReplayVerificationResult, RuntimeCapabilityManifest,
    SuiteTaskAggregateInput, SystemCommand, SystemId, benchmark_suite_by_id, benchmark_suite_v1_tasks, benchmark_suite_v3_tasks, benchmark_suite_v4_tasks, benchmark_suite_v5_tasks, benchmark_suite_v6_tasks, benchmark_suite_v7_tasks, benchmark_suite_v8_tasks, benchmark_suite_v9_tasks, benchmark_suite_v10_tasks, benchmark_suite_v11_tasks, benchmark_suite_v12_tasks, benchmark_suite_v13_tasks, benchmark_suites,
    benchmark_suites_for_task, benchmark_task_by_id, benchmark_task_by_rom_sha256,
    compare_benchmark_suites,
    compare_campaign_samples, score_benchmark_task_frame,
    summarize_benchmark_suite, summarize_benchmark_trials, SuiteTaskComparisonInput,
    AGENT_TURN_REQUEST_SCHEMA, AGENT_GYM_ID,
    AGENT_GYM_INITIAL_DISTANCE, AGENT_GYM_ROM_SHA256, AGENT_GYM_SOURCE_SHA256,
    AGENT_GYM_START, AGENT_GYM_TARGET, AUTODRIVE_RECEIPT_SCHEMA,
    AUTODRIVE_STATUS_SCHEMA, BENCHMARK_CAMPAIGN_SCHEMA, BENCHMARK_SUITE_COMPARISON_SCHEMA,
    BENCHMARK_SUITE_REPORT_SCHEMA, BENCHMARK_SUITE_V1_ID, BENCHMARK_SUITE_V2_ID,
    BENCHMARK_SUITE_V3_ID, BENCHMARK_SUITE_V4_ID, BENCHMARK_SUITE_V5_ID,
    BENCHMARK_SUITE_V6_ID, BENCHMARK_SUITE_V7_ID, BENCHMARK_SUITE_V8_ID, BENCHMARK_SUITE_V9_ID, BENCHMARK_SUITE_V10_ID, BENCHMARK_SUITE_V11_ID, BENCHMARK_SUITE_V12_ID, BENCHMARK_SUITE_V13_ID, CAMPAIGN_COMPARISON_SCHEMA,
    PHIBOT_OBSERVATION_SCHEMA,
    REPLAY_RECEIPT_SCHEMA, REPLAY_SCHEMA,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::VecDeque,
    fs,
    io::BufWriter,
    path::{Path, PathBuf},
    sync::Mutex,
};
use tauri::{AppHandle, Manager, State};

mod providers;

use providers::ollama::{
    self, OllamaModel, OllamaModelDetails, OllamaQualificationReceipt, OllamaTurnResult,
};

const MAX_LIBRARY_ENTRIES: usize = 4096;
const WEB_AUDIO_SAMPLE_RATE_HZ: u32 = 48_000;
const SRAM_FLUSH_INTERVAL_FRAMES: u64 = 300;
const REPLAY_CHECKPOINT_INTERVAL_FRAMES: u64 = 60;
const STATE_MAGIC: &[u8] = b"PHICADE_STATE_V1\0";
const MODEL_GAMEPLAY_BENCHMARK_SCHEMA: &str = "phicade.model-gameplay-benchmark.v1";
const MIN_CAMPAIGN_TRIALS: u16 = 3;
const MAX_CAMPAIGN_TRIALS: u16 = 20;
const MANUAL_AGENT_MEMORY_MAX_BYTES: u32 = 4_096;
const MANUAL_AGENT_MEMORY_UPDATE_MAX_BYTES: u32 = 1_024;
const RUNTIME_REGISTRATION_SCHEMA: &str = "phicade.runtime-registration.v1";
const PUBLIC_SUITE_RESULT_SCHEMA: &str = "phicade.public-suite-result.v1";

#[derive(Default)]
struct EmulatorState {
    session: Mutex<Option<EmulatorSession>>,
}

#[derive(Debug, Clone)]
struct StateSnapshot {
    frame: u64,
    bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
struct SessionPaths {
    save_ram: PathBuf,
    state_dir: PathBuf,
    screenshot_dir: PathBuf,
    replay_dir: PathBuf,
    autodrive_dir: PathBuf,
    autodrive_root: PathBuf,
    model_benchmark_dir: PathBuf,
    model_benchmark_root: PathBuf,
    benchmark_campaign_root: PathBuf,
    benchmark_campaign_dir: PathBuf,
    campaign_comparison_dir: PathBuf,
    suite_report_root: PathBuf,
    suite_comparison_root: PathBuf,
    profile: PathBuf,
}

#[derive(Debug, Clone)]
struct ReplayRecording {
    ledger: ReplayLedger,
    next_checkpoint_frame: u64,
}

#[derive(Debug, Clone)]
struct ReplayExport {
    replay_path: PathBuf,
    receipt_path: PathBuf,
    replay_sha256: String,
    receipt: ReplayReceipt,
}

#[derive(Debug, Clone)]
struct AutodriveExport {
    receipt_path: PathBuf,
    receipt: AutodriveReceipt,
}

#[derive(Debug, Clone)]
struct AgentMemoryState {
    content: String,
    initial_sha256: String,
    revision: u64,
    updates: u64,
    bytes_written: u64,
    refusals: u64,
}

impl AgentMemoryState {
    fn new() -> Self {
        let content = String::new();
        let initial_sha256 = sha256_bytes(content.as_bytes());
        Self {
            content,
            initial_sha256,
            revision: 0,
            updates: 0,
            bytes_written: 0,
            refusals: 0,
        }
    }

    fn sha256(&self) -> String {
        sha256_bytes(self.content.as_bytes())
    }

    fn bytes(&self) -> u32 {
        u32::try_from(self.content.len()).unwrap_or(u32::MAX)
    }
}

#[derive(Debug, Clone)]
struct ModelBenchmarkRun {
    run_id: u64,
    autodrive_run_id: u64,
    benchmark_id: String,
    benchmark_source_sha256: String,
    benchmark_rom_sha256: String,
    provider: String,
    model: String,
    model_digest: String,
    model_qualification_sha256: String,
    core_sha256: String,
    started_frame: u64,
    start_player: PixelPoint,
    target: PixelPoint,
    initial_distance: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ModelGameplayBenchmarkReceipt {
    schema: String,
    record_status: String,
    benchmark_id: String,
    benchmark_run_id: u64,
    provider: String,
    model: String,
    model_digest: String,
    model_qualification_sha256: String,
    gym_source_sha256: String,
    gym_rom_sha256: String,
    core_sha256: String,
    core_name: String,
    core_version: String,
    autodrive_receipt_sha256: String,
    autodrive_run_id: u64,
    policy: AutodrivePolicy,
    stop_reason: AutodriveStopReason,
    started_frame: u64,
    ended_frame: u64,
    start_player: PixelPoint,
    final_player: Option<PixelPoint>,
    target: PixelPoint,
    initial_distance: i32,
    final_distance: Option<i32>,
    progress: Option<i32>,
    score_1000: Option<u16>,
    task_success: bool,
    turns_issued: u16,
    turns_completed: u16,
    total_actions: u32,
    final_frame_sha256: String,
    scoring_error: Option<String>,
}

#[derive(Debug, Clone)]
struct ModelBenchmarkExport {
    receipt_path: PathBuf,
    receipt: ModelGameplayBenchmarkReceipt,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelBenchmarkArtifact {
    receipt_path: String,
    receipt: ModelGameplayBenchmarkReceipt,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelBenchmarkStart {
    benchmark_run_id: u64,
    autodrive: AutodriveStatus,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CampaignTrialEvidence {
    benchmark_run_id: u64,
    receipt_sha256: String,
    record_status: String,
    score_1000: Option<u16>,
    task_success: bool,
    stop_reason: AutodriveStopReason,
}

#[derive(Debug, Clone)]
struct BenchmarkCampaignRun {
    campaign_id: u64,
    active: bool,
    total_trials: u16,
    benchmark_id: String,
    benchmark_source_sha256: String,
    benchmark_rom_sha256: String,
    provider: String,
    model: String,
    model_digest: String,
    model_qualification_sha256: String,
    core_sha256: String,
    policy: AutodrivePolicy,
    trials: Vec<CampaignTrialEvidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BenchmarkCampaignReceipt {
    schema: String,
    record_status: String,
    campaign_id: u64,
    benchmark_id: String,
    provider: String,
    model: String,
    model_digest: String,
    model_qualification_sha256: String,
    gym_source_sha256: String,
    gym_rom_sha256: String,
    core_sha256: String,
    core_name: String,
    core_version: String,
    policy: AutodrivePolicy,
    total_trials: u16,
    completed_trials: u16,
    trials: Vec<CampaignTrialEvidence>,
    stats: BenchmarkCampaignStats,
}

#[derive(Debug, Clone)]
struct BenchmarkCampaignExport {
    receipt_path: PathBuf,
    receipt: BenchmarkCampaignReceipt,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BenchmarkCampaignArtifact {
    receipt_path: String,
    receipt: BenchmarkCampaignReceipt,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BenchmarkCampaignStatus {
    schema: String,
    campaign_id: u64,
    active: bool,
    total_trials: u16,
    completed_trials: u16,
    provider: String,
    model: String,
    model_digest: String,
    policy: AutodrivePolicy,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BenchmarkCampaignStart {
    status: BenchmarkCampaignStatus,
    benchmark: ModelBenchmarkStart,
}


#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CampaignListEntry {
    campaign_id: u64,
    receipt_sha256: String,
    record_status: String,
    model: String,
    model_digest: String,
    total_trials: u16,
    completed_trials: u16,
    mean_score_1000: Option<f64>,
    success_rate: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ComparisonCampaignRef {
    campaign_id: u64,
    receipt_sha256: String,
    provider: String,
    model: String,
    model_digest: String,
    model_qualification_sha256: String,
    completed_trials: u16,
    stats: BenchmarkCampaignStats,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CampaignComparisonReceipt {
    schema: String,
    record_status: String,
    comparison_id: u64,
    benchmark_id: String,
    gym_source_sha256: String,
    gym_rom_sha256: String,
    core_sha256: String,
    core_name: String,
    core_version: String,
    policy: AutodrivePolicy,
    total_trials: u16,
    campaign_a: ComparisonCampaignRef,
    campaign_b: ComparisonCampaignRef,
    stats: CampaignComparisonStats,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CampaignComparisonArtifact {
    receipt_path: String,
    receipt: CampaignComparisonReceipt,
}


#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BenchmarkSuiteTaskEntry {
    id: String,
    title: String,
    introduced_in_suite_id: String,
    rom_sha256: String,
    source_sha256: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BenchmarkSuiteListEntry {
    id: String,
    title: String,
    version: u16,
    task_count: u16,
    tasks: Vec<BenchmarkSuiteTaskEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SuiteTaskCampaignRef {
    task_id: String,
    task_title: String,
    campaign_id: u64,
    campaign_receipt_sha256: String,
    source_sha256: String,
    rom_sha256: String,
    stats: BenchmarkCampaignStats,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BenchmarkSuiteReportReceipt {
    schema: String,
    record_status: String,
    report_id: u64,
    suite_id: String,
    cohort_id: String,
    provider: String,
    model: String,
    model_digest: String,
    model_qualification_sha256: String,
    core_sha256: String,
    core_name: String,
    core_version: String,
    policy: AutodrivePolicy,
    trials_per_task: u16,
    tasks: Vec<SuiteTaskCampaignRef>,
    stats: BenchmarkSuiteAggregateStats,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BenchmarkSuiteReportArtifact {
    receipt_path: String,
    receipt: BenchmarkSuiteReportReceipt,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BenchmarkSuiteReportListEntry {
    report_id: u64,
    suite_id: String,
    provider: String,
    model: String,
    model_digest: String,
    core_sha256: String,
    trials_per_task: u16,
    task_count: u16,
    macro_mean_score_1000: f64,
    overall_success_rate: f64,
    receipt_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PublicSuiteTaskResult {
    task_id: String,
    task_title: String,
    campaign_id: u64,
    campaign_receipt_sha256: String,
    observed_trials: u16,
    scored_trials: u16,
    scoring_error_trials: u16,
    successful_trials: u16,
    success_rate: f64,
    mean_score_1000: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PublicSuiteResultReceipt {
    schema: String,
    record_status: String,
    suite_id: String,
    suite_version: u16,
    suite_title: String,
    report_id: u64,
    suite_report_sha256: String,
    cohort_id: String,
    provider: String,
    model: String,
    model_digest: String,
    model_qualification_sha256: String,
    core_sha256: String,
    core_name: String,
    core_version: String,
    policy: AutodrivePolicy,
    trials_per_task: u16,
    total_scored_trials: u32,
    total_scoring_error_trials: u32,
    stats: BenchmarkSuiteAggregateStats,
    tasks: Vec<PublicSuiteTaskResult>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PublicSuiteResultArtifact {
    json_path: String,
    markdown_path: String,
    json_sha256: String,
    markdown_sha256: String,
    receipt: PublicSuiteResultReceipt,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SuiteComparisonReportRef {
    report_id: u64,
    receipt_sha256: String,
    cohort_id: String,
    provider: String,
    model: String,
    model_digest: String,
    model_qualification_sha256: String,
    stats: BenchmarkSuiteAggregateStats,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SuiteTaskComparisonRef {
    task_id: String,
    task_title: String,
    campaign_a_id: u64,
    campaign_b_id: u64,
    stats: CampaignComparisonStats,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BenchmarkSuiteComparisonReceipt {
    schema: String,
    record_status: String,
    comparison_id: u64,
    suite_id: String,
    core_sha256: String,
    core_name: String,
    core_version: String,
    policy: AutodrivePolicy,
    trials_per_task: u16,
    report_a: SuiteComparisonReportRef,
    report_b: SuiteComparisonReportRef,
    stats: BenchmarkSuiteComparisonStats,
    tasks: Vec<SuiteTaskComparisonRef>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BenchmarkSuiteComparisonArtifact {
    receipt_path: String,
    receipt: BenchmarkSuiteComparisonReceipt,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SuiteCandidateTask {
    task_id: String,
    task_title: String,
    campaign_id: u64,
    mean_score_1000: f64,
    success_rate: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BenchmarkSuiteReportCandidate {
    suite_id: String,
    cohort_id: String,
    provider: String,
    model: String,
    model_digest: String,
    trials_per_task: u16,
    covered_tasks: u16,
    suite_task_count: u16,
    ready: bool,
    tasks: Vec<SuiteCandidateTask>,
}

#[derive(Debug, Clone, Serialize)]
struct SuiteCohortIdentity {
    provider: String,
    model: String,
    model_digest: String,
    model_qualification_sha256: String,
    core_sha256: String,
    core_name: String,
    core_version: String,
    policy: AutodrivePolicy,
    total_trials: u16,
}

#[derive(Debug, Clone)]
struct SuiteCohortEvidence {
    identity: SuiteCohortIdentity,
    tasks: std::collections::BTreeMap<String, (PathBuf, BenchmarkCampaignReceipt)>,
}

struct EmulatorSession {
    core: LibretroCore,
    runtime_features: RuntimeSessionFeatures,
    game_path: String,
    game_key: String,
    profile: GameProfile,
    paths: SessionPaths,
    rewind: VecDeque<StateSnapshot>,
    next_rewind_frame: u64,
    last_sram_flush_frame: u64,
    last_frame: FrameBuffer,
    recording: Option<ReplayRecording>,
    last_replay: Option<ReplayExport>,
    authority: AuthorityPolicy,
    authority_rejections: u64,
    last_authority_reason: Option<String>,
    pending_agent_turn: Option<AgentTurnRequest>,
    agent_inbox: VecDeque<ActionEnvelope>,
    next_driver_turn_id: u64,
    next_action_sequence: u64,
    agent_memory: AgentMemoryState,
    autodrive: Option<AutodriveStatus>,
    last_autodrive: Option<AutodriveExport>,
    next_autodrive_run_id: u64,
    model_benchmark: Option<ModelBenchmarkRun>,
    last_model_benchmark: Option<ModelBenchmarkExport>,
    next_model_benchmark_run_id: u64,
    benchmark_campaign: Option<BenchmarkCampaignRun>,
    last_benchmark_campaign: Option<BenchmarkCampaignExport>,
    next_benchmark_campaign_id: u64,
    next_campaign_comparison_id: u64,
}

impl Drop for EmulatorSession {
    fn drop(&mut self) {
        let _ = flush_save_ram(self);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct AppSettings {
    rom_directory: Option<String>,
    auto_scan: bool,
    controller_deadzone: f32,
    sameboy_core_path: Option<String>,
    ollama_base_url: String,
    ollama_model: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            rom_directory: None,
            auto_scan: false,
            controller_deadzone: 0.18,
            sameboy_core_path: None,
            ollama_base_url: "http://127.0.0.1:11434".to_owned(),
            ollama_model: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
struct GameProfile {
    fast_forward: u8,
    rewind_seconds: u16,
    rewind_interval_frames: u16,
    save_slot: u8,
}

impl Default for GameProfile {
    fn default() -> Self {
        Self {
            fast_forward: 1,
            rewind_seconds: 10,
            rewind_interval_frames: 30,
            save_slot: 0,
        }
    }
}

impl GameProfile {
    fn validate(&self) -> Result<(), String> {
        if !matches!(self.fast_forward, 1 | 2 | 4) {
            return Err("fast-forward multiplier must be 1, 2, or 4".into());
        }
        if !(2..=60).contains(&self.rewind_seconds) {
            return Err("rewindSeconds must be in 2..=60".into());
        }
        if !(10..=120).contains(&self.rewind_interval_frames) {
            return Err("rewindIntervalFrames must be in 10..=120".into());
        }
        if self.save_slot > 9 {
            return Err("saveSlot must be in 0..=9".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RomEntry {
    path: String,
    display_name: String,
    system: &'static str,
    extension: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BenchmarkTaskInfo {
    suite_id: String,
    suite_ids: Vec<String>,
    id: String,
    title: String,
    rom_sha256: String,
    source_sha256: String,
    start: PixelPoint,
    target: PixelPoint,
    initial_distance: i32,
    success_distance: i32,
    warmup_frames: u64,
    allowed_buttons: Vec<String>,
}

fn benchmark_task_info(task: &BenchmarkTaskSpec) -> BenchmarkTaskInfo {
    BenchmarkTaskInfo {
        suite_id: task.suite_id.into(),
        suite_ids: benchmark_suites_for_task(task.id)
            .into_iter()
            .map(|suite| suite.id.to_owned())
            .collect(),
        id: task.id.into(),
        title: task.title.into(),
        rom_sha256: task.rom_sha256.into(),
        source_sha256: task.source_sha256.into(),
        start: task.start,
        target: task.target,
        initial_distance: task.initial_distance,
        success_distance: task.success_distance,
        warmup_frames: task.warmup_frames,
        allowed_buttons: task.allowed_buttons.iter().map(|button| (*button).to_owned()).collect(),
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct RuntimeSessionFeatures {
    state_snapshots: bool,
    exact_replay: bool,
    persistent_save_data: bool,
}

fn runtime_session_features(manifest: &RuntimeCapabilityManifest) -> RuntimeSessionFeatures {
    RuntimeSessionFeatures {
        state_snapshots: manifest.capabilities.state_snapshots.is_supported(),
        exact_replay: manifest.capabilities.exact_replay.is_supported(),
        persistent_save_data: manifest.capabilities.persistent_save_data.is_supported(),
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionRouteEvidence {
    route: String,
    registered_core_sha256: Option<String>,
    registration_binary_evidence_bound: bool,
    registration_authority_granted: bool,
    session_operator_approved: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionInfo {
    game_path: String,
    game_key: String,
    core_path: String,
    core: CoreIdentity,
    capability_manifest: RuntimeCapabilityManifest,
    runtime_features: RuntimeSessionFeatures,
    route_evidence: SessionRouteEvidence,
    profile: GameProfile,
    benchmark_task: Option<BenchmarkTaskInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeRegistrationReceipt {
    schema: String,
    record_status: String,
    core_path: String,
    core_sha256: String,
    core: CoreIdentity,
    capability_manifest: RuntimeCapabilityManifest,
    qualification_profile_id: Option<String>,
    binary_evidence_bound: bool,
    authority_granted: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct FramePacket {
    frame: u64,
    width: u32,
    height: u32,
    rgba_base64: String,
    audio_base64: String,
    sample_rate_hz: u32,
    shutdown_requested: bool,
    rewind_snapshots: usize,
    fast_forward: u8,
    replay_recording: bool,
    replay_actions: usize,
    replay_checkpoints: usize,
    control_mode: ControlMode,
    authority_rejections: u64,
    last_authority_reason: Option<String>,
    driver_pending_turn_id: Option<u64>,
    driver_queued_actions: usize,
    autodrive: Option<AutodriveStatus>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AuthorityStatus {
    mode: ControlMode,
    playable_ports: u8,
    agent_id: Option<String>,
    agent_seat: Option<u8>,
    allowed_buttons: Vec<String>,
    allowed_axes: Vec<String>,
    expires_at_frame: Option<u64>,
    rejected_actions: u64,
    last_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DriverStatus {
    pending_turn_id: Option<u64>,
    queued_actions: usize,
    next_turn_id: u64,
    memory_content: String,
    memory_sha256: String,
    memory_bytes: u32,
    memory_revision: u64,
    memory_updates: u64,
    memory_bytes_written: u64,
    memory_refusals: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AutodriveArtifact {
    receipt_path: String,
    receipt: AutodriveReceipt,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReplayStatus {
    recording: bool,
    action_count: usize,
    checkpoint_count: usize,
    last_replay_path: Option<String>,
    last_receipt_path: Option<String>,
    last_replay_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReplayArtifact {
    replay_path: String,
    receipt_path: String,
    replay_sha256: String,
    receipt: ReplayReceipt,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct OllamaQualificationArtifact {
    receipt_path: String,
    receipt: OllamaQualificationReceipt,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct OllamaQualificationStatus {
    details: OllamaModelDetails,
    qualified: bool,
    receipt: Option<OllamaQualificationReceipt>,
    receipt_path: Option<String>,
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let directory = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("cannot resolve app config directory: {error}"))?;
    Ok(directory.join("settings.json"))
}

fn ollama_qualification_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("cannot resolve app data directory: {error}"))?;
    Ok(root.join("model-qualifications").join("ollama"))
}

fn ollama_qualification_path(app: &AppHandle, digest: &str) -> Result<PathBuf, String> {
    if digest.trim().is_empty() {
        return Err("model qualification requires a non-empty digest".into());
    }
    Ok(ollama_qualification_dir(app)?
        .join(format!("{}.json", sanitize_component(digest))))
}

fn ollama_qualification_archive_dir(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(ollama_qualification_dir(app)?.join("receipts"))
}

fn ollama_qualification_archive_path(
    app: &AppHandle,
    receipt_sha256: &str,
) -> Result<PathBuf, String> {
    let digest = receipt_sha256.trim().to_ascii_lowercase();
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("model qualification receipt SHA-256 must be 64 hexadecimal characters".into());
    }
    Ok(ollama_qualification_archive_dir(app)?.join(format!("{digest}.json")))
}

fn persist_ollama_qualification(
    app: &AppHandle,
    receipt: &OllamaQualificationReceipt,
) -> Result<OllamaQualificationArtifact, String> {
    let path = ollama_qualification_path(app, &receipt.digest)?;
    let parent = path
        .parent()
        .ok_or_else(|| "qualification receipt path has no parent".to_owned())?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    let json = serde_json::to_vec_pretty(receipt)
        .map_err(|error| format!("serialize Ollama qualification receipt: {error}"))?;
    let receipt_sha256 = sha256_bytes(&json);
    let archive_path = ollama_qualification_archive_path(app, &receipt_sha256)?;
    if archive_path.exists() {
        let observed = sha256_file(&archive_path)?;
        if observed != receipt_sha256 {
            return Err(format!(
                "qualification archive {} does not match its SHA-256 filename",
                archive_path.display()
            ));
        }
    } else {
        write_atomic(&archive_path, &json)?;
    }
    write_atomic(&path, &json)?;

    Ok(OllamaQualificationArtifact {
        receipt_path: path.to_string_lossy().to_string(),
        receipt: receipt.clone(),
    })
}

fn load_ollama_qualification(
    app: &AppHandle,
    digest: &str,
) -> Result<Option<OllamaQualificationArtifact>, String> {
    let path = ollama_qualification_path(app, digest)?;
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let receipt = serde_json::from_slice::<OllamaQualificationReceipt>(&bytes)
        .map_err(|error| format!("cannot parse {}: {error}", path.display()))?;

    Ok(Some(OllamaQualificationArtifact {
        receipt_path: path.to_string_lossy().to_string(),
        receipt,
    }))
}

fn load_ollama_qualification_by_sha(
    app: &AppHandle,
    digest: &str,
    expected_sha256: &str,
) -> Result<OllamaQualificationArtifact, String> {
    let archive_path = ollama_qualification_archive_path(app, expected_sha256)?;
    let path = if archive_path.exists() {
        archive_path
    } else {
        let alias = ollama_qualification_path(app, digest)?;
        if !alias.exists() {
            return Err(format!(
                "model qualification receipt {} is missing",
                expected_sha256
            ));
        }
        let observed = sha256_file(&alias)?;
        if observed != expected_sha256 {
            return Err(format!(
                "model qualification receipt {} is not available in immutable archive or current digest alias",
                expected_sha256
            ));
        }
        alias
    };

    let observed = sha256_file(&path)?;
    if observed != expected_sha256 {
        return Err(format!(
            "model qualification receipt {} hash mismatch at {}",
            expected_sha256,
            path.display()
        ));
    }
    let bytes = fs::read(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let receipt = serde_json::from_slice::<OllamaQualificationReceipt>(&bytes)
        .map_err(|error| format!("cannot parse {}: {error}", path.display()))?;

    Ok(OllamaQualificationArtifact {
        receipt_path: path.to_string_lossy().to_string(),
        receipt,
    })
}

fn qualification_receipt_passes(
    receipt: &OllamaQualificationReceipt,
    model: &str,
    digest: &str,
) -> bool {
    receipt.schema == "phicade.ollama-model-qualification.v1"
        && receipt.result == "PASS"
        && receipt.provider == "ollama"
        && receipt.model == model
        && receipt.digest == digest
        && receipt.vision_advertised
        && receipt.structured_output_pass
        && receipt.vision_probe_pass
}

#[tauri::command]
fn load_settings(app: AppHandle) -> Result<AppSettings, String> {
    let path = settings_path(&app)?;
    if !path.exists() {
        return Ok(AppSettings::default());
    }

    let json = fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;

    serde_json::from_str(&json)
        .map_err(|error| format!("cannot parse {}: {error}", path.display()))
}

#[tauri::command]
fn save_settings(app: AppHandle, settings: AppSettings) -> Result<(), String> {
    let path = settings_path(&app)?;
    let parent = path
        .parent()
        .ok_or_else(|| "settings path has no parent directory".to_owned())?;

    fs::create_dir_all(parent)
        .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;

    let json = serde_json::to_string_pretty(&settings)
        .map_err(|error| format!("cannot serialize settings: {error}"))?;

    fs::write(&path, json)
        .map_err(|error| format!("cannot write {}: {error}", path.display()))
}

fn classify_extension(extension: &str) -> Option<&'static str> {
    match extension {
        "nes" => Some("NES"),
        "sfc" | "smc" => Some("SNES"),
        "gb" => Some("GB"),
        "gbc" => Some("GBC"),
        "gba" => Some("GBA"),
        "md" | "gen" | "32x" => Some("GENESIS"),
        "cue" | "chd" | "pbp" => Some("PS1"),
        _ => None,
    }
}

fn system_id_from_extension(extension: &str) -> Option<SystemId> {
    match extension {
        "gb" => Some(SystemId::GameBoy),
        "gbc" => Some(SystemId::GameBoyColor),
        _ => None,
    }
}

fn entry_from_path(path: &Path) -> Option<RomEntry> {
    let extension = path.extension()?.to_string_lossy().to_ascii_lowercase();
    let system = classify_extension(&extension)?;
    let display_name = path.file_stem()?.to_string_lossy().to_string();

    Some(RomEntry {
        path: path.to_string_lossy().to_string(),
        display_name,
        system,
        extension,
    })
}

#[tauri::command]
fn scan_rom_directory(path: String) -> Result<Vec<RomEntry>, String> {
    let root = PathBuf::from(&path)
        .canonicalize()
        .map_err(|error| format!("cannot resolve selected directory: {error}"))?;

    if !root.is_dir() {
        return Err("selected path is not a directory".to_owned());
    }

    let entries = fs::read_dir(&root)
        .map_err(|error| format!("cannot read {}: {error}", root.display()))?;

    let mut games = Vec::new();

    for entry in entries.take(MAX_LIBRARY_ENTRIES) {
        let entry = entry.map_err(|error| format!("cannot read directory entry: {error}"))?;
        let candidate = entry.path();

        if candidate.is_file() {
            if let Some(game) = entry_from_path(&candidate) {
                games.push(game);
            }
        }
    }

    games.sort_by(|left, right| {
        left.system
            .cmp(right.system)
            .then_with(|| left.display_name.to_lowercase().cmp(&right.display_name.to_lowercase()))
    });

    Ok(games)
}

#[tauri::command]
fn validate_action_envelope(envelope: ActionEnvelope) -> Result<ActionEnvelope, String> {
    envelope.validate()?;
    Ok(envelope)
}

fn runtime_registry_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("cannot resolve app data directory: {error}"))?;
    let registry = root.join("runtime-registry");
    fs::create_dir_all(&registry)
        .map_err(|error| format!("cannot create {}: {error}", registry.display()))?;
    Ok(registry)
}

fn runtime_registration_receipt(
    core_path: &Path,
    core_sha256: String,
    core: CoreIdentity,
    capability_manifest: RuntimeCapabilityManifest,
) -> RuntimeRegistrationReceipt {
    let qualification_profile_id = capability_manifest
        .qualification_profile
        .as_ref()
        .map(|profile| profile.profile_id.clone());

    RuntimeRegistrationReceipt {
        schema: RUNTIME_REGISTRATION_SCHEMA.into(),
        record_status: "REGISTERED".into(),
        core_path: core_path.to_string_lossy().to_string(),
        core_sha256,
        core,
        capability_manifest,
        qualification_profile_id,
        binary_evidence_bound: false,
        authority_granted: false,
    }
}

#[tauri::command]
fn register_runtime_core(
    app: AppHandle,
    core_path: String,
) -> Result<RuntimeRegistrationReceipt, String> {
    let core_file = PathBuf::from(&core_path)
        .canonicalize()
        .map_err(|error| format!("cannot resolve selected core: {error}"))?;
    if !core_file.is_file() {
        return Err("selected runtime core is not a file".into());
    }

    let core_sha256 = sha256_file(&core_file)?;
    let (_, system_dir, save_dir) = app_runtime_dirs(&app)?;
    let core = LibretroCore::open(&core_file, &system_dir, &save_dir)
        .map_err(|error| format!("cannot inspect libretro core: {error:?}"))?;
    let identity = core.identity().clone();
    let capability_manifest = core.capability_manifest();
    drop(core);

    let receipt = runtime_registration_receipt(
        &core_file,
        core_sha256.clone(),
        identity,
        capability_manifest,
    );

    let registry = runtime_registry_dir(&app)?;
    let path = registry.join(format!("{core_sha256}.json"));
    let json = serde_json::to_vec_pretty(&receipt)
        .map_err(|error| format!("cannot serialize runtime registration: {error}"))?;
    write_atomic(&path, &json)?;

    Ok(receipt)
}

#[tauri::command]
fn list_runtime_registrations(
    app: AppHandle,
) -> Result<Vec<RuntimeRegistrationReceipt>, String> {
    let registry = runtime_registry_dir(&app)?;
    let mut receipts = Vec::new();

    for entry in fs::read_dir(&registry)
        .map_err(|error| format!("cannot list {}: {error}", registry.display()))?
    {
        let entry = entry.map_err(|error| format!("cannot read runtime registry entry: {error}"))?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let bytes = fs::read(&path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        let receipt = serde_json::from_slice::<RuntimeRegistrationReceipt>(&bytes)
            .map_err(|error| format!("cannot parse {}: {error}", path.display()))?;
        if receipt.schema != RUNTIME_REGISTRATION_SCHEMA {
            return Err(format!(
                "unsupported runtime registration schema in {}: {}",
                path.display(),
                receipt.schema
            ));
        }
        receipts.push(receipt);
    }

    receipts.sort_by(|left, right| {
        left.core
            .library_name
            .cmp(&right.core.library_name)
            .then_with(|| left.core.library_version.cmp(&right.core.library_version))
            .then_with(|| left.core_sha256.cmp(&right.core_sha256))
    });
    Ok(receipts)
}

fn load_runtime_registration(
    app: &AppHandle,
    core_sha256: &str,
) -> Result<RuntimeRegistrationReceipt, String> {
    let digest = core_sha256.trim().to_ascii_lowercase();
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("runtime registration requires a 64-character SHA-256".into());
    }

    let path = runtime_registry_dir(app)?.join(format!("{digest}.json"));
    let bytes = fs::read(&path)
        .map_err(|error| format!("cannot read runtime registration {}: {error}", path.display()))?;
    let receipt = serde_json::from_slice::<RuntimeRegistrationReceipt>(&bytes)
        .map_err(|error| format!("cannot parse runtime registration {}: {error}", path.display()))?;

    if receipt.schema != RUNTIME_REGISTRATION_SCHEMA {
        return Err(format!(
            "unsupported runtime registration schema: {}",
            receipt.schema
        ));
    }
    if receipt.core_sha256.to_ascii_lowercase() != digest {
        return Err("runtime registration digest does not match its registry filename".into());
    }

    Ok(receipt)
}

#[tauri::command]
fn start_registered_emulation(
    app: AppHandle,
    state: State<'_, EmulatorState>,
    core_sha256: String,
    content_path: Option<String>,
    operator_approved: bool,
) -> Result<SessionInfo, String> {
    if !operator_approved {
        return Err("registered runtime launch requires explicit operator approval".into());
    }

    let registration = load_runtime_registration(&app, &core_sha256)?;
    let core_file = PathBuf::from(&registration.core_path)
        .canonicalize()
        .map_err(|error| format!("cannot resolve registered core: {error}"))?;
    let observed_core_sha256 = sha256_file(&core_file)?;
    if !observed_core_sha256.eq_ignore_ascii_case(&registration.core_sha256) {
        return Err(format!(
            "registered runtime binary changed: expected {}, observed {}",
            registration.core_sha256, observed_core_sha256
        ));
    }

    let selected_content = match content_path {
        Some(path) => {
            let resolved = PathBuf::from(path)
                .canonicalize()
                .map_err(|error| format!("cannot resolve selected content: {error}"))?;
            if !resolved.is_file() {
                return Err("registered libretro sessions currently accept FILE content only".into());
            }
            Some(resolved)
        }
        None => None,
    };

    {
        let mut session = state
            .session
            .lock()
            .map_err(|_| "emulator session lock poisoned".to_owned())?;
        *session = None;
    }

    let (root, system_dir, save_dir) = app_runtime_dirs(&app)?;
    let mut core = LibretroCore::open(&core_file, &system_dir, &save_dir)
        .map_err(|error| format!("cannot open registered libretro core: {error:?}"))?;
    let identity = core.identity().clone();

    if identity.library_name != registration.core.library_name
        || identity.library_version != registration.core.library_version
    {
        return Err(format!(
            "registered runtime identity changed: expected {} {}, observed {} {}",
            registration.core.library_name,
            registration.core.library_version,
            identity.library_name,
            identity.library_version
        ));
    }

    let (game_path, game_key, benchmark_task) = if let Some(content) = selected_content.as_ref() {
        let display_name = content
            .file_stem()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| "libretro content".to_owned());
        let descriptor = ContentDescriptor::file(content, display_name)
            .with_runtime_hint(identity.library_name.clone());
        core.load_content(&descriptor)
            .map_err(|error| format!("cannot load registered runtime content: {error:?}"))?;
        let key = sha256_file(content)?;
        let benchmark = benchmark_task_by_rom_sha256(&key).map(benchmark_task_info);
        (content.to_string_lossy().to_string(), key, benchmark)
    } else {
        if !core.supports_no_game() {
            return Err("registered runtime does not support a no-content launcher session".into());
        }
        core.load_no_content()
            .map_err(|error| format!("cannot start registered runtime launcher: {error:?}"))?;
        let key = sha256_bytes(format!("launcher:{}", registration.core_sha256).as_bytes());
        (
            format!("launcher://{}-{}", identity.library_name, identity.library_version),
            key,
            None,
        )
    };

    let capability_manifest = core.capability_manifest();
    if !capability_manifest.capabilities.frame_step.is_supported()
        || !capability_manifest.capabilities.rendered_framebuffer.is_supported()
        || !capability_manifest.capabilities.governed_actions.is_supported()
    {
        return Err("registered runtime lacks the minimum governed frame-session capabilities".into());
    }
    let runtime_features = runtime_session_features(&capability_manifest);

    let paths = session_paths(&root, &game_key, &identity)?;
    if runtime_features.persistent_save_data {
        restore_save_ram(&mut core, &paths.save_ram)?;
    }
    let profile = load_game_profile(&paths.profile)?;
    persist_game_profile(&paths.profile, &profile)?;

    let info = SessionInfo {
        game_path: game_path.clone(),
        game_key: game_key.clone(),
        core_path: core_file.to_string_lossy().to_string(),
        core: identity,
        capability_manifest,
        runtime_features: runtime_features.clone(),
        route_evidence: SessionRouteEvidence {
            route: "registered-libretro".into(),
            registered_core_sha256: Some(registration.core_sha256.clone()),
            registration_binary_evidence_bound: registration.binary_evidence_bound,
            registration_authority_granted: registration.authority_granted,
            session_operator_approved: true,
        },
        profile: profile.clone(),
        benchmark_task,
    };

    let next_autodrive_run_id =
        next_numbered_receipt_id(&paths.autodrive_dir, "run-", ".json")?;
    let next_model_benchmark_run_id =
        next_numbered_receipt_id(&paths.model_benchmark_dir, "run-", ".json")?;
    let next_benchmark_campaign_id =
        next_numbered_receipt_id(&paths.benchmark_campaign_dir, "campaign-", ".json")?;
    let next_campaign_comparison_id =
        next_numbered_receipt_id(&paths.campaign_comparison_dir, "comparison-", ".json")?;

    let mut emulator_session = EmulatorSession {
        core,
        runtime_features: runtime_features.clone(),
        game_path,
        game_key,
        profile,
        paths,
        rewind: VecDeque::new(),
        next_rewind_frame: 0,
        last_sram_flush_frame: 0,
        last_frame: FrameBuffer::default(),
        recording: None,
        last_replay: None,
        authority: AuthorityPolicy::new(1),
        authority_rejections: 0,
        last_authority_reason: None,
        pending_agent_turn: None,
        agent_inbox: VecDeque::new(),
        next_driver_turn_id: 1,
        next_action_sequence: 0,
        agent_memory: AgentMemoryState::new(),
        autodrive: None,
        last_autodrive: None,
        next_autodrive_run_id,
        model_benchmark: None,
        last_model_benchmark: None,
        next_model_benchmark_run_id,
        benchmark_campaign: None,
        last_benchmark_campaign: None,
        next_benchmark_campaign_id,
        next_campaign_comparison_id,
    };

    if emulator_session.runtime_features.state_snapshots {
        push_rewind_snapshot(&mut emulator_session)?;
        emulator_session.next_rewind_frame =
            u64::from(emulator_session.profile.rewind_interval_frames);
    }

    let mut session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    *session = Some(emulator_session);

    Ok(info)
}

fn app_runtime_dirs(app: &AppHandle) -> Result<(PathBuf, PathBuf, PathBuf), String> {
    let root = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("cannot resolve app data directory: {error}"))?;
    let system = root.join("system");
    let saves = root.join("saves");
    fs::create_dir_all(&system)
        .map_err(|error| format!("cannot create {}: {error}", system.display()))?;
    fs::create_dir_all(&saves)
        .map_err(|error| format!("cannot create {}: {error}", saves.display()))?;
    Ok((root, system, saves))
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path)
        .map_err(|error| format!("cannot hash {}: {error}", path.display()))?;
    Ok(sha256_bytes(&bytes))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn sanitize_component(value: &str) -> String {
    let cleaned: String = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                character
            } else {
                '-'
            }
        })
        .collect();
    let trimmed = cleaned.trim_matches('-');
    if trimmed.is_empty() {
        "unknown".to_owned()
    } else {
        trimmed.to_owned()
    }
}


fn next_numbered_receipt_id(
    directory: &Path,
    prefix: &str,
    suffix: &str,
) -> Result<u64, String> {
    let mut max_id = 0u64;
    let entries = fs::read_dir(directory)
        .map_err(|error| format!("cannot list {}: {error}", directory.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("cannot read directory entry: {error}"))?;
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if !name.starts_with(prefix) || !name.ends_with(suffix) {
            continue;
        }
        let numeric = &name[prefix.len()..name.len() - suffix.len()];
        if let Ok(id) = numeric.parse::<u64>() {
            max_id = max_id.max(id);
        }
    }
    Ok(max_id.saturating_add(1).max(1))
}

fn session_paths(
    root: &Path,
    game_key: &str,
    identity: &CoreIdentity,
) -> Result<SessionPaths, String> {
    let core_namespace = format!(
        "{}-{}",
        sanitize_component(&identity.library_name),
        sanitize_component(&identity.library_version)
    );
    let state_dir = root.join("states").join(game_key).join(core_namespace);
    let screenshot_dir = root.join("screenshots").join(game_key);
    let profile_dir = root.join("profiles");
    let replay_dir = root.join("replays").join(game_key);
    let autodrive_root = root.join("autodrive");
    let autodrive_dir = autodrive_root.join(game_key);
    let model_benchmark_root = root.join("model-benchmarks");
    let benchmark_campaign_root = root.join("benchmark-campaigns");
    let model_benchmark_dir = model_benchmark_root.join(game_key);
    let benchmark_campaign_dir = benchmark_campaign_root.join(game_key);
    let campaign_comparison_dir = root.join("campaign-comparisons").join(game_key);
    let suite_report_root = root.join("suite-reports");
    let suite_comparison_root = root.join("suite-comparisons");

    fs::create_dir_all(&state_dir)
        .map_err(|error| format!("cannot create {}: {error}", state_dir.display()))?;
    fs::create_dir_all(&screenshot_dir)
        .map_err(|error| format!("cannot create {}: {error}", screenshot_dir.display()))?;
    fs::create_dir_all(&profile_dir)
        .map_err(|error| format!("cannot create {}: {error}", profile_dir.display()))?;
    fs::create_dir_all(&replay_dir)
        .map_err(|error| format!("cannot create {}: {error}", replay_dir.display()))?;
    fs::create_dir_all(&autodrive_dir)
        .map_err(|error| format!("cannot create {}: {error}", autodrive_dir.display()))?;
    fs::create_dir_all(&model_benchmark_dir)
        .map_err(|error| format!("cannot create {}: {error}", model_benchmark_dir.display()))?;
    fs::create_dir_all(&benchmark_campaign_dir)
        .map_err(|error| format!("cannot create {}: {error}", benchmark_campaign_dir.display()))?;
    fs::create_dir_all(&campaign_comparison_dir)
        .map_err(|error| format!("cannot create {}: {error}", campaign_comparison_dir.display()))?;
    fs::create_dir_all(&suite_report_root)
        .map_err(|error| format!("cannot create {}: {error}", suite_report_root.display()))?;
    fs::create_dir_all(&suite_comparison_root)
        .map_err(|error| format!("cannot create {}: {error}", suite_comparison_root.display()))?;
    for suite in benchmark_suites() {
        fs::create_dir_all(suite_report_root.join(suite.id))
            .map_err(|error| format!("cannot create suite report namespace {}: {error}", suite.id))?;
        fs::create_dir_all(suite_comparison_root.join(suite.id))
            .map_err(|error| format!("cannot create suite comparison namespace {}: {error}", suite.id))?;
    }

    Ok(SessionPaths {
        save_ram: root.join("saves").join(format!("{game_key}.srm")),
        state_dir,
        screenshot_dir,
        replay_dir,
        autodrive_dir,
        autodrive_root,
        model_benchmark_dir,
        model_benchmark_root,
        benchmark_campaign_root,
        benchmark_campaign_dir,
        campaign_comparison_dir,
        suite_report_root,
        suite_comparison_root,
        profile: profile_dir.join(format!("{game_key}.json")),
    })
}

fn load_game_profile(path: &Path) -> Result<GameProfile, String> {
    if !path.exists() {
        return Ok(GameProfile::default());
    }
    let json = fs::read_to_string(path)
        .map_err(|error| format!("cannot read profile {}: {error}", path.display()))?;
    let profile: GameProfile = serde_json::from_str(&json)
        .map_err(|error| format!("cannot parse profile {}: {error}", path.display()))?;
    profile.validate()?;
    Ok(profile)
}

fn persist_game_profile(path: &Path, profile: &GameProfile) -> Result<(), String> {
    profile.validate()?;
    let json = serde_json::to_string_pretty(profile)
        .map_err(|error| format!("cannot serialize game profile: {error}"))?;
    write_atomic(path, json.as_bytes())
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    }
    let temp = path.with_extension("tmp");
    fs::write(&temp, bytes)
        .map_err(|error| format!("cannot write {}: {error}", temp.display()))?;
    if path.exists() {
        fs::remove_file(path)
            .map_err(|error| format!("cannot replace {}: {error}", path.display()))?;
    }
    fs::rename(&temp, path)
        .map_err(|error| format!("cannot replace {}: {error}", path.display()))
}

fn flush_save_ram(session: &EmulatorSession) -> Result<(), String> {
    let bytes = session
        .core
        .read_save_ram()
        .map_err(|error| format!("read save RAM: {error:?}"))?;
    if bytes.is_empty() {
        return Ok(());
    }
    write_atomic(&session.paths.save_ram, &bytes)
}

fn restore_save_ram(core: &mut LibretroCore, path: &Path) -> Result<(), String> {
    if !path.exists() || core.save_ram_size() == 0 {
        return Ok(());
    }
    let bytes = fs::read(path)
        .map_err(|error| format!("cannot read save RAM {}: {error}", path.display()))?;
    core.write_save_ram(&bytes)
        .map_err(|error| format!("restore save RAM: {error:?}"))
}

fn state_path(session: &EmulatorSession, slot: u8) -> PathBuf {
    session.paths.state_dir.join(format!("slot-{slot}.state"))
}

fn encode_state(frame: u64, state: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(STATE_MAGIC.len() + 8 + state.len());
    bytes.extend_from_slice(STATE_MAGIC);
    bytes.extend_from_slice(&frame.to_le_bytes());
    bytes.extend_from_slice(state);
    bytes
}

fn decode_state(bytes: &[u8]) -> Result<(u64, &[u8]), String> {
    let header_len = STATE_MAGIC.len() + 8;
    if bytes.len() < header_len || !bytes.starts_with(STATE_MAGIC) {
        return Err("invalid PhiCade state header".into());
    }
    let frame_offset = STATE_MAGIC.len();
    let frame = u64::from_le_bytes(
        bytes[frame_offset..frame_offset + 8]
            .try_into()
            .map_err(|_| "invalid PhiCade state frame header")?,
    );
    Ok((frame, &bytes[header_len..]))
}

fn save_state_slot(session: &mut EmulatorSession, slot: u8) -> Result<(), String> {
    if slot > 9 {
        return Err("state slot must be in 0..=9".into());
    }
    let state = session
        .core
        .serialize_state()
        .map_err(|error| format!("serialize slot {slot}: {error:?}"))?;
    let payload = encode_state(session.core.frame_count(), &state);
    write_atomic(&state_path(session, slot), &payload)
}

fn load_state_slot(session: &mut EmulatorSession, slot: u8) -> Result<(), String> {
    if slot > 9 {
        return Err("state slot must be in 0..=9".into());
    }
    let path = state_path(session, slot);
    let payload = fs::read(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let (frame, state) = decode_state(&payload)?;
    session
        .core
        .restore_state(state, frame)
        .map_err(|error| format!("restore slot {slot}: {error:?}"))?;
    session.rewind.clear();
    session.next_rewind_frame =
        session.core.frame_count() + u64::from(session.profile.rewind_interval_frames);
    push_rewind_snapshot(session)?;
    Ok(())
}

fn rewind_capacity(session: &EmulatorSession) -> usize {
    let fps = session.core.identity().fps.max(1.0);
    let interval = f64::from(session.profile.rewind_interval_frames.max(1));
    ((f64::from(session.profile.rewind_seconds) * fps / interval).ceil() as usize).max(2) + 1
}

fn push_rewind_snapshot(session: &mut EmulatorSession) -> Result<(), String> {
    let state = session
        .core
        .serialize_state()
        .map_err(|error| format!("serialize rewind snapshot: {error:?}"))?;
    session.rewind.push_back(StateSnapshot {
        frame: session.core.frame_count(),
        bytes: state,
    });
    let capacity = rewind_capacity(session);
    while session.rewind.len() > capacity {
        session.rewind.pop_front();
    }
    Ok(())
}

fn rewind_session(session: &mut EmulatorSession, seconds: u8) -> Result<(), String> {
    if session.rewind.is_empty() {
        return Err("rewind buffer is empty".into());
    }

    let fps = session.core.identity().fps.max(1.0).round() as u64;
    let target = session
        .core
        .frame_count()
        .saturating_sub(fps.saturating_mul(u64::from(seconds.max(1))));

    while session.rewind.len() > 1 {
        let should_pop = session
            .rewind
            .back()
            .is_some_and(|snapshot| snapshot.frame > target);
        if should_pop {
            session.rewind.pop_back();
        } else {
            break;
        }
    }

    let snapshot = session
        .rewind
        .back()
        .cloned()
        .ok_or_else(|| "rewind buffer unexpectedly empty".to_owned())?;

    session
        .core
        .restore_state(&snapshot.bytes, snapshot.frame)
        .map_err(|error| format!("rewind restore failed: {error:?}"))?;
    session.next_rewind_frame =
        snapshot.frame + u64::from(session.profile.rewind_interval_frames);
    Ok(())
}


fn replay_status_for(session: &EmulatorSession) -> ReplayStatus {
    let (action_count, checkpoint_count) = session
        .recording
        .as_ref()
        .map(|recording| {
            (
                recording.ledger.actions.len(),
                recording.ledger.checkpoints.len(),
            )
        })
        .or_else(|| {
            session.last_replay.as_ref().map(|replay| {
                (
                    replay.receipt.action_count,
                    replay.receipt.checkpoint_count,
                )
            })
        })
        .unwrap_or((0, 0));

    ReplayStatus {
        recording: session.recording.is_some(),
        action_count,
        checkpoint_count,
        last_replay_path: session
            .last_replay
            .as_ref()
            .map(|replay| replay.replay_path.to_string_lossy().to_string()),
        last_receipt_path: session
            .last_replay
            .as_ref()
            .map(|replay| replay.receipt_path.to_string_lossy().to_string()),
        last_replay_sha256: session
            .last_replay
            .as_ref()
            .map(|replay| replay.replay_sha256.clone()),
    }
}

fn make_replay_checkpoint(
    session: &EmulatorSession,
    frame: &FrameBuffer,
) -> Result<ReplayCheckpoint, String> {
    let state = session
        .core
        .serialize_state()
        .map_err(|error| format!("serialize replay checkpoint: {error:?}"))?;

    Ok(ReplayCheckpoint {
        frame: session.core.frame_count(),
        state_sha256: sha256_bytes(&state),
        frame_sha256: sha256_bytes(&frame.rgba8),
        input_mask: session.core.input_mask_snapshot(),
    })
}

fn upsert_replay_checkpoint(
    session: &mut EmulatorSession,
    checkpoint: ReplayCheckpoint,
) -> Result<(), String> {
    let recording = session
        .recording
        .as_mut()
        .ok_or_else(|| "replay recorder is not active".to_owned())?;

    if recording
        .ledger
        .checkpoints
        .last()
        .is_some_and(|last| last.frame == checkpoint.frame)
    {
        if let Some(last) = recording.ledger.checkpoints.last_mut() {
            *last = checkpoint;
        }
    } else {
        recording.ledger.checkpoints.push(checkpoint);
    }

    Ok(())
}

fn maybe_record_replay_checkpoint(session: &mut EmulatorSession) -> Result<(), String> {
    let frame = session.core.frame_count();
    let due = session
        .recording
        .as_ref()
        .is_some_and(|recording| frame >= recording.next_checkpoint_frame);

    if !due {
        return Ok(());
    }

    let checkpoint = make_replay_checkpoint(session, &session.last_frame)?;
    upsert_replay_checkpoint(session, checkpoint)?;

    if let Some(recording) = session.recording.as_mut() {
        recording.next_checkpoint_frame =
            frame.saturating_add(REPLAY_CHECKPOINT_INTERVAL_FRAMES);
    }

    Ok(())
}

fn validate_recording_actions(
    session: &EmulatorSession,
    actions: &[ActionEnvelope],
) -> Result<(), String> {
    if session.recording.is_none() {
        return Ok(());
    }

    for envelope in actions {
        if let ActionKind::System { command, .. } = &envelope.action {
            return Err(format!(
                "{command:?} is disabled while Replay v1 recording is active; v1 records controller/axis input only"
            ));
        }
    }

    Ok(())
}

fn record_applied_actions(
    session: &mut EmulatorSession,
    actions: &[ActionEnvelope],
) -> Result<(), String> {
    if session.recording.is_none() || actions.is_empty() {
        return Ok(());
    }

    let applied_frame = session.core.frame_count();
    let mut normalized = Vec::with_capacity(actions.len());

    for envelope in actions {
        let mut event = envelope.clone();
        event.frame = applied_frame;
        event.validate()?;
        normalized.push(event);
    }

    let recording = session
        .recording
        .as_mut()
        .ok_or_else(|| "replay recorder disappeared".to_owned())?;
    recording.ledger.actions.extend(normalized);
    Ok(())
}

fn replay_receipt(
    ledger: &ReplayLedger,
    replay_sha256: &str,
    verification: Option<ReplayVerification>,
) -> ReplayReceipt {
    ReplayReceipt {
        schema: REPLAY_RECEIPT_SCHEMA.to_owned(),
        replay_sha256: replay_sha256.to_owned(),
        game_sha256: ledger.game_sha256.clone(),
        core_name: ledger.core_name.clone(),
        core_version: ledger.core_version.clone(),
        core_sha256: ledger.core_sha256.clone(),
        start_frame: ledger.start_frame,
        end_frame: ledger.end_frame,
        action_count: ledger.actions.len(),
        checkpoint_count: ledger.checkpoints.len(),
        verification,
    }
}

fn artifact_view(export: &ReplayExport) -> ReplayArtifact {
    ReplayArtifact {
        replay_path: export.replay_path.to_string_lossy().to_string(),
        receipt_path: export.receipt_path.to_string_lossy().to_string(),
        replay_sha256: export.replay_sha256.clone(),
        receipt: export.receipt.clone(),
    }
}

fn finalize_replay_recording(session: &mut EmulatorSession) -> Result<ReplayExport, String> {
    let start_frame = session
        .recording
        .as_ref()
        .ok_or_else(|| "replay recorder is not active".to_owned())?
        .ledger
        .start_frame;
    if session.core.frame_count() <= start_frame {
        return Err("record at least one emulated frame before exporting the replay".into());
    }

    let mut recording = session
        .recording
        .take()
        .ok_or_else(|| "replay recorder is not active".to_owned())?;

    let final_checkpoint = make_replay_checkpoint(session, &session.last_frame)?;
    if recording
        .ledger
        .checkpoints
        .last()
        .is_some_and(|last| last.frame == final_checkpoint.frame)
    {
        if let Some(last) = recording.ledger.checkpoints.last_mut() {
            *last = final_checkpoint.clone();
        }
    } else {
        recording.ledger.checkpoints.push(final_checkpoint.clone());
    }

    recording.ledger.end_frame = session.core.frame_count();
    recording.ledger.final_state_sha256 = final_checkpoint.state_sha256.clone();
    recording.ledger.final_frame_sha256 = final_checkpoint.frame_sha256.clone();
    recording.ledger.validate()?;

    let replay_json = serde_json::to_vec_pretty(&recording.ledger)
        .map_err(|error| format!("serialize replay ledger: {error}"))?;
    let replay_sha256 = sha256_bytes(&replay_json);
    let replay_path = session
        .paths
        .replay_dir
        .join(format!("{replay_sha256}.replay.json"));
    let receipt_path = session
        .paths
        .replay_dir
        .join(format!("{replay_sha256}.receipt.json"));

    write_atomic(&replay_path, &replay_json)?;

    let receipt = replay_receipt(&recording.ledger, &replay_sha256, None);
    let receipt_json = serde_json::to_vec_pretty(&receipt)
        .map_err(|error| format!("serialize replay receipt: {error}"))?;
    write_atomic(&receipt_path, &receipt_json)?;

    let export = ReplayExport {
        replay_path,
        receipt_path,
        replay_sha256,
        receipt,
    };
    session.last_replay = Some(export.clone());
    Ok(export)
}

fn replay_divergence(
    checked_checkpoints: usize,
    frame: u64,
    expected: &ReplayCheckpoint,
    actual: &ReplayCheckpoint,
) -> ReplayVerification {
    ReplayVerification {
        result: ReplayVerificationResult::Diverged,
        checked_checkpoints,
        first_divergence_frame: Some(frame),
        expected_state_sha256: Some(expected.state_sha256.clone()),
        actual_state_sha256: Some(actual.state_sha256.clone()),
        expected_frame_sha256: Some(expected.frame_sha256.clone()),
        actual_frame_sha256: Some(actual.frame_sha256.clone()),
    }
}

fn compare_replay_checkpoint(
    expected: &ReplayCheckpoint,
    actual: &ReplayCheckpoint,
    checked_checkpoints: usize,
    compare_framebuffer: bool,
) -> Option<ReplayVerification> {
    let state_differs = expected.state_sha256 != actual.state_sha256;
    let input_differs = expected.input_mask != actual.input_mask;
    let frame_differs =
        compare_framebuffer && expected.frame_sha256 != actual.frame_sha256;

    if state_differs || input_differs || frame_differs {
        Some(replay_divergence(
            checked_checkpoints,
            expected.frame,
            expected,
            actual,
        ))
    } else {
        None
    }
}

fn verify_replay_execution(
    session: &mut EmulatorSession,
    ledger: &ReplayLedger,
) -> Result<ReplayVerification, String> {
    ledger.validate()?;

    if ledger.game_sha256 != session.game_key {
        return Err("replay game fingerprint does not match running game".into());
    }
    if ledger.core_name != session.core.identity().library_name
        || ledger.core_version != session.core.identity().library_version
    {
        return Err("replay core identity/version does not match running core".into());
    }

    let running_core_sha256 = sha256_file(session.core.core_path())?;
    if ledger.core_sha256 != running_core_sha256 {
        return Err("replay core binary hash does not match running core".into());
    }

    let initial_state = BASE64
        .decode(&ledger.initial_state_base64)
        .map_err(|error| format!("decode replay initial state: {error}"))?;
    session
        .core
        .restore_state(&initial_state, ledger.start_frame)
        .map_err(|error| format!("restore replay initial state: {error:?}"))?;
    session.core.restore_input_mask(ledger.initial_input_mask);

    let mut checked = 0usize;
    let mut checkpoint_index = 0usize;
    let mut video = FrameBuffer::default();
    let mut audio = AudioBuffer::default();

    if let Some(expected) = ledger.checkpoints.first() {
        if expected.frame == ledger.start_frame {
            let actual = make_replay_checkpoint(session, &FrameBuffer::default())?;
            checked += 1;

            let state_or_input_differs =
                expected.state_sha256 != actual.state_sha256
                    || expected.input_mask != actual.input_mask;

            if state_or_input_differs {
                return Ok(replay_divergence(
                    checked,
                    expected.frame,
                    expected,
                    &actual,
                ));
            }
            checkpoint_index = 1;
        }
    }

    let mut action_index = 0usize;

    while session.core.frame_count() < ledger.end_frame {
        let current_frame = session.core.frame_count();
        let mut frame_actions = Vec::new();

        while let Some(action) = ledger.actions.get(action_index) {
            if action.frame != current_frame {
                break;
            }

            if let ActionKind::System { command, .. } = &action.action {
                return Err(format!(
                    "Replay v1 contains unsupported system command {command:?}"
                ));
            }

            let mut replay_action = action.clone();
            replay_action.source = ActionSource::Replay;
            frame_actions.push(replay_action);
            action_index += 1;
        }

        session
            .core
            .step_frame(&frame_actions, &mut video, &mut audio)
            .map_err(|error| format!("verify replay frame: {error:?}"))?;

        while let Some(expected) = ledger.checkpoints.get(checkpoint_index) {
            if expected.frame > session.core.frame_count() {
                break;
            }
            if expected.frame < session.core.frame_count() {
                return Err(format!(
                    "replay checkpoint {} was skipped at runtime frame {}",
                    expected.frame,
                    session.core.frame_count()
                ));
            }

            let actual = make_replay_checkpoint(session, &video)?;
            checked += 1;
            if let Some(divergence) =
                compare_replay_checkpoint(expected, &actual, checked, true)
            {
                return Ok(divergence);
            }
            checkpoint_index += 1;
        }
    }

    let final_state = session
        .core
        .serialize_state()
        .map_err(|error| format!("serialize replay final state: {error:?}"))?;
    let final_state_sha256 = sha256_bytes(&final_state);
    let final_frame_sha256 = sha256_bytes(&video.rgba8);

    if final_state_sha256 != ledger.final_state_sha256
        || final_frame_sha256 != ledger.final_frame_sha256
    {
        let expected = ReplayCheckpoint {
            frame: ledger.end_frame,
            state_sha256: ledger.final_state_sha256.clone(),
            frame_sha256: ledger.final_frame_sha256.clone(),
            input_mask: session.core.input_mask_snapshot(),
        };
        let actual = ReplayCheckpoint {
            frame: ledger.end_frame,
            state_sha256: final_state_sha256,
            frame_sha256: final_frame_sha256,
            input_mask: session.core.input_mask_snapshot(),
        };
        return Ok(replay_divergence(
            checked.saturating_add(1),
            ledger.end_frame,
            &expected,
            &actual,
        ));
    }

    Ok(ReplayVerification {
        result: ReplayVerificationResult::Pass,
        checked_checkpoints: checked,
        first_divergence_frame: None,
        expected_state_sha256: None,
        actual_state_sha256: None,
        expected_frame_sha256: None,
        actual_frame_sha256: None,
    })
}

fn process_session_actions(
    session: &mut EmulatorSession,
    actions: &[ActionEnvelope],
) -> Result<(), String> {
    for envelope in actions {
        match &envelope.action {
            ActionKind::System { command, .. }
                if matches!(
                    command,
                    SystemCommand::SaveState | SystemCommand::LoadState | SystemCommand::Rewind
                ) && !session.runtime_features.state_snapshots =>
            {
                return Err(format!(
                    "{command:?} is unavailable: the active runtime does not support state snapshots"
                ));
            }
            ActionKind::System {
                command: SystemCommand::SaveState,
                slot,
            } => {
                let selected = slot.unwrap_or(session.profile.save_slot);
                session.profile.save_slot = selected;
                persist_game_profile(&session.paths.profile, &session.profile)?;
                save_state_slot(session, selected)?;
            }
            ActionKind::System {
                command: SystemCommand::LoadState,
                slot,
            } => {
                let selected = slot.unwrap_or(session.profile.save_slot);
                session.profile.save_slot = selected;
                persist_game_profile(&session.paths.profile, &session.profile)?;
                load_state_slot(session, selected)?;
            }
            ActionKind::System {
                command: SystemCommand::Rewind,
                slot,
            } => {
                rewind_session(session, slot.unwrap_or(2).max(1))?;
            }
            _ => {}
        }
    }
    Ok(())
}

#[tauri::command]
fn start_emulation(
    app: AppHandle,
    state: State<'_, EmulatorState>,
    core_path: String,
    game_path: String,
) -> Result<SessionInfo, String> {
    let game = PathBuf::from(&game_path)
        .canonicalize()
        .map_err(|error| format!("cannot resolve selected game: {error}"))?;
    let core_file = PathBuf::from(&core_path)
        .canonicalize()
        .map_err(|error| format!("cannot resolve selected core: {error}"))?;

    let extension = game
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| "selected game has no supported extension".to_owned())?;
    let system = system_id_from_extension(&extension)
        .ok_or_else(|| "Rung 4 SameBoy sessions accept .gb or .gbc images only".to_owned())?;
    let display_name = game
        .file_stem()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| "Game Boy image".to_owned());
    let game_key = sha256_file(&game)?;

    {
        let mut session = state
            .session
            .lock()
            .map_err(|_| "emulator session lock poisoned".to_owned())?;
        *session = None;
    }

    let (root, system_dir, save_dir) = app_runtime_dirs(&app)?;
    let mut core = LibretroCore::open(&core_file, &system_dir, &save_dir)
        .map_err(|error| format!("cannot open libretro core: {error:?}"))?;
    let identity = core.identity().clone();

    if !identity.library_name.to_ascii_lowercase().contains("sameboy") {
        return Err(format!(
            "Rung 4 qualification expects SameBoy, got {} {}",
            identity.library_name, identity.library_version
        ));
    }

    core.load_game(&GameImage::new(&game, system, display_name))
        .map_err(|error| format!("cannot load game image: {error:?}"))?;
    let capability_manifest = core.capability_manifest();
    let runtime_features = runtime_session_features(&capability_manifest);

    let paths = session_paths(&root, &game_key, &identity)?;
    restore_save_ram(&mut core, &paths.save_ram)?;
    let profile = load_game_profile(&paths.profile)?;
    persist_game_profile(&paths.profile, &profile)?;

    let info = SessionInfo {
        game_path: game.to_string_lossy().to_string(),
        game_key: game_key.clone(),
        core_path: core_file.to_string_lossy().to_string(),
        core: identity,
        capability_manifest,
        runtime_features: runtime_features.clone(),
        route_evidence: SessionRouteEvidence {
            route: "legacy-direct-sameboy".into(),
            registered_core_sha256: None,
            registration_binary_evidence_bound: false,
            registration_authority_granted: false,
            session_operator_approved: true,
        },
        profile: profile.clone(),
        benchmark_task: benchmark_task_by_rom_sha256(&game_key).map(benchmark_task_info),
    };

    let next_autodrive_run_id =
        next_numbered_receipt_id(&paths.autodrive_dir, "run-", ".json")?;
    let next_model_benchmark_run_id =
        next_numbered_receipt_id(&paths.model_benchmark_dir, "run-", ".json")?;
    let next_benchmark_campaign_id =
        next_numbered_receipt_id(&paths.benchmark_campaign_dir, "campaign-", ".json")?;
    let next_campaign_comparison_id =
        next_numbered_receipt_id(&paths.campaign_comparison_dir, "comparison-", ".json")?;

    let mut emulator_session = EmulatorSession {
        core,
        runtime_features: runtime_features.clone(),
        game_path: info.game_path.clone(),
        game_key,
        profile,
        paths,
        rewind: VecDeque::new(),
        next_rewind_frame: 0,
        last_sram_flush_frame: 0,
        last_frame: FrameBuffer::default(),
        recording: None,
        last_replay: None,
        authority: AuthorityPolicy::new(1),
        authority_rejections: 0,
        last_authority_reason: None,
        pending_agent_turn: None,
        agent_inbox: VecDeque::new(),
        next_driver_turn_id: 1,
        next_action_sequence: 0,
        agent_memory: AgentMemoryState::new(),
        autodrive: None,
        last_autodrive: None,
        next_autodrive_run_id,
        model_benchmark: None,
        last_model_benchmark: None,
        next_model_benchmark_run_id,
        benchmark_campaign: None,
        last_benchmark_campaign: None,
        next_benchmark_campaign_id,
        next_campaign_comparison_id,
    };

    if emulator_session.runtime_features.state_snapshots {
        push_rewind_snapshot(&mut emulator_session)?;
        emulator_session.next_rewind_frame =
            u64::from(emulator_session.profile.rewind_interval_frames);
    }

    let mut session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    *session = Some(emulator_session);

    Ok(info)
}

fn resample_stereo(input: &[f32], source_rate: u32, target_rate: u32) -> Vec<f32> {
    let source_frames = input.len() / 2;
    if source_frames == 0 || source_rate == 0 || target_rate == 0 {
        return Vec::new();
    }
    if source_rate == target_rate {
        return input[..source_frames * 2].to_vec();
    }

    let output_frames =
        ((source_frames as u64 * target_rate as u64 + source_rate as u64 - 1)
            / source_rate as u64)
            .max(1) as usize;
    let ratio = source_rate as f64 / target_rate as f64;
    let mut output = Vec::with_capacity(output_frames * 2);

    for output_frame in 0..output_frames {
        let position = (output_frame as f64 * ratio).min((source_frames - 1) as f64);
        let left_index = position.floor() as usize;
        let right_index = (left_index + 1).min(source_frames - 1);
        let fraction = (position - left_index as f64) as f32;

        for channel in 0..2 {
            let left = input[left_index * 2 + channel];
            let right = input[right_index * 2 + channel];
            output.push(left + (right - left) * fraction);
        }
    }

    output
}


fn control_mode_label(mode: ControlMode) -> &'static str {
    match mode {
        ControlMode::Human => "human",
        ControlMode::PhiBot => "phi-bot",
        ControlMode::Coop => "coop",
        ControlMode::Versus => "versus",
    }
}

fn authority_status_for(session: &EmulatorSession) -> AuthorityStatus {
    let grant = session.authority.agent_grant.as_ref();
    AuthorityStatus {
        mode: session.authority.mode,
        playable_ports: session.authority.playable_ports,
        agent_id: grant.map(|value| value.agent_id.clone()),
        agent_seat: grant.map(|value| value.seat),
        allowed_buttons: grant
            .map(|value| value.allowed_buttons.iter().cloned().collect())
            .unwrap_or_default(),
        allowed_axes: grant
            .map(|value| value.allowed_axes.iter().cloned().collect())
            .unwrap_or_default(),
        expires_at_frame: grant.and_then(|value| value.expires_at_frame),
        rejected_actions: session.authority_rejections,
        last_reason: session.last_authority_reason.clone(),
    }
}

#[tauri::command]
fn authority_status(state: State<'_, EmulatorState>) -> Result<AuthorityStatus, String> {
    let session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_ref()
        .ok_or_else(|| "no emulator session is running".to_owned())?;
    Ok(authority_status_for(session))
}

#[tauri::command]
fn set_control_mode(
    state: State<'_, EmulatorState>,
    mode: ControlMode,
    agent_id: Option<String>,
    allowed_buttons: Option<Vec<String>>,
    grant_frames: Option<u64>,
) -> Result<AuthorityStatus, String> {
    let mut session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_mut()
        .ok_or_else(|| "no emulator session is running".to_owned())?;

    if session.recording.is_some() {
        return Err("control-mode changes are disabled during replay recording".into());
    }

    if session
        .autodrive
        .as_ref()
        .is_some_and(|status| status.active)
    {
        let reason = if mode == ControlMode::Human {
            AutodriveStopReason::HumanTakeover
        } else {
            AutodriveStopReason::OperatorStop
        };
        let _ = finish_autodrive(session, reason)?;
    }

    let grant = match mode {
        ControlMode::Human => None,
        ControlMode::PhiBot | ControlMode::Coop | ControlMode::Versus => {
            let agent_id = agent_id
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| "Phi-Bot modes require a non-empty agentId".to_owned())?;
            let seat = if mode == ControlMode::Versus { 2 } else { 1 };
            let mut grant = AgentGrant::game_boy(agent_id, seat);

            if let Some(buttons) = allowed_buttons {
                let known = ["A", "B", "SELECT", "START", "UP", "DOWN", "LEFT", "RIGHT"];
                let normalized: Vec<String> = buttons
                    .into_iter()
                    .map(|button| button.trim().to_ascii_uppercase())
                    .collect();
                if normalized.iter().any(|button| !known.contains(&button.as_str())) {
                    return Err("allowedButtons contains an unsupported Game Boy button".into());
                }
                grant.allowed_buttons = normalized.into_iter().collect();
            }

            let lifetime = grant_frames.unwrap_or(3_600);
            if !(1..=216_000).contains(&lifetime) {
                return Err("grantFrames must be in 1..=216000".into());
            }
            grant.expires_at_frame = Some(session.core.frame_count().saturating_add(lifetime));
            Some(grant)
        }
    };

    session.authority.set_mode(mode, grant)?;
    session.pending_agent_turn = None;
    session.agent_inbox.clear();
    session.agent_memory = AgentMemoryState::new();
    session.core.restore_input_mask(0);
    session.last_authority_reason = Some(format!(
        "operator set control mode to {}",
        control_mode_label(mode)
    ));
    Ok(authority_status_for(session))
}

fn build_phi_bot_observation(
    session: &EmulatorSession,
    agent_id: String,
    seat: u8,
) -> Result<PhiBotObservation, String> {
    let grant = session
        .authority
        .agent_grant
        .as_ref()
        .ok_or_else(|| "no Phi-Bot grant is active".to_owned())?;

    if grant.agent_id != agent_id || grant.seat != seat {
        return Err("observation request does not match the active Phi-Bot grant".into());
    }
    if grant
        .expires_at_frame
        .is_some_and(|expires| session.core.frame_count() > expires)
    {
        return Err("Phi-Bot grant has expired".into());
    }
    if session.last_frame.width == 0
        || session.last_frame.height == 0
        || session.last_frame.rgba8.is_empty()
    {
        return Err("no rendered framebuffer is available yet".into());
    }

    let observation = PhiBotObservation {
        schema: PHIBOT_OBSERVATION_SCHEMA.to_owned(),
        frame: session.core.frame_count(),
        width: session.last_frame.width,
        height: session.last_frame.height,
        rgba_base64: BASE64.encode(&session.last_frame.rgba8),
        frame_sha256: sha256_bytes(&session.last_frame.rgba8),
        input_mask: session.core.input_mask_snapshot(),
        game_sha256: session.game_key.clone(),
        core_name: session.core.identity().library_name.clone(),
        core_version: session.core.identity().library_version.clone(),
        agent_id,
        seat,
        control_mode: control_mode_label(session.authority.mode).to_owned(),
        allowed_buttons: grant.allowed_buttons.iter().cloned().collect(),
        allowed_axes: grant.allowed_axes.iter().cloned().collect(),
        expires_at_frame: grant.expires_at_frame,
    };
    observation.validate()?;
    Ok(observation)
}

#[tauri::command]
fn phi_bot_observation(
    state: State<'_, EmulatorState>,
    agent_id: String,
    seat: u8,
) -> Result<PhiBotObservation, String> {
    let session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_ref()
        .ok_or_else(|| "no emulator session is running".to_owned())?;
    build_phi_bot_observation(session, agent_id, seat)
}

fn autodrive_status_for(session: &EmulatorSession) -> Option<AutodriveStatus> {
    session.autodrive.as_ref().map(|status| {
        let mut snapshot = status.clone();
        snapshot.current_frame = session.core.frame_count();
        snapshot
    })
}

fn autodrive_artifact(export: &AutodriveExport) -> AutodriveArtifact {
    AutodriveArtifact {
        receipt_path: export.receipt_path.to_string_lossy().to_string(),
        receipt: export.receipt.clone(),
    }
}

fn model_benchmark_artifact(export: &ModelBenchmarkExport) -> ModelBenchmarkArtifact {
    ModelBenchmarkArtifact {
        receipt_path: export.receipt_path.to_string_lossy().to_string(),
        receipt: export.receipt.clone(),
    }
}

fn benchmark_campaign_status_for(
    campaign: &BenchmarkCampaignRun,
) -> BenchmarkCampaignStatus {
    BenchmarkCampaignStatus {
        schema: BENCHMARK_CAMPAIGN_SCHEMA.to_owned(),
        campaign_id: campaign.campaign_id,
        active: campaign.active,
        total_trials: campaign.total_trials,
        completed_trials: u16::try_from(campaign.trials.len()).unwrap_or(u16::MAX),
        provider: campaign.provider.clone(),
        model: campaign.model.clone(),
        model_digest: campaign.model_digest.clone(),
        policy: campaign.policy.clone(),
    }
}

fn benchmark_campaign_artifact(
    export: &BenchmarkCampaignExport,
) -> BenchmarkCampaignArtifact {
    BenchmarkCampaignArtifact {
        receipt_path: export.receipt_path.to_string_lossy().to_string(),
        receipt: export.receipt.clone(),
    }
}

fn finish_benchmark_campaign(
    session: &mut EmulatorSession,
    record_status: &str,
) -> Result<BenchmarkCampaignArtifact, String> {
    let snapshot = session
        .benchmark_campaign
        .as_ref()
        .ok_or_else(|| "no benchmark campaign has been started".to_owned())?
        .clone();

    let outcomes: Vec<BenchmarkTrialOutcome> = snapshot
        .trials
        .iter()
        .map(|trial| BenchmarkTrialOutcome {
            score_1000: trial.score_1000,
            task_success: trial.task_success,
        })
        .collect();
    let stats = summarize_benchmark_trials(&outcomes);

    let receipt = BenchmarkCampaignReceipt {
        schema: BENCHMARK_CAMPAIGN_SCHEMA.to_owned(),
        record_status: record_status.to_owned(),
        campaign_id: snapshot.campaign_id,
        benchmark_id: snapshot.benchmark_id.clone(),
        provider: snapshot.provider,
        model: snapshot.model,
        model_digest: snapshot.model_digest,
        model_qualification_sha256: snapshot.model_qualification_sha256,
        gym_source_sha256: snapshot.benchmark_source_sha256.clone(),
        gym_rom_sha256: snapshot.benchmark_rom_sha256.clone(),
        core_sha256: snapshot.core_sha256,
        core_name: session.core.identity().library_name.clone(),
        core_version: session.core.identity().library_version.clone(),
        policy: snapshot.policy,
        total_trials: snapshot.total_trials,
        completed_trials: u16::try_from(snapshot.trials.len()).unwrap_or(u16::MAX),
        trials: snapshot.trials,
        stats,
    };

    let receipt_path = session
        .paths
        .benchmark_campaign_dir
        .join(format!("campaign-{:06}.json", receipt.campaign_id));
    let json = serde_json::to_vec_pretty(&receipt)
        .map_err(|error| format!("serialize benchmark campaign receipt: {error}"))?;
    write_atomic(&receipt_path, &json)?;

    if let Some(campaign) = session.benchmark_campaign.as_mut() {
        campaign.active = false;
    }

    let export = BenchmarkCampaignExport {
        receipt_path,
        receipt,
    };
    session.last_benchmark_campaign = Some(export.clone());
    Ok(benchmark_campaign_artifact(&export))
}

fn note_benchmark_campaign_trial(
    session: &mut EmulatorSession,
    export: &ModelBenchmarkExport,
) -> Result<(), String> {
    let Some(campaign) = session
        .benchmark_campaign
        .as_ref()
        .filter(|campaign| campaign.active)
        .cloned()
    else {
        return Ok(());
    };

    let receipt = &export.receipt;
    if receipt.provider != campaign.provider
        || receipt.model != campaign.model
        || receipt.model_digest != campaign.model_digest
        || receipt.model_qualification_sha256 != campaign.model_qualification_sha256
        || receipt.core_sha256 != campaign.core_sha256
        || receipt.benchmark_id != campaign.benchmark_id
        || receipt.gym_source_sha256 != campaign.benchmark_source_sha256
        || receipt.gym_rom_sha256 != campaign.benchmark_rom_sha256
        || receipt.policy != campaign.policy
    {
        return Err("benchmark campaign trial does not match pinned campaign evidence".into());
    }

    let evidence = CampaignTrialEvidence {
        benchmark_run_id: receipt.benchmark_run_id,
        receipt_sha256: sha256_file(&export.receipt_path)?,
        record_status: receipt.record_status.clone(),
        score_1000: receipt.score_1000,
        task_success: receipt.task_success,
        stop_reason: receipt.stop_reason,
    };

    let completed = {
        let campaign = session
            .benchmark_campaign
            .as_mut()
            .ok_or_else(|| "benchmark campaign disappeared during trial finalization".to_owned())?;
        if campaign.trials.len() >= usize::from(campaign.total_trials) {
            return Err("benchmark campaign received more trials than configured".into());
        }
        campaign.trials.push(evidence);
        campaign.trials.len() == usize::from(campaign.total_trials)
    };

    if completed {
        let _ = finish_benchmark_campaign(session, "COMPLETE")?;
    }
    Ok(())
}

fn finish_model_benchmark(
    session: &mut EmulatorSession,
    autodrive: &AutodriveExport,
) -> Result<Option<ModelBenchmarkArtifact>, String> {
    let Some(run) = session.model_benchmark.clone() else {
        return Ok(None);
    };
    if run.autodrive_run_id != autodrive.receipt.run_id {
        return Ok(None);
    }

    let task = benchmark_task_by_id(&run.benchmark_id)
        .ok_or_else(|| format!("benchmark registry no longer contains {}", run.benchmark_id))?;
    let scored: Result<AgentGymScore, String> =
        score_benchmark_task_frame(&session.last_frame, task);
    let (
        record_status,
        final_player,
        final_distance,
        progress,
        score_1000,
        task_success,
        scoring_error,
    ) = match scored {
        Ok(score) => (
            "COMPLETE".to_owned(),
            Some(score.player),
            Some(score.final_distance),
            Some(score.progress),
            Some(score.score_1000),
            score.success,
            None,
        ),
        Err(error) => (
            "SCORING_ERROR".to_owned(),
            None,
            None,
            None,
            None,
            false,
            Some(error),
        ),
    };

    let receipt = ModelGameplayBenchmarkReceipt {
        schema: MODEL_GAMEPLAY_BENCHMARK_SCHEMA.to_owned(),
        record_status,
        benchmark_id: run.benchmark_id.clone(),
        benchmark_run_id: run.run_id,
        provider: run.provider,
        model: run.model,
        model_digest: run.model_digest,
        model_qualification_sha256: run.model_qualification_sha256,
        gym_source_sha256: run.benchmark_source_sha256.clone(),
        gym_rom_sha256: run.benchmark_rom_sha256.clone(),
        core_sha256: run.core_sha256,
        core_name: session.core.identity().library_name.clone(),
        core_version: session.core.identity().library_version.clone(),
        autodrive_receipt_sha256: sha256_file(&autodrive.receipt_path)?,
        autodrive_run_id: autodrive.receipt.run_id,
        policy: autodrive.receipt.policy.clone(),
        stop_reason: autodrive.receipt.stop_reason,
        started_frame: run.started_frame,
        ended_frame: autodrive.receipt.ended_frame,
        start_player: run.start_player,
        final_player,
        target: run.target,
        initial_distance: run.initial_distance,
        final_distance,
        progress,
        score_1000,
        task_success,
        turns_issued: autodrive.receipt.turns_issued,
        turns_completed: autodrive.receipt.turns_completed,
        total_actions: autodrive.receipt.total_actions,
        final_frame_sha256: autodrive.receipt.final_frame_sha256.clone(),
        scoring_error,
    };

    let receipt_path = session
        .paths
        .model_benchmark_dir
        .join(format!("run-{:06}.json", run.run_id));
    let json = serde_json::to_vec_pretty(&receipt)
        .map_err(|error| format!("serialize model gameplay benchmark receipt: {error}"))?;
    write_atomic(&receipt_path, &json)?;

    let export = ModelBenchmarkExport {
        receipt_path,
        receipt,
    };
    session.last_model_benchmark = Some(export.clone());
    session.model_benchmark = None;
    note_benchmark_campaign_trial(session, &export)?;
    Ok(Some(model_benchmark_artifact(&export)))
}

fn finish_autodrive(
    session: &mut EmulatorSession,
    reason: AutodriveStopReason,
) -> Result<AutodriveArtifact, String> {
    let current_frame = session.core.frame_count();
    let memory = session.agent_memory.clone();
    let status = session
        .autodrive
        .as_mut()
        .ok_or_else(|| "no autonomous run has been started".to_owned())?;

    if !status.active {
        if let Some(last) = session.last_autodrive.as_ref() {
            return Ok(autodrive_artifact(last));
        }
        return Err("autonomous run is already stopped".into());
    }

    status.current_frame = current_frame;
    status.active = false;
    status.stop_reason = Some(reason);

    session.pending_agent_turn = None;
    session.agent_inbox.clear();
    session.core.restore_input_mask(0);

    let receipt = AutodriveReceipt {
        schema: AUTODRIVE_RECEIPT_SCHEMA.to_owned(),
        run_id: status.run_id,
        provider: status.provider.clone(),
        model: status.model.clone(),
        game_sha256: session.game_key.clone(),
        core_name: session.core.identity().library_name.clone(),
        core_version: session.core.identity().library_version.clone(),
        started_frame: status.started_frame,
        ended_frame: current_frame,
        turns_issued: status.turns_issued,
        turns_completed: status.turns_completed,
        total_actions: status.total_actions,
        total_scheduled_cadence_wait_frames: status.total_scheduled_cadence_wait_frames,
        max_scheduled_cadence_wait_frames: status.max_scheduled_cadence_wait_frames,
        last_observation_frame: status.last_observation_frame,
        stop_reason: reason,
        final_frame_sha256: sha256_bytes(&session.last_frame.rgba8),
        initial_memory_sha256: memory.initial_sha256.clone(),
        final_memory_sha256: memory.sha256(),
        final_memory_content: memory.content.clone(),
        final_memory_bytes: memory.bytes(),
        memory_revision: memory.revision,
        memory_updates: memory.updates,
        memory_bytes_written: memory.bytes_written,
        memory_refusals: memory.refusals,
        policy: status.policy.clone(),
    };

    let receipt_path = session
        .paths
        .autodrive_dir
        .join(format!("run-{:06}.json", status.run_id));
    let json = serde_json::to_vec_pretty(&receipt)
        .map_err(|error| format!("serialize autonomous run receipt: {error}"))?;
    write_atomic(&receipt_path, &json)?;

    let export = AutodriveExport {
        receipt_path,
        receipt,
    };
    session.last_autodrive = Some(export.clone());
    let _ = finish_model_benchmark(session, &export)?;
    session.agent_memory = AgentMemoryState::new();
    Ok(autodrive_artifact(&export))
}

fn autodrive_pre_turn_guard(session: &mut EmulatorSession) -> Result<(), String> {
    let current_frame = session.core.frame_count();
    let reason = session.autodrive.as_mut().and_then(|status| {
        if !status.active {
            return None;
        }
        status.current_frame = current_frame;
        status.pre_turn_stop_reason()
    });

    if let Some(reason) = reason {
        let _ = finish_autodrive(session, reason)?;
        return Err(format!("autonomous run stopped: {reason:?}"));
    }

    Ok(())
}

fn autodrive_post_step_guard(session: &mut EmulatorSession) -> Result<(), String> {
    let current_frame = session.core.frame_count();
    let driver_idle = session.pending_agent_turn.is_none() && session.agent_inbox.is_empty();
    let benchmark_success = session
        .model_benchmark
        .as_ref()
        .and_then(|run| benchmark_task_by_id(&run.benchmark_id))
        .and_then(|task| score_benchmark_task_frame(&session.last_frame, task).ok())
        .map(|score| score.success)
        .unwrap_or(false);
    let reason = session.autodrive.as_mut().and_then(|status| {
        if !status.active {
            return None;
        }
        status.current_frame = current_frame;

        if benchmark_success {
            return Some(AutodriveStopReason::TaskSuccess);
        }

        if status.frame_span() >= status.policy.max_emulated_frames {
            return Some(AutodriveStopReason::FrameBudget);
        }

        if driver_idle {
            if status.turns_completed >= status.policy.max_turns {
                return Some(AutodriveStopReason::TurnBudget);
            }
            if status.total_actions >= status.policy.max_total_actions {
                return Some(AutodriveStopReason::ActionBudget);
            }
            if let Some(reason) = status.post_turn_stop_reason() {
                return Some(reason);
            }
        }

        None
    });

    if let Some(reason) = reason {
        let _ = finish_autodrive(session, reason)?;
    }

    Ok(())
}

fn start_autodrive_inner(
    app: &AppHandle,
    session: &mut EmulatorSession,
    provider: String,
    model: String,
    model_digest: String,
    policy: AutodrivePolicy,
) -> Result<(AutodriveStatus, OllamaQualificationArtifact), String> {
    if session.recording.is_some() {
        return Err("autonomous driving is disabled during replay recording".into());
    }
    if session.profile.fast_forward != 1 {
        return Err("autonomous driving requires 1x game speed".into());
    }
    if session.authority.mode != ControlMode::PhiBot {
        return Err("autonomous driving requires PHI-BOT handoff mode".into());
    }
    if session.pending_agent_turn.is_some() || !session.agent_inbox.is_empty() {
        return Err("driver must be idle before starting autonomous mode".into());
    }
    if session
        .autodrive
        .as_ref()
        .is_some_and(|status| status.active)
    {
        return Err("an autonomous run is already active".into());
    }

    let provider = provider.trim().to_ascii_lowercase();
    if provider != "ollama" {
        return Err("autonomous mode currently supports the ollama provider".into());
    }
    let model = model.trim().to_owned();
    if model.is_empty() {
        return Err("autonomous driving requires a selected model".into());
    }
    let model_digest = model_digest.trim().to_owned();
    if model_digest.is_empty() {
        return Err("autonomous driving requires a qualified model digest".into());
    }
    let artifact = load_ollama_qualification(app, &model_digest)?
        .ok_or_else(|| "AUTO DRIVE requires a qualification receipt for this model digest".to_owned())?;
    if !qualification_receipt_passes(&artifact.receipt, &model, &model_digest) {
        return Err("AUTO DRIVE qualification receipt does not match the selected model digest".into());
    }
    policy.validate()?;
    if policy.policy_version != 1 {
        return Err("new autonomous runs require Autodrive policyVersion 1".into());
    }
    session.agent_memory = AgentMemoryState::new();

    let frame = session.core.frame_count();
    let status = AutodriveStatus {
        schema: AUTODRIVE_STATUS_SCHEMA.to_owned(),
        run_id: session.next_autodrive_run_id,
        active: true,
        provider,
        model: format!("{model}@{model_digest}"),
        started_frame: frame,
        current_frame: frame,
        turns_issued: 0,
        turns_completed: 0,
        total_actions: 0,
        consecutive_empty_turns: 0,
        next_observation_frame: frame,
        last_observation_frame: None,
        total_scheduled_cadence_wait_frames: 0,
        max_scheduled_cadence_wait_frames: 0,
        policy,
        stop_reason: None,
    };
    status.validate()?;

    session.next_autodrive_run_id = session.next_autodrive_run_id.saturating_add(1);
    session.autodrive = Some(status.clone());
    Ok((status, artifact))
}

#[tauri::command]
fn start_autodrive(
    app: AppHandle,
    state: State<'_, EmulatorState>,
    provider: String,
    model: String,
    model_digest: String,
    policy: AutodrivePolicy,
) -> Result<AutodriveStatus, String> {
    let mut session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_mut()
        .ok_or_else(|| "no emulator session is running".to_owned())?;

    start_autodrive_inner(&app, session, provider, model, model_digest, policy)
        .map(|(status, _)| status)
}

fn start_model_gameplay_benchmark_inner(
    app: &AppHandle,
    session: &mut EmulatorSession,
    provider: String,
    model: String,
    model_digest: String,
    policy: AutodrivePolicy,
) -> Result<ModelBenchmarkStart, String> {
    let task = benchmark_task_by_rom_sha256(&session.game_key)
        .ok_or_else(|| "model gameplay benchmark requires a registered benchmark-suite ROM".to_owned())?;
    if session
        .model_benchmark
        .as_ref()
        .is_some()
        || session
            .autodrive
            .as_ref()
            .is_some_and(|status| status.active)
    {
        return Err("a benchmark or autonomous run is already active".into());
    }
    if session.recording.is_some() {
        return Err("model benchmark is disabled during replay recording".into());
    }
    if session.profile.fast_forward != 1 {
        return Err("model benchmark requires 1x game speed".into());
    }
    if session.authority.mode != ControlMode::PhiBot {
        return Err("model benchmark requires PHI-BOT handoff mode".into());
    }

    policy.validate()?;
    let model = model.trim().to_owned();
    let model_digest = model_digest.trim().to_owned();
    let qualification = load_ollama_qualification(&app, &model_digest)?
        .ok_or_else(|| "model benchmark requires a qualification receipt for this digest".to_owned())?;
    if !qualification_receipt_passes(&qualification.receipt, &model, &model_digest) {
        return Err("model benchmark qualification does not match the selected model digest".into());
    }
    let qualification_sha256 = sha256_file(Path::new(&qualification.receipt_path))?;

    session.pending_agent_turn = None;
    session.agent_inbox.clear();
    session.core.restore_input_mask(0);
    session
        .core
        .reset()
        .map_err(|error| format!("reset benchmark {}: {error:?}", task.id))?;
    session.last_frame = FrameBuffer::default();
    session.rewind.clear();
    session.next_rewind_frame = 0;

    let mut warmup_audio = AudioBuffer::default();
    for _ in 0..task.warmup_frames {
        session
            .core
            .step_frame(&[], &mut session.last_frame, &mut warmup_audio)
            .map_err(|error| format!("warm benchmark {}: {error:?}", task.id))?;
    }

    let start_score = score_benchmark_task_frame(&session.last_frame, task)?;
    if start_score.player != task.start
        || start_score.final_distance != task.initial_distance
    {
        return Err(format!(
            "benchmark {} start geometry drifted: expected ({}, {}) / {} got ({}, {}) / {}",
            task.id,
            task.start.x,
            task.start.y,
            task.initial_distance,
            start_score.player.x,
            start_score.player.y,
            start_score.final_distance
        ));
    }

    let agent_id = session
        .authority
        .agent_grant
        .as_ref()
        .map(|grant| grant.agent_id.clone())
        .ok_or_else(|| "model benchmark requires an active Phi-Bot grant".to_owned())?;

    let mut grant = AgentGrant::game_boy(agent_id, 1);
    grant.allowed_buttons = task
        .allowed_buttons
        .iter()
        .map(|button| (*button).to_owned())
        .collect();
    grant.expires_at_frame = Some(
        session
            .core
            .frame_count()
            .saturating_add(policy.max_emulated_frames)
            .saturating_add(600),
    );
    session.authority.set_mode(ControlMode::PhiBot, Some(grant))?;

    let core_sha256 = sha256_file(session.core.core_path())?;
    let benchmark_run_id = session.next_model_benchmark_run_id;
    let (autodrive, _) = start_autodrive_inner(
        &app,
        session,
        provider.clone(),
        model.clone(),
        model_digest.clone(),
        policy,
    )?;

    session.next_model_benchmark_run_id =
        session.next_model_benchmark_run_id.saturating_add(1);
    session.model_benchmark = Some(ModelBenchmarkRun {
        run_id: benchmark_run_id,
        autodrive_run_id: autodrive.run_id,
        benchmark_id: task.id.into(),
        benchmark_source_sha256: task.source_sha256.into(),
        benchmark_rom_sha256: task.rom_sha256.into(),
        provider: provider.trim().to_ascii_lowercase(),
        model,
        model_digest,
        model_qualification_sha256: qualification_sha256,
        core_sha256,
        started_frame: session.core.frame_count(),
        start_player: start_score.player,
        target: task.target,
        initial_distance: task.initial_distance,
    });

    Ok(ModelBenchmarkStart {
        benchmark_run_id,
        autodrive,
    })

}

#[tauri::command]
fn start_model_gameplay_benchmark(
    app: AppHandle,
    state: State<'_, EmulatorState>,
    provider: String,
    model: String,
    model_digest: String,
    policy: AutodrivePolicy,
) -> Result<ModelBenchmarkStart, String> {
    let mut session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_mut()
        .ok_or_else(|| "no emulator session is running".to_owned())?;

    if session
        .benchmark_campaign
        .as_ref()
        .is_some_and(|campaign| campaign.active)
    {
        return Err("single benchmark start is disabled while a campaign is active".into());
    }

    start_model_gameplay_benchmark_inner(
        &app,
        session,
        provider,
        model,
        model_digest,
        policy,
    )
}

fn validate_campaign_trial_count(total_trials: u16) -> Result<(), String> {
    if !(MIN_CAMPAIGN_TRIALS..=MAX_CAMPAIGN_TRIALS).contains(&total_trials) {
        return Err(format!(
            "benchmark campaign trials must be in {}..={}",
            MIN_CAMPAIGN_TRIALS, MAX_CAMPAIGN_TRIALS
        ));
    }
    Ok(())
}

async fn inspect_campaign_model_digest(
    base_url: &str,
    model: &str,
    expected_digest: &str,
) -> Result<(), String> {
    let details = ollama::inspect_model(base_url, model).await?;
    if details.digest != expected_digest {
        return Err(format!(
            "benchmark campaign digest drift: expected {}, installed {}",
            expected_digest, details.digest
        ));
    }
    Ok(())
}

#[tauri::command]
async fn start_benchmark_campaign(
    app: AppHandle,
    state: State<'_, EmulatorState>,
    base_url: String,
    provider: String,
    model: String,
    model_digest: String,
    policy: AutodrivePolicy,
    total_trials: u16,
) -> Result<BenchmarkCampaignStart, String> {
    validate_campaign_trial_count(total_trials)?;
    let provider_normalized = provider.trim().to_ascii_lowercase();
    if provider_normalized != "ollama" {
        return Err("benchmark campaigns currently support the ollama provider".into());
    }
    inspect_campaign_model_digest(&base_url, &model, &model_digest).await?;

    let mut session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_mut()
        .ok_or_else(|| "no emulator session is running".to_owned())?;

    if session
        .benchmark_campaign
        .as_ref()
        .is_some_and(|campaign| campaign.active)
    {
        return Err("a benchmark campaign is already active".into());
    }

    let benchmark = start_model_gameplay_benchmark_inner(
        &app,
        session,
        provider_normalized,
        model,
        model_digest,
        policy,
    )?;
    let run = session
        .model_benchmark
        .as_ref()
        .ok_or_else(|| "benchmark start did not create native benchmark state".to_owned())?
        .clone();

    let campaign_id = session.next_benchmark_campaign_id;
    session.next_benchmark_campaign_id =
        session.next_benchmark_campaign_id.saturating_add(1);
    session.last_benchmark_campaign = None;
    session.benchmark_campaign = Some(BenchmarkCampaignRun {
        campaign_id,
        active: true,
        total_trials,
        benchmark_id: run.benchmark_id,
        benchmark_source_sha256: run.benchmark_source_sha256,
        benchmark_rom_sha256: run.benchmark_rom_sha256,
        provider: run.provider,
        model: run.model,
        model_digest: run.model_digest,
        model_qualification_sha256: run.model_qualification_sha256,
        core_sha256: run.core_sha256,
        policy: benchmark.autodrive.policy.clone(),
        trials: Vec::with_capacity(usize::from(total_trials)),
    });

    let status = benchmark_campaign_status_for(
        session
            .benchmark_campaign
            .as_ref()
            .ok_or_else(|| "benchmark campaign state disappeared".to_owned())?,
    );
    Ok(BenchmarkCampaignStart { status, benchmark })
}

#[tauri::command]
async fn continue_benchmark_campaign(
    app: AppHandle,
    state: State<'_, EmulatorState>,
    base_url: String,
) -> Result<BenchmarkCampaignStart, String> {
    let snapshot = {
        let session = state
            .session
            .lock()
            .map_err(|_| "emulator session lock poisoned".to_owned())?;
        let session = session
            .as_ref()
            .ok_or_else(|| "no emulator session is running".to_owned())?;
        let campaign = session
            .benchmark_campaign
            .as_ref()
            .filter(|campaign| campaign.active)
            .ok_or_else(|| "no active benchmark campaign".to_owned())?;
        if campaign.trials.len() >= usize::from(campaign.total_trials) {
            return Err("benchmark campaign already has all configured trials".into());
        }
        campaign.clone()
    };

    inspect_campaign_model_digest(&base_url, &snapshot.model, &snapshot.model_digest).await?;

    let mut session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_mut()
        .ok_or_else(|| "no emulator session is running".to_owned())?;
    let current = session
        .benchmark_campaign
        .as_ref()
        .filter(|campaign| campaign.active)
        .ok_or_else(|| "benchmark campaign stopped before continuation".to_owned())?;

    if current.campaign_id != snapshot.campaign_id
        || current.model_digest != snapshot.model_digest
        || current.model_qualification_sha256 != snapshot.model_qualification_sha256
        || current.core_sha256 != snapshot.core_sha256
        || current.benchmark_id != snapshot.benchmark_id
        || current.benchmark_source_sha256 != snapshot.benchmark_source_sha256
        || current.benchmark_rom_sha256 != snapshot.benchmark_rom_sha256
        || current.policy != snapshot.policy
    {
        return Err("benchmark campaign pins changed before continuation".into());
    }
    if session.model_benchmark.is_some()
        || session
            .autodrive
            .as_ref()
            .is_some_and(|status| status.active)
    {
        return Err("current campaign trial has not finished".into());
    }

    let qualification = load_ollama_qualification(&app, &snapshot.model_digest)?
        .ok_or_else(|| "campaign qualification receipt disappeared".to_owned())?;
    if !qualification_receipt_passes(
        &qualification.receipt,
        &snapshot.model,
        &snapshot.model_digest,
    ) {
        return Err("campaign model qualification no longer passes".into());
    }
    if sha256_file(Path::new(&qualification.receipt_path))?
        != snapshot.model_qualification_sha256
    {
        return Err("campaign model qualification receipt changed".into());
    }
    if sha256_file(session.core.core_path())? != snapshot.core_sha256 {
        return Err("campaign emulator core binary changed".into());
    }

    let benchmark = start_model_gameplay_benchmark_inner(
        &app,
        session,
        snapshot.provider,
        snapshot.model,
        snapshot.model_digest,
        snapshot.policy,
    )?;
    let status = benchmark_campaign_status_for(
        session
            .benchmark_campaign
            .as_ref()
            .ok_or_else(|| "benchmark campaign state disappeared".to_owned())?,
    );
    Ok(BenchmarkCampaignStart { status, benchmark })
}

#[tauri::command]
fn benchmark_campaign_status(
    state: State<'_, EmulatorState>,
) -> Result<Option<BenchmarkCampaignStatus>, String> {
    let session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_ref()
        .ok_or_else(|| "no emulator session is running".to_owned())?;
    Ok(session
        .benchmark_campaign
        .as_ref()
        .map(benchmark_campaign_status_for))
}

fn benchmark_campaign_receipt_path(session: &EmulatorSession, campaign_id: u64) -> PathBuf {
    session
        .paths
        .benchmark_campaign_dir
        .join(format!("campaign-{campaign_id:06}.json"))
}

fn load_benchmark_campaign_receipt(
    session: &EmulatorSession,
    campaign_id: u64,
) -> Result<(PathBuf, BenchmarkCampaignReceipt), String> {
    let path = benchmark_campaign_receipt_path(session, campaign_id);
    let bytes = fs::read(&path)
        .map_err(|error| format!("cannot read campaign {}: {error}", path.display()))?;
    let receipt = serde_json::from_slice::<BenchmarkCampaignReceipt>(&bytes)
        .map_err(|error| format!("cannot parse campaign {}: {error}", path.display()))?;
    if receipt.schema != BENCHMARK_CAMPAIGN_SCHEMA {
        return Err(format!("unsupported campaign schema: {}", receipt.schema));
    }
    if receipt.campaign_id != campaign_id {
        return Err("campaign file ID does not match receipt ID".into());
    }
    Ok((path, receipt))
}

fn verify_campaign_trial_receipts(
    model_benchmark_dir: &Path,
    campaign: &BenchmarkCampaignReceipt,
) -> Result<(), String> {
    for trial in &campaign.trials {
        let path = model_benchmark_dir
            .join(format!("run-{:06}.json", trial.benchmark_run_id));
        if !path.exists() {
            return Err(format!(
                "campaign {} references missing benchmark receipt {}",
                campaign.campaign_id,
                path.display()
            ));
        }
        let observed = sha256_file(&path)?;
        if observed != trial.receipt_sha256 {
            return Err(format!(
                "campaign {} trial {} receipt hash mismatch",
                campaign.campaign_id, trial.benchmark_run_id
            ));
        }
    }
    Ok(())
}

fn validate_model_gameplay_receipt_against_campaign(
    campaign: &BenchmarkCampaignReceipt,
    trial: &CampaignTrialEvidence,
    receipt: &ModelGameplayBenchmarkReceipt,
) -> Result<(), String> {
    if receipt.schema != MODEL_GAMEPLAY_BENCHMARK_SCHEMA {
        return Err(format!(
            "campaign {} trial {} has unsupported model benchmark schema {}",
            campaign.campaign_id, trial.benchmark_run_id, receipt.schema
        ));
    }
    if receipt.benchmark_run_id != trial.benchmark_run_id
        || receipt.record_status != trial.record_status
        || receipt.benchmark_id != campaign.benchmark_id
        || receipt.provider != campaign.provider
        || receipt.model != campaign.model
        || receipt.model_digest != campaign.model_digest
        || receipt.model_qualification_sha256 != campaign.model_qualification_sha256
        || receipt.gym_source_sha256 != campaign.gym_source_sha256
        || receipt.gym_rom_sha256 != campaign.gym_rom_sha256
        || receipt.core_sha256 != campaign.core_sha256
        || receipt.core_name != campaign.core_name
        || receipt.core_version != campaign.core_version
        || receipt.policy != campaign.policy
        || receipt.stop_reason != trial.stop_reason
        || receipt.score_1000 != trial.score_1000
        || receipt.task_success != trial.task_success
    {
        return Err(format!(
            "campaign {} trial {} model benchmark identity differs from campaign evidence",
            campaign.campaign_id, trial.benchmark_run_id
        ));
    }
    if receipt.record_status == "COMPLETE" && receipt.scoring_error.is_some() {
        return Err(format!(
            "campaign {} trial {} is COMPLETE but carries a scoring error",
            campaign.campaign_id, trial.benchmark_run_id
        ));
    }
    Ok(())
}

fn validate_autodrive_receipt_against_benchmark(
    campaign: &BenchmarkCampaignReceipt,
    benchmark: &ModelGameplayBenchmarkReceipt,
    receipt: &AutodriveReceipt,
) -> Result<(), String> {
    if receipt.schema != AUTODRIVE_RECEIPT_SCHEMA {
        return Err(format!(
            "campaign {} benchmark {} has unsupported Autodrive schema {}",
            campaign.campaign_id, benchmark.benchmark_run_id, receipt.schema
        ));
    }

    let expected_model = format!("{}@{}", benchmark.model, benchmark.model_digest);
    if receipt.run_id != benchmark.autodrive_run_id
        || receipt.provider != benchmark.provider
        || receipt.model != expected_model
        || receipt.game_sha256 != benchmark.gym_rom_sha256
        || receipt.core_name != benchmark.core_name
        || receipt.core_version != benchmark.core_version
        || receipt.policy != benchmark.policy
        || receipt.stop_reason != benchmark.stop_reason
        || receipt.started_frame != benchmark.started_frame
        || receipt.ended_frame != benchmark.ended_frame
        || receipt.turns_issued != benchmark.turns_issued
        || receipt.turns_completed != benchmark.turns_completed
        || receipt.total_actions != benchmark.total_actions
        || receipt.final_frame_sha256 != benchmark.final_frame_sha256
    {
        return Err(format!(
            "campaign {} benchmark {} Autodrive receipt identity differs",
            campaign.campaign_id, benchmark.benchmark_run_id
        ));
    }
    Ok(())
}

fn validate_campaign_evidence_closure(
    session: &EmulatorSession,
    campaign: &BenchmarkCampaignReceipt,
) -> Result<(), String> {
    let model_benchmark_dir = session
        .paths
        .model_benchmark_root
        .join(&campaign.gym_rom_sha256);
    validate_campaign_for_comparison(&model_benchmark_dir, campaign)?;

    for trial in &campaign.trials {
        let benchmark_path =
            model_benchmark_dir.join(format!("run-{:06}.json", trial.benchmark_run_id));
        let observed_benchmark_sha = sha256_file(&benchmark_path)?;
        if observed_benchmark_sha != trial.receipt_sha256 {
            return Err(format!(
                "campaign {} trial {} model benchmark hash mismatch",
                campaign.campaign_id, trial.benchmark_run_id
            ));
        }

        let benchmark_bytes = fs::read(&benchmark_path)
            .map_err(|error| format!("cannot read {}: {error}", benchmark_path.display()))?;
        let benchmark = serde_json::from_slice::<ModelGameplayBenchmarkReceipt>(&benchmark_bytes)
            .map_err(|error| format!("cannot parse {}: {error}", benchmark_path.display()))?;
        validate_model_gameplay_receipt_against_campaign(campaign, trial, &benchmark)?;

        let autodrive_path = session
            .paths
            .autodrive_root
            .join(&campaign.gym_rom_sha256)
            .join(format!("run-{:06}.json", benchmark.autodrive_run_id));
        let observed_autodrive_sha = sha256_file(&autodrive_path)?;
        if observed_autodrive_sha != benchmark.autodrive_receipt_sha256 {
            return Err(format!(
                "campaign {} benchmark {} Autodrive receipt hash mismatch",
                campaign.campaign_id, benchmark.benchmark_run_id
            ));
        }

        let autodrive_bytes = fs::read(&autodrive_path)
            .map_err(|error| format!("cannot read {}: {error}", autodrive_path.display()))?;
        let autodrive = serde_json::from_slice::<AutodriveReceipt>(&autodrive_bytes)
            .map_err(|error| format!("cannot parse {}: {error}", autodrive_path.display()))?;
        validate_autodrive_receipt_against_benchmark(campaign, &benchmark, &autodrive)?;
    }

    Ok(())
}

fn validate_suite_report_evidence_closure(
    app: &AppHandle,
    session: &EmulatorSession,
    receipt: &BenchmarkSuiteReportReceipt,
) -> Result<std::collections::BTreeMap<String, BenchmarkCampaignReceipt>, String> {
    let campaigns = validate_suite_report_provenance(session, receipt)?;

    if receipt.provider != "ollama" {
        return Err(format!(
            "suite report {} evidence closure does not support provider {}",
            receipt.report_id, receipt.provider
        ));
    }
    let qualification = load_ollama_qualification_by_sha(
        app,
        &receipt.model_digest,
        &receipt.model_qualification_sha256,
    )?;
    if !qualification_receipt_passes(
        &qualification.receipt,
        &receipt.model,
        &receipt.model_digest,
    ) {
        return Err(format!(
            "suite report {} model qualification receipt no longer passes exact model identity",
            receipt.report_id
        ));
    }

    for campaign in campaigns.values() {
        validate_campaign_evidence_closure(session, campaign)?;
    }

    Ok(campaigns)
}

fn validate_campaign_for_comparison(
    model_benchmark_dir: &Path,
    campaign: &BenchmarkCampaignReceipt,
) -> Result<(), String> {
    if campaign.record_status != "COMPLETE" {
        return Err(format!(
            "campaign {} is {}, not COMPLETE",
            campaign.campaign_id, campaign.record_status
        ));
    }
    if campaign.completed_trials != campaign.total_trials
        || campaign.trials.len() != usize::from(campaign.total_trials)
    {
        return Err(format!(
            "campaign {} does not contain all configured trials",
            campaign.campaign_id
        ));
    }
    if campaign.stats.observed_trials != campaign.completed_trials {
        return Err(format!(
            "campaign {} observed-trial count does not match receipt",
            campaign.campaign_id
        ));
    }
    if campaign.stats.scoring_error_trials != 0 {
        return Err(format!(
            "campaign {} contains scoring-error trials",
            campaign.campaign_id
        ));
    }
    if campaign.stats.scored_trials != campaign.completed_trials {
        return Err(format!(
            "campaign {} does not have a numeric score for every trial",
            campaign.campaign_id
        ));
    }
    if campaign.stats.scored_trials < 2 {
        return Err(format!(
            "campaign {} needs at least two scored trials for comparison",
            campaign.campaign_id
        ));
    }
    verify_campaign_trial_receipts(model_benchmark_dir, campaign)
}

fn validate_campaign_compatibility(
    a: &BenchmarkCampaignReceipt,
    b: &BenchmarkCampaignReceipt,
) -> Result<(), String> {
    if a.campaign_id == b.campaign_id {
        return Err("comparison requires two distinct campaign IDs".into());
    }
    if a.benchmark_id != b.benchmark_id {
        return Err("campaign benchmark IDs differ".into());
    }
    if a.provider != b.provider {
        return Err("campaign providers differ".into());
    }
    if a.gym_source_sha256 != b.gym_source_sha256 || a.gym_rom_sha256 != b.gym_rom_sha256 {
        return Err("campaign Gym source/ROM hashes differ".into());
    }
    if a.core_sha256 != b.core_sha256
        || a.core_name != b.core_name
        || a.core_version != b.core_version
    {
        return Err("campaign emulator core provenance differs".into());
    }
    if a.policy != b.policy {
        return Err("campaign Autodrive policies differ".into());
    }
    if a.total_trials != b.total_trials {
        return Err("campaign configured trial counts differ".into());
    }
    Ok(())
}

fn comparison_campaign_ref(
    receipt_path: &Path,
    campaign: &BenchmarkCampaignReceipt,
) -> Result<ComparisonCampaignRef, String> {
    Ok(ComparisonCampaignRef {
        campaign_id: campaign.campaign_id,
        receipt_sha256: sha256_file(receipt_path)?,
        provider: campaign.provider.clone(),
        model: campaign.model.clone(),
        model_digest: campaign.model_digest.clone(),
        model_qualification_sha256: campaign.model_qualification_sha256.clone(),
        completed_trials: campaign.completed_trials,
        stats: campaign.stats.clone(),
    })
}

fn campaign_list_entry(
    path: &Path,
    receipt: &BenchmarkCampaignReceipt,
) -> Result<CampaignListEntry, String> {
    Ok(CampaignListEntry {
        campaign_id: receipt.campaign_id,
        receipt_sha256: sha256_file(path)?,
        record_status: receipt.record_status.clone(),
        model: receipt.model.clone(),
        model_digest: receipt.model_digest.clone(),
        total_trials: receipt.total_trials,
        completed_trials: receipt.completed_trials,
        mean_score_1000: receipt.stats.mean_score_1000,
        success_rate: receipt.stats.success_rate,
    })
}

fn suite_cohort_id(identity: &SuiteCohortIdentity) -> Result<String, String> {
    let bytes = serde_json::to_vec(identity)
        .map_err(|error| format!("serialize suite cohort identity: {error}"))?;
    Ok(sha256_bytes(&bytes))
}

fn suite_report_dir_for(session: &EmulatorSession, suite_id: &str) -> Result<PathBuf, String> {
    let suite = benchmark_suite_by_id(suite_id)
        .ok_or_else(|| format!("unknown benchmark suite {suite_id}"))?;
    Ok(session.paths.suite_report_root.join(suite.id))
}

fn suite_comparison_dir_for(
    session: &EmulatorSession,
    suite_id: &str,
) -> Result<PathBuf, String> {
    let suite = benchmark_suite_by_id(suite_id)
        .ok_or_else(|| format!("unknown benchmark suite {suite_id}"))?;
    Ok(session.paths.suite_comparison_root.join(suite.id))
}

#[tauri::command]
fn list_benchmark_suites() -> Vec<BenchmarkSuiteListEntry> {
    benchmark_suites()
        .iter()
        .map(|suite| BenchmarkSuiteListEntry {
            id: suite.id.to_owned(),
            title: suite.title.to_owned(),
            version: suite.version,
            task_count: u16::try_from(suite.tasks.len()).unwrap_or(u16::MAX),
            tasks: suite
                .tasks
                .iter()
                .map(|task| BenchmarkSuiteTaskEntry {
                    id: task.id.to_owned(),
                    title: task.title.to_owned(),
                    introduced_in_suite_id: task.suite_id.to_owned(),
                    rom_sha256: task.rom_sha256.to_owned(),
                    source_sha256: task.source_sha256.to_owned(),
                })
                .collect(),
        })
        .collect()
}

fn scan_benchmark_suite_cohorts_from_roots(
    benchmark_campaign_root: &Path,
    model_benchmark_root: &Path,
    suite_id: &str,
) -> Result<std::collections::BTreeMap<String, SuiteCohortEvidence>, String> {
    let suite = benchmark_suite_by_id(suite_id)
        .ok_or_else(|| format!("unknown benchmark suite {suite_id}"))?;
    let mut cohorts = std::collections::BTreeMap::new();

    for task in suite.tasks {
        let campaign_dir = benchmark_campaign_root.join(task.rom_sha256);
        let model_benchmark_dir = model_benchmark_root.join(task.rom_sha256);
        if !campaign_dir.exists() {
            continue;
        }

        for entry in fs::read_dir(&campaign_dir)
            .map_err(|error| format!("cannot list {}: {error}", campaign_dir.display()))?
        {
            let entry =
                entry.map_err(|error| format!("cannot read campaign directory entry: {error}"))?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }

            let bytes = fs::read(&path)
                .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
            let receipt = serde_json::from_slice::<BenchmarkCampaignReceipt>(&bytes)
                .map_err(|error| format!("cannot parse {}: {error}", path.display()))?;

            if receipt.schema != BENCHMARK_CAMPAIGN_SCHEMA {
                return Err(format!(
                    "unsupported campaign schema in {}: {}",
                    path.display(),
                    receipt.schema
                ));
            }
            if receipt.benchmark_id != task.id
                || receipt.gym_source_sha256 != task.source_sha256
                || receipt.gym_rom_sha256 != task.rom_sha256
            {
                return Err(format!(
                    "campaign {} does not match benchmark registry task {}",
                    path.display(),
                    task.id
                ));
            }

            validate_campaign_for_comparison(&model_benchmark_dir, &receipt)?;

            let identity = SuiteCohortIdentity {
                provider: receipt.provider.clone(),
                model: receipt.model.clone(),
                model_digest: receipt.model_digest.clone(),
                model_qualification_sha256: receipt.model_qualification_sha256.clone(),
                core_sha256: receipt.core_sha256.clone(),
                core_name: receipt.core_name.clone(),
                core_version: receipt.core_version.clone(),
                policy: receipt.policy.clone(),
                total_trials: receipt.total_trials,
            };
            let cohort_id = suite_cohort_id(&identity)?;
            let cohort = cohorts
                .entry(cohort_id)
                .or_insert_with(|| SuiteCohortEvidence {
                    identity,
                    tasks: std::collections::BTreeMap::new(),
                });

            let replace = cohort
                .tasks
                .get(task.id)
                .map(|(_, existing)| receipt.campaign_id > existing.campaign_id)
                .unwrap_or(true);
            if replace {
                cohort
                    .tasks
                    .insert(task.id.to_owned(), (path, receipt));
            }
        }
    }

    Ok(cohorts)
}

fn scan_benchmark_suite_cohorts(
    session: &EmulatorSession,
    suite_id: &str,
) -> Result<std::collections::BTreeMap<String, SuiteCohortEvidence>, String> {
    scan_benchmark_suite_cohorts_from_roots(
        &session.paths.benchmark_campaign_root,
        &session.paths.model_benchmark_root,
        suite_id,
    )
}

fn suite_report_candidate(
    suite_id: &str,
    cohort_id: &str,
    evidence: &SuiteCohortEvidence,
) -> Result<BenchmarkSuiteReportCandidate, String> {
    let suite = benchmark_suite_by_id(suite_id)
        .ok_or_else(|| format!("unknown benchmark suite {suite_id}"))?;
    let mut tasks = Vec::new();
    for task in suite.tasks {
        let Some((_, campaign)) = evidence.tasks.get(task.id) else {
            continue;
        };
        let mean_score_1000 = campaign
            .stats
            .mean_score_1000
            .ok_or_else(|| format!("campaign {} has no mean score", campaign.campaign_id))?;
        tasks.push(SuiteCandidateTask {
            task_id: task.id.into(),
            task_title: task.title.into(),
            campaign_id: campaign.campaign_id,
            mean_score_1000,
            success_rate: campaign.stats.success_rate,
        });
    }

    let suite_task_count = u16::try_from(suite.tasks.len()).unwrap_or(u16::MAX);
    let covered_tasks = u16::try_from(tasks.len()).unwrap_or(u16::MAX);

    Ok(BenchmarkSuiteReportCandidate {
        suite_id: suite.id.to_owned(),
        cohort_id: cohort_id.to_owned(),
        provider: evidence.identity.provider.clone(),
        model: evidence.identity.model.clone(),
        model_digest: evidence.identity.model_digest.clone(),
        trials_per_task: evidence.identity.total_trials,
        covered_tasks,
        suite_task_count,
        ready: covered_tasks == suite_task_count,
        tasks,
    })
}

fn build_suite_report_from_cohort(
    session: &mut EmulatorSession,
    suite_id: &str,
    cohort_id: &str,
) -> Result<BenchmarkSuiteReportArtifact, String> {
    let suite = benchmark_suite_by_id(suite_id)
        .ok_or_else(|| format!("unknown benchmark suite {suite_id}"))?;
    let cohorts = scan_benchmark_suite_cohorts(session, suite.id)?;
    let evidence = cohorts
        .get(cohort_id)
        .ok_or_else(|| "suite cohort no longer exists".to_owned())?;

    if evidence.tasks.len() != suite.tasks.len() {
        return Err(format!(
            "suite cohort covers {}/{} tasks",
            evidence.tasks.len(),
            suite.tasks.len()
        ));
    }

    let mut task_refs = Vec::new();
    let mut aggregate_inputs = Vec::new();

    for task in suite.tasks {
        let (campaign_path, campaign) = evidence
            .tasks
            .get(task.id)
            .ok_or_else(|| format!("suite cohort is missing task {}", task.id))?;

        let model_benchmark_dir = session.paths.model_benchmark_root.join(task.rom_sha256);
        validate_campaign_for_comparison(&model_benchmark_dir, campaign)?;

        if campaign.benchmark_id != task.id
            || campaign.gym_source_sha256 != task.source_sha256
            || campaign.gym_rom_sha256 != task.rom_sha256
        {
            return Err(format!("suite campaign {} drifted from registry", campaign.campaign_id));
        }

        let mean_score_1000 = campaign
            .stats
            .mean_score_1000
            .ok_or_else(|| format!("campaign {} has no mean score", campaign.campaign_id))?;

        aggregate_inputs.push(SuiteTaskAggregateInput {
            task_id: task.id.into(),
            mean_score_1000,
            observed_trials: campaign.stats.observed_trials,
            successful_trials: campaign.stats.successful_trials,
        });

        task_refs.push(SuiteTaskCampaignRef {
            task_id: task.id.into(),
            task_title: task.title.into(),
            campaign_id: campaign.campaign_id,
            campaign_receipt_sha256: sha256_file(campaign_path)?,
            source_sha256: task.source_sha256.into(),
            rom_sha256: task.rom_sha256.into(),
            stats: campaign.stats.clone(),
        });
    }

    let stats = summarize_benchmark_suite(&aggregate_inputs)?;
    let report_dir = suite_report_dir_for(session, suite.id)?;
    let report_id = next_numbered_receipt_id(&report_dir, "suite-report-", ".json")?;
    let identity = &evidence.identity;
    let receipt = BenchmarkSuiteReportReceipt {
        schema: BENCHMARK_SUITE_REPORT_SCHEMA.into(),
        record_status: "COMPLETE".into(),
        report_id,
        suite_id: suite.id.into(),
        cohort_id: cohort_id.to_owned(),
        provider: identity.provider.clone(),
        model: identity.model.clone(),
        model_digest: identity.model_digest.clone(),
        model_qualification_sha256: identity.model_qualification_sha256.clone(),
        core_sha256: identity.core_sha256.clone(),
        core_name: identity.core_name.clone(),
        core_version: identity.core_version.clone(),
        policy: identity.policy.clone(),
        trials_per_task: identity.total_trials,
        tasks: task_refs,
        stats,
    };

    let receipt_path = report_dir.join(format!("suite-report-{report_id:06}.json"));
    let json = serde_json::to_vec_pretty(&receipt)
        .map_err(|error| format!("serialize Benchmark Suite report: {error}"))?;
    write_atomic(&receipt_path, &json)?;

    Ok(BenchmarkSuiteReportArtifact {
        receipt_path: receipt_path.to_string_lossy().to_string(),
        receipt,
    })
}

#[tauri::command]
fn list_benchmark_suite_report_candidates(
    state: State<'_, EmulatorState>,
    suite_id: String,
) -> Result<Vec<BenchmarkSuiteReportCandidate>, String> {
    let session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_ref()
        .ok_or_else(|| "no emulator session is running".to_owned())?;

    let suite_id = suite_id.trim();
    let cohorts = scan_benchmark_suite_cohorts(session, suite_id)?;
    cohorts
        .iter()
        .map(|(cohort_id, evidence)| suite_report_candidate(suite_id, cohort_id, evidence))
        .collect()
}

#[tauri::command]
fn build_benchmark_suite_report(
    state: State<'_, EmulatorState>,
    suite_id: String,
    cohort_id: String,
) -> Result<BenchmarkSuiteReportArtifact, String> {
    let mut session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_mut()
        .ok_or_else(|| "no emulator session is running".to_owned())?;
    build_suite_report_from_cohort(session, suite_id.trim(), cohort_id.trim())
}

fn load_benchmark_suite_report_receipt(
    session: &EmulatorSession,
    suite_id: &str,
    report_id: u64,
) -> Result<(PathBuf, BenchmarkSuiteReportReceipt), String> {
    let report_dir = suite_report_dir_for(session, suite_id)?;
    let path = report_dir.join(format!("suite-report-{report_id:06}.json"));
    let bytes = fs::read(&path)
        .map_err(|error| format!("cannot read suite report {}: {error}", path.display()))?;
    let receipt = serde_json::from_slice::<BenchmarkSuiteReportReceipt>(&bytes)
        .map_err(|error| format!("cannot parse suite report {}: {error}", path.display()))?;
    if receipt.report_id != report_id {
        return Err(format!(
            "suite report path ID {} does not match receipt ID {}",
            report_id, receipt.report_id
        ));
    }
    if receipt.suite_id != suite_id {
        return Err(format!(
            "suite report {} belongs to {}, not {}",
            report_id, receipt.suite_id, suite_id
        ));
    }
    Ok((path, receipt))
}

fn validate_suite_report_provenance(
    session: &EmulatorSession,
    receipt: &BenchmarkSuiteReportReceipt,
) -> Result<std::collections::BTreeMap<String, BenchmarkCampaignReceipt>, String> {
    if receipt.schema != BENCHMARK_SUITE_REPORT_SCHEMA {
        return Err(format!("unsupported suite report schema {}", receipt.schema));
    }
    if receipt.record_status != "COMPLETE" {
        return Err(format!(
            "suite report {} is {}, not COMPLETE",
            receipt.report_id, receipt.record_status
        ));
    }
    let suite = benchmark_suite_by_id(&receipt.suite_id)
        .ok_or_else(|| format!("suite report {} targets unknown suite {}", receipt.report_id, receipt.suite_id))?;
    if receipt.tasks.len() != suite.tasks.len() {
        return Err(format!(
            "suite report {} covers {}/{} registered tasks",
            receipt.report_id,
            receipt.tasks.len(),
            suite.tasks.len()
        ));
    }
    if usize::from(receipt.stats.task_count) != receipt.tasks.len() {
        return Err(format!("suite report {} task count is inconsistent", receipt.report_id));
    }

    let mut campaigns = std::collections::BTreeMap::new();
    let mut aggregate_inputs = Vec::new();

    for task in suite.tasks {
        let task_ref = receipt
            .tasks
            .iter()
            .find(|candidate| candidate.task_id == task.id)
            .ok_or_else(|| {
                format!(
                    "suite report {} is missing task {}",
                    receipt.report_id, task.id
                )
            })?;

        if task_ref.source_sha256 != task.source_sha256 || task_ref.rom_sha256 != task.rom_sha256 {
            return Err(format!(
                "suite report {} task {} registry hashes differ",
                receipt.report_id, task.id
            ));
        }

        let campaign_path = session
            .paths
            .benchmark_campaign_root
            .join(task.rom_sha256)
            .join(format!("campaign-{:06}.json", task_ref.campaign_id));
        if !campaign_path.exists() {
            return Err(format!(
                "suite report {} campaign {} is missing",
                receipt.report_id, task_ref.campaign_id
            ));
        }
        let campaign_hash = sha256_file(&campaign_path)?;
        if campaign_hash != task_ref.campaign_receipt_sha256 {
            return Err(format!(
                "suite report {} campaign {} hash mismatch",
                receipt.report_id, task_ref.campaign_id
            ));
        }
        let campaign_bytes = fs::read(&campaign_path)
            .map_err(|error| format!("cannot read {}: {error}", campaign_path.display()))?;
        let campaign = serde_json::from_slice::<BenchmarkCampaignReceipt>(&campaign_bytes)
            .map_err(|error| format!("cannot parse {}: {error}", campaign_path.display()))?;

        let model_benchmark_dir = session.paths.model_benchmark_root.join(task.rom_sha256);
        validate_campaign_for_comparison(&model_benchmark_dir, &campaign)?;

        if campaign.benchmark_id != task.id
            || campaign.gym_source_sha256 != task.source_sha256
            || campaign.gym_rom_sha256 != task.rom_sha256
        {
            return Err(format!(
                "suite report {} campaign {} task identity differs",
                receipt.report_id, campaign.campaign_id
            ));
        }
        if campaign.provider != receipt.provider
            || campaign.model != receipt.model
            || campaign.model_digest != receipt.model_digest
            || campaign.model_qualification_sha256 != receipt.model_qualification_sha256
            || campaign.core_sha256 != receipt.core_sha256
            || campaign.core_name != receipt.core_name
            || campaign.core_version != receipt.core_version
            || campaign.policy != receipt.policy
            || campaign.total_trials != receipt.trials_per_task
        {
            return Err(format!(
                "suite report {} campaign {} cohort identity differs",
                receipt.report_id, campaign.campaign_id
            ));
        }
        if campaign.stats != task_ref.stats {
            return Err(format!(
                "suite report {} campaign {} statistics differ",
                receipt.report_id, campaign.campaign_id
            ));
        }

        let mean_score_1000 = campaign
            .stats
            .mean_score_1000
            .ok_or_else(|| format!("campaign {} has no mean score", campaign.campaign_id))?;
        aggregate_inputs.push(SuiteTaskAggregateInput {
            task_id: task.id.into(),
            mean_score_1000,
            observed_trials: campaign.stats.observed_trials,
            successful_trials: campaign.stats.successful_trials,
        });
        campaigns.insert(task.id.to_owned(), campaign);
    }

    let recomputed = summarize_benchmark_suite(&aggregate_inputs)?;
    if recomputed != receipt.stats {
        return Err(format!(
            "suite report {} aggregate statistics do not replay exactly",
            receipt.report_id
        ));
    }

    Ok(campaigns)
}

fn validate_suite_report_compatibility(
    a: &BenchmarkSuiteReportReceipt,
    b: &BenchmarkSuiteReportReceipt,
) -> Result<(), String> {
    if a.report_id == b.report_id {
        return Err("suite comparison requires two distinct report IDs".into());
    }
    if a.suite_id != b.suite_id {
        return Err("suite report IDs differ".into());
    }
    if a.core_sha256 != b.core_sha256 || a.core_name != b.core_name || a.core_version != b.core_version {
        return Err("suite report emulator core provenance differs".into());
    }
    if a.policy != b.policy {
        return Err("suite report Autodrive policies differ".into());
    }
    if a.trials_per_task != b.trials_per_task {
        return Err("suite report configured trial counts differ".into());
    }
    Ok(())
}

fn suite_comparison_report_ref(
    path: &Path,
    receipt: &BenchmarkSuiteReportReceipt,
) -> Result<SuiteComparisonReportRef, String> {
    Ok(SuiteComparisonReportRef {
        report_id: receipt.report_id,
        receipt_sha256: sha256_file(path)?,
        cohort_id: receipt.cohort_id.clone(),
        provider: receipt.provider.clone(),
        model: receipt.model.clone(),
        model_digest: receipt.model_digest.clone(),
        model_qualification_sha256: receipt.model_qualification_sha256.clone(),
        stats: receipt.stats.clone(),
    })
}

#[tauri::command]
fn list_benchmark_suite_reports(
    state: State<'_, EmulatorState>,
) -> Result<Vec<BenchmarkSuiteReportListEntry>, String> {
    let session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_ref()
        .ok_or_else(|| "no emulator session is running".to_owned())?;

    let mut entries = Vec::new();
    for suite in benchmark_suites() {
        let report_dir = suite_report_dir_for(session, suite.id)?;
        for entry in fs::read_dir(&report_dir)
            .map_err(|error| format!("cannot list suite reports for {}: {error}", suite.id))?
        {
            let entry = entry.map_err(|error| format!("cannot read suite report entry: {error}"))?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(&path)
                .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
            let receipt = serde_json::from_slice::<BenchmarkSuiteReportReceipt>(&bytes)
                .map_err(|error| format!("cannot parse {}: {error}", path.display()))?;
            if receipt.schema != BENCHMARK_SUITE_REPORT_SCHEMA
                || receipt.record_status != "COMPLETE"
                || receipt.suite_id != suite.id
            {
                continue;
            }
            entries.push(BenchmarkSuiteReportListEntry {
                report_id: receipt.report_id,
                suite_id: receipt.suite_id.clone(),
                provider: receipt.provider.clone(),
                model: receipt.model.clone(),
                model_digest: receipt.model_digest.clone(),
                core_sha256: receipt.core_sha256.clone(),
                trials_per_task: receipt.trials_per_task,
                task_count: receipt.stats.task_count,
                macro_mean_score_1000: receipt.stats.macro_mean_score_1000,
                overall_success_rate: receipt.stats.overall_success_rate,
                receipt_sha256: sha256_file(&path)?,
            });
        }
    }
    entries.sort_by(|a, b| {
        a.suite_id
            .cmp(&b.suite_id)
            .then(a.report_id.cmp(&b.report_id))
    });
    Ok(entries)
}


fn markdown_cell(value: &str) -> String {
    value
        .replace('|', "\\|")
        .replace('\r', " ")
        .replace('\n', " ")
        .trim()
        .to_owned()
}

fn public_suite_result_receipt(
    suite_id: &str,
    report_path: &Path,
    report: &BenchmarkSuiteReportReceipt,
) -> Result<PublicSuiteResultReceipt, String> {
    let suite = benchmark_suite_by_id(suite_id)
        .ok_or_else(|| format!("unknown benchmark suite {suite_id}"))?;

    let mut total_scored_trials = 0u32;
    let mut total_scoring_error_trials = 0u32;
    let mut tasks = Vec::with_capacity(suite.tasks.len());

    for task in suite.tasks {
        let task_ref = report
            .tasks
            .iter()
            .find(|candidate| candidate.task_id == task.id)
            .ok_or_else(|| {
                format!(
                    "suite report {} is missing task {}",
                    report.report_id, task.id
                )
            })?;
        let mean_score_1000 = task_ref.stats.mean_score_1000.ok_or_else(|| {
            format!(
                "suite report {} task {} has no public mean score",
                report.report_id, task.id
            )
        })?;

        total_scored_trials =
            total_scored_trials.saturating_add(u32::from(task_ref.stats.scored_trials));
        total_scoring_error_trials = total_scoring_error_trials
            .saturating_add(u32::from(task_ref.stats.scoring_error_trials));

        tasks.push(PublicSuiteTaskResult {
            task_id: task_ref.task_id.clone(),
            task_title: task_ref.task_title.clone(),
            campaign_id: task_ref.campaign_id,
            campaign_receipt_sha256: task_ref.campaign_receipt_sha256.clone(),
            observed_trials: task_ref.stats.observed_trials,
            scored_trials: task_ref.stats.scored_trials,
            scoring_error_trials: task_ref.stats.scoring_error_trials,
            successful_trials: task_ref.stats.successful_trials,
            success_rate: task_ref.stats.success_rate,
            mean_score_1000,
        });
    }

    Ok(PublicSuiteResultReceipt {
        schema: PUBLIC_SUITE_RESULT_SCHEMA.into(),
        record_status: "VERIFIED_EXPORT".into(),
        suite_id: report.suite_id.clone(),
        suite_version: suite.version,
        suite_title: suite.title.into(),
        report_id: report.report_id,
        suite_report_sha256: sha256_file(report_path)?,
        cohort_id: report.cohort_id.clone(),
        provider: report.provider.clone(),
        model: report.model.clone(),
        model_digest: report.model_digest.clone(),
        model_qualification_sha256: report.model_qualification_sha256.clone(),
        core_sha256: report.core_sha256.clone(),
        core_name: report.core_name.clone(),
        core_version: report.core_version.clone(),
        policy: report.policy.clone(),
        trials_per_task: report.trials_per_task,
        total_scored_trials,
        total_scoring_error_trials,
        stats: report.stats.clone(),
        tasks,
    })
}

fn render_public_suite_result_markdown(
    receipt: &PublicSuiteResultReceipt,
    json_sha256: &str,
) -> String {
    let stats = &receipt.stats;
    let mut markdown = String::new();
    markdown.push_str("# PhiCade Public Suite Result\n\n");
    markdown.push_str("> Verified export derived from a revalidated PhiCade Suite Report.\n\n");
    markdown.push_str("## Result\n\n");
    markdown.push_str("| Field | Value |\n|---|---|\n");
    markdown.push_str(&format!("| Suite | {} (v{}) |\n", markdown_cell(&receipt.suite_title), receipt.suite_version));
    markdown.push_str(&format!("| Suite ID | {} |\n", markdown_cell(&receipt.suite_id)));
    markdown.push_str(&format!("| Report ID | #{} |\n", receipt.report_id));
    markdown.push_str(&format!("| Model | {} |\n", markdown_cell(&receipt.model)));
    markdown.push_str(&format!("| Model digest | `{}` |\n", receipt.model_digest));
    markdown.push_str(&format!("| Provider | {} |\n", markdown_cell(&receipt.provider)));
    markdown.push_str(&format!("| Core | {} {} |\n", markdown_cell(&receipt.core_name), markdown_cell(&receipt.core_version)));
    markdown.push_str(&format!("| Tasks | {} |\n", stats.task_count));
    markdown.push_str(&format!("| Trials per task | {} |\n", receipt.trials_per_task));
    markdown.push_str(&format!("| Macro mean score | {:.1} / 1000 |\n", stats.macro_mean_score_1000));
    markdown.push_str(&format!("| Overall success rate | {:.1}% |\n", stats.overall_success_rate * 100.0));
    markdown.push_str(&format!("| Task mean range | {:.1} .. {:.1} |\n", stats.min_task_mean_score_1000, stats.max_task_mean_score_1000));
    markdown.push_str(&format!("| Task-mean population SD | {:.1} |\n", stats.population_stddev_task_mean_score_1000));
    markdown.push_str(&format!("| Observed trials | {} |\n", stats.total_observed_trials));
    markdown.push_str(&format!("| Scored trials | {} |\n", receipt.total_scored_trials));
    markdown.push_str(&format!("| Scoring-error trials | {} |\n", receipt.total_scoring_error_trials));
    markdown.push_str(&format!("| Successful trials | {} |\n", stats.total_successful_trials));
    markdown.push_str("\n## Provenance\n\n");
    markdown.push_str(&format!("- Suite Report SHA-256: `{}`\n", receipt.suite_report_sha256));
    markdown.push_str(&format!("- Public JSON SHA-256: `{}`\n", json_sha256));
    markdown.push_str(&format!("- Model qualification SHA-256: `{}`\n", receipt.model_qualification_sha256));
    markdown.push_str(&format!("- Core SHA-256: `{}`\n", receipt.core_sha256));
    markdown.push_str(&format!("- Cohort ID: `{}`\n", receipt.cohort_id));
    markdown.push_str("\n## Per-task evidence\n\n");
    markdown.push_str("| Task | Mean /1000 | Success | Trials | Campaign | Campaign receipt SHA |\n");
    markdown.push_str("|---|---:|---:|---:|---:|---|\n");
    for task in &receipt.tasks {
        markdown.push_str(&format!(
            "| {} | {:.1} | {:.1}% | {}/{} | #{} | `{}` |\n",
            markdown_cell(&task.task_title),
            task.mean_score_1000,
            task.success_rate * 100.0,
            task.scored_trials,
            task.observed_trials,
            task.campaign_id,
            task.campaign_receipt_sha256,
        ));
    }
    markdown.push_str("\n## Interpretation boundary\n\n");
    markdown.push_str(
        "This export reports one verified empirical cohort. It does not convert structural shortcut controls into empirical model baselines, and it does not declare a universal winner. Re-run provenance validation from the referenced Suite Report and campaign receipts before making derivative claims.\n",
    );
    markdown
}

#[tauri::command]
fn export_benchmark_suite_public_result(
    app: AppHandle,
    state: State<'_, EmulatorState>,
    suite_id: String,
    report_id: u64,
) -> Result<PublicSuiteResultArtifact, String> {
    let session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_ref()
        .ok_or_else(|| "no emulator session is running".to_owned())?;

    let suite_id = suite_id.trim();
    let (report_path, report) =
        load_benchmark_suite_report_receipt(session, suite_id, report_id)?;
    validate_suite_report_evidence_closure(&app, session, &report)?;

    let receipt = public_suite_result_receipt(suite_id, &report_path, &report)?;
    let report_dir = suite_report_dir_for(session, suite_id)?;
    let public_dir = report_dir.join("public");
    fs::create_dir_all(&public_dir)
        .map_err(|error| format!("cannot create public result directory {}: {error}", public_dir.display()))?;

    let stem = format!("suite-report-{report_id:06}-public");
    let json_path = public_dir.join(format!("{stem}.json"));
    let markdown_path = public_dir.join(format!("{stem}.md"));

    let json = serde_json::to_vec_pretty(&receipt)
        .map_err(|error| format!("serialize public suite result: {error}"))?;
    write_atomic(&json_path, &json)?;
    let json_sha256 = sha256_file(&json_path)?;

    let markdown = render_public_suite_result_markdown(&receipt, &json_sha256);
    write_atomic(&markdown_path, markdown.as_bytes())?;
    let markdown_sha256 = sha256_file(&markdown_path)?;

    Ok(PublicSuiteResultArtifact {
        json_path: json_path.to_string_lossy().to_string(),
        markdown_path: markdown_path.to_string_lossy().to_string(),
        json_sha256,
        markdown_sha256,
        receipt,
    })
}

#[tauri::command]
fn compare_benchmark_suite_reports(
    app: AppHandle,
    state: State<'_, EmulatorState>,
    suite_id: String,
    report_a_id: u64,
    report_b_id: u64,
) -> Result<BenchmarkSuiteComparisonArtifact, String> {
    let mut session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_mut()
        .ok_or_else(|| "no emulator session is running".to_owned())?;

    let suite_id = suite_id.trim();
    let suite = benchmark_suite_by_id(suite_id)
        .ok_or_else(|| format!("unknown benchmark suite {suite_id}"))?;
    let (path_a, report_a) = load_benchmark_suite_report_receipt(session, suite.id, report_a_id)?;
    let (path_b, report_b) = load_benchmark_suite_report_receipt(session, suite.id, report_b_id)?;
    let campaigns_a = validate_suite_report_evidence_closure(&app, session, &report_a)?;
    let campaigns_b = validate_suite_report_evidence_closure(&app, session, &report_b)?;
    validate_suite_report_compatibility(&report_a, &report_b)?;

    let mut aggregate_inputs = Vec::new();
    let mut task_refs = Vec::new();

    for task in suite.tasks {
        let campaign_a = campaigns_a
            .get(task.id)
            .ok_or_else(|| format!("suite report A is missing task {}", task.id))?;
        let campaign_b = campaigns_b
            .get(task.id)
            .ok_or_else(|| format!("suite report B is missing task {}", task.id))?;

        let scores_a: Vec<u16> = campaign_a
            .trials
            .iter()
            .filter_map(|trial| trial.score_1000)
            .collect();
        let scores_b: Vec<u16> = campaign_b
            .trials
            .iter()
            .filter_map(|trial| trial.score_1000)
            .collect();
        let task_stats = compare_campaign_samples(
            &scores_a,
            campaign_a.stats.successful_trials,
            campaign_a.stats.observed_trials,
            &scores_b,
            campaign_b.stats.successful_trials,
            campaign_b.stats.observed_trials,
        )?;

        aggregate_inputs.push(SuiteTaskComparisonInput {
            task_id: task.id.into(),
            mean_score_a_1000: task_stats.sample_a.mean_score_1000,
            mean_score_b_1000: task_stats.sample_b.mean_score_1000,
            success_rate_a: task_stats.sample_a.success_rate,
            success_rate_b: task_stats.sample_b.success_rate,
        });
        task_refs.push(SuiteTaskComparisonRef {
            task_id: task.id.into(),
            task_title: task.title.into(),
            campaign_a_id: campaign_a.campaign_id,
            campaign_b_id: campaign_b.campaign_id,
            stats: task_stats,
        });
    }

    let stats = compare_benchmark_suites(
        &aggregate_inputs,
        report_a.stats.overall_success_rate,
        report_b.stats.overall_success_rate,
    )?;
    let comparison_dir = suite_comparison_dir_for(session, suite.id)?;
    let comparison_id = next_numbered_receipt_id(&comparison_dir, "comparison-", ".json")?;
    let receipt = BenchmarkSuiteComparisonReceipt {
        schema: BENCHMARK_SUITE_COMPARISON_SCHEMA.into(),
        record_status: "COMPLETE".into(),
        comparison_id,
        suite_id: report_a.suite_id.clone(),
        core_sha256: report_a.core_sha256.clone(),
        core_name: report_a.core_name.clone(),
        core_version: report_a.core_version.clone(),
        policy: report_a.policy.clone(),
        trials_per_task: report_a.trials_per_task,
        report_a: suite_comparison_report_ref(&path_a, &report_a)?,
        report_b: suite_comparison_report_ref(&path_b, &report_b)?,
        stats,
        tasks: task_refs,
    };

    let receipt_path = comparison_dir.join(format!("comparison-{comparison_id:06}.json"));
    let json = serde_json::to_vec_pretty(&receipt)
        .map_err(|error| format!("serialize suite comparison receipt: {error}"))?;
    write_atomic(&receipt_path, &json)?;

    Ok(BenchmarkSuiteComparisonArtifact {
        receipt_path: receipt_path.to_string_lossy().to_string(),
        receipt,
    })
}

#[tauri::command]
fn list_benchmark_campaign_receipts(
    state: State<'_, EmulatorState>,
) -> Result<Vec<CampaignListEntry>, String> {
    let session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_ref()
        .ok_or_else(|| "no emulator session is running".to_owned())?;

    let mut entries = Vec::new();
    for entry in fs::read_dir(&session.paths.benchmark_campaign_dir)
        .map_err(|error| format!("cannot list campaign receipts: {error}"))?
    {
        let entry = entry.map_err(|error| format!("cannot read campaign directory entry: {error}"))?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let bytes = fs::read(&path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        let receipt = serde_json::from_slice::<BenchmarkCampaignReceipt>(&bytes)
            .map_err(|error| format!("cannot parse {}: {error}", path.display()))?;
        entries.push(campaign_list_entry(&path, &receipt)?);
    }
    entries.sort_by_key(|entry| entry.campaign_id);
    Ok(entries)
}

#[tauri::command]
fn compare_benchmark_campaigns(
    state: State<'_, EmulatorState>,
    campaign_a_id: u64,
    campaign_b_id: u64,
) -> Result<CampaignComparisonArtifact, String> {
    let mut session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_mut()
        .ok_or_else(|| "no emulator session is running".to_owned())?;

    let (path_a, a) = load_benchmark_campaign_receipt(session, campaign_a_id)?;
    let (path_b, b) = load_benchmark_campaign_receipt(session, campaign_b_id)?;

    validate_campaign_for_comparison(&session.paths.model_benchmark_dir, &a)?;
    validate_campaign_for_comparison(&session.paths.model_benchmark_dir, &b)?;
    validate_campaign_compatibility(&a, &b)?;

    let scores_a: Vec<u16> = a.trials.iter().filter_map(|trial| trial.score_1000).collect();
    let scores_b: Vec<u16> = b.trials.iter().filter_map(|trial| trial.score_1000).collect();
    let stats = compare_campaign_samples(
        &scores_a,
        a.stats.successful_trials,
        a.stats.observed_trials,
        &scores_b,
        b.stats.successful_trials,
        b.stats.observed_trials,
    )?;

    let comparison_id = session.next_campaign_comparison_id;
    let receipt = CampaignComparisonReceipt {
        schema: CAMPAIGN_COMPARISON_SCHEMA.to_owned(),
        record_status: "COMPLETE".into(),
        comparison_id,
        benchmark_id: a.benchmark_id.clone(),
        gym_source_sha256: a.gym_source_sha256.clone(),
        gym_rom_sha256: a.gym_rom_sha256.clone(),
        core_sha256: a.core_sha256.clone(),
        core_name: a.core_name.clone(),
        core_version: a.core_version.clone(),
        policy: a.policy.clone(),
        total_trials: a.total_trials,
        campaign_a: comparison_campaign_ref(&path_a, &a)?,
        campaign_b: comparison_campaign_ref(&path_b, &b)?,
        stats,
    };

    let receipt_path = session
        .paths
        .campaign_comparison_dir
        .join(format!("comparison-{comparison_id:06}.json"));
    let json = serde_json::to_vec_pretty(&receipt)
        .map_err(|error| format!("serialize campaign comparison receipt: {error}"))?;
    write_atomic(&receipt_path, &json)?;
    session.next_campaign_comparison_id =
        session.next_campaign_comparison_id.saturating_add(1);

    Ok(CampaignComparisonArtifact {
        receipt_path: receipt_path.to_string_lossy().to_string(),
        receipt,
    })
}

#[tauri::command]
fn last_benchmark_campaign_receipt(
    state: State<'_, EmulatorState>,
) -> Result<Option<BenchmarkCampaignArtifact>, String> {
    let session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_ref()
        .ok_or_else(|| "no emulator session is running".to_owned())?;
    Ok(session
        .last_benchmark_campaign
        .as_ref()
        .map(benchmark_campaign_artifact))
}

#[tauri::command]
fn cancel_benchmark_campaign(
    state: State<'_, EmulatorState>,
) -> Result<BenchmarkCampaignArtifact, String> {
    let mut session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_mut()
        .ok_or_else(|| "no emulator session is running".to_owned())?;

    if !session
        .benchmark_campaign
        .as_ref()
        .is_some_and(|campaign| campaign.active)
    {
        if let Some(last) = session.last_benchmark_campaign.as_ref() {
            return Ok(benchmark_campaign_artifact(last));
        }
        return Err("no active benchmark campaign".into());
    }

    if session
        .autodrive
        .as_ref()
        .is_some_and(|status| status.active)
    {
        let _ = finish_autodrive(session, AutodriveStopReason::OperatorStop)?;
    }

    if session
        .benchmark_campaign
        .as_ref()
        .is_some_and(|campaign| campaign.active)
    {
        finish_benchmark_campaign(session, "PARTIAL")
    } else {
        session
            .last_benchmark_campaign
            .as_ref()
            .map(benchmark_campaign_artifact)
            .ok_or_else(|| "campaign completed without a summary receipt".to_owned())
    }
}

#[tauri::command]
fn last_model_gameplay_benchmark(
    state: State<'_, EmulatorState>,
) -> Result<Option<ModelBenchmarkArtifact>, String> {
    let session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_ref()
        .ok_or_else(|| "no emulator session is running".to_owned())?;
    Ok(session
        .last_model_benchmark
        .as_ref()
        .map(model_benchmark_artifact))
}

#[tauri::command]
fn autodrive_status(state: State<'_, EmulatorState>) -> Result<Option<AutodriveStatus>, String> {
    let session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_ref()
        .ok_or_else(|| "no emulator session is running".to_owned())?;
    Ok(autodrive_status_for(session))
}

#[tauri::command]
fn stop_autodrive(state: State<'_, EmulatorState>) -> Result<AutodriveArtifact, String> {
    let mut session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_mut()
        .ok_or_else(|| "no emulator session is running".to_owned())?;
    finish_autodrive(session, AutodriveStopReason::OperatorStop)
}

#[tauri::command]
fn fail_autodrive_provider(state: State<'_, EmulatorState>) -> Result<AutodriveArtifact, String> {
    let mut session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_mut()
        .ok_or_else(|| "no emulator session is running".to_owned())?;
    finish_autodrive(session, AutodriveStopReason::ProviderFailure)
}

fn driver_status_for(session: &EmulatorSession) -> DriverStatus {
    DriverStatus {
        pending_turn_id: session.pending_agent_turn.as_ref().map(|request| request.turn_id),
        queued_actions: session.agent_inbox.len(),
        next_turn_id: session.next_driver_turn_id,
        memory_content: session.agent_memory.content.clone(),
        memory_sha256: session.agent_memory.sha256(),
        memory_bytes: session.agent_memory.bytes(),
        memory_revision: session.agent_memory.revision,
        memory_updates: session.agent_memory.updates,
        memory_bytes_written: session.agent_memory.bytes_written,
        memory_refusals: session.agent_memory.refusals,
    }
}

#[tauri::command]
fn driver_status(state: State<'_, EmulatorState>) -> Result<DriverStatus, String> {
    let session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_ref()
        .ok_or_else(|| "no emulator session is running".to_owned())?;
    Ok(driver_status_for(session))
}

#[tauri::command]
fn issue_agent_turn(
    state: State<'_, EmulatorState>,
    agent_id: String,
    seat: u8,
) -> Result<AgentTurnRequest, String> {
    let mut session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_mut()
        .ok_or_else(|| "no emulator session is running".to_owned())?;

    if session.recording.is_some() {
        return Err("agent turn issuance is disabled during replay recording".into());
    }

    autodrive_pre_turn_guard(session)?;

    if let Some(status) = session.autodrive.as_ref().filter(|status| status.active) {
        let current_frame = session.core.frame_count();
        if !status.observation_ready(current_frame) {
            return Err(format!(
                "autonomous observation cadence waits until frame {} (current {})",
                status.next_observation_frame, current_frame
            ));
        }
    }

    if session
        .pending_agent_turn
        .as_ref()
        .is_some_and(|request| session.core.frame_count() <= request.valid_until_frame)
    {
        return Err("an unexpired agent turn is already pending".into());
    }

    let observation = build_phi_bot_observation(session, agent_id, seat)?;
    let (max_memory_bytes, max_memory_update_bytes) = session
        .autodrive
        .as_ref()
        .filter(|status| status.active)
        .map(|status| (
            status.policy.max_memory_bytes,
            status.policy.max_memory_update_bytes,
        ))
        .unwrap_or((
            MANUAL_AGENT_MEMORY_MAX_BYTES,
            MANUAL_AGENT_MEMORY_UPDATE_MAX_BYTES,
        ));
    let request = AgentTurnRequest {
        schema: AGENT_TURN_REQUEST_SCHEMA.to_owned(),
        turn_id: session.next_driver_turn_id,
        max_actions: 8,
        max_delay_frames: 30,
        valid_until_frame: observation.frame.saturating_add(120),
        memory: session.agent_memory.content.clone(),
        memory_sha256: session.agent_memory.sha256(),
        max_memory_bytes,
        max_memory_update_bytes,
        observation,
    };
    request.validate()?;

    session.next_driver_turn_id = session.next_driver_turn_id.saturating_add(1);
    session.pending_agent_turn = Some(request.clone());
    if let Some(status) = session.autodrive.as_mut().filter(|status| status.active) {
        status.note_turn_issued_at(session.core.frame_count());
    }
    Ok(request)
}

#[tauri::command]
fn cancel_agent_turn(state: State<'_, EmulatorState>) -> Result<DriverStatus, String> {
    let mut session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_mut()
        .ok_or_else(|| "no emulator session is running".to_owned())?;

    if session
        .autodrive
        .as_ref()
        .is_some_and(|status| status.active)
    {
        let _ = finish_autodrive(session, AutodriveStopReason::OperatorStop)?;
    } else {
        session.pending_agent_turn = None;
        session.last_authority_reason = Some("operator cancelled pending agent turn".to_owned());
    }
    Ok(driver_status_for(session))
}

#[tauri::command]
async fn list_ollama_models(base_url: String) -> Result<Vec<OllamaModel>, String> {
    ollama::list_models(&base_url).await
}

#[tauri::command]
async fn inspect_ollama_model(
    base_url: String,
    model: String,
) -> Result<OllamaModelDetails, String> {
    ollama::inspect_model(&base_url, &model).await
}

#[tauri::command]
async fn qualify_ollama_model(
    app: AppHandle,
    base_url: String,
    model: String,
) -> Result<OllamaQualificationArtifact, String> {
    let receipt = ollama::qualify_model(&base_url, &model).await?;
    persist_ollama_qualification(&app, &receipt)
}

#[tauri::command]
async fn ollama_qualification_status(
    app: AppHandle,
    base_url: String,
    model: String,
) -> Result<OllamaQualificationStatus, String> {
    let details = ollama::inspect_model(&base_url, &model).await?;
    let artifact = load_ollama_qualification(&app, &details.digest)?;
    let (qualified, receipt, receipt_path) = match artifact {
        Some(artifact) => {
            let qualified =
                qualification_receipt_passes(&artifact.receipt, &details.name, &details.digest);
            (
                qualified,
                Some(artifact.receipt),
                Some(artifact.receipt_path),
            )
        }
        None => (false, None, None),
    };

    Ok(OllamaQualificationStatus {
        details,
        qualified,
        receipt,
        receipt_path,
    })
}

#[tauri::command]
async fn complete_ollama_turn(
    request: AgentTurnRequest,
    base_url: String,
    model: String,
) -> Result<OllamaTurnResult, String> {
    ollama::complete_turn(request, &base_url, &model).await
}

#[tauri::command]
fn submit_agent_turn(
    state: State<'_, EmulatorState>,
    response: AgentTurnResponse,
) -> Result<DriverStatus, String> {
    let mut session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_mut()
        .ok_or_else(|| "no emulator session is running".to_owned())?;

    if session.recording.is_some() {
        return Err("agent turn submission is disabled during replay recording".into());
    }

    let request = session
        .pending_agent_turn
        .as_ref()
        .cloned()
        .ok_or_else(|| "no agent turn is pending".to_owned())?;
    let apply_frame = session.core.frame_count();
    if let Err(error) = response.validate_against(&request, apply_frame) {
        if error.to_ascii_lowercase().contains("memory") {
            session.agent_memory.refusals = session.agent_memory.refusals.saturating_add(1);
        }
        return Err(error);
    }
    let max_action_delay_frames = response
        .actions
        .iter()
        .map(|action| action.delay_frames)
        .max()
        .unwrap_or(0);
    let compiled = compile_agent_turn(&request, &response, apply_frame)?;
    let compiled_count = compiled.len();

    if let Some(status) = session.autodrive.as_ref().filter(|status| status.active) {
        if !status.can_accept_actions(compiled_count) {
            let _ = finish_autodrive(session, AutodriveStopReason::ActionBudget)?;
            return Err("autonomous run action budget would be exceeded".into());
        }
    }

    if let Some(memory_update) = response.memory_update.as_ref() {
        if memory_update != &session.agent_memory.content {
            session.agent_memory.content = memory_update.clone();
            session.agent_memory.revision = session.agent_memory.revision.saturating_add(1);
            session.agent_memory.updates = session.agent_memory.updates.saturating_add(1);
            session.agent_memory.bytes_written = session
                .agent_memory
                .bytes_written
                .saturating_add(memory_update.len() as u64);
        }
    }

    for action in compiled {
        session.agent_inbox.push_back(action);
    }
    session
        .agent_inbox
        .make_contiguous()
        .sort_by_key(|action| action.frame);
    session.pending_agent_turn = None;

    let empty_stop = if let Some(status) = session.autodrive.as_mut().filter(|status| status.active) {
        status.note_turn_completed_with_delay(
            compiled_count,
            max_action_delay_frames,
            apply_frame,
        );
        status.post_turn_stop_reason()
    } else {
        None
    };

    if let Some(reason) = empty_stop {
        let _ = finish_autodrive(session, reason)?;
    }

    Ok(driver_status_for(session))
}

#[tauri::command]
fn step_emulation(
    state: State<'_, EmulatorState>,
    actions: Vec<ActionEnvelope>,
) -> Result<FramePacket, String> {
    let mut session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_mut()
        .ok_or_else(|| "no emulator session is running".to_owned())?;

    let authority_frame = session.core.frame_count();

    let grant_expired = session
        .authority
        .agent_grant
        .as_ref()
        .and_then(|grant| grant.expires_at_frame)
        .is_some_and(|expires| authority_frame > expires);

    if grant_expired {
        if session
            .autodrive
            .as_ref()
            .is_some_and(|status| status.active)
        {
            let _ = finish_autodrive(session, AutodriveStopReason::GrantExpired)?;
        }
        session.authority.set_mode(ControlMode::Human, None)?;
        session.pending_agent_turn = None;
        session.agent_inbox.clear();
        session.agent_memory = AgentMemoryState::new();
        session.core.restore_input_mask(0);
        session.last_authority_reason =
            Some("Phi-Bot grant expired; control returned to HUMAN".to_owned());
    }

    if session
        .pending_agent_turn
        .as_ref()
        .is_some_and(|request| authority_frame > request.valid_until_frame)
    {
        session.pending_agent_turn = None;
        session.last_authority_reason =
            Some("agent driver turn expired before response".to_owned());
    }

    let mut incoming_actions = actions;

    while session
        .agent_inbox
        .front()
        .is_some_and(|event| event.frame <= authority_frame)
    {
        if let Some(event) = session.agent_inbox.pop_front() {
            incoming_actions.push(event);
        }
    }

    incoming_actions.sort_by_key(|event| {
        (
            event.frame,
            live_source_order(&event.source),
            event.sequence,
        )
    });

    let decisions = session
        .authority
        .authorize_batch(&incoming_actions, authority_frame);
    let mut accepted_actions = Vec::with_capacity(incoming_actions.len());
    for (mut event, decision) in decisions {
        let autodrive_blocks_system = session
            .autodrive
            .as_ref()
            .is_some_and(|status| status.active)
            && matches!(&event.action, ActionKind::System { .. });

        if decision.accepted && !autodrive_blocks_system {
            event.sequence = session.next_action_sequence;
            event.frame = authority_frame;
            session.next_action_sequence = session.next_action_sequence.saturating_add(1);
            accepted_actions.push(event);
        } else {
            session.authority_rejections = session.authority_rejections.saturating_add(1);
            session.last_authority_reason = Some(if autodrive_blocks_system {
                "system/timeline actions are disabled during autonomous driving".to_owned()
            } else {
                decision.reason
            });
        }
    }

    validate_recording_actions(session, &accepted_actions)?;
    record_applied_actions(session, &accepted_actions)?;
    process_session_actions(session, &accepted_actions)?;

    let speed = session.profile.fast_forward.clamp(1, 4);
    let mut video = FrameBuffer::default();
    let mut audio = AudioBuffer::default();

    for index in 0..speed {
        let frame_actions = if index == 0 {
            accepted_actions.as_slice()
        } else {
            &[]
        };
        session
            .core
            .step_frame(frame_actions, &mut video, &mut audio)
            .map_err(|error| format!("core frame failed: {error:?}"))?;

        session.last_frame = FrameBuffer {
            width: video.width,
            height: video.height,
            rgba8: video.rgba8.clone(),
        };

        let frame = session.core.frame_count();
        if session.runtime_features.state_snapshots && frame >= session.next_rewind_frame {
            push_rewind_snapshot(session)?;
            session.next_rewind_frame =
                frame + u64::from(session.profile.rewind_interval_frames);
        }

        maybe_record_replay_checkpoint(session)?;

        if frame.saturating_sub(session.last_sram_flush_frame) >= SRAM_FLUSH_INTERVAL_FRAMES {
            flush_save_ram(session)?;
            session.last_sram_flush_frame = frame;
        }
    }

    if session.core.shutdown_requested()
        && session
            .autodrive
            .as_ref()
            .is_some_and(|status| status.active)
    {
        let _ = finish_autodrive(session, AutodriveStopReason::CoreShutdown)?;
    } else {
        autodrive_post_step_guard(session)?;
    }

    let output_rate = if audio.sample_rate_hz > 96_000 {
        WEB_AUDIO_SAMPLE_RATE_HZ
    } else {
        audio.sample_rate_hz
    };
    let output_samples = resample_stereo(
        &audio.interleaved_stereo_f32,
        audio.sample_rate_hz,
        output_rate,
    );
    let mut audio_bytes = Vec::with_capacity(output_samples.len() * 2);
    for sample in output_samples {
        let quantized = (sample.clamp(-1.0, 1.0) * 32767.0).round() as i16;
        audio_bytes.extend_from_slice(&quantized.to_le_bytes());
    }

    let replay_recording = session.recording.is_some();
    let (replay_actions, replay_checkpoints) = session
        .recording
        .as_ref()
        .map(|recording| {
            (
                recording.ledger.actions.len(),
                recording.ledger.checkpoints.len(),
            )
        })
        .or_else(|| {
            session.last_replay.as_ref().map(|replay| {
                (
                    replay.receipt.action_count,
                    replay.receipt.checkpoint_count,
                )
            })
        })
        .unwrap_or((0, 0));

    Ok(FramePacket {
        frame: session.core.frame_count(),
        width: video.width,
        height: video.height,
        rgba_base64: BASE64.encode(video.rgba8),
        audio_base64: BASE64.encode(audio_bytes),
        sample_rate_hz: output_rate,
        shutdown_requested: session.core.shutdown_requested(),
        rewind_snapshots: session.rewind.len(),
        fast_forward: speed,
        replay_recording,
        replay_actions,
        replay_checkpoints,
        control_mode: session.authority.mode,
        authority_rejections: session.authority_rejections,
        last_authority_reason: session.last_authority_reason.clone(),
        driver_pending_turn_id: session
            .pending_agent_turn
            .as_ref()
            .map(|request| request.turn_id),
        driver_queued_actions: session.agent_inbox.len(),
        autodrive: autodrive_status_for(session),
    })
}

#[tauri::command]
fn set_game_profile(
    state: State<'_, EmulatorState>,
    profile: GameProfile,
) -> Result<GameProfile, String> {
    profile.validate()?;
    let mut session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_mut()
        .ok_or_else(|| "no emulator session is running".to_owned())?;

    if session.recording.is_some() {
        return Err("per-game profile changes are disabled during replay recording".into());
    }
    if session
        .autodrive
        .as_ref()
        .is_some_and(|status| status.active)
    {
        return Err("per-game profile changes are disabled during autonomous driving".into());
    }

    persist_game_profile(&session.paths.profile, &profile)?;
    session.profile = profile.clone();
    let capacity = rewind_capacity(session);
    while session.rewind.len() > capacity {
        session.rewind.pop_front();
    }
    session.next_rewind_frame =
        session.core.frame_count() + u64::from(session.profile.rewind_interval_frames);
    Ok(profile)
}


#[tauri::command]
fn replay_status(state: State<'_, EmulatorState>) -> Result<ReplayStatus, String> {
    let session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_ref()
        .ok_or_else(|| "no emulator session is running".to_owned())?;
    Ok(replay_status_for(session))
}

#[tauri::command]
fn start_replay_recording(state: State<'_, EmulatorState>) -> Result<ReplayStatus, String> {
    let mut session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_mut()
        .ok_or_else(|| "no emulator session is running".to_owned())?;

    if session.recording.is_some() {
        return Err("replay recording is already active".into());
    }
    if !session.runtime_features.exact_replay {
        return Err("Replay v1 is unavailable: the active runtime does not support exact replay".into());
    }
    if session
        .autodrive
        .as_ref()
        .is_some_and(|status| status.active)
    {
        return Err("stop autonomous driving before starting replay recording".into());
    }
    if session.profile.fast_forward != 1 {
        return Err("replay recording requires the per-game profile to be at 1x".into());
    }

    let start_frame = session.core.frame_count();
    let initial_state = session
        .core
        .serialize_state()
        .map_err(|error| format!("serialize replay initial state: {error:?}"))?;
    let initial_checkpoint = make_replay_checkpoint(session, &session.last_frame)?;
    let core_sha256 = sha256_file(session.core.core_path())?;

    session.recording = Some(ReplayRecording {
        ledger: ReplayLedger {
            schema: REPLAY_SCHEMA.to_owned(),
            game_sha256: session.game_key.clone(),
            core_name: session.core.identity().library_name.clone(),
            core_version: session.core.identity().library_version.clone(),
            core_sha256,
            start_frame,
            end_frame: start_frame,
            initial_state_base64: BASE64.encode(initial_state),
            initial_input_mask: session.core.input_mask_snapshot(),
            actions: Vec::new(),
            checkpoints: vec![initial_checkpoint.clone()],
            final_state_sha256: initial_checkpoint.state_sha256,
            final_frame_sha256: initial_checkpoint.frame_sha256,
        },
        next_checkpoint_frame: start_frame.saturating_add(REPLAY_CHECKPOINT_INTERVAL_FRAMES),
    });

    Ok(replay_status_for(session))
}

#[tauri::command]
fn stop_replay_recording(state: State<'_, EmulatorState>) -> Result<ReplayArtifact, String> {
    let mut session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_mut()
        .ok_or_else(|| "no emulator session is running".to_owned())?;

    let export = finalize_replay_recording(session)?;
    Ok(artifact_view(&export))
}

#[tauri::command]
fn verify_last_replay(state: State<'_, EmulatorState>) -> Result<ReplayArtifact, String> {
    let mut session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_mut()
        .ok_or_else(|| "no emulator session is running".to_owned())?;

    if session.recording.is_some() {
        return Err("stop replay recording before verification".into());
    }
    if !session.runtime_features.exact_replay {
        return Err("Replay v1 verification is unavailable: the active runtime does not support exact replay".into());
    }

    let mut export = session
        .last_replay
        .clone()
        .ok_or_else(|| "no replay has been exported in this session".to_owned())?;
    let replay_json = fs::read(&export.replay_path)
        .map_err(|error| format!("cannot read {}: {error}", export.replay_path.display()))?;
    let ledger: ReplayLedger = serde_json::from_slice(&replay_json)
        .map_err(|error| format!("cannot parse replay ledger: {error}"))?;
    let observed_replay_sha256 = sha256_bytes(&replay_json);

    if observed_replay_sha256 != export.replay_sha256 {
        return Err("exported replay hash no longer matches its content".into());
    }

    let backup_state = session
        .core
        .serialize_state()
        .map_err(|error| format!("serialize live state before verification: {error:?}"))?;
    let backup_frame = session.core.frame_count();
    let backup_input_mask = session.core.input_mask_snapshot();
    let backup_save_ram = session
        .core
        .read_save_ram()
        .map_err(|error| format!("read live save RAM before verification: {error:?}"))?;
    let backup_last_frame = session.last_frame.clone();
    let backup_rewind = session.rewind.clone();
    let backup_next_rewind_frame = session.next_rewind_frame;
    let backup_last_sram_flush_frame = session.last_sram_flush_frame;

    let verification = verify_replay_execution(session, &ledger);

    let restore_result = session
        .core
        .restore_state(&backup_state, backup_frame)
        .map_err(|error| format!("restore live state after verification: {error:?}"))
        .and_then(|_| {
            session.core.restore_input_mask(backup_input_mask);
            session
                .core
                .write_save_ram(&backup_save_ram)
                .map_err(|error| format!("restore live save RAM after verification: {error:?}"))
        });

    session.last_frame = backup_last_frame;
    session.rewind = backup_rewind;
    session.next_rewind_frame = backup_next_rewind_frame;
    session.last_sram_flush_frame = backup_last_sram_flush_frame;

    restore_result?;
    let verification = verification?;

    export.receipt = replay_receipt(
        &ledger,
        &export.replay_sha256,
        Some(verification),
    );
    let receipt_json = serde_json::to_vec_pretty(&export.receipt)
        .map_err(|error| format!("serialize verified replay receipt: {error}"))?;
    write_atomic(&export.receipt_path, &receipt_json)?;

    session.last_replay = Some(export.clone());
    Ok(artifact_view(&export))
}

#[tauri::command]
fn capture_screenshot(state: State<'_, EmulatorState>) -> Result<String, String> {
    let session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_ref()
        .ok_or_else(|| "no emulator session is running".to_owned())?;

    if session.last_frame.width == 0
        || session.last_frame.height == 0
        || session.last_frame.rgba8.is_empty()
    {
        return Err("no rendered frame is available yet".into());
    }

    let path = session.paths.screenshot_dir.join(format!(
        "frame-{:012}.png",
        session.core.frame_count()
    ));
    let file = fs::File::create(&path)
        .map_err(|error| format!("cannot create {}: {error}", path.display()))?;
    let writer = BufWriter::new(file);
    let mut encoder = png::Encoder::new(
        writer,
        session.last_frame.width,
        session.last_frame.height,
    );
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut png_writer = encoder
        .write_header()
        .map_err(|error| format!("cannot encode PNG header: {error}"))?;
    png_writer
        .write_image_data(&session.last_frame.rgba8)
        .map_err(|error| format!("cannot encode PNG pixels: {error}"))?;

    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
fn flush_game_save(state: State<'_, EmulatorState>) -> Result<(), String> {
    let session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_ref()
        .ok_or_else(|| "no emulator session is running".to_owned())?;
    flush_save_ram(session)
}

#[tauri::command]
fn stop_emulation(state: State<'_, EmulatorState>) -> Result<(), String> {
    let mut session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    if let Some(active) = session.as_mut() {
        if active.recording.is_some() {
            return Err("stop replay recording before ejecting the game".into());
        }
        if active
            .autodrive
            .as_ref()
            .is_some_and(|status| status.active)
        {
            let _ = finish_autodrive(active, AutodriveStopReason::CoreShutdown)?;
        }
        if active
            .benchmark_campaign
            .as_ref()
            .is_some_and(|campaign| campaign.active)
        {
            let _ = finish_benchmark_campaign(active, "PARTIAL")?;
        }
        flush_save_ram(active)?;
    }
    *session = None;
    Ok(())
}

#[tauri::command]
fn last_autodrive_receipt(
    state: State<'_, EmulatorState>,
) -> Result<Option<AutodriveArtifact>, String> {
    let session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    let session = session
        .as_ref()
        .ok_or_else(|| "no emulator session is running".to_owned())?;
    Ok(session.last_autodrive.as_ref().map(autodrive_artifact))
}

#[tauri::command]
fn running_game(state: State<'_, EmulatorState>) -> Result<Option<String>, String> {
    let session = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned".to_owned())?;
    Ok(session.as_ref().map(|value| value.game_path.clone()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(EmulatorState::default())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            load_settings,
            save_settings,
            register_runtime_core,
            list_runtime_registrations,
            start_registered_emulation,
            scan_rom_directory,
            validate_action_envelope,
            start_emulation,
            step_emulation,
            authority_status,
            set_control_mode,
            phi_bot_observation,
            driver_status,
            start_autodrive,
            start_model_gameplay_benchmark,
            last_model_gameplay_benchmark,
            start_benchmark_campaign,
            continue_benchmark_campaign,
            benchmark_campaign_status,
            last_benchmark_campaign_receipt,
            list_benchmark_campaign_receipts,
            list_benchmark_suites,
            list_benchmark_suite_report_candidates,
            build_benchmark_suite_report,
            list_benchmark_suite_reports,
            export_benchmark_suite_public_result,
            compare_benchmark_suite_reports,
            compare_benchmark_campaigns,
            cancel_benchmark_campaign,
            autodrive_status,
            stop_autodrive,
            fail_autodrive_provider,
            last_autodrive_receipt,
            issue_agent_turn,
            cancel_agent_turn,
            list_ollama_models,
            inspect_ollama_model,
            qualify_ollama_model,
            ollama_qualification_status,
            complete_ollama_turn,
            submit_agent_turn,
            set_game_profile,
            replay_status,
            start_replay_recording,
            stop_replay_recording,
            verify_last_replay,
            capture_screenshot,
            flush_game_save,
            stop_emulation,
            running_game
        ])
        .run(tauri::generate_context!())
        .expect("error while running PhiCade");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scummvm_session_features_disable_timeline_state() {
        let identity = CoreIdentity {
            library_name: "ScummVM".into(),
            library_version: phicade_libretro::SCUMMVM_VERSION.into(),
            valid_extensions: "scummvm".into(),
            need_fullpath: true,
            fps: 60.0,
            sample_rate_hz: 48_000,
        };
        let manifest = phicade_libretro::libretro_capability_manifest(&identity, false, false);
        let features = runtime_session_features(&manifest);

        assert!(!features.state_snapshots);
        assert!(!features.exact_replay);
        assert!(!features.persistent_save_data);
    }

    #[test]
    fn sameboy_session_features_keep_qualified_timeline_state() {
        let identity = CoreIdentity {
            library_name: "SameBoy".into(),
            library_version: phicade_libretro::SAMEBOY_VERSION.into(),
            valid_extensions: "gb|gbc".into(),
            need_fullpath: false,
            fps: 59.7,
            sample_rate_hz: 48_000,
        };
        let manifest = phicade_libretro::libretro_capability_manifest(&identity, true, true);
        let features = runtime_session_features(&manifest);

        assert!(features.state_snapshots);
        assert!(features.exact_replay);
        assert!(features.persistent_save_data);
    }

    #[test]
    fn runtime_registration_is_provenance_only_not_authority() {
        let identity = CoreIdentity {
            library_name: "ScummVM".into(),
            library_version: "v2026.3.0".into(),
            valid_extensions: "scummvm".into(),
            need_fullpath: true,
            fps: 60.0,
            sample_rate_hz: 48_000,
        };
        let manifest = phicade_libretro::libretro_capability_manifest(&identity, false, false);
        let receipt = runtime_registration_receipt(
            Path::new("/tmp/scummvm_libretro.so"),
            "a".repeat(64),
            identity,
            manifest,
        );

        assert_eq!(receipt.schema, RUNTIME_REGISTRATION_SCHEMA);
        assert_eq!(receipt.record_status, "REGISTERED");
        assert_eq!(
            receipt.qualification_profile_id.as_deref(),
            Some(phicade_libretro::SCUMMVM_QUALIFICATION_PROFILE_ID)
        );
        assert!(!receipt.binary_evidence_bound);
        assert!(!receipt.authority_granted);
    }

    #[test]
    fn public_suite_markdown_preserves_provenance_and_boundaries() {
        let receipt = PublicSuiteResultReceipt {
            schema: PUBLIC_SUITE_RESULT_SCHEMA.into(),
            record_status: "VERIFIED_EXPORT".into(),
            suite_id: "suite-v13".into(),
            suite_version: 13,
            suite_title: "Benchmark Suite v13".into(),
            report_id: 7,
            suite_report_sha256: "a".repeat(64),
            cohort_id: "cohort-a".into(),
            provider: "ollama".into(),
            model: "vision|model".into(),
            model_digest: "b".repeat(64),
            model_qualification_sha256: "c".repeat(64),
            core_sha256: "d".repeat(64),
            core_name: "SameBoy".into(),
            core_version: "1.0.3".into(),
            policy: AutodrivePolicy::default(),
            trials_per_task: 5,
            total_scored_trials: 5,
            total_scoring_error_trials: 0,
            stats: BenchmarkSuiteAggregateStats {
                task_count: 1,
                total_observed_trials: 5,
                total_successful_trials: 4,
                overall_success_rate: 0.8,
                macro_mean_score_1000: 850.0,
                min_task_mean_score_1000: 850.0,
                max_task_mean_score_1000: 850.0,
                population_stddev_task_mean_score_1000: 0.0,
            },
            tasks: vec![PublicSuiteTaskResult {
                task_id: "task-a".into(),
                task_title: "Task | A".into(),
                campaign_id: 3,
                campaign_receipt_sha256: "e".repeat(64),
                observed_trials: 5,
                scored_trials: 5,
                scoring_error_trials: 0,
                successful_trials: 4,
                success_rate: 0.8,
                mean_score_1000: 850.0,
            }],
        };

        let markdown = render_public_suite_result_markdown(&receipt, &"f".repeat(64));
        assert!(markdown.contains("850.0 / 1000"));
        assert!(markdown.contains("80.0%"));
        assert!(markdown.contains(&"a".repeat(64)));
        assert!(markdown.contains(&"f".repeat(64)));
        assert!(markdown.contains("Task \\| A"));
        assert!(markdown.contains("does not declare a universal winner"));
    }

    #[test]
    fn public_suite_markdown_is_deterministic() {
        let receipt = PublicSuiteResultReceipt {
            schema: PUBLIC_SUITE_RESULT_SCHEMA.into(),
            record_status: "VERIFIED_EXPORT".into(),
            suite_id: "suite-v1".into(),
            suite_version: 1,
            suite_title: "Benchmark Suite v1".into(),
            report_id: 1,
            suite_report_sha256: "1".repeat(64),
            cohort_id: "cohort".into(),
            provider: "ollama".into(),
            model: "model".into(),
            model_digest: "2".repeat(64),
            model_qualification_sha256: "3".repeat(64),
            core_sha256: "4".repeat(64),
            core_name: "SameBoy".into(),
            core_version: "1.0.3".into(),
            policy: AutodrivePolicy::default(),
            trials_per_task: 3,
            total_scored_trials: 3,
            total_scoring_error_trials: 0,
            stats: BenchmarkSuiteAggregateStats {
                task_count: 1,
                total_observed_trials: 3,
                total_successful_trials: 1,
                overall_success_rate: 1.0 / 3.0,
                macro_mean_score_1000: 500.0,
                min_task_mean_score_1000: 500.0,
                max_task_mean_score_1000: 500.0,
                population_stddev_task_mean_score_1000: 0.0,
            },
            tasks: vec![PublicSuiteTaskResult {
                task_id: "task".into(),
                task_title: "Task".into(),
                campaign_id: 1,
                campaign_receipt_sha256: "5".repeat(64),
                observed_trials: 3,
                scored_trials: 3,
                scoring_error_trials: 0,
                successful_trials: 1,
                success_rate: 1.0 / 3.0,
                mean_score_1000: 500.0,
            }],
        };

        let first = render_public_suite_result_markdown(&receipt, &"6".repeat(64));
        let second = render_public_suite_result_markdown(&receipt, &"6".repeat(64));
        assert_eq!(first, second);
    }

    #[test]
    fn classifier_accepts_known_cartridge_formats() {
        assert_eq!(classify_extension("nes"), Some("NES"));
        assert_eq!(classify_extension("sfc"), Some("SNES"));
        assert_eq!(classify_extension("gba"), Some("GBA"));
        assert_eq!(classify_extension("md"), Some("GENESIS"));
    }

    #[test]
    fn classifier_rejects_unknown_files() {
        assert_eq!(classify_extension("txt"), None);
        assert_eq!(classify_extension("exe"), None);
        assert_eq!(classify_extension("zip"), None);
    }

    #[test]
    fn resamples_core_audio_into_web_audio_range() {
        let input = vec![0.25, -0.25, 0.25, -0.25, 0.25, -0.25, 0.25, -0.25];
        let output = resample_stereo(&input, 96_000, 48_000);
        assert_eq!(output.len(), 4);
        assert!(output.iter().all(|sample| sample.abs() <= 0.25));
    }

    #[test]
    fn state_envelope_round_trips_frame_and_bytes() {
        let encoded = encode_state(369, &[1, 2, 3, 4]);
        let (frame, state) = decode_state(&encoded).expect("decode state");
        assert_eq!(frame, 369);
        assert_eq!(state, &[1, 2, 3, 4]);
    }

    #[test]
    fn game_profile_rejects_ungoverned_speed() {
        let profile = GameProfile {
            fast_forward: 3,
            ..GameProfile::default()
        };
        assert!(profile.validate().is_err());
    }

    #[test]
    fn rung_four_maps_game_boy_extensions() {
        assert_eq!(system_id_from_extension("gb"), Some(SystemId::GameBoy));
        assert_eq!(system_id_from_extension("gbc"), Some(SystemId::GameBoyColor));
        assert_eq!(system_id_from_extension("gba"), None);
    }

    fn qualified_receipt(digest: &str) -> OllamaQualificationReceipt {
        OllamaQualificationReceipt {
            schema: "phicade.ollama-model-qualification.v1".into(),
            result: "PASS".into(),
            provider: "ollama".into(),
            model: "vision-model".into(),
            digest: digest.into(),
            capabilities: vec!["completion".into(), "vision".into()],
            vision_advertised: true,
            structured_output_pass: true,
            vision_probe_pass: true,
            probe_expected: "red".into(),
            probe_observed: Some("red".into()),
            total_duration_ns: Some(1),
            eval_count: Some(1),
            error: None,
        }
    }

    #[test]
    fn model_qualification_gate_accepts_exact_digest() {
        let receipt = qualified_receipt("digest-a");
        assert!(qualification_receipt_passes(
            &receipt,
            "vision-model",
            "digest-a"
        ));
    }

    #[test]
    fn model_qualification_gate_rejects_changed_digest() {
        let receipt = qualified_receipt("digest-a");
        assert!(!qualification_receipt_passes(
            &receipt,
            "vision-model",
            "digest-b"
        ));
    }

    #[test]
    fn campaign_trial_count_is_bounded() {
        assert!(validate_campaign_trial_count(2).is_err());
        assert!(validate_campaign_trial_count(3).is_ok());
        assert!(validate_campaign_trial_count(20).is_ok());
        assert!(validate_campaign_trial_count(21).is_err());
    }

    #[test]
    fn campaign_receipt_serializes_trial_hashes_and_stats() {
        let stats = summarize_benchmark_trials(&[
            BenchmarkTrialOutcome {
                score_1000: Some(1000),
                task_success: true,
            },
            BenchmarkTrialOutcome {
                score_1000: Some(500),
                task_success: false,
            },
            BenchmarkTrialOutcome {
                score_1000: None,
                task_success: false,
            },
        ]);
        let receipt = BenchmarkCampaignReceipt {
            schema: BENCHMARK_CAMPAIGN_SCHEMA.into(),
            record_status: "COMPLETE".into(),
            campaign_id: 3,
            benchmark_id: AGENT_GYM_ID.into(),
            provider: "ollama".into(),
            model: "vision-model".into(),
            model_digest: "digest-a".into(),
            model_qualification_sha256: "q".repeat(64),
            gym_source_sha256: AGENT_GYM_SOURCE_SHA256.into(),
            gym_rom_sha256: AGENT_GYM_ROM_SHA256.into(),
            core_sha256: "c".repeat(64),
            core_name: "SameBoy".into(),
            core_version: "1.0.3".into(),
            policy: AutodrivePolicy::default(),
            total_trials: 3,
            completed_trials: 3,
            trials: vec![
                CampaignTrialEvidence {
                    benchmark_run_id: 1,
                    receipt_sha256: "a".repeat(64),
                    record_status: "COMPLETE".into(),
                    score_1000: Some(1000),
                    task_success: true,
                    stop_reason: AutodriveStopReason::TaskSuccess,
                },
                CampaignTrialEvidence {
                    benchmark_run_id: 2,
                    receipt_sha256: "b".repeat(64),
                    record_status: "COMPLETE".into(),
                    score_1000: Some(500),
                    task_success: false,
                    stop_reason: AutodriveStopReason::TurnBudget,
                },
                CampaignTrialEvidence {
                    benchmark_run_id: 3,
                    receipt_sha256: "d".repeat(64),
                    record_status: "SCORING_ERROR".into(),
                    score_1000: None,
                    task_success: false,
                    stop_reason: AutodriveStopReason::ProviderFailure,
                },
            ],
            stats,
        };

        let json = serde_json::to_value(&receipt).expect("serialize campaign receipt");
        assert_eq!(json["schema"], BENCHMARK_CAMPAIGN_SCHEMA);
        assert_eq!(json["trials"][0]["receiptSha256"], "a".repeat(64));
        assert_eq!(json["stats"]["scoredTrials"], 2);
        assert_eq!(json["stats"]["scoringErrorTrials"], 1);
        assert_eq!(json["stats"]["meanScore1000"], 750.0);
    }

    fn comparison_test_campaign(
        campaign_id: u64,
        model: &str,
        digest: &str,
    ) -> BenchmarkCampaignReceipt {
        let outcomes = [
            BenchmarkTrialOutcome {
                score_1000: Some(900),
                task_success: true,
            },
            BenchmarkTrialOutcome {
                score_1000: Some(750),
                task_success: false,
            },
            BenchmarkTrialOutcome {
                score_1000: Some(600),
                task_success: false,
            },
        ];
        BenchmarkCampaignReceipt {
            schema: BENCHMARK_CAMPAIGN_SCHEMA.into(),
            record_status: "COMPLETE".into(),
            campaign_id,
            benchmark_id: AGENT_GYM_ID.into(),
            provider: "ollama".into(),
            model: model.into(),
            model_digest: digest.into(),
            model_qualification_sha256: format!("q-{digest}"),
            gym_source_sha256: AGENT_GYM_SOURCE_SHA256.into(),
            gym_rom_sha256: AGENT_GYM_ROM_SHA256.into(),
            core_sha256: "c".repeat(64),
            core_name: "SameBoy".into(),
            core_version: "1.0.3".into(),
            policy: AutodrivePolicy::default(),
            total_trials: 3,
            completed_trials: 3,
            trials: vec![
                CampaignTrialEvidence {
                    benchmark_run_id: 1,
                    receipt_sha256: "a".repeat(64),
                    record_status: "COMPLETE".into(),
                    score_1000: Some(900),
                    task_success: true,
                    stop_reason: AutodriveStopReason::TaskSuccess,
                },
                CampaignTrialEvidence {
                    benchmark_run_id: 2,
                    receipt_sha256: "b".repeat(64),
                    record_status: "COMPLETE".into(),
                    score_1000: Some(750),
                    task_success: false,
                    stop_reason: AutodriveStopReason::TurnBudget,
                },
                CampaignTrialEvidence {
                    benchmark_run_id: 3,
                    receipt_sha256: "d".repeat(64),
                    record_status: "COMPLETE".into(),
                    score_1000: Some(600),
                    task_success: false,
                    stop_reason: AutodriveStopReason::TurnBudget,
                },
            ],
            stats: summarize_benchmark_trials(&outcomes),
        }
    }

    fn comparison_test_dir(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "phicade-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create test dir");
        path
    }

    #[test]
    fn comparison_accepts_different_models_on_identical_environment() {
        let a = comparison_test_campaign(1, "model-a", "digest-a");
        let b = comparison_test_campaign(2, "model-b", "digest-b");
        validate_campaign_compatibility(&a, &b).expect("compatible campaigns");
    }

    #[test]
    fn comparison_refuses_same_campaign_and_policy_drift() {
        let a = comparison_test_campaign(1, "model-a", "digest-a");
        let same = comparison_test_campaign(1, "model-b", "digest-b");
        assert!(validate_campaign_compatibility(&a, &same).is_err());

        let mut drifted = comparison_test_campaign(2, "model-b", "digest-b");
        drifted.policy.max_turns += 1;
        assert!(validate_campaign_compatibility(&a, &drifted).is_err());
    }

    #[test]
    fn comparison_refuses_partial_and_scoring_error_campaigns_before_hash_walk() {
        let directory = comparison_test_dir("comparison-refusal");

        let mut partial = comparison_test_campaign(1, "model-a", "digest-a");
        partial.record_status = "PARTIAL".into();
        assert!(validate_campaign_for_comparison(&directory, &partial).is_err());

        let mut scoring_error = comparison_test_campaign(2, "model-b", "digest-b");
        scoring_error.trials[2].record_status = "SCORING_ERROR".into();
        scoring_error.trials[2].score_1000 = None;
        scoring_error.stats = summarize_benchmark_trials(&[
            BenchmarkTrialOutcome { score_1000: Some(900), task_success: true },
            BenchmarkTrialOutcome { score_1000: Some(750), task_success: false },
            BenchmarkTrialOutcome { score_1000: None, task_success: false },
        ]);
        assert!(validate_campaign_for_comparison(&directory, &scoring_error).is_err());

        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn comparison_trial_hash_verifier_detects_mutation() {
        let directory = comparison_test_dir("comparison-hash");
        let mut campaign = comparison_test_campaign(1, "model-a", "digest-a");

        for (index, trial) in campaign.trials.iter_mut().enumerate() {
            let bytes = format!("trial-evidence-{index}").into_bytes();
            let path = directory.join(format!("run-{:06}.json", trial.benchmark_run_id));
            fs::write(&path, &bytes).expect("write trial");
            trial.receipt_sha256 = sha256_bytes(&bytes);
        }

        verify_campaign_trial_receipts(&directory, &campaign).expect("hashes pass");

        let first = directory.join("run-000001.json");
        fs::write(&first, b"mutated-evidence").expect("mutate trial");
        assert!(verify_campaign_trial_receipts(&directory, &campaign).is_err());

        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn numbered_receipt_ids_advance_past_existing_evidence() {
        let directory = comparison_test_dir("receipt-ids");
        fs::write(directory.join("run-000001.json"), b"one").expect("write one");
        fs::write(directory.join("run-000007.json"), b"seven").expect("write seven");
        fs::write(directory.join("ignore-me.txt"), b"noise").expect("write noise");

        assert_eq!(
            next_numbered_receipt_id(&directory, "run-", ".json").expect("next id"),
            8
        );

        let _ = fs::remove_dir_all(directory);
    }

    fn suite_test_campaign(
        task: &BenchmarkTaskSpec,
        campaign_id: u64,
        digest: &str,
    ) -> BenchmarkCampaignReceipt {
        let outcomes = [
            BenchmarkTrialOutcome {
                score_1000: Some(900),
                task_success: true,
            },
            BenchmarkTrialOutcome {
                score_1000: Some(700),
                task_success: false,
            },
            BenchmarkTrialOutcome {
                score_1000: Some(500),
                task_success: false,
            },
        ];
        BenchmarkCampaignReceipt {
            schema: BENCHMARK_CAMPAIGN_SCHEMA.into(),
            record_status: "COMPLETE".into(),
            campaign_id,
            benchmark_id: task.id.into(),
            provider: "ollama".into(),
            model: "suite-model".into(),
            model_digest: digest.into(),
            model_qualification_sha256: format!("qualification-{digest}"),
            gym_source_sha256: task.source_sha256.into(),
            gym_rom_sha256: task.rom_sha256.into(),
            core_sha256: "c".repeat(64),
            core_name: "SameBoy".into(),
            core_version: "1.0.3".into(),
            policy: AutodrivePolicy::default(),
            total_trials: 3,
            completed_trials: 3,
            trials: vec![
                CampaignTrialEvidence {
                    benchmark_run_id: 1,
                    receipt_sha256: String::new(),
                    record_status: "COMPLETE".into(),
                    score_1000: Some(900),
                    task_success: true,
                    stop_reason: AutodriveStopReason::TaskSuccess,
                },
                CampaignTrialEvidence {
                    benchmark_run_id: 2,
                    receipt_sha256: String::new(),
                    record_status: "COMPLETE".into(),
                    score_1000: Some(700),
                    task_success: false,
                    stop_reason: AutodriveStopReason::TurnBudget,
                },
                CampaignTrialEvidence {
                    benchmark_run_id: 3,
                    receipt_sha256: String::new(),
                    record_status: "COMPLETE".into(),
                    score_1000: Some(500),
                    task_success: false,
                    stop_reason: AutodriveStopReason::TurnBudget,
                },
            ],
            stats: summarize_benchmark_trials(&outcomes),
        }
    }

    fn write_suite_test_campaign(
        campaign_root: &Path,
        model_root: &Path,
        task: &BenchmarkTaskSpec,
        campaign_id: u64,
        digest: &str,
    ) -> BenchmarkCampaignReceipt {
        let mut campaign = suite_test_campaign(task, campaign_id, digest);
        let campaign_dir = campaign_root.join(task.rom_sha256);
        let model_dir = model_root.join(task.rom_sha256);
        fs::create_dir_all(&campaign_dir).expect("create suite campaign task dir");
        fs::create_dir_all(&model_dir).expect("create suite model task dir");

        for (index, trial) in campaign.trials.iter_mut().enumerate() {
            let bytes = format!("{}-trial-{index}", task.id).into_bytes();
            let path = model_dir.join(format!("run-{:06}.json", trial.benchmark_run_id));
            fs::write(&path, &bytes).expect("write suite trial evidence");
            trial.receipt_sha256 = sha256_bytes(&bytes);
        }

        let campaign_path = campaign_dir.join(format!("campaign-{campaign_id:06}.json"));
        fs::write(
            &campaign_path,
            serde_json::to_vec_pretty(&campaign).expect("serialize suite campaign"),
        )
        .expect("write suite campaign");
        campaign
    }

    #[test]
    fn suite_cohort_becomes_ready_only_with_every_registered_task() {
        let root = comparison_test_dir("suite-ready");
        let campaign_root = root.join("campaigns");
        let model_root = root.join("models");
        fs::create_dir_all(&campaign_root).expect("create campaign root");
        fs::create_dir_all(&model_root).expect("create model root");

        for task in benchmark_suite_v1_tasks() {
            write_suite_test_campaign(&campaign_root, &model_root, task, 1, "digest-suite");
        }

        let cohorts =
            scan_benchmark_suite_cohorts_from_roots(&campaign_root, &model_root, BENCHMARK_SUITE_V1_ID)
                .expect("scan suite cohorts");
        assert_eq!(cohorts.len(), 1);
        let (cohort_id, evidence) = cohorts.iter().next().expect("cohort");
        let candidate = suite_report_candidate(BENCHMARK_SUITE_V1_ID, cohort_id, evidence).expect("candidate");
        assert!(candidate.ready);
        assert_eq!(usize::from(candidate.covered_tasks), benchmark_suite_v1_tasks().len());
        assert_eq!(candidate.tasks.len(), benchmark_suite_v1_tasks().len());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn suite_v2_adds_wall_detour_without_mutating_v1_coverage() {
        let root = comparison_test_dir("suite-v2-membership");
        let campaign_root = root.join("campaigns");
        let model_root = root.join("models");
        fs::create_dir_all(&campaign_root).expect("create campaign root");
        fs::create_dir_all(&model_root).expect("create model root");

        for task in benchmark_suite_v1_tasks() {
            write_suite_test_campaign(&campaign_root, &model_root, task, 1, "digest-suite");
        }

        let v1_cohorts = scan_benchmark_suite_cohorts_from_roots(
            &campaign_root,
            &model_root,
            BENCHMARK_SUITE_V1_ID,
        )
        .expect("scan v1 suite");
        let (v1_id, v1_evidence) = v1_cohorts.iter().next().expect("v1 cohort");
        let v1_candidate =
            suite_report_candidate(BENCHMARK_SUITE_V1_ID, v1_id, v1_evidence)
                .expect("v1 candidate");
        assert!(v1_candidate.ready);
        assert_eq!(v1_candidate.covered_tasks, 2);
        assert_eq!(v1_candidate.suite_task_count, 2);

        let v2_cohorts = scan_benchmark_suite_cohorts_from_roots(
            &campaign_root,
            &model_root,
            BENCHMARK_SUITE_V2_ID,
        )
        .expect("scan incomplete v2 suite");
        let (v2_id, v2_evidence) = v2_cohorts.iter().next().expect("v2 cohort");
        let v2_candidate =
            suite_report_candidate(BENCHMARK_SUITE_V2_ID, v2_id, v2_evidence)
                .expect("v2 candidate");
        assert!(!v2_candidate.ready);
        assert_eq!(v2_candidate.covered_tasks, 2);
        assert_eq!(v2_candidate.suite_task_count, 3);

        let v2 = benchmark_suite_by_id(BENCHMARK_SUITE_V2_ID).expect("v2 registry");
        let wall = v2.tasks.iter().find(|task| task.id == "wall-detour-v1").expect("wall task");
        write_suite_test_campaign(&campaign_root, &model_root, wall, 1, "digest-suite");

        let v2_complete = scan_benchmark_suite_cohorts_from_roots(
            &campaign_root,
            &model_root,
            BENCHMARK_SUITE_V2_ID,
        )
        .expect("scan complete v2 suite");
        let (v2_complete_id, v2_complete_evidence) =
            v2_complete.iter().next().expect("complete v2 cohort");
        let v2_complete_candidate = suite_report_candidate(
            BENCHMARK_SUITE_V2_ID,
            v2_complete_id,
            v2_complete_evidence,
        )
        .expect("complete v2 candidate");
        assert!(v2_complete_candidate.ready);
        assert_eq!(v2_complete_candidate.covered_tasks, 3);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn suite_v3_requires_both_temporal_cues_without_mutating_v2_coverage() {
        let root = comparison_test_dir("suite-v3-membership");
        let campaign_root = root.join("campaigns");
        let model_root = root.join("models");
        fs::create_dir_all(&campaign_root).expect("create campaign root");
        fs::create_dir_all(&model_root).expect("create model root");

        let v3 = benchmark_suite_by_id(BENCHMARK_SUITE_V3_ID).expect("v3 registry");
        assert_eq!(v3.tasks.len(), 5);

        for task in &v3.tasks[..3] {
            write_suite_test_campaign(&campaign_root, &model_root, task, 1, "digest-suite");
        }

        let v2_cohorts = scan_benchmark_suite_cohorts_from_roots(
            &campaign_root,
            &model_root,
            BENCHMARK_SUITE_V2_ID,
        )
        .expect("scan complete v2 suite");
        let (v2_id, v2_evidence) = v2_cohorts.iter().next().expect("v2 cohort");
        let v2_candidate = suite_report_candidate(BENCHMARK_SUITE_V2_ID, v2_id, v2_evidence)
            .expect("v2 candidate");
        assert!(v2_candidate.ready);
        assert_eq!(v2_candidate.covered_tasks, 3);

        let v3_cohorts = scan_benchmark_suite_cohorts_from_roots(
            &campaign_root,
            &model_root,
            BENCHMARK_SUITE_V3_ID,
        )
        .expect("scan incomplete v3 suite");
        let (v3_id, v3_evidence) = v3_cohorts.iter().next().expect("v3 cohort");
        let v3_candidate = suite_report_candidate(BENCHMARK_SUITE_V3_ID, v3_id, v3_evidence)
            .expect("v3 candidate");
        assert!(!v3_candidate.ready);
        assert_eq!(v3_candidate.covered_tasks, 3);
        assert_eq!(v3_candidate.suite_task_count, 5);

        write_suite_test_campaign(
            &campaign_root,
            &model_root,
            &benchmark_suite_v3_tasks()[3],
            1,
            "digest-suite",
        );
        let one_cue = scan_benchmark_suite_cohorts_from_roots(
            &campaign_root,
            &model_root,
            BENCHMARK_SUITE_V3_ID,
        )
        .expect("scan one-cue v3 suite");
        let (one_id, one_evidence) = one_cue.iter().next().expect("one-cue cohort");
        let one_candidate = suite_report_candidate(BENCHMARK_SUITE_V3_ID, one_id, one_evidence)
            .expect("one-cue candidate");
        assert!(!one_candidate.ready);
        assert_eq!(one_candidate.covered_tasks, 4);

        write_suite_test_campaign(
            &campaign_root,
            &model_root,
            &benchmark_suite_v3_tasks()[4],
            1,
            "digest-suite",
        );
        let complete = scan_benchmark_suite_cohorts_from_roots(
            &campaign_root,
            &model_root,
            BENCHMARK_SUITE_V3_ID,
        )
        .expect("scan complete v3 suite");
        let (complete_id, complete_evidence) = complete.iter().next().expect("complete v3 cohort");
        let complete_candidate =
            suite_report_candidate(BENCHMARK_SUITE_V3_ID, complete_id, complete_evidence)
                .expect("complete v3 candidate");
        assert!(complete_candidate.ready);
        assert_eq!(complete_candidate.covered_tasks, 5);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn suite_v4_requires_both_relays_without_mutating_v3_coverage() {
        let root = comparison_test_dir("suite-v4-membership");
        let campaign_root = root.join("campaigns");
        let model_root = root.join("models");
        fs::create_dir_all(&campaign_root).expect("create campaign root");
        fs::create_dir_all(&model_root).expect("create model root");

        let v4 = benchmark_suite_by_id(BENCHMARK_SUITE_V4_ID).expect("v4 registry");
        assert_eq!(v4.tasks.len(), 7);

        for task in &v4.tasks[..5] {
            write_suite_test_campaign(&campaign_root, &model_root, task, 1, "digest-suite");
        }

        let v3_cohorts = scan_benchmark_suite_cohorts_from_roots(
            &campaign_root,
            &model_root,
            BENCHMARK_SUITE_V3_ID,
        )
        .expect("scan complete v3 suite");
        let (v3_id, v3_evidence) = v3_cohorts.iter().next().expect("v3 cohort");
        let v3_candidate = suite_report_candidate(BENCHMARK_SUITE_V3_ID, v3_id, v3_evidence)
            .expect("v3 candidate");
        assert!(v3_candidate.ready);
        assert_eq!(v3_candidate.covered_tasks, 5);

        let v4_cohorts = scan_benchmark_suite_cohorts_from_roots(
            &campaign_root,
            &model_root,
            BENCHMARK_SUITE_V4_ID,
        )
        .expect("scan incomplete v4 suite");
        let (v4_id, v4_evidence) = v4_cohorts.iter().next().expect("v4 cohort");
        let v4_candidate = suite_report_candidate(BENCHMARK_SUITE_V4_ID, v4_id, v4_evidence)
            .expect("v4 candidate");
        assert!(!v4_candidate.ready);
        assert_eq!(v4_candidate.covered_tasks, 5);
        assert_eq!(v4_candidate.suite_task_count, 7);

        write_suite_test_campaign(
            &campaign_root,
            &model_root,
            &benchmark_suite_v4_tasks()[5],
            1,
            "digest-suite",
        );
        let one_relay = scan_benchmark_suite_cohorts_from_roots(
            &campaign_root,
            &model_root,
            BENCHMARK_SUITE_V4_ID,
        )
        .expect("scan one-relay v4 suite");
        let (one_id, one_evidence) = one_relay.iter().next().expect("one-relay cohort");
        let one_candidate = suite_report_candidate(BENCHMARK_SUITE_V4_ID, one_id, one_evidence)
            .expect("one-relay candidate");
        assert!(!one_candidate.ready);
        assert_eq!(one_candidate.covered_tasks, 6);

        write_suite_test_campaign(
            &campaign_root,
            &model_root,
            &benchmark_suite_v4_tasks()[6],
            1,
            "digest-suite",
        );
        let complete = scan_benchmark_suite_cohorts_from_roots(
            &campaign_root,
            &model_root,
            BENCHMARK_SUITE_V4_ID,
        )
        .expect("scan complete v4 suite");
        let (complete_id, complete_evidence) = complete.iter().next().expect("complete v4 cohort");
        let complete_candidate =
            suite_report_candidate(BENCHMARK_SUITE_V4_ID, complete_id, complete_evidence)
                .expect("complete v4 candidate");
        assert!(complete_candidate.ready);
        assert_eq!(complete_candidate.covered_tasks, 7);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn suite_v5_requires_both_key_gates_without_mutating_v4_coverage() {
        let root = comparison_test_dir("suite-v5-membership");
        let campaign_root = root.join("campaigns");
        let model_root = root.join("models");
        fs::create_dir_all(&campaign_root).expect("create campaign root");
        fs::create_dir_all(&model_root).expect("create model root");

        let v5 = benchmark_suite_by_id(BENCHMARK_SUITE_V5_ID).expect("v5 registry");
        assert_eq!(v5.tasks.len(), 9);

        for task in &v5.tasks[..7] {
            write_suite_test_campaign(&campaign_root, &model_root, task, 1, "digest-suite");
        }

        let v4_cohorts = scan_benchmark_suite_cohorts_from_roots(
            &campaign_root,
            &model_root,
            BENCHMARK_SUITE_V4_ID,
        )
        .expect("scan complete v4 suite");
        let (v4_id, v4_evidence) = v4_cohorts.iter().next().expect("v4 cohort");
        let v4_candidate = suite_report_candidate(BENCHMARK_SUITE_V4_ID, v4_id, v4_evidence)
            .expect("v4 candidate");
        assert!(v4_candidate.ready);
        assert_eq!(v4_candidate.covered_tasks, 7);

        let v5_cohorts = scan_benchmark_suite_cohorts_from_roots(
            &campaign_root,
            &model_root,
            BENCHMARK_SUITE_V5_ID,
        )
        .expect("scan incomplete v5 suite");
        let (v5_id, v5_evidence) = v5_cohorts.iter().next().expect("v5 cohort");
        let v5_candidate = suite_report_candidate(BENCHMARK_SUITE_V5_ID, v5_id, v5_evidence)
            .expect("v5 candidate");
        assert!(!v5_candidate.ready);
        assert_eq!(v5_candidate.covered_tasks, 7);
        assert_eq!(v5_candidate.suite_task_count, 9);

        write_suite_test_campaign(
            &campaign_root,
            &model_root,
            &benchmark_suite_v5_tasks()[7],
            1,
            "digest-suite",
        );
        let one_key = scan_benchmark_suite_cohorts_from_roots(
            &campaign_root,
            &model_root,
            BENCHMARK_SUITE_V5_ID,
        )
        .expect("scan one-key v5 suite");
        let (one_id, one_evidence) = one_key.iter().next().expect("one-key cohort");
        let one_candidate = suite_report_candidate(BENCHMARK_SUITE_V5_ID, one_id, one_evidence)
            .expect("one-key candidate");
        assert!(!one_candidate.ready);
        assert_eq!(one_candidate.covered_tasks, 8);

        write_suite_test_campaign(
            &campaign_root,
            &model_root,
            &benchmark_suite_v5_tasks()[8],
            1,
            "digest-suite",
        );
        let complete = scan_benchmark_suite_cohorts_from_roots(
            &campaign_root,
            &model_root,
            BENCHMARK_SUITE_V5_ID,
        )
        .expect("scan complete v5 suite");
        let (complete_id, complete_evidence) = complete.iter().next().expect("complete v5 cohort");
        let complete_candidate =
            suite_report_candidate(BENCHMARK_SUITE_V5_ID, complete_id, complete_evidence)
                .expect("complete v5 candidate");
        assert!(complete_candidate.ready);
        assert_eq!(complete_candidate.covered_tasks, 9);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn suite_v6_requires_both_power_chains_without_mutating_v5_coverage() {
        let root = comparison_test_dir("suite-v6-membership");
        let campaign_root = root.join("campaigns");
        let model_root = root.join("models");
        fs::create_dir_all(&campaign_root).expect("create campaign root");
        fs::create_dir_all(&model_root).expect("create model root");

        let v6 = benchmark_suite_by_id(BENCHMARK_SUITE_V6_ID).expect("v6 registry");
        assert_eq!(v6.tasks.len(), 11);

        for task in &v6.tasks[..9] {
            write_suite_test_campaign(&campaign_root, &model_root, task, 1, "digest-suite");
        }

        let v5_cohorts = scan_benchmark_suite_cohorts_from_roots(
            &campaign_root,
            &model_root,
            BENCHMARK_SUITE_V5_ID,
        )
        .expect("scan complete v5 suite");
        let (v5_id, v5_evidence) = v5_cohorts.iter().next().expect("v5 cohort");
        let v5_candidate = suite_report_candidate(BENCHMARK_SUITE_V5_ID, v5_id, v5_evidence)
            .expect("v5 candidate");
        assert!(v5_candidate.ready);
        assert_eq!(v5_candidate.covered_tasks, 9);

        let v6_cohorts = scan_benchmark_suite_cohorts_from_roots(
            &campaign_root,
            &model_root,
            BENCHMARK_SUITE_V6_ID,
        )
        .expect("scan incomplete v6 suite");
        let (v6_id, v6_evidence) = v6_cohorts.iter().next().expect("v6 cohort");
        let v6_candidate = suite_report_candidate(BENCHMARK_SUITE_V6_ID, v6_id, v6_evidence)
            .expect("v6 candidate");
        assert!(!v6_candidate.ready);
        assert_eq!(v6_candidate.covered_tasks, 9);
        assert_eq!(v6_candidate.suite_task_count, 11);

        write_suite_test_campaign(
            &campaign_root,
            &model_root,
            &benchmark_suite_v6_tasks()[9],
            1,
            "digest-suite",
        );
        let one_power = scan_benchmark_suite_cohorts_from_roots(
            &campaign_root,
            &model_root,
            BENCHMARK_SUITE_V6_ID,
        )
        .expect("scan one-power v6 suite");
        let (one_id, one_evidence) = one_power.iter().next().expect("one-power cohort");
        let one_candidate = suite_report_candidate(BENCHMARK_SUITE_V6_ID, one_id, one_evidence)
            .expect("one-power candidate");
        assert!(!one_candidate.ready);
        assert_eq!(one_candidate.covered_tasks, 10);

        write_suite_test_campaign(
            &campaign_root,
            &model_root,
            &benchmark_suite_v6_tasks()[10],
            1,
            "digest-suite",
        );
        let complete = scan_benchmark_suite_cohorts_from_roots(
            &campaign_root,
            &model_root,
            BENCHMARK_SUITE_V6_ID,
        )
        .expect("scan complete v6 suite");
        let (complete_id, complete_evidence) = complete.iter().next().expect("complete v6 cohort");
        let complete_candidate =
            suite_report_candidate(BENCHMARK_SUITE_V6_ID, complete_id, complete_evidence)
                .expect("complete v6 candidate");
        assert!(complete_candidate.ready);
        assert_eq!(complete_candidate.covered_tasks, 11);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn suite_v7_requires_both_branch_selectors_without_mutating_v6_coverage() {
        let root = comparison_test_dir("suite-v7-membership");
        let campaign_root = root.join("campaigns");
        let model_root = root.join("models");
        fs::create_dir_all(&campaign_root).expect("create campaign root");
        fs::create_dir_all(&model_root).expect("create model root");

        let v7 = benchmark_suite_by_id(BENCHMARK_SUITE_V7_ID).expect("v7 registry");
        assert_eq!(v7.tasks.len(), 13);

        for task in &v7.tasks[..11] {
            write_suite_test_campaign(&campaign_root, &model_root, task, 1, "digest-suite");
        }

        let v6_cohorts = scan_benchmark_suite_cohorts_from_roots(
            &campaign_root,
            &model_root,
            BENCHMARK_SUITE_V6_ID,
        )
        .expect("scan complete v6 suite");
        let (v6_id, v6_evidence) = v6_cohorts.iter().next().expect("v6 cohort");
        let v6_candidate = suite_report_candidate(BENCHMARK_SUITE_V6_ID, v6_id, v6_evidence)
            .expect("v6 candidate");
        assert!(v6_candidate.ready);
        assert_eq!(v6_candidate.covered_tasks, 11);

        let v7_cohorts = scan_benchmark_suite_cohorts_from_roots(
            &campaign_root,
            &model_root,
            BENCHMARK_SUITE_V7_ID,
        )
        .expect("scan incomplete v7 suite");
        let (v7_id, v7_evidence) = v7_cohorts.iter().next().expect("v7 cohort");
        let v7_candidate = suite_report_candidate(BENCHMARK_SUITE_V7_ID, v7_id, v7_evidence)
            .expect("v7 candidate");
        assert!(!v7_candidate.ready);
        assert_eq!(v7_candidate.covered_tasks, 11);
        assert_eq!(v7_candidate.suite_task_count, 13);

        write_suite_test_campaign(
            &campaign_root,
            &model_root,
            &benchmark_suite_v7_tasks()[11],
            1,
            "digest-suite",
        );
        let one_branch = scan_benchmark_suite_cohorts_from_roots(
            &campaign_root,
            &model_root,
            BENCHMARK_SUITE_V7_ID,
        )
        .expect("scan one-branch v7 suite");
        let (one_id, one_evidence) = one_branch.iter().next().expect("one-branch cohort");
        let one_candidate = suite_report_candidate(BENCHMARK_SUITE_V7_ID, one_id, one_evidence)
            .expect("one-branch candidate");
        assert!(!one_candidate.ready);
        assert_eq!(one_candidate.covered_tasks, 12);

        write_suite_test_campaign(
            &campaign_root,
            &model_root,
            &benchmark_suite_v7_tasks()[12],
            1,
            "digest-suite",
        );
        let complete = scan_benchmark_suite_cohorts_from_roots(
            &campaign_root,
            &model_root,
            BENCHMARK_SUITE_V7_ID,
        )
        .expect("scan complete v7 suite");
        let (complete_id, complete_evidence) =
            complete.iter().next().expect("complete v7 cohort");
        let complete_candidate =
            suite_report_candidate(BENCHMARK_SUITE_V7_ID, complete_id, complete_evidence)
                .expect("complete v7 candidate");
        assert!(complete_candidate.ready);
        assert_eq!(complete_candidate.covered_tasks, 13);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn suite_v8_requires_all_four_nested_branches_without_mutating_v7_coverage() {
        let root = comparison_test_dir("suite-v8-membership");
        let campaign_root = root.join("campaigns");
        let model_root = root.join("models");
        fs::create_dir_all(&campaign_root).expect("create campaign root");
        fs::create_dir_all(&model_root).expect("create model root");

        let v8 = benchmark_suite_by_id(BENCHMARK_SUITE_V8_ID).expect("v8 registry");
        assert_eq!(v8.tasks.len(), 17);

        for task in &v8.tasks[..13] {
            write_suite_test_campaign(&campaign_root, &model_root, task, 1, "digest-suite");
        }

        let v7_cohorts = scan_benchmark_suite_cohorts_from_roots(
            &campaign_root, &model_root, BENCHMARK_SUITE_V7_ID,
        ).expect("scan complete v7 suite");
        let (v7_id, v7_evidence) = v7_cohorts.iter().next().expect("v7 cohort");
        let v7_candidate = suite_report_candidate(BENCHMARK_SUITE_V7_ID, v7_id, v7_evidence)
            .expect("v7 candidate");
        assert!(v7_candidate.ready);
        assert_eq!(v7_candidate.covered_tasks, 13);

        for expected_coverage in 13..=16 {
            let cohorts = scan_benchmark_suite_cohorts_from_roots(
                &campaign_root, &model_root, BENCHMARK_SUITE_V8_ID,
            ).expect("scan incomplete v8 suite");
            let (id, evidence) = cohorts.iter().next().expect("v8 cohort");
            let candidate = suite_report_candidate(BENCHMARK_SUITE_V8_ID, id, evidence)
                .expect("v8 candidate");
            assert!(!candidate.ready);
            assert_eq!(candidate.covered_tasks, expected_coverage);
            assert_eq!(candidate.suite_task_count, 17);

            if expected_coverage < 16 {
                write_suite_test_campaign(
                    &campaign_root,
                    &model_root,
                    &benchmark_suite_v8_tasks()[expected_coverage as usize],
                    1,
                    "digest-suite",
                );
            }
        }

        write_suite_test_campaign(
            &campaign_root,
            &model_root,
            &benchmark_suite_v8_tasks()[16],
            1,
            "digest-suite",
        );
        let complete = scan_benchmark_suite_cohorts_from_roots(
            &campaign_root, &model_root, BENCHMARK_SUITE_V8_ID,
        ).expect("scan complete v8 suite");
        let (complete_id, complete_evidence) = complete.iter().next().expect("complete v8 cohort");
        let complete_candidate =
            suite_report_candidate(BENCHMARK_SUITE_V8_ID, complete_id, complete_evidence)
                .expect("complete v8 candidate");
        assert!(complete_candidate.ready);
        assert_eq!(complete_candidate.covered_tasks, 17);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn suite_v9_requires_all_four_binding_memory_tasks_without_mutating_v8_coverage() {
        let root = comparison_test_dir("suite-v9-membership");
        let campaign_root = root.join("campaigns");
        let model_root = root.join("models");
        fs::create_dir_all(&campaign_root).expect("create campaign root");
        fs::create_dir_all(&model_root).expect("create model root");

        let v9 = benchmark_suite_by_id(BENCHMARK_SUITE_V9_ID).expect("v9 registry");
        assert_eq!(v9.tasks.len(), 21);

        for task in &v9.tasks[..17] {
            write_suite_test_campaign(&campaign_root, &model_root, task, 1, "digest-suite");
        }

        let v8_cohorts = scan_benchmark_suite_cohorts_from_roots(
            &campaign_root, &model_root, BENCHMARK_SUITE_V8_ID,
        ).expect("scan complete v8 suite");
        let (v8_id, v8_evidence) = v8_cohorts.iter().next().expect("v8 cohort");
        let v8_candidate = suite_report_candidate(BENCHMARK_SUITE_V8_ID, v8_id, v8_evidence)
            .expect("v8 candidate");
        assert!(v8_candidate.ready);
        assert_eq!(v8_candidate.covered_tasks, 17);

        for expected_coverage in 17..=20 {
            let cohorts = scan_benchmark_suite_cohorts_from_roots(
                &campaign_root, &model_root, BENCHMARK_SUITE_V9_ID,
            ).expect("scan incomplete v9 suite");
            let (id, evidence) = cohorts.iter().next().expect("v9 cohort");
            let candidate = suite_report_candidate(BENCHMARK_SUITE_V9_ID, id, evidence)
                .expect("v9 candidate");
            assert!(!candidate.ready);
            assert_eq!(candidate.covered_tasks, expected_coverage);
            assert_eq!(candidate.suite_task_count, 21);

            if expected_coverage < 20 {
                write_suite_test_campaign(
                    &campaign_root,
                    &model_root,
                    &benchmark_suite_v9_tasks()[expected_coverage as usize],
                    1,
                    "digest-suite",
                );
            }
        }

        write_suite_test_campaign(
            &campaign_root,
            &model_root,
            &benchmark_suite_v9_tasks()[20],
            1,
            "digest-suite",
        );
        let complete = scan_benchmark_suite_cohorts_from_roots(
            &campaign_root, &model_root, BENCHMARK_SUITE_V9_ID,
        ).expect("scan complete v9 suite");
        let (complete_id, complete_evidence) = complete.iter().next().expect("complete v9 cohort");
        let complete_candidate =
            suite_report_candidate(BENCHMARK_SUITE_V9_ID, complete_id, complete_evidence)
                .expect("complete v9 candidate");
        assert!(complete_candidate.ready);
        assert_eq!(complete_candidate.covered_tasks, 21);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn suite_v10_requires_all_eight_compositional_tasks_without_mutating_v9_coverage() {
        let root = comparison_test_dir("suite-v10-membership");
        let campaign_root = root.join("campaigns");
        let model_root = root.join("models");
        fs::create_dir_all(&campaign_root).expect("create campaign root");
        fs::create_dir_all(&model_root).expect("create model root");

        let v10 = benchmark_suite_by_id(BENCHMARK_SUITE_V10_ID).expect("v10 registry");
        assert_eq!(v10.tasks.len(), 29);

        for task in &v10.tasks[..21] {
            write_suite_test_campaign(&campaign_root, &model_root, task, 1, "digest-suite");
        }

        let v9_cohorts = scan_benchmark_suite_cohorts_from_roots(
            &campaign_root, &model_root, BENCHMARK_SUITE_V9_ID,
        ).expect("scan complete v9 suite");
        let (v9_id, v9_evidence) = v9_cohorts.iter().next().expect("v9 cohort");
        let v9_candidate = suite_report_candidate(BENCHMARK_SUITE_V9_ID, v9_id, v9_evidence)
            .expect("v9 candidate");
        assert!(v9_candidate.ready);
        assert_eq!(v9_candidate.covered_tasks, 21);

        for expected_coverage in 21..=28 {
            let cohorts = scan_benchmark_suite_cohorts_from_roots(
                &campaign_root, &model_root, BENCHMARK_SUITE_V10_ID,
            ).expect("scan incomplete v10 suite");
            let (id, evidence) = cohorts.iter().next().expect("v10 cohort");
            let candidate = suite_report_candidate(BENCHMARK_SUITE_V10_ID, id, evidence)
                .expect("v10 candidate");
            assert!(!candidate.ready);
            assert_eq!(candidate.covered_tasks, expected_coverage);
            assert_eq!(candidate.suite_task_count, 29);

            if expected_coverage < 28 {
                write_suite_test_campaign(
                    &campaign_root,
                    &model_root,
                    &benchmark_suite_v10_tasks()[expected_coverage as usize],
                    1,
                    "digest-suite",
                );
            }
        }

        write_suite_test_campaign(
            &campaign_root,
            &model_root,
            &benchmark_suite_v10_tasks()[28],
            1,
            "digest-suite",
        );
        let complete = scan_benchmark_suite_cohorts_from_roots(
            &campaign_root, &model_root, BENCHMARK_SUITE_V10_ID,
        ).expect("scan complete v10 suite");
        let (complete_id, complete_evidence) = complete.iter().next().expect("complete v10 cohort");
        let complete_candidate =
            suite_report_candidate(BENCHMARK_SUITE_V10_ID, complete_id, complete_evidence)
                .expect("complete v10 candidate");
        assert!(complete_candidate.ready);
        assert_eq!(complete_candidate.covered_tasks, 29);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn suite_v11_requires_all_sixteen_sequential_rule_tasks_without_mutating_v10_coverage() {
        let root = comparison_test_dir("suite-v11-membership");
        let campaign_root = root.join("campaigns");
        let model_root = root.join("models");
        fs::create_dir_all(&campaign_root).expect("create campaign root");
        fs::create_dir_all(&model_root).expect("create model root");

        let v11 = benchmark_suite_by_id(BENCHMARK_SUITE_V11_ID).expect("v11 registry");
        assert_eq!(v11.tasks.len(), 45);

        for task in &v11.tasks[..29] {
            write_suite_test_campaign(&campaign_root, &model_root, task, 1, "digest-suite");
        }

        let v10_cohorts = scan_benchmark_suite_cohorts_from_roots(
            &campaign_root, &model_root, BENCHMARK_SUITE_V10_ID,
        ).expect("scan complete v10 suite");
        let (v10_id, v10_evidence) = v10_cohorts.iter().next().expect("v10 cohort");
        let v10_candidate = suite_report_candidate(BENCHMARK_SUITE_V10_ID, v10_id, v10_evidence)
            .expect("v10 candidate");
        assert!(v10_candidate.ready);
        assert_eq!(v10_candidate.covered_tasks, 29);

        for expected_coverage in 29..=44 {
            let cohorts = scan_benchmark_suite_cohorts_from_roots(
                &campaign_root, &model_root, BENCHMARK_SUITE_V11_ID,
            ).expect("scan incomplete v11 suite");
            let (id, evidence) = cohorts.iter().next().expect("v11 cohort");
            let candidate = suite_report_candidate(BENCHMARK_SUITE_V11_ID, id, evidence)
                .expect("v11 candidate");
            assert!(!candidate.ready);
            assert_eq!(candidate.covered_tasks, expected_coverage);
            assert_eq!(candidate.suite_task_count, 45);

            if expected_coverage < 44 {
                write_suite_test_campaign(
                    &campaign_root,
                    &model_root,
                    &benchmark_suite_v11_tasks()[expected_coverage as usize],
                    1,
                    "digest-suite",
                );
            }
        }

        write_suite_test_campaign(
            &campaign_root,
            &model_root,
            &benchmark_suite_v11_tasks()[44],
            1,
            "digest-suite",
        );
        let complete = scan_benchmark_suite_cohorts_from_roots(
            &campaign_root, &model_root, BENCHMARK_SUITE_V11_ID,
        ).expect("scan complete v11 suite");
        let (complete_id, complete_evidence) = complete.iter().next().expect("complete v11 cohort");
        let complete_candidate =
            suite_report_candidate(BENCHMARK_SUITE_V11_ID, complete_id, complete_evidence)
                .expect("complete v11 candidate");
        assert!(complete_candidate.ready);
        assert_eq!(complete_candidate.covered_tasks, 45);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn suite_v12_requires_all_sixteen_selective_context_tasks_without_mutating_v11_coverage() {
        let root = comparison_test_dir("suite-v12-membership");
        let campaign_root = root.join("campaigns");
        let model_root = root.join("models");
        fs::create_dir_all(&campaign_root).expect("create campaign root");
        fs::create_dir_all(&model_root).expect("create model root");

        let v12 = benchmark_suite_by_id(BENCHMARK_SUITE_V12_ID).expect("v12 registry");
        assert_eq!(v12.tasks.len(), 61);

        for task in &v12.tasks[..45] {
            write_suite_test_campaign(&campaign_root, &model_root, task, 1, "digest-suite");
        }

        let v11_cohorts = scan_benchmark_suite_cohorts_from_roots(
            &campaign_root, &model_root, BENCHMARK_SUITE_V11_ID,
        ).expect("scan complete v11 suite");
        let (v11_id, v11_evidence) = v11_cohorts.iter().next().expect("v11 cohort");
        let v11_candidate = suite_report_candidate(BENCHMARK_SUITE_V11_ID, v11_id, v11_evidence)
            .expect("v11 candidate");
        assert!(v11_candidate.ready);
        assert_eq!(v11_candidate.covered_tasks, 45);

        for expected_coverage in 45..=60 {
            let cohorts = scan_benchmark_suite_cohorts_from_roots(
                &campaign_root, &model_root, BENCHMARK_SUITE_V12_ID,
            ).expect("scan incomplete v12 suite");
            let (id, evidence) = cohorts.iter().next().expect("v12 cohort");
            let candidate = suite_report_candidate(BENCHMARK_SUITE_V12_ID, id, evidence)
                .expect("v12 candidate");
            assert!(!candidate.ready);
            assert_eq!(candidate.covered_tasks, expected_coverage);
            assert_eq!(candidate.suite_task_count, 61);

            if expected_coverage < 60 {
                write_suite_test_campaign(
                    &campaign_root,
                    &model_root,
                    &benchmark_suite_v12_tasks()[expected_coverage as usize],
                    1,
                    "digest-suite",
                );
            }
        }

        write_suite_test_campaign(
            &campaign_root,
            &model_root,
            &benchmark_suite_v12_tasks()[60],
            1,
            "digest-suite",
        );
        let complete = scan_benchmark_suite_cohorts_from_roots(
            &campaign_root, &model_root, BENCHMARK_SUITE_V12_ID,
        ).expect("scan complete v12 suite");
        let (complete_id, complete_evidence) = complete.iter().next().expect("complete v12 cohort");
        let complete_candidate =
            suite_report_candidate(BENCHMARK_SUITE_V12_ID, complete_id, complete_evidence)
                .expect("complete v12 candidate");
        assert!(complete_candidate.ready);
        assert_eq!(complete_candidate.covered_tasks, 61);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn suite_v13_requires_all_sixteen_indirect_context_tasks_without_mutating_v12_coverage() {
        let root = comparison_test_dir("suite-v13-membership");
        let campaign_root = root.join("campaigns");
        let model_root = root.join("models");
        fs::create_dir_all(&campaign_root).expect("create campaign root");
        fs::create_dir_all(&model_root).expect("create model root");

        let v13 = benchmark_suite_by_id(BENCHMARK_SUITE_V13_ID).expect("v13 registry");
        assert_eq!(v13.tasks.len(), 77);

        for task in &v13.tasks[..61] {
            write_suite_test_campaign(&campaign_root, &model_root, task, 1, "digest-suite");
        }

        let v12_cohorts = scan_benchmark_suite_cohorts_from_roots(
            &campaign_root, &model_root, BENCHMARK_SUITE_V12_ID,
        ).expect("scan complete v12 suite");
        let (v12_id, v12_evidence) = v12_cohorts.iter().next().expect("v12 cohort");
        let v12_candidate = suite_report_candidate(BENCHMARK_SUITE_V12_ID, v12_id, v12_evidence)
            .expect("v12 candidate");
        assert!(v12_candidate.ready);
        assert_eq!(v12_candidate.covered_tasks, 61);

        for expected_coverage in 61..=76 {
            let cohorts = scan_benchmark_suite_cohorts_from_roots(
                &campaign_root, &model_root, BENCHMARK_SUITE_V13_ID,
            ).expect("scan incomplete v13 suite");
            let (id, evidence) = cohorts.iter().next().expect("v13 cohort");
            let candidate = suite_report_candidate(BENCHMARK_SUITE_V13_ID, id, evidence)
                .expect("v13 candidate");
            assert!(!candidate.ready);
            assert_eq!(candidate.covered_tasks, expected_coverage);
            assert_eq!(candidate.suite_task_count, 77);

            if expected_coverage < 76 {
                write_suite_test_campaign(
                    &campaign_root,
                    &model_root,
                    &benchmark_suite_v13_tasks()[expected_coverage as usize],
                    1,
                    "digest-suite",
                );
            }
        }

        write_suite_test_campaign(
            &campaign_root,
            &model_root,
            &benchmark_suite_v13_tasks()[76],
            1,
            "digest-suite",
        );
        let complete = scan_benchmark_suite_cohorts_from_roots(
            &campaign_root, &model_root, BENCHMARK_SUITE_V13_ID,
        ).expect("scan complete v13 suite");
        let (complete_id, complete_evidence) = complete.iter().next().expect("complete v13 cohort");
        let complete_candidate =
            suite_report_candidate(BENCHMARK_SUITE_V13_ID, complete_id, complete_evidence)
                .expect("complete v13 candidate");
        assert!(complete_candidate.ready);
        assert_eq!(complete_candidate.covered_tasks, 77);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn suite_cohort_stays_incomplete_when_one_task_is_missing() {
        let root = comparison_test_dir("suite-incomplete");
        let campaign_root = root.join("campaigns");
        let model_root = root.join("models");
        fs::create_dir_all(&campaign_root).expect("create campaign root");
        fs::create_dir_all(&model_root).expect("create model root");

        let first = &benchmark_suite_v1_tasks()[0];
        write_suite_test_campaign(&campaign_root, &model_root, first, 1, "digest-suite");

        let cohorts =
            scan_benchmark_suite_cohorts_from_roots(&campaign_root, &model_root, BENCHMARK_SUITE_V1_ID)
                .expect("scan incomplete suite");
        assert_eq!(cohorts.len(), 1);
        let (cohort_id, evidence) = cohorts.iter().next().expect("cohort");
        let candidate = suite_report_candidate(BENCHMARK_SUITE_V1_ID, cohort_id, evidence).expect("candidate");
        assert!(!candidate.ready);
        assert_eq!(candidate.covered_tasks, 1);
        assert_eq!(
            usize::from(candidate.suite_task_count),
            benchmark_suite_v1_tasks().len()
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn suite_cohort_refuses_digest_mismatch_across_tasks() {
        let root = comparison_test_dir("suite-digest-split");
        let campaign_root = root.join("campaigns");
        let model_root = root.join("models");
        fs::create_dir_all(&campaign_root).expect("create campaign root");
        fs::create_dir_all(&model_root).expect("create model root");

        let tasks = benchmark_suite_v1_tasks();
        write_suite_test_campaign(&campaign_root, &model_root, &tasks[0], 1, "digest-a");
        write_suite_test_campaign(&campaign_root, &model_root, &tasks[1], 1, "digest-b");

        let cohorts =
            scan_benchmark_suite_cohorts_from_roots(&campaign_root, &model_root, BENCHMARK_SUITE_V1_ID)
                .expect("scan split suite");
        assert_eq!(cohorts.len(), 2);
        assert!(cohorts
            .iter()
            .all(|(id, evidence)| !suite_report_candidate(BENCHMARK_SUITE_V1_ID, id, evidence).expect("candidate").ready));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn suite_cohort_scan_rejects_mutated_underlying_trial() {
        let root = comparison_test_dir("suite-tamper");
        let campaign_root = root.join("campaigns");
        let model_root = root.join("models");
        fs::create_dir_all(&campaign_root).expect("create campaign root");
        fs::create_dir_all(&model_root).expect("create model root");

        for task in benchmark_suite_v1_tasks() {
            write_suite_test_campaign(&campaign_root, &model_root, task, 1, "digest-suite");
        }

        let first = &benchmark_suite_v1_tasks()[0];
        fs::write(
            model_root.join(first.rom_sha256).join("run-000001.json"),
            b"mutated-suite-trial",
        )
        .expect("mutate suite trial");

        assert!(
            scan_benchmark_suite_cohorts_from_roots(&campaign_root, &model_root, BENCHMARK_SUITE_V1_ID).is_err()
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn model_gameplay_receipt_serializes_score_and_digest_evidence() {
        let receipt = ModelGameplayBenchmarkReceipt {
            schema: MODEL_GAMEPLAY_BENCHMARK_SCHEMA.into(),
            record_status: "COMPLETE".into(),
            benchmark_id: AGENT_GYM_ID.into(),
            benchmark_run_id: 7,
            provider: "ollama".into(),
            model: "vision-model".into(),
            model_digest: "digest-a".into(),
            model_qualification_sha256: "q".repeat(64),
            gym_source_sha256: AGENT_GYM_SOURCE_SHA256.into(),
            gym_rom_sha256: AGENT_GYM_ROM_SHA256.into(),
            core_sha256: "c".repeat(64),
            core_name: "SameBoy".into(),
            core_version: "1.0.3".into(),
            autodrive_receipt_sha256: "a".repeat(64),
            autodrive_run_id: 8,
            policy: AutodrivePolicy::default(),
            stop_reason: AutodriveStopReason::TaskSuccess,
            started_frame: 120,
            ended_frame: 240,
            start_player: AGENT_GYM_START,
            final_player: Some(AGENT_GYM_TARGET),
            target: AGENT_GYM_TARGET,
            initial_distance: AGENT_GYM_INITIAL_DISTANCE,
            final_distance: Some(0),
            progress: Some(AGENT_GYM_INITIAL_DISTANCE),
            score_1000: Some(1000),
            task_success: true,
            turns_issued: 4,
            turns_completed: 4,
            total_actions: 8,
            final_frame_sha256: "f".repeat(64),
            scoring_error: None,
        };

        let json = serde_json::to_value(&receipt).expect("serialize receipt");
        assert_eq!(json["schema"], MODEL_GAMEPLAY_BENCHMARK_SCHEMA);
        assert_eq!(json["modelDigest"], "digest-a");
        assert_eq!(json["score1000"], 1000);
        assert_eq!(json["taskSuccess"], true);
        assert_eq!(json["stopReason"], "task-success");
    }
}
