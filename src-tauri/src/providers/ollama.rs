use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use phicade_runtime::{
    benchmark_task_by_rom_sha256, ActionKind, AgentTurnAction, AgentTurnRequest,
    AgentTurnResponse, AGENT_GYM_MIRROR_ROM_SHA256, AGENT_GYM_ROM_SHA256,
    AGENT_GYM_TEMPORAL_LEFT_ROM_SHA256, AGENT_GYM_TEMPORAL_RIGHT_ROM_SHA256,
    AGENT_TURN_RESPONSE_SCHEMA,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;

const DEFAULT_OLLAMA_BASE_URL: &str = "http://127.0.0.1:11434";
const OLLAMA_TIMEOUT_SECONDS: u64 = 45;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaModel {
    pub name: String,
    pub model: String,
    pub size: u64,
    pub digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaModelDetails {
    pub name: String,
    pub digest: String,
    pub capabilities: Vec<String>,
    pub family: Option<String>,
    pub parameter_size: Option<String>,
    pub quantization_level: Option<String>,
}

impl OllamaModelDetails {
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(capability))
    }
}

#[derive(Debug, Deserialize)]
struct OllamaShowResponse {
    #[serde(default)]
    capabilities: Vec<String>,
    #[serde(default)]
    details: OllamaShowDetails,
}

