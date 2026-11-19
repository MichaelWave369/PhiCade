use phicade_xr_bridge::{Installation, LaunchOptions, LaunchPlan};
use serde::Serialize;
use std::{
    path::Path,
    process::{Child, Command},
    sync::Mutex,
};
use tauri::{AppHandle, Manager, State};

#[derive(Default)]
pub struct DolphinXrState {
    process: Mutex<Option<Child>>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchReceipt {
    pid: u32,
    plan: LaunchPlan,
    vr_session_verified: bool,
}
#[tauri::command]
pub fn inspect_dolphinxr(executable: String) -> Result<Installation, String> {
    phicade_xr_bridge::inspect(Path::new(&executable))
}
#[tauri::command]
pub fn launch_dolphinxr(
    app: AppHandle,
    state: State<'_, DolphinXrState>,
    executable: String,
    expected_hash: String,
    game: String,
    options: LaunchOptions,
    operator_approved: bool,
) -> Result<LaunchReceipt, String> {
    if !operator_approved {
        return Err("DolphinXR launch requires a local session approval".into());
    }
    let mut process = state
        .process
        .lock()
        .map_err(|_| "DolphinXR process lock poisoned")?;
    if let Some(child) = process.as_mut() {
        if child.try_wait().map_err(|e| e.to_string())?.is_none() {
            return Err(
                "a DolphinXR session is already active; close it in DolphinXR first".into(),
            );
        }
    }
    let root = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("xr/dolphinxr");
    let plan = phicade_xr_bridge::prepare(
        Path::new(&executable),
        &expected_hash,
        Path::new(&game),
        &root,
        &options,
    )?;
    // Exact argument vector, no shell, no arbitrary flags from peers or agents.
    let child = Command::new(&plan.executable)
        .args(&plan.arguments)
        .spawn()
        .map_err(|e| format!("DolphinXR did not launch: {e}"))?;
    let pid = child.id();
    *process = Some(child);
    Ok(LaunchReceipt {
        pid,
        plan,
        vr_session_verified: false,
    })
}
#[tauri::command]
pub fn dolphinxr_status(state: State<'_, DolphinXrState>) -> Result<Option<u32>, String> {
    let mut process = state
        .process
        .lock()
        .map_err(|_| "DolphinXR process lock poisoned")?;
    if let Some(child) = process.as_mut() {
        if child.try_wait().map_err(|e| e.to_string())?.is_none() {
            return Ok(Some(child.id()));
        }
    }
    *process = None;
    Ok(None)
}
