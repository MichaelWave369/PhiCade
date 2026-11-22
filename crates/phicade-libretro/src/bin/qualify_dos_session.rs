//! Rung 6: exercise the production DOS session-input API against the original
//! 16-bit DOS VGA fixture. Lab feature is used ONLY to set the core's automatic
//! shutdown test option, not to send keyboard or mouse events.
use phicade_libretro::LibretroCore;
use phicade_runtime::{AudioBuffer, ContentDescriptor, EmulatorCore, FrameBuffer};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{env, fs, path::{Path, PathBuf}, process};

const UPSTREAM: &str = "e39497e7c2a32c67c26e6598dd14f6d58c384bfd";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionReceipt {
    schema: &'static str,
    result: &'static str,
    scope: &'static str,
    core_name: String,
    core_version: String,
    upstream_revision: &'static str,
    core_sha256: String,
    fixture_sha256: String,
    frames_executed: u64,
    magenta_frames: u64,
    green_frames: u64,
    cyan_frames: u64,
    shutdown_frame: Option<u64>,
    preapproval_input_refused: bool,
    revoked_input_refused: bool,
    forbidden_key_refused: bool,
    unbounded_mouse_refused: bool,
    session_approval_used: bool,
    production_keyboard_guest_observed: bool,
    production_mouse_click_guest_observed: bool,
    guest_shutdown_observed: bool,
    retail_game_compatibility_qualified: bool,
    windows_physical_device_qualified: bool,
}

fn sha256(path: &Path) -> Result<String, String> {
    let data = fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    Ok(format!("{:x}", Sha256::digest(data)))
}

fn filled(frame: &FrameBuffer, color: &str) -> bool {
    let pixels = (frame.width as usize).saturating_mul(frame.height as usize);
    if pixels == 0 || frame.rgba8.len() < pixels.saturating_mul(4) { return false; }
    let count = frame.rgba8.chunks_exact(4).filter(|p| match color {
        "magenta" => p[0] >= 145 && p[2] >= 145 && p[1] < 135,
        "green" => p[1] >= 145 && p[0] < 135 && p[2] < 135,
        "cyan" => p[1] >= 145 && p[2] >= 145 && p[0] < 135,
        _ => false,
    }).count();
    count > pixels / 3
}

