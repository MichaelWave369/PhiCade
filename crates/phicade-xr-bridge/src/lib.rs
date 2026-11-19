//! Process boundary only. DolphinXR retains its native OpenXR controllers, license
//! and identity. No Dolphin implementation, binaries or game data are distributed.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};
pub const INSPECTED_SOURCE: &str = "aeb496accb6e2f333e4995b893c01ed04a8d876d";
pub const WII_PROFILE: &str = "Profiles/Wiimote/OpenXR Wii Remote.ini";
pub const GC_PROFILE: &str = "Profiles/GCPad/Quest Touch GameCube.ini";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Installation {
    pub executable: String,
    pub binary_sha256: String,
    pub sys_directory: String,
    pub wii_profile_sha256: Option<String>,
    pub gamecube_profile_sha256: Option<String>,
    pub identity_evidence: String,
    pub inspected_reference_revision: String,
    pub hardware_verified: bool,
}
pub fn hash_file(path: &Path) -> Result<String, String> {
    let mut file =
        fs::File::open(path).map_err(|e| format!("cannot open {}: {e}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}
fn canonical_file(path: &Path) -> Result<PathBuf, String> {
    let p = fs::canonicalize(path).map_err(|e| format!("invalid local file: {e}"))?;
    if !p.is_file() {
        return Err("select a local file".into());
    }
    Ok(p)
}
fn profile(sys: &Path, relative: &str) -> Result<String, String> {
    let p = canonical_file(&sys.join(relative))?;
    if !p.starts_with(sys) || fs::metadata(&p).map_err(|e| e.to_string())?.len() > 65536 {
        return Err("profile outside selected installation or too large".into());
    }
    let text = fs::read_to_string(p).map_err(|e| e.to_string())?;
    if !text.contains("[Profile]") || !text.contains("OpenXR/0/OpenXR Controller") {
        return Err("stock OpenXR controller profile missing or incompatible".into());
    }
    Ok(text)
}
pub fn inspect(executable: &Path) -> Result<Installation, String> {
    let executable = canonical_file(executable)?;
    let name = executable
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if ![
        "dolphin.exe",
        "dolphinxr.exe",
        "dolphin-emu",
        "dolphin-emu.exe",
    ]
    .contains(&name.as_str())
    {
        return Err("select the installed DolphinXR executable".into());
    }
    let root = executable.parent().ok_or("executable has no parent")?;
    let sys = fs::canonicalize(root.join("Sys"))
        .map_err(|_| "portable DolphinXR Sys folder must be beside the selected executable")?;
    let wii = profile(&sys, WII_PROFILE).ok();
    let gc = profile(&sys, GC_PROFILE).ok();
    if wii.is_none() && gc.is_none() {
        return Err(
            "installation lacks recognized stock OpenXR profiles; unsupported build".into(),
        );
    }
    Ok(Installation {
        executable: executable.to_string_lossy().into_owned(),
        binary_sha256: hash_file(&executable)?,
        sys_directory: sys.to_string_lossy().into_owned(),
        wii_profile_sha256: wii.map(|s| hex::encode(Sha256::digest(s.as_bytes()))),
        gamecube_profile_sha256: gc.map(|s| hex::encode(Sha256::digest(s.as_bytes()))),
        identity_evidence:
            "selected-local-executable + stock-profile hashes; not proof of a VR-enabled binary"
                .into(),
        inspected_reference_revision: INSPECTED_SOURCE.into(),
        hardware_verified: false,
    })
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ControllerProfile {
    WiiRemoteNunchuk,
    GamecubeTouch,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LaunchOptions {
    pub profile: ControllerProfile,
    pub flat_screen: bool,
    pub passthrough: bool,
    pub left_handed: bool,
    pub motion_pitch_degrees: f32,
    pub units_per_meter: f32,
    pub operator_confirmed_openxr_build: bool,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchPlan {
    pub executable: String,
    pub arguments: Vec<String>,
    pub user_directory: String,
    pub binary_sha256: String,
    pub game_sha256: String,
    pub profile_sha256: String,
    pub native_openxr_input: bool,
    pub browser_pose_injection: bool,
    pub commercial_game_verified: bool,
}
fn handed_profile(text: &str, left: bool) -> String {
    if !left {
        return text.into();
    }
    text.replace("Right", "__PHICADE_SWAP__")
        .replace("Left", "Right")
        .replace("__PHICADE_SWAP__", "Left")
        .replace("Left Button A", "Left Button X")
        .replace("Left Button B", "Left Button Y")
        .replace("Right Button X", "Right Button A")
        .replace("Right Button Y", "Right Button B")
}
fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
    {
        Ok(mut f) => f.write_all(bytes).map_err(|e| e.to_string()),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}
/// Called by a local operator command, with installation hash returned by inspect.
pub fn prepare(
    executable: &Path,
    expected_hash: &str,
    game: &Path,
    profile_root: &Path,
    options: &LaunchOptions,
) -> Result<LaunchPlan, String> {
    if !options.operator_confirmed_openxr_build {
        return Err(
            "confirm that the selected installation is an OpenXR-enabled DolphinXR build".into(),
        );
    }
    if !options.motion_pitch_degrees.is_finite()
        || !(-90.0..=90.0).contains(&options.motion_pitch_degrees)
        || !options.units_per_meter.is_finite()
        || !(0.01..=100.0).contains(&options.units_per_meter)
    {
        return Err("XR settings outside supported range".into());
    }
    let install = inspect(executable)?;
    if install.binary_sha256 != expected_hash {
        return Err("DolphinXR binary changed since inspection; inspect it again".into());
    }
    let game = canonical_file(game)?;
    let ext = game
        .extension()
        .and_then(|v| v.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if !["iso", "gcm", "wbfs", "rvz", "wia", "ciso", "dol", "elf"].contains(&ext.as_str()) {
        return Err("unsupported local Wii/GameCube content format".into());
    }
    let game_hash = hash_file(&game)?;
    let (relative, section, filename) = match options.profile {
        ControllerProfile::WiiRemoteNunchuk => (WII_PROFILE, "Wiimote1", "WiimoteNew.ini"),
        ControllerProfile::GamecubeTouch => (GC_PROFILE, "GCPad1", "GCPadNew.ini"),
    };
    let source = profile(Path::new(&install.sys_directory), relative)?;

    let key = format!(
        "{}-{}-{}",
        &game_hash[..16],
        section,
        if options.left_handed { "left" } else { "right" }
    );
    let user = profile_root.join(key);
    let config = user.join("Config");
    fs::create_dir_all(&config).map_err(|e| e.to_string())?;
    let source = handed_profile(&source, options.left_handed).replacen(
        "[Profile]",
        &format!("[{section}]"),
        1,
    );
    // This is the user's installed stock configuration, preserved locally. Keep
    // DolphinXR's distribution notices/source obligations with that installation.
    let source = if matches!(options.profile, ControllerProfile::WiiRemoteNunchuk) {
        format!("{source}\nSource = 3\n")
    } else {
        source
    };
    write_new(&config.join(filename), source.as_bytes())?;
    let mut arguments = vec![
        "--user".into(),
        user.to_string_lossy().into_owned(),
        "--exec".into(),
        game.to_string_lossy().into_owned(),
    ];
    for option in [
        "Graphics.VR.EnableOpenXR=True".into(),
        format!("Graphics.VR.FlatScreen={}", options.flat_screen),
        format!("Graphics.VR.Passthrough={}", options.passthrough),
        format!(
            "Graphics.VR.WiimoteMotionPitch={}",
            options.motion_pitch_degrees
        ),
        format!("Graphics.VR.UnitsPerMeter={}", options.units_per_meter),
        "Graphics.VR.AgentServer=False".into(),
    ] {
        arguments.push("--config".into());
        arguments.push(option);
    }
    Ok(LaunchPlan {
        executable: install.executable,
        arguments,
        user_directory: user.to_string_lossy().into_owned(),
        binary_sha256: install.binary_sha256,
        game_sha256: game_hash,
        profile_sha256: hash_file(&config.join(filename))?,
        native_openxr_input: true,
        browser_pose_injection: false,
        commercial_game_verified: false,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (PathBuf, PathBuf, PathBuf) {
        let root = std::env::temp_dir().join(format!(
            "phicade-dolphin-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let sys = root.join("Sys/Profiles/Wiimote");
        fs::create_dir_all(&sys).unwrap();
        let exe = root.join("Dolphin.exe");
        let game = root.join("original homebrew.dol");
        fs::write(&exe, b"synthetic test-only binary, never executed").unwrap();
        fs::write(&game, b"original synthetic homebrew fixture").unwrap();
        fs::write(
            sys.join("OpenXR Wii Remote.ini"),
            "[Profile]\nDevice = OpenXR/0/OpenXR Controller\nButtons/A = `Right Button A`\n",
        )
        .unwrap();
        (root, exe, game)
    }
    fn options() -> LaunchOptions {
        LaunchOptions {
            profile: ControllerProfile::WiiRemoteNunchuk,
            flat_screen: false,
            passthrough: false,
            left_handed: false,
            motion_pitch_degrees: 20.0,
            units_per_meter: 1.0,
            operator_confirmed_openxr_build: true,
        }
    }
    #[test]
    fn hashes_and_separate_process_arguments_are_bound() {
        let (root, exe, game) = fixture();
        let i = inspect(&exe).unwrap();
        let plan = prepare(
            &exe,
            &i.binary_sha256,
            &game,
            &root.join("user"),
            &options(),
        )
        .unwrap();
        let canonical_game = fs::canonicalize(&game).unwrap();
        assert!(plan
            .arguments
            .contains(&canonical_game.to_string_lossy().into_owned()));
        assert!(plan
            .arguments
            .contains(&"Graphics.VR.AgentServer=False".into()));
        assert!(!plan.browser_pose_injection);
        assert!(
            fs::read_to_string(Path::new(&plan.user_directory).join("Config/WiimoteNew.ini"))
                .unwrap()
                .contains("Source = 3")
        );
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn rejects_changed_binary_unconfirmed_build_and_invalid_settings() {
        let (root, exe, game) = fixture();
        let i = inspect(&exe).unwrap();
        assert!(prepare(&exe, "bad", &game, &root.join("user"), &options()).is_err());
        let mut o = options();
        o.operator_confirmed_openxr_build = false;
        assert!(prepare(&exe, &i.binary_sha256, &game, &root.join("user"), &o).is_err());
        o.operator_confirmed_openxr_build = true;
        o.units_per_meter = f32::NAN;
        assert!(prepare(&exe, &i.binary_sha256, &game, &root.join("user"), &o).is_err());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn handedness_swaps_button_labels_without_losing_device_identity() {
        let p = handed_profile(
            "Device = OpenXR/0/OpenXR Controller\nButtons/A = `Right Button A`\nZ = `Left Trigger`",
            true,
        );
        assert!(p.contains("Left Button X"));
        assert!(p.contains("Right Trigger"));
        assert!(p.contains("OpenXR/0/OpenXR Controller"));
    }
}
