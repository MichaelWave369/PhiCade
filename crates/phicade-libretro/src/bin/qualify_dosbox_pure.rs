//! Run an original DOS VGA+speaker fixture through PhiCade's real libretro host.
//! This is NOT a claim that commercial DOS games, keyboard, mouse or exit work.
use phicade_libretro::LibretroCore;
use phicade_runtime::{
    ActionEnvelope, ActionKind, ActionSource, AudioBuffer, ContentDescriptor, EmulatorCore,
    FrameBuffer, RuntimeCapabilityManifest,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{env, fs, path::{Path, PathBuf}, process};

const MIN_FRAMES: u64 = 120;
const MAX_FRAMES: u64 = 600;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DosQualificationReceipt {
    schema: &'static str,
    result: &'static str,
    scope: &'static str,
    core_name: String,
    core_version: String,
    upstream_source_revision: &'static str,
    core_sha256: String,
    original_fixture_sha256: String,
    requested_frames: u64,
    frames_with_video: u64,
    frames_with_audio_callback: u64,
    frames_with_nonzero_audio: u64,
    frames_with_magenta_vga: u64,
    frames_with_green_vga: u64,
    keyboard_callback_registered: bool,
    escape_guest_visual_transition_observed: bool,
    q_post_exit_visual_transition_observed: bool,
    mouse_device_polled_in_core: bool,
    mouse_device_poll_count: u64,
    final_width: u32,
    final_height: u32,
    final_frame_sha256: String,
    libretro_audio_callback_observed: bool,
    nonzero_audio_samples_observed: bool,
    magenta_graphics_observed: bool,
    retropad_host_routing_observed: bool,
    keyboard_guest_delivery_qualified: bool,
    mouse_guest_delivery_qualified: bool,
    guest_clean_exit_qualified: bool,
    retail_dos_game_compatibility_qualified: bool,
    capability_manifest: RuntimeCapabilityManifest,
}

fn sha256_path(path: &Path) -> Result<String, String> {
    fs::read(path).map(|data| format!("{:x}", Sha256::digest(data)))
        .map_err(|e| format!("cannot hash {}: {e}", path.display()))
}

fn button(sequence: u64, frame: u64, pressed: bool) -> ActionEnvelope {
    ActionEnvelope {
        sequence,
        frame,
        source: ActionSource::Script { name: "original-dos-fixture-qualification".into() },
        action: ActionKind::Button { button: "RIGHT".into(), pressed },
    }
}

fn count_vga_magenta(frame: &FrameBuffer) -> usize {
    frame.rgba8.chunks_exact(4)
        .filter(|p| p[0] >= 145 && p[2] >= 145 && p[1] < 135)
        .count()
}
fn count_vga_green(frame: &FrameBuffer) -> usize {
    frame.rgba8.chunks_exact(4)
        .filter(|p| p[1] >= 145 && p[0] < 135 && p[2] < 135)
        .count()
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let mut core_path: Option<PathBuf> = None;
    let mut fixture_path: Option<PathBuf> = None;
    let mut receipt_path: Option<PathBuf> = None;
    let mut frames = 240u64;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--core" => core_path = args.next().map(PathBuf::from),
            "--fixture" => fixture_path = args.next().map(PathBuf::from),
            "--receipt" => receipt_path = args.next().map(PathBuf::from),
            "--frames" => frames = args.next().and_then(|s| s.parse::<u64>().ok())
                .ok_or_else(|| "invalid frames".to_owned())?,
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }
    let core_path = core_path.ok_or("missing --core argument")?;
    let fixture_path = fixture_path.ok_or("missing --fixture argument")?;
    if !(MIN_FRAMES..=MAX_FRAMES).contains(&frames) {
        return Err(format!("frames must be between {MIN_FRAMES} and {MAX_FRAMES}"));
    }
    if !fixture_path.is_file() || !fixture_path.extension()
        .and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case("dosz")) {
        return Err("fixture must be an existing .dosz archive".into());
    }

    let root = env::temp_dir().join(format!("phicade-dos-qualify-{}", process::id()));
    let system_dir = root.join("system");
    let save_dir = root.join("save");
    fs::create_dir_all(&system_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&save_dir).map_err(|e| e.to_string())?;
    let mut core = LibretroCore::open(&core_path, &system_dir, &save_dir)
        .map_err(|e| format!("could not open DOSBox Pure: {e:?}"))?;
    let identity = core.identity().clone();
    if !identity.library_name.eq_ignore_ascii_case("DOSBox-pure") {
        return Err(format!("wrong core: expected DOSBox-pure, found {}", identity.library_name));
    }
    if !identity.valid_extensions.split('|').any(|e| e.eq_ignore_ascii_case("dosz")) {
        return Err("core does not advertise .dosz support".into());
    }
    let fixture = ContentDescriptor::file(&fixture_path, "PhiCade original DOS VGA test")
        .with_runtime_hint(identity.library_name.clone());
    core.load_content(&fixture)
        .map_err(|e| format!("DOSBox Pure could not load DOSZ: {e:?}"))?;
    let capability_manifest = core.capability_manifest();
    if !core.lab_keyboard_registered() {
        return Err("DOSBox Pure did not register libretro keyboard callback".into());
    }

    let mut video = FrameBuffer::default();
    let mut audio = AudioBuffer::default();
    let mut frames_with_video = 0;
    let mut frames_with_audio_callback = 0;
    let mut frames_with_nonzero_audio = 0;
    let mut frames_with_magenta_vga = 0;
    let mut frames_with_green_vga = 0;
    let mut q_post_exit_visual_transition_observed = false;
    let mut retropad_host_routing_observed = false;
    for i in 0..frames {
        // Test-only host input hooks, never exposed to Tauri or agent sessions.
        if i == 75 { core.lab_mouse_state(8, -3, true).map_err(|e| format!("mouse press: {e:?}"))?; }
        if i == 82 { core.lab_mouse_state(0, 0, false).map_err(|e| format!("mouse release: {e:?}"))?; }
        if i == 110 { core.lab_send_escape(true).map_err(|e| format!("escape down: {e:?}"))?; }
        if i == 115 { core.lab_send_escape(false).map_err(|e| format!("escape up: {e:?}"))?; }
        if i == 175 { core.lab_send_q(true).map_err(|e| format!("Q down: {e:?}"))?; }
        if i == 180 { core.lab_send_q(false).map_err(|e| format!("Q up: {e:?}"))?; }
        let actions = if i == 60 { vec![button(1, i, true)] }
        else if i == 70 { vec![button(2, i, false)] }
        else { Vec::new() };
        core.step_frame(&actions, &mut video, &mut audio)
            .map_err(|e| format!("frame {i} failed: {e:?}"))?;
        if !video.rgba8.is_empty() && video.width > 0 && video.height > 0 {
            frames_with_video += 1;
            let pixels = (video.width as usize).saturating_mul(video.height as usize);
            if pixels > 0 {
                let magenta = count_vga_magenta(&video) > pixels / 3;
                let green = count_vga_green(&video) > pixels / 3;
                if magenta { frames_with_magenta_vga += 1; }
                if green { frames_with_green_vga += 1; }
                // After the guest Q handler, a returned menu/text screen
                // is corroborating evidence, NOT direct proof of DOS exit.
                if i > 190 && !green && frames_with_green_vga >= 3 {
                    q_post_exit_visual_transition_observed = true;
                }
            }
        }
        if !audio.interleaved_stereo_f32.is_empty() {
            frames_with_audio_callback += 1;
        }
        if audio.interleaved_stereo_f32.iter().any(|sample| sample.abs() > 0.001) {
            frames_with_nonzero_audio += 1;
        }
        if i == 60 && core.input_mask_snapshot() != 0 {
            retropad_host_routing_observed = true;
        }
        if i == 70 && core.input_mask_snapshot() != 0 {
            return Err("retropad release did not clear input state".into());
        }
    }
    if frames_with_video == 0 || video.width == 0 || video.height == 0 {
        return Err("DOSBox Pure provided no renderable video".into());
    }
    if frames_with_magenta_vga < 3 {
        return Err(format!("original VGA magenta screen not observed; matching frames={frames_with_magenta_vga}"));
    }
    if frames_with_green_vga < 3 {
        return Err(format!("DOS BIOS did not observably respond to Escape; matching green frames={frames_with_green_vga}"));
    }
    if !retropad_host_routing_observed {
        return Err("governed retropad button did not reach host state".into());
    }

    let receipt = DosQualificationReceipt {
        schema: "phicade.dosbox-pure-original-fixture.v1",
        result: "PASS",
        scope: "original DOS .COM fixture only; no commercial game compatibility claim",
        core_name: identity.library_name,
        core_version: identity.library_version,
        upstream_source_revision: "e39497e7c2a32c67c26e6598dd14f6d58c384bfd",
        core_sha256: sha256_path(&core_path)?,
        original_fixture_sha256: sha256_path(&fixture_path)?,
        requested_frames: frames,
        frames_with_video,
        frames_with_audio_callback,
        frames_with_nonzero_audio,
        frames_with_magenta_vga,
        frames_with_green_vga,
        keyboard_callback_registered: true,
        escape_guest_visual_transition_observed: true,
        q_post_exit_visual_transition_observed,
        mouse_device_polled_in_core: core.lab_mouse_poll_count() > 0,
        mouse_device_poll_count: core.lab_mouse_poll_count(),
        final_width: video.width,
        final_height: video.height,
        final_frame_sha256: format!("{:x}", Sha256::digest(&video.rgba8)),
        libretro_audio_callback_observed: frames_with_audio_callback > 0,
        nonzero_audio_samples_observed: frames_with_nonzero_audio > 0,
        magenta_graphics_observed: true,
        retropad_host_routing_observed,
        // Green state is only reachable via BIOS keyboard Esc branch.
        keyboard_guest_delivery_qualified: true,
        mouse_guest_delivery_qualified: false,
        guest_clean_exit_qualified: false,
        retail_dos_game_compatibility_qualified: false,
        capability_manifest,
    };
    let json = serde_json::to_string_pretty(&receipt).map_err(|e| e.to_string())?;
    println!("{json}");
    if let Some(path) = receipt_path {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(path, format!("{json}\n")).map_err(|e| e.to_string())?;
    }
    drop(core); // Host RAII cleanup does not prove DOS guest exit.
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Original DOS fixture qualification FAILED: {error}");
        process::exit(1);
    }
}