fn run() -> Result<(), String> {
    let args: Vec<_> = env::args().collect();
    fn value(args: &[String], name: &str) -> Result<PathBuf, String> {
        let pos = args.iter().position(|v| v == name)
            .ok_or_else(|| format!("missing {name}"))?;
        args.get(pos + 1).map(PathBuf::from)
            .ok_or_else(|| format!("missing value for {name}"))
    }
    let core_path = value(&args, "--core")?;
    let fixture_path = value(&args, "--fixture")?;
    let receipt_path = value(&args, "--receipt")?;
    if !core_path.is_file() || !fixture_path.is_file() ||
        !fixture_path.extension().and_then(|v| v.to_str()).is_some_and(|v| v == "dosz") {
        return Err("requires a real core and original .dosz fixture".into());
    }
    let root = env::temp_dir().join(format!("phicade-dos-session-{}", process::id()));
    fs::create_dir_all(root.join("system")).map_err(|e| e.to_string())?;
    fs::create_dir_all(root.join("save")).map_err(|e| e.to_string())?;
    let mut core = LibretroCore::open(&core_path, root.join("system"), root.join("save"))
        .map_err(|e| format!("open core: {e:?}"))?;
    let identity = core.identity().clone();
    if !identity.library_name.eq_ignore_ascii_case("DOSBox-pure") {
        return Err(format!("unexpected core {}", identity.library_name));
    }
    // This method only changes DOSBox Pure's fixture exit policy; it does not
    // synthesize input. All guest input below uses the normal session API.
    core.lab_request_shutdown_when_guest_exits()
        .map_err(|e| format!("enable fixture shutdown policy: {e:?}"))?;
    core.load_content(&ContentDescriptor::file(&fixture_path, "Original PhiCade DOS input fixture")
        .with_runtime_hint(identity.library_name.clone()))
        .map_err(|e| format!("load original DOS fixture: {e:?}"))?;

    let preapproval_input_refused =
        core.dos_session_key("Escape", true).is_err() &&
        core.dos_session_mouse(0, 0, true).is_err();
    if !preapproval_input_refused { return Err("input accepted without approval".into()); }
    core.enable_dos_session_input().map_err(|e| format!("session approval: {e:?}"))?;
    if !core.dos_session_input_enabled() { return Err("session approval did not take effect".into()); }

    let forbidden_key_refused = core.dos_session_key("F11", true).is_err();
    let unbounded_mouse_refused = core.dos_session_mouse(65, 0, false).is_err()
        && core.dos_session_mouse(0, -65, false).is_err();
    if !forbidden_key_refused || !unbounded_mouse_refused {
        return Err("production session accepted forbidden input".into());
    }

    let mut frame = FrameBuffer::default();
    let mut audio = AudioBuffer::default();
    let (mut magenta, mut green, mut cyan) = (0, 0, 0);
    let mut frames_executed = 0;
    let mut shutdown_frame = None;
    let mut revoked_input_refused = false;
    for i in 0..240_u64 {
        if i == 45 {
            core.dos_session_key("w", true).map_err(|e| format!("hold test: {e:?}"))?;
        }
        if i == 48 {
            core.disable_dos_session_input();
            revoked_input_refused = !core.dos_session_input_enabled()
                && core.dos_session_key("w", false).is_err()
                && core.dos_session_mouse(0, 0, true).is_err();
            if !revoked_input_refused { return Err("revoked session still accepted input".into()); }
        }
        if i == 50 {
            core.enable_dos_session_input().map_err(|e| format!("reapproval: {e:?}"))?;
        }
        if i == 110 { core.dos_session_key("Escape", true).map_err(|e| format!("Esc down: {e:?}"))?; }
        if i == 115 { core.dos_session_key("Escape", false).map_err(|e| format!("Esc up: {e:?}"))?; }
        if i == 145 { core.dos_session_mouse(8, -3, true).map_err(|e| format!("mouse down: {e:?}"))?; }
        if i == 170 { core.dos_session_mouse(0, 0, false).map_err(|e| format!("mouse up: {e:?}"))?; }
        if i == 200 { core.dos_session_key("q", true).map_err(|e| format!("Q down: {e:?}"))?; }
        if i == 205 { core.dos_session_key("q", false).map_err(|e| format!("Q up: {e:?}"))?; }
        core.step_frame(&[], &mut frame, &mut audio)
            .map_err(|e| format!("guest frame {i}: {e:?}"))?;
        frames_executed += 1;
        if filled(&frame, "magenta") { magenta += 1; }
        if filled(&frame, "green") { green += 1; }
        if filled(&frame, "cyan") { cyan += 1; }
        if core.shutdown_requested() {
            if i <= 200 { return Err(format!("premature DOS shutdown at frame {i}")); }
            shutdown_frame = Some(i);
            break;
        }
    }
    if magenta < 3 || green < 3 || cyan < 3 || shutdown_frame.is_none() {
        return Err(format!(
            "guest transitions incomplete: magenta={magenta}, green={green}, cyan={cyan}, shutdown={shutdown_frame:?}"
        ));
    }
    let receipt = SessionReceipt {
        schema: "phicade.dos-session-original-fixture.v1",
        result: "PASS",
        scope: "original 16-bit DOS source fixture via production libretro session input only",
        core_name: identity.library_name,
        core_version: identity.library_version,
        upstream_revision: UPSTREAM,
        core_sha256: sha256(&core_path)?,
        fixture_sha256: sha256(&fixture_path)?,
        frames_executed,
        magenta_frames: magenta,
        green_frames: green,
        cyan_frames: cyan,
        shutdown_frame,
        preapproval_input_refused,
        revoked_input_refused,
        forbidden_key_refused,
        unbounded_mouse_refused,
        session_approval_used: true,
        production_keyboard_guest_observed: true,
        production_mouse_click_guest_observed: true,
        guest_shutdown_observed: true,
        retail_game_compatibility_qualified: false,
        windows_physical_device_qualified: false,
    };
    let json = serde_json::to_string_pretty(&receipt).map_err(|e| e.to_string())?;
    if let Some(parent) = receipt_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&receipt_path, format!("{json}\n")).map_err(|e| e.to_string())?;
    println!("{json}");
    core.disable_dos_session_input();
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("Production DOS session qualification FAILED: {e}");
        process::exit(1);
    }
}
