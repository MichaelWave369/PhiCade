//! Explicit, Windows-only Steam client handoff. No Steamworks API replacement,
//! DRM circumvention, filesystem access, account scanning or direct game launch.
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamHandoffReceipt {
    pub app_id: u32,
    pub protocol_uri: String,
    pub handoff_requested: bool,
}

fn checked_steam_uri(app_id: u32, operator_approved: bool) -> Result<String, String> {
    if !operator_approved {
        return Err("Steam handoff requires an explicit operator-approved action".into());
    }
    if app_id == 0 {
        return Err("Steam App ID must be nonzero".into());
    }
    Ok(format!("steam://run/{app_id}"))
}

#[tauri::command]
pub fn launch_steam_app(app_id: u32, operator_approved: bool) -> Result<SteamHandoffReceipt, String> {
    let protocol_uri = checked_steam_uri(app_id, operator_approved)?;

    #[cfg(target_os = "windows")]
    {
        // No shell, CMD, PowerShell, custom user commands or supplied paths.
        // Windows Explorer delegates an exact, numeric steam://run/<appid> URI
        // to the user's registered handler; Steam decides whether it can launch.
        let system_root = std::env::var_os("SystemRoot")
            .ok_or_else(|| "Windows SystemRoot is unavailable".to_owned())?;
        let explorer = std::path::PathBuf::from(system_root).join("explorer.exe");
        if !explorer.is_file() {
            return Err("Windows Explorer was not found".into());
        }
        std::process::Command::new(explorer)
            .arg(&protocol_uri)
            .spawn()
            .map_err(|e| format!("could not request Steam client handoff: {e}"))?;
        Ok(SteamHandoffReceipt {
            app_id,
            protocol_uri,
            handoff_requested: true,
        })
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = protocol_uri;
        Err("Steam client handoff is available only in PhiCade for Windows".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_operator_approval_is_required() {
        assert!(checked_steam_uri(480, false).is_err());
        assert!(checked_steam_uri(0, true).is_err());
    }

    #[test]
    fn steam_launch_uri_is_numeric_only() {
        assert_eq!(checked_steam_uri(480, true).unwrap(), "steam://run/480");
        assert_eq!(checked_steam_uri(u32::MAX, true).unwrap(), "steam://run/4294967295");
    }

    #[test]
    fn no_non_windows_client_launch_is_allowed() {
        #[cfg(not(target_os = "windows"))]
        assert!(launch_steam_app(480, true).is_err());
    }
}