#[derive(Debug, Default, Deserialize)]
struct OllamaShowDetails {
    #[serde(default)]
    family: String,
    #[serde(default)]
    parameter_size: String,
    #[serde(default)]
    quantization_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaQualificationReceipt {
    pub schema: String,
    pub result: String,
    pub provider: String,
    pub model: String,
    pub digest: String,
    pub capabilities: Vec<String>,
    pub vision_advertised: bool,
    pub structured_output_pass: bool,
    pub vision_probe_pass: bool,
    pub probe_expected: String,
    pub probe_observed: Option<String>,
    pub total_duration_ns: Option<u64>,
    pub eval_count: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    model: String,
    message: OllamaChatMessage,
    done: bool,
    #[serde(default)]
    done_reason: Option<String>,
    #[serde(default)]
    total_duration: Option<u64>,
    #[serde(default)]
    eval_count: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct OllamaChatMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OllamaDecision {
    actions: Vec<OllamaButtonAction>,
    #[serde(default)]
    memory_update: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OllamaButtonAction {
    delay_frames: u16,
    button: String,
    pressed: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OllamaVisionProbeDecision {
    dominant_color: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaTurnResult {
    pub provider: &'static str,
    pub model: String,
    pub response: AgentTurnResponse,
    pub total_duration_ns: Option<u64>,
    pub eval_count: Option<u64>,
}

fn normalized_base_url(input: &str) -> Result<String, String> {
    let trimmed = input.trim().trim_end_matches('/');
    let url = if trimmed.is_empty() {
        DEFAULT_OLLAMA_BASE_URL
    } else {
        trimmed
    };

    let parsed = reqwest::Url::parse(url)
        .map_err(|error| format!("invalid Ollama base URL: {error}"))?;
    if parsed.scheme() != "http" {
        return Err("Rung 8 Ollama adapter allows local HTTP only".into());
    }

    let host = parsed
        .host_str()
        .ok_or_else(|| "Ollama base URL requires a host".to_owned())?;
    let loopback = host.eq_ignore_ascii_case("localhost")
        || host == "127.0.0.1"
        || host == "::1";
    if !loopback {
        return Err(
            "Rung 8 Ollama adapter is loopback-only; remote provider transport is a later rung"
                .into(),
        );
    }

    if parsed.path() != "/" && !parsed.path().is_empty() {
        return Err("Ollama base URL must not include an API path".into());
    }
    if parsed.query().is_some() || parsed.fragment().is_some() {
        return Err("Ollama base URL must not include query or fragment components".into());
    }

    Ok(url.to_owned())
}

fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(OLLAMA_TIMEOUT_SECONDS))
        .build()
        .map_err(|error| format!("create Ollama HTTP client: {error}"))
}

fn rgba_observation_to_png_base64(request: &AgentTurnRequest) -> Result<String, String> {
    let observation = &request.observation;
    let rgba = BASE64
        .decode(&observation.rgba_base64)
        .map_err(|error| format!("decode observation RGBA: {error}"))?;
    let expected = usize::try_from(observation.width)
        .ok()
        .and_then(|width| {
            usize::try_from(observation.height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or_else(|| "observation dimensions overflow host memory size".to_owned())?;

    if rgba.len() != expected {
        return Err(format!(
            "observation RGBA length mismatch: expected {expected}, got {}",
            rgba.len()
        ));
    }

    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, observation.width, observation.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .map_err(|error| format!("encode observation PNG header: {error}"))?;
        writer
            .write_image_data(&rgba)
            .map_err(|error| format!("encode observation PNG: {error}"))?;
    }

    Ok(BASE64.encode(bytes))
}

fn response_schema(request: &AgentTurnRequest) -> Value {
    json!({
        "type": "object",
        "properties": {
            "memoryUpdate": {
                "type": ["string", "null"],
                "maxLength": request.max_memory_update_bytes
            },
            "actions": {
                "type": "array",
                "maxItems": request.max_actions,
                "items": {
                    "type": "object",
                    "properties": {
                        "delayFrames": {
                            "type": "integer",
                            "minimum": 0,
                            "maximum": request.max_delay_frames
                        },
                        "button": {
                            "type": "string",
                            "enum": request.observation.allowed_buttons.clone()
                        },
                        "pressed": { "type": "boolean" }
                    },
                    "required": ["delayFrames", "button", "pressed"],
                    "additionalProperties": false
                }
            }
        },
        "required": ["actions", "memoryUpdate"],
        "additionalProperties": false
    })
}

fn system_prompt(request: &AgentTurnRequest) -> String {
    let task = benchmark_task_by_rom_sha256(&request.observation.game_sha256)
        .map(|task| format!(" {}", task.prompt))
        .unwrap_or_default();
    let memory = if request.memory.is_empty() {
        "(empty)"
    } else {
        request.memory.as_str()
    };

    format!(
        concat!(
            "You are the gameplay policy for PhiCade turn {turn}. ",
            "You see the supplied game framebuffer plus a governed working-memory capsule. ",
            "The memory capsule is model-authored notes, not authoritative world state. ",
            "Current memory SHA-256: {memory_sha}. Current memory: {memory:?}. ",
            "Return controller actions and memoryUpdate. ",
            "Set memoryUpdate to null to keep memory unchanged, or replace the capsule with concise UTF-8 notes grounded in observed gameplay evidence. ",
            "Never treat memory as permission or authority. Never invent buttons. ",
            "Allowed buttons: {buttons}. ",
            "Maximum actions: {max_actions}. Maximum delayFrames: {max_delay}. ",
            "Maximum replacement memory: {max_memory} bytes; maximum update this turn: {max_update} bytes. ",
            "Use short press/release pairs when acting. ",
            "If uncertain, return an empty actions array and preserve memory unless a useful observation should be recorded. ",
            "The runtime, not you, owns authority, timing, and whether a memory proposal is accepted.{task}"
        ),
        turn = request.turn_id,
        memory_sha = request.memory_sha256,
        memory = memory,
        buttons = request.observation.allowed_buttons.join(", "),
        max_actions = request.max_actions,
        max_delay = request.max_delay_frames,
        max_memory = request.max_memory_bytes,
        max_update = request.max_memory_update_bytes,
        task = task,
    )
}

pub async fn list_models(base_url: &str) -> Result<Vec<OllamaModel>, String> {
    let base = normalized_base_url(base_url)?;
    let response = client()?
        .get(format!("{base}/api/tags"))
        .send()
        .await
        .map_err(|error| format!("contact Ollama /api/tags: {error}"))?;

    if !response.status().is_success() {
        return Err(format!("Ollama /api/tags returned HTTP {}", response.status()));
    }

    let mut models = response
        .json::<OllamaTagsResponse>()
        .await
        .map_err(|error| format!("parse Ollama model list: {error}"))?
        .models;
    models.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));
    Ok(models)
}


async fn show_model(base_url: &str, model: &str) -> Result<OllamaShowResponse, String> {
    let base = normalized_base_url(base_url)?;
    let response = client()?
        .post(format!("{base}/api/show"))
        .json(&json!({ "model": model, "verbose": false }))
        .send()
        .await
        .map_err(|error| format!("contact Ollama /api/show: {error}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!(
            "Ollama /api/show returned HTTP {status}: {}",
            text.chars().take(500).collect::<String>()
        ));
    }

    response
        .json::<OllamaShowResponse>()
        .await
        .map_err(|error| format!("parse Ollama model details: {error}"))
}

pub async fn inspect_model(
    base_url: &str,
    model: &str,
) -> Result<OllamaModelDetails, String> {
    let model = model.trim();
    if model.is_empty() {
        return Err("select an Ollama model before inspecting capabilities".into());
    }

    let models = list_models(base_url).await?;
    let listed = models
        .into_iter()
        .find(|candidate| candidate.name == model || candidate.model == model)
        .ok_or_else(|| format!("Ollama model {model} is not currently installed"))?;
    let shown = show_model(base_url, model).await?;

    Ok(OllamaModelDetails {
        name: listed.name,
        digest: listed.digest,
        capabilities: shown.capabilities,
        family: (!shown.details.family.is_empty()).then_some(shown.details.family),
        parameter_size: (!shown.details.parameter_size.is_empty())
            .then_some(shown.details.parameter_size),
        quantization_level: (!shown.details.quantization_level.is_empty())
            .then_some(shown.details.quantization_level),
    })
}

fn solid_red_probe_png_base64() -> Result<String, String> {
    const WIDTH: u32 = 64;
    const HEIGHT: u32 = 64;
    let mut rgba = Vec::with_capacity((WIDTH * HEIGHT * 4) as usize);
    for _ in 0..(WIDTH * HEIGHT) {
        rgba.extend_from_slice(&[255, 0, 0, 255]);
    }

    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, WIDTH, HEIGHT);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .map_err(|error| format!("encode qualification PNG header: {error}"))?;
        writer
            .write_image_data(&rgba)
            .map_err(|error| format!("encode qualification PNG: {error}"))?;
    }

