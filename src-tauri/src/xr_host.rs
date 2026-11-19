//! No HTTP server or filesystem API. WebRTC terminates in the local webview;
//! native commands bind its input to this exact approved emulator session.
use crate::EmulatorState;
use phicade_xr_protocol::{InputPacket, MAX_GRANT_MS, PROTOCOL};
use serde::Serialize;
use std::sync::OnceLock;
use std::time::Instant;
use tauri::State;

pub fn now_ms() -> u64 {
    static EPOCH: OnceLock<Instant> = OnceLock::new();
    EPOCH
        .get_or_init(Instant::now)
        .elapsed()
        .as_millis()
        .min(u64::MAX as u128) as u64
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XrHostReceipt {
    protocol: &'static str,
    peer_id: String,
    playable_ports: u8,
    lightgun_enabled: bool,
    expires_in_ms: u64,
}
#[tauri::command]
pub fn xr_open_host(
    state: State<'_, EmulatorState>,
    peer_id: String,
    token: String,
    lightgun: bool,
    operator_approved: bool,
) -> Result<XrHostReceipt, String> {
    if !operator_approved {
        return Err("XR hosting requires local operator approval".into());
    }
    let mut guard = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned")?;
    let session = guard
        .as_mut()
        .ok_or("launch a governed game session before hosting XR")?;
    if session.recording.is_some() || session.autodrive.as_ref().is_some_and(|s| s.active) {
        return Err("stop replay recording and Autodrive before assigning XR control".into());
    }
    if lightgun {
        session
            .core
            .configure_lightgun(true)
            .map_err(|e| format!("{e:?}"))?;
        // Existing Replay v1 checkpoints only cover the joypad mask. Do not
        // inherit replay qualification for an absolute lightgun device.
        session.runtime_features.exact_replay = false;
    } else {
        // Restore the ordinary controller when re-pairing after a gun session.
        let _ = session.core.configure_lightgun(false);
        session.core.neutralize_inputs();
    }
    session
        .xr_gateway
        .approve(peer_id.clone(), token, lightgun, now_ms())?;
    Ok(XrHostReceipt {
        protocol: PROTOCOL,
        peer_id,
        playable_ports: 1,
        lightgun_enabled: lightgun,
        expires_in_ms: MAX_GRANT_MS,
    })
}
#[tauri::command]
pub fn xr_submit_input(
    state: State<'_, EmulatorState>,
    peer_id: String,
    token: String,
    packet: InputPacket,
) -> Result<(), String> {
    let mut guard = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned")?;
    let session = guard.as_mut().ok_or("no emulator session is running")?;
    session
        .xr_gateway
        .submit(&peer_id, &token, packet, now_ms())
}
#[tauri::command]
pub fn xr_revoke_host(state: State<'_, EmulatorState>) -> Result<(), String> {
    let mut guard = state
        .session
        .lock()
        .map_err(|_| "emulator session lock poisoned")?;
    if let Some(session) = guard.as_mut() {
        session.xr_gateway.revoke();
        session.core.neutralize_inputs();
    }
    Ok(())
}
