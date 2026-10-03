use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use phicade_runtime::{
    ActionKind, AgentTurnAction, AgentTurnRequest, AgentTurnResponse,
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
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OllamaButtonAction {
    delay_frames: u16,
    button: String,
    pressed: bool,
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
                            "enum": request.observation.allowed_buttons
                        },
                        "pressed": { "type": "boolean" }
                    },
                    "required": ["delayFrames", "button", "pressed"],
                    "additionalProperties": false
                }
            }
        },
        "required": ["actions"],
        "additionalProperties": false
    })
}

fn system_prompt(request: &AgentTurnRequest) -> String {
    format!(
        concat!(
            "You are the gameplay policy for PhiCade turn {turn}. ",
            "You see only the supplied game framebuffer. ",
            "Return controller actions only. Never invent buttons. ",
            "Allowed buttons: {buttons}. ",
            "Maximum actions: {max_actions}. Maximum delayFrames: {max_delay}. ",
            "Use short press/release pairs when acting. ",
            "If uncertain, return an empty actions array. ",
            "The runtime, not you, owns authority and timing."
        ),
        turn = request.turn_id,
        buttons = request.observation.allowed_buttons.join(", "),
        max_actions = request.max_actions,
        max_delay = request.max_delay_frames,
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
    fn response_schema_uses_request_budgets() {
        let schema = response_schema(&request());
        assert_eq!(schema["properties"]["actions"]["maxItems"], 4);
        assert_eq!(
            schema["properties"]["actions"]["items"]["properties"]["delayFrames"]["maximum"],
            8
        );
    }
}