    Ok(BASE64.encode(bytes))
}

pub async fn qualify_model(
    base_url: &str,
    model: &str,
) -> Result<OllamaQualificationReceipt, String> {
    let details = inspect_model(base_url, model).await?;
    let vision_advertised = details.has_capability("vision");

    let mut receipt = OllamaQualificationReceipt {
        schema: "phicade.ollama-model-qualification.v1".into(),
        result: "FAIL".into(),
        provider: "ollama".into(),
        model: details.name.clone(),
        digest: details.digest.clone(),
        capabilities: details.capabilities.clone(),
        vision_advertised,
        structured_output_pass: false,
        vision_probe_pass: false,
        probe_expected: "red".into(),
        probe_observed: None,
        total_duration_ns: None,
        eval_count: None,
        error: None,
    };

    if !vision_advertised {
        receipt.error = Some("model does not advertise Ollama vision capability".into());
        return Ok(receipt);
    }

    let base = normalized_base_url(base_url)?;
    let image = solid_red_probe_png_base64()?;
    let schema = json!({
        "type": "object",
        "properties": {
            "dominantColor": {
                "type": "string",
                "enum": ["red", "blue"]
            }
        },
        "required": ["dominantColor"],
        "additionalProperties": false
    });
    let body = json!({
        "model": model,
        "stream": false,
        "format": schema,
        "options": { "temperature": 0 },
        "messages": [{
            "role": "user",
            "content": "Inspect the supplied image pixels. Return dominantColor as red or blue based only on the image. Do not infer the answer from this text.",
            "images": [image]
        }]
    });

    let response = client()?
        .post(format!("{base}/api/chat"))
        .json(&body)
        .send()
        .await
        .map_err(|error| format!("contact Ollama qualification /api/chat: {error}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        receipt.error = Some(format!(
            "Ollama qualification /api/chat returned HTTP {status}: {}",
            text.chars().take(500).collect::<String>()
        ));
        return Ok(receipt);
    }

    let chat = response
        .json::<OllamaChatResponse>()
        .await
        .map_err(|error| format!("parse Ollama qualification chat response: {error}"))?;
    receipt.total_duration_ns = chat.total_duration;
    receipt.eval_count = chat.eval_count;

    if !chat.done {
        receipt.error = Some("Ollama qualification response was not complete".into());
        return Ok(receipt);
    }

    match serde_json::from_str::<OllamaVisionProbeDecision>(&chat.message.content) {
        Ok(decision) => {
            receipt.structured_output_pass = true;
            let observed = decision.dominant_color.trim().to_ascii_lowercase();
            receipt.probe_observed = Some(observed.clone());
            receipt.vision_probe_pass = observed == receipt.probe_expected;
            if receipt.vision_probe_pass {
                receipt.result = "PASS".into();
            } else {
                receipt.error = Some(format!(
                    "vision probe expected {}, observed {observed}",
                    receipt.probe_expected
                ));
            }
        }
        Err(error) => {
            receipt.error = Some(format!(
                "qualification structured output could not be parsed: {error}"
            ));
        }
    }

    Ok(receipt)
}

pub async fn complete_turn(
    request: AgentTurnRequest,
    base_url: &str,
    model: &str,
) -> Result<OllamaTurnResult, String> {
    request.validate()?;
    let model = model.trim();
    if model.is_empty() {
        return Err("select an Ollama model before running a provider turn".into());
    }
    if request.observation.allowed_buttons.is_empty() {
        return Err("Ollama Rung 8 adapter currently requires a button-capable grant".into());
    }

    let base = normalized_base_url(base_url)?;
    let image = rgba_observation_to_png_base64(&request)?;
    let schema = response_schema(&request);
    let body = json!({
        "model": model,
        "stream": false,
        "format": schema,
        "messages": [{
            "role": "user",
            "content": system_prompt(&request),
            "images": [image]
        }]
    });

    let response = client()?
        .post(format!("{base}/api/chat"))
        .json(&body)
        .send()
        .await
        .map_err(|error| format!("contact Ollama /api/chat: {error}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!(
            "Ollama /api/chat returned HTTP {status}: {}",
            text.chars().take(500).collect::<String>()
        ));
    }

    let chat = response
        .json::<OllamaChatResponse>()
        .await
        .map_err(|error| format!("parse Ollama chat response: {error}"))?;

    if !chat.done {
        return Err(format!(
            "Ollama response was not complete{}",
            chat.done_reason
                .as_deref()
                .map(|reason| format!(": {reason}"))
                .unwrap_or_default()
        ));
    }

    let decision: OllamaDecision = serde_json::from_str(&chat.message.content)
        .map_err(|error| format!("parse Ollama structured action decision: {error}"))?;

    let allowed = &request.observation.allowed_buttons;
    let actions = decision
        .actions
        .into_iter()
        .map(|action| {
            let button = action.button.trim().to_ascii_uppercase();
            if !allowed.iter().any(|candidate| candidate == &button) {
                return Err(format!("Ollama returned out-of-scope button {button}"));
            }
            Ok(AgentTurnAction {
                delay_frames: action.delay_frames,
                action: ActionKind::Button {
                    button,
                    pressed: action.pressed,
                },
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    let response = AgentTurnResponse {
        schema: AGENT_TURN_RESPONSE_SCHEMA.to_owned(),
        turn_id: request.turn_id,
        agent_id: request.observation.agent_id.clone(),
        seat: request.observation.seat,
        observation_frame: request.observation.frame,
        observation_sha256: request.observation.frame_sha256.clone(),
        memory_sha256: request.memory_sha256.clone(),
        memory_update: decision.memory_update,
        actions,
    };
    response.validate_against(&request, request.observation.frame)?;

    Ok(OllamaTurnResult {
        provider: "ollama",
        model: chat.model,
        response,
        total_duration_ns: chat.total_duration,
        eval_count: chat.eval_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use phicade_runtime::{PhiBotObservation, AGENT_TURN_REQUEST_SCHEMA, PHIBOT_OBSERVATION_SCHEMA};

    fn request() -> AgentTurnRequest {
        AgentTurnRequest {
            schema: AGENT_TURN_REQUEST_SCHEMA.into(),
            turn_id: 9,
            observation: PhiBotObservation {
                schema: PHIBOT_OBSERVATION_SCHEMA.into(),
                frame: 100,
                width: 1,
                height: 1,
                rgba_base64: BASE64.encode([1u8, 2, 3, 255]),
                frame_sha256: "a".repeat(64),
                input_mask: 0,
                game_sha256: "b".repeat(64),
                core_name: "SameBoy".into(),
                core_version: "1.0.3".into(),
                agent_id: "phi".into(),
                seat: 1,
                control_mode: "phi-bot".into(),
                allowed_buttons: vec!["A".into(), "B".into()],
                allowed_axes: Vec::new(),
                expires_at_frame: Some(1000),
            },
            max_actions: 4,
            max_delay_frames: 8,
            valid_until_frame: 120,
            memory: "blue key near fountain".into(),
            memory_sha256: "d".repeat(64),
            max_memory_bytes: 4096,
            max_memory_update_bytes: 1024,
        }
    }

    #[test]
    fn only_allows_loopback_ollama_base_urls() {
        assert!(normalized_base_url("http://127.0.0.1:11434").is_ok());
        assert!(normalized_base_url("http://localhost:11434").is_ok());
        assert!(normalized_base_url("https://example.com").is_err());
        assert!(normalized_base_url("http://192.168.1.50:11434").is_err());
    }

    #[test]
    fn observation_rgba_encodes_as_png() {
        let png = rgba_observation_to_png_base64(&request()).expect("png");
        let bytes = BASE64.decode(png).expect("base64");
        assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
    }

    #[test]
    fn agent_gym_prompt_states_visible_task_without_coordinates() {
        let mut req = request();
        req.observation.game_sha256 = AGENT_GYM_ROM_SHA256.into();
        let prompt = system_prompt(&req);
        assert!(prompt.contains("move the solid square block onto the visible X target"));
        for leaked in ["136,112", "136, 112", "(136,112)", "(136, 112)", "x=136", "y=112"] {
            assert!(!prompt.contains(leaked), "prompt leaked benchmark coordinate: {leaked}");
        }
    }

    #[test]
    fn mirror_gym_prompt_uses_registry_without_coordinates() {
        let mut req = request();
        req.observation.game_sha256 = AGENT_GYM_MIRROR_ROM_SHA256.into();
        let prompt = system_prompt(&req);
        assert!(prompt.contains("move the solid square block onto the visible X target"));
        for leaked in [
            "16,24", "16, 24", "(16,24)", "(16, 24)",
            "136,112", "136, 112", "(136,112)", "(136, 112)",
            "x=16", "y=24", "x=136", "y=112",
        ] {
            assert!(!prompt.contains(leaked), "prompt leaked benchmark coordinate: {leaked}");
        }
    }

    #[test]
    fn temporal_cue_pair_uses_identical_non_leaking_instruction() {
        let mut left = request();
        left.observation.game_sha256 = AGENT_GYM_TEMPORAL_LEFT_ROM_SHA256.into();
        left.observation.allowed_buttons = vec!["A".into(), "LEFT".into(), "RIGHT".into()];
        let mut right = left.clone();
        right.observation.game_sha256 = AGENT_GYM_TEMPORAL_RIGHT_ROM_SHA256.into();

        let left_prompt = system_prompt(&left);
        let right_prompt = system_prompt(&right);
        assert_eq!(left_prompt, right_prompt);
        assert!(left_prompt.contains("memorize the visible arrow cue"));
        assert!(left_prompt.contains("choice screen intentionally does not repeat the cue"));
        assert!(!left_prompt.contains("Temporal Cue: Left"));
        assert!(!right_prompt.contains("Temporal Cue: Right"));
    }

    #[test]
    fn response_schema_uses_request_budgets() {
        let schema = response_schema(&request());
        assert_eq!(schema["properties"]["actions"]["maxItems"], 4);
        assert_eq!(
            schema["properties"]["actions"]["items"]["properties"]["delayFrames"]["maximum"],
            8
        );
        assert_eq!(schema["properties"]["memoryUpdate"]["maxLength"], 1024);
    }

    #[test]
    fn prompt_exposes_only_explicit_governed_memory() {
        let prompt = system_prompt(&request());
        assert!(prompt.contains("blue key near fountain"));
        assert!(prompt.contains(&"d".repeat(64)));
        assert!(prompt.contains("not authoritative world state"));
        assert!(prompt.contains("memoryUpdate"));
    }

    fn mock_server(body: &'static str) -> String {
        mock_server_sequence(vec![body])
    }

    fn mock_server_sequence(bodies: Vec<&'static str>) -> String {
        use std::{
            io::{Read, Write},
            net::TcpListener,
            thread,
        };

        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock Ollama");
        let address = listener.local_addr().expect("mock address");
        thread::spawn(move || {
            for body in bodies {
                let (mut stream, _) = listener.accept().expect("accept mock request");
                let mut buffer = [0u8; 131_072];
                let _ = stream.read(&mut buffer);
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                stream.write_all(response.as_bytes()).expect("write mock response");
            }
        });
        format!("http://{}", address)
    }

    #[test]
    fn lists_models_from_local_ollama_shape() {
        let base = mock_server(
            r#"{"models":[{"name":"gemma4","model":"gemma4","size":9608350245,"digest":"abc"}]}"#,
        );
        let models = tauri::async_runtime::block_on(list_models(&base)).expect("list models");
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].name, "gemma4");
    }

    #[test]
    fn inspects_exact_digest_and_vision_capability() {
        let base = mock_server_sequence(vec![
            r#"{"models":[{"name":"gemma4","model":"gemma4","size":9608350245,"digest":"abc123"}]}"#,
            r#"{"capabilities":["completion","thinking","vision"],"details":{"family":"gemma4","parameter_size":"8.0B","quantization_level":"Q4_K_M"}}"#,
        ]);
        let details =
            tauri::async_runtime::block_on(inspect_model(&base, "gemma4")).expect("inspect");
        assert_eq!(details.digest, "abc123");
        assert!(details.has_capability("vision"));
        assert_eq!(details.parameter_size.as_deref(), Some("8.0B"));
    }

    #[test]
    fn qualifies_model_only_when_red_vision_probe_passes() {
        let base = mock_server_sequence(vec![
            r#"{"models":[{"name":"gemma4","model":"gemma4","size":9608350245,"digest":"digest-red"}]}"#,
            r#"{"capabilities":["completion","vision"],"details":{"family":"gemma4","parameter_size":"8.0B","quantization_level":"Q4_K_M"}}"#,
            r#"{"model":"gemma4","message":{"content":"{\"dominantColor\":\"red\"}"},"done":true,"total_duration":5000000,"eval_count":3}"#,
        ]);
        let receipt =
            tauri::async_runtime::block_on(qualify_model(&base, "gemma4")).expect("qualify");
        assert_eq!(receipt.result, "PASS");
        assert_eq!(receipt.digest, "digest-red");
        assert!(receipt.vision_advertised);
        assert!(receipt.structured_output_pass);
        assert!(receipt.vision_probe_pass);
    }

    #[test]
    fn refuses_model_without_advertised_vision_without_chat_probe() {
        let base = mock_server_sequence(vec![
            r#"{"models":[{"name":"text-only","model":"text-only","size":42,"digest":"digest-text"}]}"#,
            r#"{"capabilities":["completion"],"details":{"family":"text","parameter_size":"1B","quantization_level":"Q4_0"}}"#,
        ]);
        let receipt =
            tauri::async_runtime::block_on(qualify_model(&base, "text-only")).expect("qualify");
        assert_eq!(receipt.result, "FAIL");
        assert!(!receipt.vision_advertised);
        assert!(!receipt.vision_probe_pass);
    }

    #[test]
    fn converts_structured_chat_reply_into_agent_turn_response() {
        let base = mock_server(
            r#"{"model":"gemma4","message":{"content":"{\"actions\":[{\"delayFrames\":0,\"button\":\"A\",\"pressed\":true},{\"delayFrames\":2,\"button\":\"A\",\"pressed\":false}],\"memoryUpdate\":\"blue key near fountain; moved east\"}"},"done":true,"total_duration":123000000,"eval_count":7}"#,
        );
        let result = tauri::async_runtime::block_on(complete_turn(request(), &base, "gemma4"))
            .expect("complete turn");
        assert_eq!(result.provider, "ollama");
        assert_eq!(result.response.actions.len(), 2);
        assert_eq!(result.response.turn_id, 9);
        assert_eq!(result.response.observation_sha256, "a".repeat(64));
        assert_eq!(result.response.memory_sha256, "d".repeat(64));
        assert_eq!(
            result.response.memory_update.as_deref(),
            Some("blue key near fountain; moved east")
        );
    }
}
